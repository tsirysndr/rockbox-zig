use anyhow::Error;
use clap::Command;
use owo_colors::OwoColorize;
use rockbox_library::audio_scan::{save_audio_metadata, scan_audio_files};
use rockbox_library::{create_connection_pool, repo};
use rockbox_typesense::client::*;
use rockbox_typesense::types::*;
use std::process::Stdio;
use std::thread::sleep;
use std::time::Duration;
use std::{env, ffi::CStr};
use std::{fs, thread};

#[no_mangle]
pub extern "C" fn parse_args(argc: usize, argv: *const *const u8) -> i32 {
    let string_array = unsafe { std::slice::from_raw_parts(argv, argc) };
    let args: Vec<&str> = string_array
        .iter()
        .map(|&ptr| {
            let c_str = unsafe { CStr::from_ptr(ptr as *const std::ffi::c_char) };
            c_str
                .to_str()
                .unwrap_or("[Invalid UTF-8 or Non Null-Terminated String]")
        })
        .collect();

    const VERSION: &str = match option_env!("TAG") {
        Some(tag) => tag,
        None => env!("CARGO_PKG_VERSION"),
    };

    let banner = format!(
        "{}\nA fork of the original Rockbox project, with a focus on modernization and more features.",
        r#"
              __________               __   ___.
    Open      \______   \ ____   ____ |  | _\_ |__   _______  ___
    Source     |       _//  _ \_/ ___\|  |/ /| __ \ /  _ \  \/  /
    Jukebox    |    |   (  <_> )  \___|    < | \_\ (  <_> > <  <
    Firmware   |____|_  /\____/ \___  >__|_ \|___  /\____/__/\_ \
                      \/            \/     \/    \/            \/
    "#
        .yellow()
    );
    let cli = Command::new("rockboxd").version(VERSION).about(&banner);

    cli.get_matches_from(args);

    thread::spawn(move || {
        let home = env::var("HOME").unwrap();

        match fs::create_dir_all(format!("{}/Music", home)) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Failed to create Music directory: {}", e);
            }
        }

        let update_library = match env::var("ROCKBOX_UPDATE_LIBRARY")
            .as_ref()
            .map(|s| s.as_str())
        {
            Ok("1") => true,
            Ok("true") => true,
            Ok(_) => false,
            Err(_) => false,
        };
        let path = rockbox_settings::get_music_dir().unwrap_or(format!("{}/Music", home));
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let pool = create_connection_pool().await?;
            let tracks = repo::track::all(pool.clone()).await?;
            if tracks.is_empty() || update_library {
                match scan_audio_files(pool.clone(), path.into()).await {
                    Ok(_) => println!("Finished scanning audio files"),
                    Err(e) => eprintln!("Failed to scan audio files: {}", e),
                }
                let tracks = repo::track::all(pool.clone()).await?;
                let albums = repo::album::all(pool.clone()).await?;
                let artists = repo::artist::all(pool.clone()).await?;

                create_tracks_collection().await?;
                create_albums_collection().await?;
                create_artists_collection().await?;

                insert_tracks(tracks.into_iter().map(Track::from).collect()).await?;
                insert_artists(artists.into_iter().map(Artist::from).collect()).await?;
                insert_albums(albums.into_iter().map(Album::from).collect()).await?;
            }
            Ok::<(), Error>(())
        })
        .unwrap();

        thread::spawn(move || {
            sleep(Duration::from_secs(5));
            match rockbox_rocksky::register_rockbox() {
                Ok(_) => println!("Successfully registered Rockbox with Rocksky server"),
                Err(e) => eprintln!("Failed to register Rockbox with Rocksky server: {}", e),
            };
        });

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

        println!(
            "{} server is running on {}",
            "Rockbox TCP".bright_purple(),
            addr.bright_green()
        );

        let graphql_port = env::var("ROCKBOX_GRAPHQL_PORT").unwrap_or("6062".to_string());
        let addr = format!("{}:{}", "0.0.0.0", graphql_port);

        println!(
            "{} server is running on {}",
            "Rockbox GraphQL".bright_purple(),
            addr.bright_green()
        );

        let rockbox_port: u16 = std::env::var("ROCKBOX_PORT")
            .unwrap_or_else(|_| "6061".to_string())
            .parse()
            .expect("ROCKBOX_PORT must be a number");

        let host_and_port = format!("0.0.0.0:{}", rockbox_port);

        println!(
            "{} server is running on {}",
            "Rockbox gRPC".bright_purple(),
            host_and_port.bright_green()
        );

        println!(
            "Rockbox Web UI is running on {} ⚡",
            "http://localhost:6062".bright_green()
        );
    });

    thread::spawn(move || {
        rockbox_typesense::setup()?;

        let api_key = uuid::Uuid::new_v4().to_string();
        let api_key = std::env::var("RB_TYPESENSE_API_KEY").unwrap_or(api_key);
        std::env::set_var("RB_TYPESENSE_API_KEY", &api_key);
        println!("Using Typesense API key: {}", api_key);

        let port = std::env::var("RB_TYPESENSE_PORT").unwrap_or_else(|_| "8109".to_string());
        std::env::set_var("RB_TYPESENSE_PORT", &port);

        let homedir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
        let data_dir = homedir.join(".config/rockbox.org/typesense");

        let path = format!(
            "{}:{}/{}",
            std::env::var("PATH").unwrap_or_default(),
            homedir.display(),
            ".rockbox/bin"
        );

        let ts_bin = {
            let local = homedir.join(".rockbox/bin/typesense-server");
            if local.exists() {
                local
            } else {
                std::path::PathBuf::from("typesense-server")
            }
        };

        let mut cmd = std::process::Command::new(&ts_bin);
        cmd.arg("--enable-cors")
            .arg(format!("--api-port={port}"))
            .env("TYPESENSE_API_KEY", &api_key)
            .env("TYPESENSE_DATA_DIR", &data_dir)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        #[cfg(target_os = "linux")]
        unsafe {
            use std::os::unix::process::CommandExt;
            cmd.pre_exec(|| {
                libc::prctl(
                    libc::PR_SET_PDEATHSIG,
                    libc::SIGTERM as libc::c_ulong,
                    0,
                    0,
                    0,
                );
                Ok(())
            });
        }

        cmd.status()?;

        Ok::<(), Error>(())
    });
    return 0;
}

#[no_mangle]
pub extern "C" fn save_remote_track_metadata(url: *const std::ffi::c_char) -> i32 {
    if url.is_null() {
        eprintln!("save_remote_track_metadata: null url");
        return -1;
    }

    let url = unsafe { CStr::from_ptr(url) };
    let url = match url.to_str() {
        Ok(url) => url,
        Err(e) => {
            eprintln!("save_remote_track_metadata: invalid utf-8: {}", e);
            return -1;
        }
    };

    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!(
                "save_remote_track_metadata: failed to create runtime: {}",
                e
            );
            return -1;
        }
    };

    match rt.block_on(async {
        let pool = create_connection_pool().await?;
        save_audio_metadata(pool, url).await
    }) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("save_remote_track_metadata: {}", e);
            -1
        }
    }
}
