use std::{env, sync::Arc};

use anyhow::Error;
use handlers::{
    batch::{handle_command_list_begin, handle_command_list_ok_begin},
    library::{
        handle_config, handle_list_album, handle_list_artist, handle_list_title, handle_rescan,
        handle_search, handle_stats, handle_tagtypes,
    },
    playback::{
        handle_currentsong, handle_getvol, handle_next, handle_pause, handle_play, handle_playid,
        handle_previous, handle_random, handle_repeat, handle_seek, handle_seekcur, handle_seekid,
        handle_setvol, handle_single, handle_status, handle_toggle,
    },
    queue::{
        handle_add, handle_clear, handle_delete, handle_move, handle_playlistinfo, handle_shuffle,
    },
};
use rockbox_rpc::api::rockbox::v1alpha1::{
    library_service_client::LibraryServiceClient, playback_service_client::PlaybackServiceClient,
    playlist_service_client::PlaylistServiceClient, settings_service_client::SettingsServiceClient,
    sound_service_client::SoundServiceClient,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Mutex,
};
use tonic::transport::Channel;

pub mod handlers;

#[derive(Clone)]
pub struct Context {
    pub library: LibraryServiceClient<Channel>,
    pub playback: PlaybackServiceClient<Channel>,
    pub settings: SettingsServiceClient<Channel>,
    pub sound: SoundServiceClient<Channel>,
    pub playlist: PlaylistServiceClient<Channel>,
    pub single: Arc<Mutex<String>>,
    pub batch: bool,
}

pub struct MpdServer {}

impl MpdServer {
    pub async fn start() -> Result<(), Error> {
        let port = env::var("ROCKBOX_MPD_PORT").unwrap_or_else(|_| "6600".to_string());
        let addr = format!("0.0.0.0:{}", port);
        let context = setup_context(false).await?;

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
    let mut stream = tokio::io::BufReader::new(stream);
    stream.write_all(b"OK MPD 0.23.15\n").await?;

    while let Ok(n) = stream.read(&mut buf).await {
        if n == 0 {
            break;
        }
        let request = String::from_utf8_lossy(&buf[..n]);
        let command = parse_command(&request)?;

        match command.as_str() {
            "play" => handle_play(&mut ctx, &request, &mut stream).await?,
            "pause" => handle_pause(&mut ctx, &request, &mut stream).await?,
            "toggle" => handle_toggle(&mut ctx, &request, &mut stream).await?,
            "next" => handle_next(&mut ctx, &request, &mut stream).await?,
            "previous" => handle_previous(&mut ctx, &request, &mut stream).await?,
            "playid" => handle_playid(&mut ctx, &request, &mut stream).await?,
            "seek" => handle_seek(&mut ctx, &request, &mut stream).await?,
            "seekid" => handle_seekid(&mut ctx, &request, &mut stream).await?,
            "seekcur" => handle_seekcur(&mut ctx, &request, &mut stream).await?,
            "random" => handle_random(&mut ctx, &request, &mut stream).await?,
            "repeat" => handle_repeat(&mut ctx, &request, &mut stream).await?,
            "getvol" => handle_getvol(&mut ctx, &request, &mut stream).await?,
            "setvol" => handle_setvol(&mut ctx, &request, &mut stream).await?,
            "volume" => handle_setvol(&mut ctx, &request, &mut stream).await?,
            "single" => handle_single(&mut ctx, &request, &mut stream).await?,
            "shuffle" => handle_shuffle(&mut ctx, &request, &mut stream).await?,
            "add" => handle_add(&mut ctx, &request, &mut stream).await?,
            "playlistinfo" => handle_playlistinfo(&mut ctx, &request, &mut stream).await?,
            "delete" => handle_delete(&mut ctx, &request, &mut stream).await?,
            "clear" => handle_clear(&mut ctx, &request, &mut stream).await?,
            "move" => handle_move(&mut ctx, &request, &mut stream).await?,
            "list album" => handle_list_album(&mut ctx, &request, &mut stream).await?,
            "list artist" => handle_list_artist(&mut ctx, &request, &mut stream).await?,
            "list title" => handle_list_title(&mut ctx, &request, &mut stream).await?,
            "update" => handle_rescan(&mut ctx, &request, &mut stream).await?,
            "search" => handle_search(&mut ctx, &request, &mut stream).await?,
            "rescan" => handle_rescan(&mut ctx, &request, &mut stream).await?,
            "status" => handle_status(&mut ctx, &request, &mut stream).await?,
            "currentsong" => handle_currentsong(&mut ctx, &request, &mut stream).await?,
            "config" => handle_config(&mut ctx, &request, &mut stream).await?,
            "tagtypes " => handle_tagtypes(&mut ctx, &request, &mut stream).await?,
            "tagtypes clear" => handle_clear(&mut ctx, &request, &mut stream).await?,
            "stats" => handle_stats(&mut ctx, &request, &mut stream).await?,
            "command_list_begin" => {
                handle_command_list_begin(&mut ctx, &request, &mut stream).await?
            }
            "command_list_ok_begin" => {
                handle_command_list_ok_begin(&mut ctx, &request, &mut stream).await?
            }
            _ => {
                println!("Unhandled command: {}", request);
                stream
                    .write_all(b"ACK [5@0] {unhandled} unknown command\n")
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

    Ok(command.to_string())
}

pub async fn setup_context(batch: bool) -> Result<Context, Error> {
    let port = env::var("ROCKBOX_PORT").unwrap_or_else(|_| "6061".to_string());
    let host = env::var("ROCKBOX_HOST").unwrap_or_else(|_| "localhost".to_string());
    let url = format!("tcp://{}:{}", host, port);

    let library = LibraryServiceClient::connect(url.clone()).await?;
    let playback = PlaybackServiceClient::connect(url.clone()).await?;
    let settings = SettingsServiceClient::connect(url.clone()).await?;
    let sound = SoundServiceClient::connect(url.clone()).await?;
    let playlist = PlaylistServiceClient::connect(url.clone()).await?;

    Ok(Context {
        library,
        playback,
        settings,
        sound,
        playlist,
        single: Arc::new(Mutex::new("\"0\"".to_string())),
        batch,
    })
}
