use anyhow::Error;
use handlers::{
    batch::{handle_command_list_begin, handle_command_list_ok_begin},
    browse::{handle_listall, handle_listallinfo, handle_listfiles, handle_lsinfo},
    library::{
        handle_config, handle_find, handle_find_album, handle_find_artist, handle_find_title,
        handle_list_album, handle_list_artist, handle_list_title, handle_rescan, handle_search,
        handle_stats, handle_tagtypes, handle_tagtypes_enable,
    },
    playback::{
        handle_currentsong, handle_getvol, handle_next, handle_outputs, handle_pause, handle_play,
        handle_playid, handle_previous, handle_random, handle_repeat, handle_seek, handle_seekcur,
        handle_seekid, handle_setvol, handle_single, handle_status, handle_toggle,
    },
    queue::{
        handle_add, handle_addid, handle_clear, handle_delete, handle_deleteid, handle_move,
        handle_playlistinfo, handle_shuffle,
    },
    system::{handle_binarylimit, handle_commands, handle_decoders, handle_idle, handle_noidle},
    Subsystem,
};
use kv::{build_tracks_kv, KV};
use rockbox_graphql::{
    schema::objects::{audio_status::AudioStatus, playlist::Playlist, track::Track},
    simplebroker::SimpleBroker,
};
use rockbox_library::{create_connection_pool, entity, repo};
use rockbox_rpc::api::rockbox::v1alpha1::{
    library_service_client::LibraryServiceClient, playback_service_client::PlaybackServiceClient,
    playlist_service_client::PlaylistServiceClient, settings_service_client::SettingsServiceClient,
    sound_service_client::SoundServiceClient, system_service_client::SystemServiceClient,
    GetCurrentRequest, GetGlobalStatusRequest, PlaylistResumeRequest,
};
use rockbox_sys::{playback::current_track, types::user_settings::UserSettings};
use sqlx::{Pool, Sqlite};
use std::{env, sync::Arc, thread, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{broadcast, Mutex},
};
use tokio_stream::StreamExt;
use tonic::transport::Channel;

pub mod consts;
pub mod dir;
pub mod handlers;
pub mod kv;

#[derive(Clone)]
pub struct Context {
    pub library: LibraryServiceClient<Channel>,
    pub playback: PlaybackServiceClient<Channel>,
    pub settings: SettingsServiceClient<Channel>,
    pub sound: SoundServiceClient<Channel>,
    pub playlist: PlaylistServiceClient<Channel>,
    pub system: SystemServiceClient<Channel>,
    pub single: Arc<Mutex<String>>,
    pub batch: bool,
    pub event_sender: broadcast::Sender<Subsystem>,
    pub event_receiver: Arc<Mutex<broadcast::Receiver<Subsystem>>>,
    pub current_track: Arc<Mutex<Option<Track>>>,
    pub current_playlist: Arc<Mutex<Option<Playlist>>>,
    pub playback_status: Arc<Mutex<Option<AudioStatus>>>,
    pub pool: Pool<Sqlite>,
    pub kv: Arc<Mutex<KV<entity::track::Track>>>,
    pub current_settings: Arc<Mutex<UserSettings>>,
}

pub struct MpdServer {}

