use owo_colors::OwoColorize;
use rockbox_library::repo;
use rockbox_sys::{self as rb, events::RockboxCommand, types::playlist_amount::PlaylistAmount};
use sqlx::Sqlite;
use std::{
    ffi::c_char,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use threadpool::ThreadPool;
use types::NewPlaylist;

pub mod types;

#[no_mangle]
pub extern "C" fn debugfn(args: *const c_char) {
    let c_str = unsafe { std::ffi::CStr::from_ptr(args) };
    let str_slice = c_str.to_str().unwrap();
    println!("{}", str_slice);
}

#[no_mangle]
pub extern "C" fn start_server() {
    const BANNER: &str = r#"
              __________               __   ___.
    Open      \______   \ ____   ____ |  | _\_ |__   _______  ___
    Source     |       _//  _ \_/ ___\|  |/ /| __ \ /  _ \  \/  /
    Jukebox    |    |   (  <_> )  \___|    < | \_\ (  <_> > <  <
    Firmware   |____|_  /\____/ \___  >__|_ \|___  /\____/__/\_ \
                      \/            \/     \/    \/            \/
    "#;

    println!("{}", BANNER.yellow());

    let port = std::env::var("ROCKBOX_TCP_PORT").unwrap_or_else(|_| "6063".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).unwrap();
    listener.set_nonblocking(true).unwrap();

    println!(
        "{} server is running on {}",
        "Rockbox TCP".bright_purple(),
        addr.bright_green()
    );

    let pool = ThreadPool::new(4);
    let active_connections = Arc::new(Mutex::new(0));
    let rt = tokio::runtime::Runtime::new().unwrap();
    let db_pool = rt
        .block_on(rockbox_library::create_connection_pool())
        .unwrap();

    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                let db_pool = db_pool.clone();
                let active_connections = Arc::clone(&active_connections);
                {
                    let mut active_connections = active_connections.lock().unwrap();
                    *active_connections += 1;
                }
                pool.execute(move || {
                    handle_connection(stream, db_pool);
                    {
                        let mut active_connections = active_connections.lock().unwrap();
                        *active_connections -= 1;
                    }
                });
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No incoming connection, just sleep and retry
                rb::system::sleep(rb::HZ);
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
                break;
            }
        }

        // Check if there are no active connections (idle state)
        let active = *active_connections.lock().unwrap();
        if active == 0 {
            rb::system::sleep(rb::HZ);
        }

        // Add a small sleep to avoid tight looping when idle
        thread::sleep(Duration::from_millis(100));
    }
}

