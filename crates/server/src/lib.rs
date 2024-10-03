use handlers::*;

use http::RockboxHttpServer;
use rockbox_sys::events::RockboxCommand;
use std::{
    ffi::c_char,
    sync::{Arc, Mutex},
    thread,
};

pub mod handlers;
pub mod http;
pub mod types;

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

    app.post("/playlists", create_playlist);
    app.put("/playlists/start", start_playlist);
    app.put("/playlists/shuffle", shuffle_playlist);
    app.get("/playlists/amount", get_playlist_amount);
    app.put("/playlists/resume", resume_playlist);
    app.put("/playlists/resume-track", resume_track);
    app.get("/playlists/:id/tracks", get_playlist_tracks);
    app.post("/playlists/:id/tracks", insert_tracks);
    app.delete("/playlists/:id/tracks", remove_tracks);

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
                    client.put(&format!("{}/player/prev", url)).send().unwrap();
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