impl MpdServer {
    pub async fn start() -> Result<(), Error> {
        let port = env::var("ROCKBOX_MPD_PORT").unwrap_or_else(|_| "6600".to_string());
        let addr = format!("0.0.0.0:{}", port);
        let context = setup_context(false, None).await?;

        listen_events(context.clone());

        thread::sleep(Duration::from_millis(200));

        restore_playlist(context.clone())?;

        let listener = TcpListener::bind(&addr).await?;

        loop {
            let (stream, _) = listener.accept().await?;
            let context = context.clone();
            tokio::spawn(async move {
                match handle_client(context, stream).await {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            });
        }
    }
}

pub async fn handle_client(mut ctx: Context, stream: TcpStream) -> Result<(), Error> {
    let mut buf = [0; 4096];
    let (reader_stream, writer_stream) = tokio::io::split(stream);
    let mut reader = tokio::io::BufReader::new(reader_stream);
    let mut writer = tokio::io::BufWriter::new(writer_stream);

    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(32);

    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            writer.write_all(msg.as_bytes()).await?;
            writer.flush().await?;
        }
        Ok::<(), Error>(())
    });

    tx.send("OK MPD 0.23.15\n".to_string()).await?;

    while let Ok(n) = reader.read(&mut buf).await {
        if n == 0 {
            break;
        }
        let request = String::from_utf8_lossy(&buf[..n]);
        let command = parse_command(&request)?;
        println!("request: {}", request);

        match command.as_str() {
            "play" => handle_play(&mut ctx, &request, tx.clone()).await?,
            "pause" => handle_pause(&mut ctx, &request, tx.clone()).await?,
            "toggle" => handle_toggle(&mut ctx, &request, tx.clone()).await?,
            "next" => handle_next(&mut ctx, &request, tx.clone()).await?,
            "previous" => handle_previous(&mut ctx, &request, tx.clone()).await?,
            "playid" => handle_playid(&mut ctx, &request, tx.clone()).await?,
            "seek" => handle_seek(&mut ctx, &request, tx.clone()).await?,
            "seekid" => handle_seekid(&mut ctx, &request, tx.clone()).await?,
            "seekcur" => handle_seekcur(&mut ctx, &request, tx.clone()).await?,
            "random" => handle_random(&mut ctx, &request, tx.clone()).await?,
            "repeat" => handle_repeat(&mut ctx, &request, tx.clone()).await?,
            "getvol" => handle_getvol(&mut ctx, &request, tx.clone()).await?,
            "setvol" => handle_setvol(&mut ctx, &request, tx.clone()).await?,
            "volume" => handle_setvol(&mut ctx, &request, tx.clone()).await?,
            "single" => handle_single(&mut ctx, &request, tx.clone()).await?,
            "shuffle" => handle_shuffle(&mut ctx, &request, tx.clone()).await?,
            "add" => handle_add(&mut ctx, &request, tx.clone()).await?,
            "addid" => handle_addid(&mut ctx, &request, tx.clone()).await?,
            "deleteid" => handle_deleteid(&mut ctx, &request, tx.clone()).await?,
            "playlistinfo" => handle_playlistinfo(&mut ctx, &request, tx.clone()).await?,
            "delete" => handle_delete(&mut ctx, &request, tx.clone()).await?,
            "clear" => handle_clear(&mut ctx, &request, tx.clone()).await?,
            "move" => handle_move(&mut ctx, &request, tx.clone()).await?,
            "list album" => handle_list_album(&mut ctx, &request, tx.clone()).await?,
            "list albumartist" => handle_list_artist(&mut ctx, &request, tx.clone()).await?,
            "list artist" => handle_list_artist(&mut ctx, &request, tx.clone()).await?,
            "list title" => handle_list_title(&mut ctx, &request, tx.clone()).await?,
            "update" => handle_rescan(&mut ctx, &request, tx.clone()).await?,
            "search" => handle_search(&mut ctx, &request, tx.clone()).await?,
            "rescan" => handle_rescan(&mut ctx, &request, tx.clone()).await?,
            "status" => handle_status(&mut ctx, &request, tx.clone()).await?,
            "currentsong" => handle_currentsong(&mut ctx, &request, tx.clone()).await?,
            "config" => handle_config(&mut ctx, &request, tx.clone()).await?,
            "tagtypes " => handle_tagtypes(&mut ctx, &request, tx.clone()).await?,
            "tagtypes clear" => handle_clear(&mut ctx, &request, tx.clone()).await?,
            "tagtypes enable" => handle_tagtypes_enable(&mut ctx, &request, tx.clone()).await?,
            "stats" => handle_stats(&mut ctx, &request, tx.clone()).await?,
            "plchanges" => handle_playlistinfo(&mut ctx, &request, tx.clone()).await?,
            "outputs" => handle_outputs(&mut ctx, &request, tx.clone()).await?,
            "idle" => handle_idle(&mut ctx, &request, tx.clone()).await?,
            "noidle" => handle_noidle(&mut ctx, &request, tx.clone()).await?,
            "decoders" => handle_decoders(&mut ctx, &request, tx.clone()).await?,
            "lsinfo" => handle_lsinfo(&mut ctx, &request, tx.clone()).await?,
            "listall" => handle_listall(&mut ctx, &request, tx.clone()).await?,
            "listallinfo" => handle_listallinfo(&mut ctx, &request, tx.clone()).await?,
            "listfiles" => handle_listfiles(&mut ctx, &request, tx.clone()).await?,
            "find artist" => handle_find_artist(&mut ctx, &request, tx.clone()).await?,
            "find album" => handle_find_album(&mut ctx, &request, tx.clone()).await?,
            "find title" => handle_find_title(&mut ctx, &request, tx.clone()).await?,
            "binarylimit" => handle_binarylimit(&mut ctx, &request, tx.clone()).await?,
            "commands" => handle_commands(&mut ctx, &request, tx.clone()).await?,
            "command_list_begin" => {
                handle_command_list_begin(&mut ctx, &request, tx.clone()).await?
            }
            "command_list_ok_begin" => {
                handle_command_list_ok_begin(&mut ctx, &request, tx.clone()).await?
            }
            _ => {
                if command.starts_with("find ") {
                    handle_find(&mut ctx, &request, tx.clone()).await?;
                    return Ok(());
                }
                println!("Unhandled command: {}", command);
                println!("Unhandled request: {}", request);
                tx.send("ACK [5@0] {unhandled} unknown command\n".to_string())
                    .await?;
                "ACK [5@0] {unhandled} unknown command\n".to_string()
            }
        };
    }
    Ok(())
}

