use handlers::*;

use http::RockboxHttpServer;
use rockbox_graphql::{
    schema::objects::{self, audio_status::AudioStatus, track::Track},
    simplebroker::SimpleBroker,
};
use rockbox_library::repo;
use rockbox_sys as rb;
use rockbox_sys::events::RockboxCommand;
use std::{
    ffi::c_char,
    sync::{Arc, Mutex},
    thread,
};

pub mod cache;
pub mod handlers;
pub mod http;
pub mod types;

pub const AUDIO_EXTENSIONS: [&str; 17] = [
    "mp3", "ogg", "flac", "m4a", "aac", "mp4", "alac", "wav", "wv", "mpc", "aiff", "ac3", "opus",
    "spx", "sid", "ape", "wma",
];

#[no_mangle]
pub extern "C" fn debugfn(args: *const c_char) {
    let c_str = unsafe { std::ffi::CStr::from_ptr(args) };
    let str_slice = c_str.to_str().unwrap();
    println!("{}", str_slice);
}

#[no_mangle]
pub extern "C" fn start_server() {
    let mut app = RockboxHttpServer::new();

    app.get("/albums", get_albums);
    app.get("/albums/:id", get_album);
    app.get("/albums/:id/tracks", get_album_tracks);

    app.get("/artists", get_artists);
    app.get("/artists/:id", get_artist);
    app.get("/artists/:id/albums", get_artist_albums);
    app.get("/artists/:id/tracks", get_artist_tracks);

    app.get("/browse/tree-entries", get_tree_entries);

    app.put("/player/play", play);
    app.put("/player/pause", pause);
    app.put("/player/resume", resume);
    app.put("/player/ff-rewind", ff_rewind);
    app.get("/player/status", status);
    app.get("/player/current-track", current_track);
    app.get("/player/next-track", next_track);
    app.put("/player/flush-and-reload-tracks", flush_and_reload_tracks);
    app.put("/player/next", next);
    app.put("/player/previous", previous);
    app.put("/player/stop", stop);
    app.get("/player/file-position", get_file_position);

    app.post("/playlists", create_playlist);
    app.put("/playlists/start", start_playlist);
    app.put("/playlists/shuffle", shuffle_playlist);
    app.get("/playlists/amount", get_playlist_amount);
    app.put("/playlists/resume", resume_playlist);
    app.put("/playlists/resume-track", resume_track);
    app.get("/playlists/:id/tracks", get_playlist_tracks);
    app.post("/playlists/:id/tracks", insert_tracks);
    app.delete("/playlists/:id/tracks", remove_tracks);
    app.get("/playlists/:id", get_playlist);

    app.get("/tracks", get_tracks);
    app.get("/tracks/:id", get_track);

    app.get("/version", get_rockbox_version);
    app.get("/status", get_status);
    app.get("/settings", get_global_settings);

    app.get("/", index);
    app.get("/operations/:id", index);
    app.get("/schemas/:id", index);
    app.get("/openapi.json", get_openapi);

    match app.listen() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error starting server: {}", e);
        }
    }
}

