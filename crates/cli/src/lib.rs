use anyhow::Error;
use clap::Command;
use owo_colors::OwoColorize;
use rockbox_library::audio_scan::scan_audio_files;
use rockbox_library::{create_connection_pool, repo};
use rockbox_search::album::Album;
use rockbox_search::artist::Artist;
use rockbox_search::track::Track;
use rockbox_search::{create_indexes, delete_all_documents, index_entity};
use std::{env, ffi::CStr};
use std::{fs, thread};

#[macro_export]
macro_rules! cast_ptr {
    ($ptr:expr) => {{
        #[cfg(target_arch = "aarch64")]
        {
            $ptr as *const u8
        }

        #[cfg(not(target_arch = "aarch64"))]
        {
            $ptr as *const i8
        }
    }};
}

#[no_mangle]
pub extern "C" fn parse_args(argc: usize, argv: *const *const u8) -> i32 {
    let string_array = unsafe { std::slice::from_raw_parts(argv, argc) };
    let args: Vec<&str> = string_array
        .iter()
        .map(|&ptr| {
            let c_str = unsafe { CStr::from_ptr(cast_ptr!(ptr)) };
            c_str
                .to_str()
                .unwrap_or("[Invalid UTF-8 or Non Null-Terminated String]")
        })
        .collect();

    const VERSION: &str = env!("CARGO_PKG_VERSION");
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
    let cli = Command::new("rockbox").version(VERSION).about(&banner);

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
        let path = env::var("ROCKBOX_LIBRARY").unwrap_or(format!("{}/Music", home));
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let pool = create_connection_pool().await?;
            let tracks = repo::track::all(pool.clone()).await?;
            if tracks.is_empty() || update_library {
                scan_audio_files(pool.clone(), path.into()).await?;
                let tracks = repo::track::all(pool.clone()).await?;
                let albums = repo::album::all(pool.clone()).await?;
                let artists = repo::artist::all(pool.clone()).await?;
                let indexes = create_indexes()?;
                let tracks_index = indexes.tracks.clone();
                let albums_index = indexes.albums.clone();
                let artists_index = indexes.artists.clone();

                thread::spawn(move || {
                    match delete_all_documents(&tracks_index) {
                        Ok(_) => {}
                        Err(e) => eprintln!("Error deleting all documents: {:?}", e),
                    }
                    for track in tracks {
                        index_entity::<Track>(&tracks_index, &track.into()).unwrap();
                    }
                });

                thread::spawn(move || {
                    match delete_all_documents(&albums_index) {
                        Ok(_) => {}
                        Err(e) => eprintln!("Error deleting all documents: {:?}", e),
                    }
                    for album in albums {
                        index_entity::<Album>(&albums_index, &album.into()).unwrap();
                    }
                });

                thread::spawn(move || {
                    match delete_all_documents(&artists_index) {
                        Ok(_) => {}
                        Err(e) => eprintln!("Error deleting all documents: {:?}", e),
                    }
                    for artist in artists {
                        index_entity::<Artist>(&artists_index, &artist.into()).unwrap();
                    }
                });
            }
            Ok::<(), Error>(())
        })
        .unwrap();

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

    return 0;
}