fn parse_command(request: &str) -> Result<String, Error> {
    let command = request.split_whitespace().next().unwrap_or_default();

    if command == "list" {
        // should parse the next word, and return "list album" or "list artist" or "list title"
        let r#type = request.split_whitespace().nth(1).unwrap_or_default();
        return Ok(format!("list {}", r#type.to_lowercase()));
    }

    if command == "tagtypes" {
        let r#type = request.split_whitespace().nth(1).unwrap_or_default();
        return Ok(format!("tagtypes {}", r#type.replace("\"", "")));
    }

    if command == "find" {
        let r#type = request.split_whitespace().nth(1).unwrap_or_default();
        return Ok(format!("find {}", r#type.to_lowercase()));
    }

    Ok(command.to_string())
}

pub async fn setup_context(batch: bool, ctx: Option<Context>) -> Result<Context, Error> {
    let port = env::var("ROCKBOX_PORT").unwrap_or_else(|_| "6061".to_string());
    let host = env::var("ROCKBOX_HOST").unwrap_or_else(|_| "localhost".to_string());
    let url = format!("tcp://{}:{}", host, port);

    let pool = create_connection_pool().await?;
    let kv = Arc::new(Mutex::new(build_tracks_kv(pool.clone()).await?));

    let library = LibraryServiceClient::connect(url.clone()).await?;
    let playback = PlaybackServiceClient::connect(url.clone()).await?;
    let settings = SettingsServiceClient::connect(url.clone()).await?;
    let sound = SoundServiceClient::connect(url.clone()).await?;
    let playlist = PlaylistServiceClient::connect(url.clone()).await?;
    let system = SystemServiceClient::connect(url.clone()).await?;

    let (event_sender, event_receiver) = broadcast::channel(16);

    Ok(Context {
        library,
        playback,
        settings,
        sound,
        playlist,
        system,
        single: Arc::new(Mutex::new("\"0\"".to_string())),
        batch,
        event_sender: match ctx {
            Some(ref ctx) => ctx.clone().event_sender,
            None => event_sender,
        },
        event_receiver: match ctx {
            Some(ref ctx) => ctx.clone().event_receiver,
            None => Arc::new(Mutex::new(event_receiver)),
        },
        current_track: match ctx {
            Some(ref ctx) => ctx.clone().current_track,
            None => Arc::new(Mutex::new(None)),
        },
        current_playlist: match ctx {
            Some(ref ctx) => ctx.clone().current_playlist,
            None => Arc::new(Mutex::new(None)),
        },
        playback_status: match ctx {
            Some(ref ctx) => ctx.clone().playback_status,
            None => Arc::new(Mutex::new(None)),
        },
        pool,
        kv,
        current_settings: Arc::new(Mutex::new(rockbox_sys::settings::get_global_settings())),
    })
}

pub fn listen_events(ctx: Context) {
    let ctx_clone = ctx.clone();
    let another_ctx = ctx.clone();
    let another_cloned_ctx = ctx.clone();

    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        loop {
            let mut current_settings = rt.block_on(another_cloned_ctx.current_settings.lock());
            *current_settings = rockbox_sys::settings::get_global_settings();
            drop(current_settings);
            thread::sleep(std::time::Duration::from_millis(800));
        }
    });

    thread::spawn(move || {
        let mut subscription = SimpleBroker::<Track>::subscribe();
        let rt = tokio::runtime::Runtime::new().unwrap();

        while let Some(track) = rt.block_on(subscription.next()) {
            let mut current_track = rt.block_on(ctx.current_track.lock());
            *current_track = Some(track);
        }
    });

    thread::spawn(move || {
        let mut subscription = SimpleBroker::<Playlist>::subscribe();
        let rt = tokio::runtime::Runtime::new().unwrap();

        while let Some(playlist) = rt.block_on(subscription.next()) {
            let mut current_playlist = rt.block_on(ctx_clone.current_playlist.lock());

            // verify if current_playlist index is different from playlist index
            if (current_playlist.is_some()
                && current_playlist.as_ref().unwrap().index != playlist.index)
                || current_playlist.is_none()
            {
                let ctx = ctx_clone.clone();
                match ctx.event_sender.send(Subsystem::Playlist) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("Error: {}", e)
                    }
                }
            }

            *current_playlist = Some(playlist);
        }
    });

    thread::spawn(move || {
        let mut subscription = SimpleBroker::<AudioStatus>::subscribe();
        let rt = tokio::runtime::Runtime::new().unwrap();

        while let Some(status) = rt.block_on(subscription.next()) {
            let mut playback_status = rt.block_on(another_ctx.playback_status.lock());
            // verify if playback_status status is different from status status
            if playback_status.is_some()
                && playback_status.as_ref().unwrap().status != status.status
            {
                let ctx = another_ctx.clone();
                match ctx.event_sender.send(Subsystem::Player) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("Error: {}", e)
                    }
                }
            }
            *playback_status = Some(status);
        }
    });
}

