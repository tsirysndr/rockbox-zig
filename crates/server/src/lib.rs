use owo_colors::OwoColorize;
use rockbox_sys::{self as rb, events::RockboxCommand, types::playlist_amount::PlaylistAmount};
use std::{
    ffi::c_char,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

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
    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr).unwrap();
    listener.set_nonblocking(true).unwrap();

    println!(
        "{} server is running on {}",
        "Rockbox TCP".bright_purple(),
        addr.bright_green()
    );

    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                handle_connection(stream);
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
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    // parse request
    let request = http_request[0].split_whitespace().collect::<Vec<_>>();
    let method = request[0];
    let path = request[1];

    println!("{} {}", method.bright_cyan(), path);

    if method != "GET" {
        let response = "HTTP/1.1 405 Method Not Allowed\r\n\r\n";
        stream.write_all(response.as_bytes()).unwrap();
        return;
    }

    match path {
        "/pause" => {
            rb::playback::pause();
        }
        "/resume" => {
            rb::playback::resume();
        }
        "/next" => {
            rb::playback::next();
        }
        "/prev" => {
            rb::playback::prev();
        }
        "/stop" => {
            rb::playback::hard_stop();
        }
        "/playlist_amount" => {
            let amount = rb::playlist::amount();
            let json = PlaylistAmount { amount };
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{}",
                serde_json::to_string(&json).unwrap()
            );
            stream.write_all(response.as_bytes()).unwrap();
            return;
        }
        "/current_playlist" => {
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
        "/playlist_resume" => {
            rb::playlist::resume();
        }
        "/playlist_resume_track" => {
            let status = rb::system::get_global_status();
            rb::playlist::resume_track(
                status.resume_index,
                status.resume_crc32,
                status.resume_elapsed.into(),
                status.resume_offset.into(),
            );
        }
        "/version" => {
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
        "/flush_and_reload_tracks" => {
            rb::playback::flush_and_reload_tracks();
            return;
        }
        "/next_track" => {
            let track = rb::playback::next_track();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{}",
                serde_json::to_string(&track).unwrap()
            );
            stream.write_all(response.as_bytes()).unwrap();
            return;
        }
        "/current_track" => {
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
        "/file_position" => {
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
        _ => {
            if path.starts_with("/play?") {
                let params: Vec<_> = path.split('?').collect();
                let params: Vec<_> = params[1].split('&').collect();
                let elapsed = params[0].split('=').collect::<Vec<_>>()[1].parse().unwrap();
                let offset = params[1].split('=').collect::<Vec<_>>()[1].parse().unwrap();
                rb::playback::play(elapsed, offset);
                let response = "HTTP/1.1 200 OK\r\n\r\n";
                stream.write_all(response.as_bytes()).unwrap();
                return;
            }

            if path.starts_with("/ff_rewind") {
                let params: Vec<_> = path.split('?').collect();
                let newtime = params[1].split('=').collect::<Vec<_>>()[1].parse().unwrap();
                rb::playback::ff_rewind(newtime);
                let response = "HTTP/1.1 200 OK\r\n\r\n";
                stream.write_all(response.as_bytes()).unwrap();
                return;
            }

            if path.starts_with("/tree_entries?") {
                let params: Vec<_> = path.split('?').collect();
                let params = queryst::parse(params[1]).unwrap_or_default();
                println!("{}", params);
                rb::browse::rockbox_browse_at(params["q"].as_str().unwrap_or("/"));
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

        while let Ok(event) = cmd_rx.recv() {
            match event {
                RockboxCommand::Play(elapsed, offset) => {
                    reqwest::blocking::get(&format!(
                        "{}/play?elapsed={}&offset={}",
                        url, elapsed, offset
                    ))
                    .unwrap();
                }
                RockboxCommand::Pause => {
                    reqwest::blocking::get(&format!("{}/pause", url)).unwrap();
                }
                RockboxCommand::Resume => {
                    reqwest::blocking::get(&format!("{}/resume", url)).unwrap();
                }
                RockboxCommand::Next => {
                    reqwest::blocking::get(&format!("{}/next", url)).unwrap();
                }
                RockboxCommand::Prev => {
                    reqwest::blocking::get(&format!("{}/prev", url)).unwrap();
                }
                RockboxCommand::FfRewind(newtime) => {
                    reqwest::blocking::get(&format!("{}/ff_rewind?newtime={}", url, newtime))
                        .unwrap();
                }
                RockboxCommand::FlushAndReloadTracks => {
                    reqwest::blocking::get(&format!("{}/flush_and_reload_tracks", url)).unwrap();
                }
                RockboxCommand::Stop => {
                    reqwest::blocking::get(&format!("{}/stop", url)).unwrap();
                }
                RockboxCommand::PlaylistResume => {
                    reqwest::blocking::get(&format!("{}/playlist_resume", url)).unwrap();
                }
                RockboxCommand::PlaylistResumeTrack => {
                    reqwest::blocking::get(&format!("{}/playlist_resume_track", url)).unwrap();
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