fn handle_connection(mut stream: TcpStream, pool: sqlx::Pool<Sqlite>) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let buf_reader = BufReader::new(&mut stream);

    let mut http_request: Vec<String> = Vec::new();
    let mut req_body = String::new();
    let mut content_length = 0;
    let mut should_read_body = false;

    for line in buf_reader.lines() {
        if let Ok(line) = line {
            if line.starts_with("Content-Length") {
                let parts: Vec<_> = line.split(":").collect();
                content_length = parts[1].trim().parse().unwrap();
            }

            if line.is_empty() {
                if content_length == 0 {
                    break;
                }
                should_read_body = true;
            }

            if should_read_body {
                req_body.push_str(format!("{}\n", line).as_str());
                if req_body.len() >= content_length {
                    break;
                }
            }

            http_request.push(line.clone());
        }
    }

    // parse request
    let request = http_request[0].split_whitespace().collect::<Vec<_>>();
    let method = request[0];
    let path = request[1];

    println!("{} {}", method.bright_cyan(), path);

    if method != "GET" && method != "PUT" && method != "POST" {
        let response = "HTTP/1.1 405 Method Not Allowed\r\n\r\n";
        stream.write_all(response.as_bytes()).unwrap();
        return;
    }

    match path {
        "/player/pause" => {
            rb::playback::pause();
        }
        "/player/resume" => {
            rb::playback::resume();
        }
        "/player/next" => {
            rb::playback::next();
        }
        "/player/prev" => {
            rb::playback::prev();
        }
        "/player/stop" => {
            rb::playback::hard_stop();
        }
        "/playlists" => {
            if method == "POST" {
                let new_playslist: NewPlaylist = serde_json::from_str(&req_body).unwrap();
                if new_playslist.tracks.is_empty() {
                    return;
                }
                let dir = new_playslist.tracks[0].clone();
                let dir_parts: Vec<_> = dir.split('/').collect();
                let dir = dir_parts[0..dir_parts.len() - 1].join("/");
                let res = rb::playlist::create(&dir, None);
                if res == -1 {
                    let response = "HTTP/1.1 500 Internal Server Error\r\n\r\n";
                    let response = format!("{}{}", response, "Failed to create playlist");
                    stream.write_all(response.as_bytes()).unwrap();
                    return;
                }
                let start_index = 0;
                let start_index = rb::playlist::build_playlist(
                    new_playslist.tracks.iter().map(|t| t.as_str()).collect(),
                    start_index,
                    new_playslist.tracks.len() as i32,
                );
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{}",
                    start_index
                );
                stream.write_all(response.as_bytes()).unwrap();
            }
            return;
        }
        "/playlists/start" => {
            if method != "PUT" {
                let response = "HTTP/1.1 405 Method Not Allowed\r\n\r\n";
                stream.write_all(response.as_bytes()).unwrap();
                return;
            }
            let mut start_index: i32 = 0;
            let mut elapsed: u64 = 0;
            let mut offset: u64 = 0;

            let params = path.split('?').collect::<Vec<_>>();
            if params.len() > 1 {
                let params = queryst::parse(params[1]).unwrap();
                start_index = params
                    .get("start_index")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .parse()
                    .unwrap_or_default();
                elapsed = params.get("elapsed").unwrap().as_u64().unwrap_or_default();
                offset = params.get("offset").unwrap().as_u64().unwrap_or_default();
            }

            rb::playlist::start(start_index, elapsed, offset);
            stream
                .write_all("HTTP/1.1 200 OK\r\n\r\n".as_bytes())
                .unwrap();
            return;
        }
        "/playlists/shuffle" => {
            if method != "PUT" {
                let response = "HTTP/1.1 405 Method Not Allowed\r\n\r\n";
                stream.write_all(response.as_bytes()).unwrap();
                return;
            }
            let params = path.split('?').collect::<Vec<_>>();
            let params = queryst::parse(params[1]).unwrap();
            let start_index = params
                .get("start_index")
                .unwrap()
                .as_i64()
                .unwrap_or_default();
            let seed = rb::system::current_tick();
            let ret = rb::playlist::shuffle(seed as i32, start_index as i32);
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{}",
                ret
            );
            stream.write_all(response.as_bytes()).unwrap();
        }
        "/playlists/amount" => {
            if method != "GET" {
                let response = "HTTP/1.1 405 Method Not Allowed\r\n\r\n";
                stream.write_all(response.as_bytes()).unwrap();
                return;
            }
            let amount = rb::playlist::amount();
            let json = PlaylistAmount { amount };
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{}",
                serde_json::to_string(&json).unwrap()
            );
            stream.write_all(response.as_bytes()).unwrap();
            return;
        }
        "/playlists/current" => {
            if method != "GET" {
                let response = "HTTP/1.1 405 Method Not Allowed\r\n\r\n";
                stream.write_all(response.as_bytes()).unwrap();
                return;
            }
            let mut playlist = rb::playlist::get_current();
            let mut entries = vec![];
            let amount = rb::playlist::amount();

            for i in 0..amount {
                let info = rb::playlist::get_track_info(i);
                let entry = rb::metadata::get_metadata(-1, &info.filename);
                entries.push(entry);
            }

            playlist.entries = entries;

            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{}",
                serde_json::to_string(&playlist).unwrap()
            );
            stream.write_all(response.as_bytes()).unwrap();
            return;
        }
        "/playlists/resume" => {
            if method != "PUT" {
                let response = "HTTP/1.1 405 Method Not Allowed\r\n\r\n";
                stream.write_all(response.as_bytes()).unwrap();
                return;
            }
            rb::playlist::resume();
        }
        "/playlists/resume-track" => {
            if method != "PUT" {
                let response = "HTTP/1.1 405 Method Not Allowed\r\n\r\n";
                stream.write_all(response.as_bytes()).unwrap();
                return;
            }
            let status = rb::system::get_global_status();
            rb::playlist::resume_track(
                status.resume_index,
                status.resume_crc32,
                status.resume_elapsed.into(),
                status.resume_offset.into(),
            );
        }
        "/version" => {
            if method != "GET" {
                let response = "HTTP/1.1 405 Method Not Allowed\r\n\r\n";
                stream.write_all(response.as_bytes()).unwrap();
                return;
            }
            let version = rb::system::get_rockbox_version();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{}",
                serde_json::to_string(&version).unwrap()
            );
            stream.write_all(response.as_bytes()).unwrap();
            return;
        }
        "/status" => {
            let status = rb::system::get_global_status();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{}",
                serde_json::to_string(&status).unwrap()
            );
            stream.write_all(response.as_bytes()).unwrap();
            return;
        }
        "/settings" => {
            let settings = rb::settings::get_global_settings();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{}",
                serde_json::to_string(&settings).unwrap()
            );
            stream.write_all(response.as_bytes()).unwrap();
            return;
        }
        "/player/flush-and-reload-tracks" => {
            rb::playback::flush_and_reload_tracks();
            return;
        }
        "/player/next-track" => {
            let track = rb::playback::next_track();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{}",
                serde_json::to_string(&track).unwrap()
            );
            stream.write_all(response.as_bytes()).unwrap();
            return;
        }
        "/player/current-track" => {
            let track = rb::playback::current_track();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{}",
                serde_json::to_string(&track).unwrap()
            );
            stream.write_all(response.as_bytes()).unwrap();
            return;
        }
        "/audio_status" => {
            let status = rb::playback::status();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{}",
                serde_json::to_string(&status).unwrap()
            );
            stream.write_all(response.as_bytes()).unwrap();
            return;
        }
        "/player/file-position" => {
            let position = rb::playback::get_file_pos();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{}",
                serde_json::to_string(&position).unwrap()
            );
            stream.write_all(response.as_bytes()).unwrap();
            return;
        }
        "/tree_context" => {
            let context = rb::browse::tree_get_context();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{}",
                serde_json::to_string(&context).unwrap()
            );
            stream.write_all(response.as_bytes()).unwrap();
            return;
        }
        "/albums" => {
            let albums = rt.block_on(repo::album::all(pool)).unwrap();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{}",
                serde_json::to_string(&albums).unwrap()
            );
            stream.write_all(response.as_bytes()).unwrap();
            return;
        }
        "/artists" => {
            let artists = rt.block_on(repo::artist::all(pool)).unwrap();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{}",
                serde_json::to_string(&artists).unwrap()
            );
            stream.write_all(response.as_bytes()).unwrap();
            return;
        }
        "/tracks" => {
            let tracks = rt.block_on(repo::track::all(pool)).unwrap();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{}",
                serde_json::to_string(&tracks).unwrap()
            );
            stream.write_all(response.as_bytes()).unwrap();
            return;
        }
        "/openapi.json" => {
            let spec = include_str!("../openapi.json");
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{}",
                spec
            );
            stream.write_all(response.as_bytes()).unwrap();
            return;
        }
        "/" => {
            let index = include_str!("../docs/index.html");
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n{}",
                index
            );
            stream.write_all(response.as_bytes()).unwrap();
        }
        _ => {
            if path.starts_with("/operations/") || path.starts_with("/schemas/") {
                let index = include_str!("../docs/index.html");
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n{}",
                    index
                );
                stream.write_all(response.as_bytes()).unwrap();
                return;
            }

            if path.starts_with("/artists/") && path.ends_with("/tracks") {
                todo!("to be implemented");
            }

            if path.starts_with("/albums/") && path.ends_with("/tracks") {
                todo!("to be implemented");
            }

            if path.starts_with("/player/play?") {
                let params: Vec<_> = path.split('?').collect();
                let params = queryst::parse(params[1]).unwrap();
                let elapsed = params.get("elapsed").unwrap().as_i64().unwrap();
                let offset = params.get("offset").unwrap().as_i64().unwrap();
                rb::playback::play(elapsed, offset);
                let response = "HTTP/1.1 200 OK\r\n\r\n";
                stream.write_all(response.as_bytes()).unwrap();
                return;
            }

            if path.starts_with("/player/ff_rewind") {
                let params: Vec<_> = path.split('?').collect();
                let params = queryst::parse(params[1]).unwrap();
                let newtime = params.get("newtime").unwrap().as_str().unwrap();
                let newtime = newtime.parse().unwrap();
                rb::playback::ff_rewind(newtime);
                let response = "HTTP/1.1 200 OK\r\n\r\n";
                stream.write_all(response.as_bytes()).unwrap();
                return;
            }

            if path.starts_with("/browse/tree-entries?") {
                let params: Vec<_> = path.split('?').collect();
                let params = queryst::parse(params[1]).unwrap_or_default();
                let path = params.get("q").unwrap().as_str().unwrap();

                if let Err(e) = rb::browse::rockbox_browse_at(path) {
                    if e.to_string().starts_with("No such file or directory") {
                        let response = "HTTP/1.1 404 Not Found\r\n\r\n";
                        stream.write_all(response.as_bytes()).unwrap();
                        return;
                    }
                    let response = format!("HTTP/1.1 500 Internal Server Error\r\n\r\n{}", e);
                    stream.write_all(response.as_bytes()).unwrap();
                    return;
                }

                let mut entries = vec![];
                let context = rb::browse::tree_get_context();

                for i in 0..context.filesindir {
                    let entry = rb::browse::tree_get_entry_at(i);
                    entries.push(entry);
                }

                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{}",
                    serde_json::to_string(&entries).unwrap()
                );
                stream.write_all(response.as_bytes()).unwrap();
                return;
            }

            if path.starts_with("/albums/") {
                let album_id = path.split('/').collect::<Vec<_>>()[2];
                let album = rt.block_on(repo::album::find(pool, album_id)).unwrap();

                if album.is_none() {
                    let response = "HTTP/1.1 404 Not Found\r\n\r\n";
                    stream.write_all(response.as_bytes()).unwrap();
                    return;
                }

                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{}",
                    serde_json::to_string(&album).unwrap()
                );
                stream.write_all(response.as_bytes()).unwrap();
                return;
            }

            if path.starts_with("/artists/") {
                let artist_id = path.split('/').collect::<Vec<_>>()[2];
                let artist = rt.block_on(repo::artist::find(pool, artist_id)).unwrap();

                if artist.is_none() {
                    let response = "HTTP/1.1 404 Not Found\r\n\r\n";
                    stream.write_all(response.as_bytes()).unwrap();
                    return;
                }

                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{}",
                    serde_json::to_string(&artist).unwrap()
                );
                stream.write_all(response.as_bytes()).unwrap();
                return;
            }

            if path.starts_with("/tracks/") {
                let track_id = path.split('/').collect::<Vec<_>>()[2];
                let track = rt.block_on(repo::track::find(pool, track_id)).unwrap();

                if track.is_none() {
                    let response = "HTTP/1.1 404 Not Found\r\n\r\n";
                    stream.write_all(response.as_bytes()).unwrap();
                    return;
                }

                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{}",
                    serde_json::to_string(&track).unwrap()
                );
                stream.write_all(response.as_bytes()).unwrap();
                return;
            }

            let response = "HTTP/1.1 404 Not Found\r\n\r\n";
            stream.write_all(response.as_bytes()).unwrap();
            return;
        }
    }

    let response = "HTTP/1.1 200 OK\r\n\r\n";

    stream.write_all(response.as_bytes()).unwrap();
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