pub fn restore_playlist(ctx: Context) -> Result<(), Error> {
    let ctx_clone = ctx.clone();
    thread::spawn(move || {
        let mut ctx = ctx_clone.clone();
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let response = ctx
                .system
                .get_global_status(GetGlobalStatusRequest {})
                .await?;
            let response = response.into_inner();

            let playback_status = ctx.playback_status.lock().await;
            let mut status = 0;

            if playback_status.is_some() {
                status = playback_status.as_ref().unwrap().status;
            }

            if response.resume_index > -1 && status != 1 {
                ctx.playlist
                    .playlist_resume(PlaylistResumeRequest {})
                    .await?;
                let resume_index = response.resume_index;
                let resume_elapsed = response.resume_elapsed;
                thread::sleep(std::time::Duration::from_millis(500));
                let response = ctx.playlist.get_current(GetCurrentRequest {}).await?;
                let response = response.into_inner();
                let mut current_track = ctx.current_track.lock().await;
                let path = response.tracks[resume_index as usize].path.clone();

                let mut track: Track = Track {
                    path: path.clone(),
                    artist: response.tracks[resume_index as usize].artist.clone(),
                    album: response.tracks[resume_index as usize].album.clone(),
                    title: response.tracks[resume_index as usize].title.clone(),
                    album_artist: response.tracks[resume_index as usize].album_artist.clone(),
                    elapsed: resume_elapsed as u64,
                    length: response.tracks[resume_index as usize].length,
                    tracknum: response.tracks[resume_index as usize].tracknum,
                    year: response.tracks[resume_index as usize].year,
                    year_string: response.tracks[resume_index as usize].year_string.clone(),
                    ..Default::default()
                };

                *current_track = Some(track.clone());
                let hash = format!("{:x}", md5::compute(path.as_bytes()));
                let pool = rockbox_library::create_connection_pool().await?;

                if let Ok(Some(metadata)) = repo::track::find_by_md5(pool.clone(), &hash).await {
                    track.id = Some(metadata.id);
                    track.album_art = metadata.album_art;
                    track.album_id = Some(metadata.album_id);
                    track.artist_id = Some(metadata.artist_id);
                    SimpleBroker::publish(track);
                }
            }

            Ok::<(), Error>(())
        })?;

        Ok::<(), Error>(())
    });

    Ok(())
}