#[no_mangle]
pub extern "C" fn start_servers() {
    let (cmd_tx, cmd_rx) = std::sync::mpsc::channel::<RockboxCommand>();
    let cmd_tx = Arc::new(Mutex::new(cmd_tx));

    thread::spawn(move || {
        let port = std::env::var("ROCKBOX_TCP_PORT").unwrap_or_else(|_| "6063".to_string());
        let url = format!("http://127.0.0.1:{}", port);
        let client = reqwest::blocking::Client::new();

        while let Ok(event) = cmd_rx.recv() {
            match event {
                RockboxCommand::Play(elapsed, offset) => {
                    client
                        .put(&format!(
                            "{}/player/play?elapsed={}&offset={}",
                            url, elapsed, offset
                        ))
                        .send()
                        .unwrap();
                }
                RockboxCommand::Pause => {
                    client.put(&format!("{}/player/pause", url)).send().unwrap();
                }
                RockboxCommand::Resume => {
                    client
                        .put(&format!("{}/player/resume", url))
                        .send()
                        .unwrap();
                }
                RockboxCommand::Next => {
                    client.put(&format!("{}/player/next", url)).send().unwrap();
                }
                RockboxCommand::Prev => {
                    client
                        .put(&format!("{}/player/previous", url))
                        .send()
                        .unwrap();
                }
                RockboxCommand::FfRewind(newtime) => {
                    client
                        .put(&format!("{}/player/ff-rewind?newtime={}", url, newtime))
                        .send()
                        .unwrap();
                }
                RockboxCommand::FlushAndReloadTracks => {
                    client
                        .put(&format!("{}/player/flush-and-reload-tracks", url))
                        .send()
                        .unwrap();
                }
                RockboxCommand::Stop => {
                    client.put(&format!("{}/player/stop", url)).send().unwrap();
                }
                RockboxCommand::PlaylistResume => {
                    client
                        .put(&format!("{}/playlists/resume", url))
                        .send()
                        .unwrap();
                }
                RockboxCommand::PlaylistResumeTrack => {
                    client
                        .put(&format!("{}/playlists/resume-track", url))
                        .send()
                        .unwrap();
                }
            }
        }
    });

    let cloned_cmd_tx = cmd_tx.clone();

    thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        match runtime.block_on(rockbox_rpc::server::start(cmd_tx.clone())) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error starting server: {}", e);
            }
        }
    });

    thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        match runtime.block_on(rockbox_graphql::server::start(cloned_cmd_tx.clone())) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error starting server: {}", e);
            }
        }
    });
}

#[no_mangle]
pub extern "C" fn start_broker() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let pool = rt
        .block_on(rockbox_library::create_connection_pool())
        .unwrap();
    loop {
        let playback_status: AudioStatus = rb::playback::status().into();
        SimpleBroker::publish(playback_status);
        match rb::playback::current_track() {
            Some(current_track) => {
                let hash = format!("{:x}", md5::compute(current_track.path.as_bytes()));
                if let Ok(Some(metadata)) =
                    rt.block_on(repo::track::find_by_md5(pool.clone(), &hash))
                {
                    let mut track: Track = current_track.into();
                    track.id = Some(metadata.id);
                    track.album_art = metadata.album_art;
                    track.album_id = Some(metadata.album_id);
                    track.artist_id = Some(metadata.artist_id);
                    SimpleBroker::publish(track);
                }
            }
            None => {}
        };

        let mut current_playlist = rb::playlist::get_current();
        let amount = rb::playlist::amount();

        let mut entries = vec![];

        for i in 0..amount {
            let info = rb::playlist::get_track_info(i);
            let entry = rb::metadata::get_metadata(-1, &info.filename);
            entries.push(entry);
        }

        current_playlist.amount = amount;
        current_playlist.max_playlist_size = rb::playlist::max_playlist_size();
        current_playlist.index = rb::playlist::index();
        current_playlist.first_index = rb::playlist::first_index();
        current_playlist.last_insert_pos = rb::playlist::last_insert_pos();
        current_playlist.seed = rb::playlist::seed();
        current_playlist.last_shuffled_start = rb::playlist::last_shuffled_start();
        current_playlist.entries = entries;

        SimpleBroker::publish(objects::playlist::Playlist {
            amount: current_playlist.amount,
            index: current_playlist.index,
            max_playlist_size: current_playlist.max_playlist_size,
            first_index: current_playlist.first_index,
            last_insert_pos: current_playlist.last_insert_pos,
            seed: current_playlist.seed,
            last_shuffled_start: current_playlist.last_shuffled_start,
            tracks: current_playlist
                .entries
                .into_iter()
                .map(|t| t.into())
                .collect(),
        });

        thread::sleep(std::time::Duration::from_millis(100));
        rb::system::sleep(rb::HZ);
    }
}
