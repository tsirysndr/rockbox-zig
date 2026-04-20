use anyhow::Error;
use clap::Command;
use owo_colors::OwoColorize;
use rockbox_library::audio_scan::{save_audio_metadata, scan_audio_files};
use rockbox_library::{create_connection_pool, repo};
use rockbox_typesense::client::*;
use rockbox_typesense::types::*;
use std::io::{BufRead, BufReader};
use std::process::Stdio;
use std::sync::atomic::{AtomicI32, Ordering};
use std::thread::sleep;
use std::time::Duration;
use std::{env, ffi::CStr};
use std::{fs, thread};
use tracing::{error, info, warn};

/// PID of the spawned typesense-server child, or -1 if not yet started.
static TYPESENSE_PID: AtomicI32 = AtomicI32::new(-1);

/// SIGTERM/SIGINT handler: kill the typesense child then _exit immediately.
///
/// system-hosted.c installs a SIGTERM handler that calls system_exception_wait()
/// which loops forever waiting for an SDL quit event that never arrives on a
/// headless daemon.  We override it here with a handler that actually exits.
/// _exit is used because it is async-signal-safe (exit() is not).
#[cfg(unix)]
extern "C" fn handle_shutdown(_sig: libc::c_int) {
    let pid = TYPESENSE_PID.load(Ordering::SeqCst);
    if pid > 0 {
        unsafe { libc::kill(pid, libc::SIGTERM) };
    }
    unsafe { libc::_exit(0) };
}

#[no_mangle]
pub extern "C" fn parse_args(argc: usize, argv: *const *const u8) -> i32 {
    let subscriber = tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

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

    // Install shutdown handler before spawning typesense-server so the PID is
    // always available when the handler fires.
    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGTERM, handle_shutdown as libc::sighandler_t);
        libc::signal(libc::SIGINT, handle_shutdown as libc::sighandler_t);
    }

    // SDL (initialised after parse_args returns) installs its own SIGTERM/SIGINT
    // handlers and overwrites ours.  Reinstall after a short delay so our
    // handler — which kills typesense-server and _exit()s — wins.
    #[cfg(unix)]
    thread::spawn(|| {
        sleep(Duration::from_secs(3));
        unsafe {
            libc::signal(libc::SIGTERM, handle_shutdown as libc::sighandler_t);
            libc::signal(libc::SIGINT, handle_shutdown as libc::sighandler_t);
        }
    });

    thread::spawn(move || {
        let home = env::var("HOME").unwrap();

        match fs::create_dir_all(format!("{}/Music", home)) {
            Ok(_) => {}
            Err(e) => {
                error!("Failed to create Music directory: {}", e);
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
                    Ok(_) => info!("Finished scanning audio files"),
                    Err(e) => error!("Failed to scan audio files: {}", e),
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
                Ok(_) => info!("Successfully registered Rockbox with Rocksky server"),
                Err(e) => error!("Failed to register Rockbox with Rocksky server: {}", e),
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

        info!("Rockbox TCP server is running on {}", addr);

        let graphql_port = env::var("ROCKBOX_GRAPHQL_PORT").unwrap_or("6062".to_string());
        let addr = format!("{}:{}", "0.0.0.0", graphql_port);

        info!("Rockbox GraphQL server is running on {}", addr);

        let rockbox_port: u16 = std::env::var("ROCKBOX_PORT")
            .unwrap_or_else(|_| "6061".to_string())
            .parse()
            .expect("ROCKBOX_PORT must be a number");

        let host_and_port = format!("0.0.0.0:{}", rockbox_port);

        info!("Rockbox gRPC server is running on {}", host_and_port);

        info!("Rockbox Web UI is running on http://localhost:6062");
    });

    thread::spawn(move || {
        rockbox_typesense::setup()?;

        let api_key = uuid::Uuid::new_v4().to_string();
        let api_key = std::env::var("RB_TYPESENSE_API_KEY").unwrap_or(api_key);
        std::env::set_var("RB_TYPESENSE_API_KEY", &api_key);
        info!("Using Typesense API key: {}", api_key);

        let port = std::env::var("RB_TYPESENSE_PORT").unwrap_or_else(|_| "8109".to_string());
        std::env::set_var("RB_TYPESENSE_PORT", &port);

        let homedir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
        let data_dir = homedir.join(".config/rockbox.org/typesense");

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
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

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

        let mut child = cmd.spawn()?;
        TYPESENSE_PID.store(child.id() as i32, Ordering::SeqCst);

        if let Some(stdout) = child.stdout.take() {
            thread::spawn(move || {
                for line in BufReader::new(stdout).lines().flatten() {
                    tracing::debug!(target: "typesense", "{}", line);
                }
            });
        }
        if let Some(stderr) = child.stderr.take() {
            thread::spawn(move || {
                for line in BufReader::new(stderr).lines().flatten() {
                    tracing::warn!(target: "typesense", "{}", line);
                }
            });
        }

        // Poll instead of blocking in waitpid so SIGTERM can reach the process.
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    warn!("typesense-server exited: {status}");
                    break;
                }
                Ok(None) => sleep(Duration::from_millis(500)),
                Err(e) => {
                    error!("typesense-server monitor error: {e}");
                    break;
                }
            }
        }

        Ok::<(), Error>(())
    });
    return 0;
}

#[no_mangle]
pub extern "C" fn save_remote_track_metadata(url: *const std::ffi::c_char) -> i32 {
    if url.is_null() {
        warn!("save_remote_track_metadata: null url");
        return -1;
    }

    let url = unsafe { CStr::from_ptr(url) };
    let url = match url.to_str() {
        Ok(url) => url,
        Err(e) => {
            warn!("save_remote_track_metadata: invalid utf-8: {}", e);
            return -1;
        }
    };

    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            error!(
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
            error!("save_remote_track_metadata: {}", e);
            -1
        }
    }
}
