use std::{env, process::Command, thread};

use anyhow::Error;
use rockbox::{install_rockboxd, wait_for_rockboxd};

pub fn start() -> Result<(), Error> {
    install_rockboxd()?;
    let video_driver = std::env::var("SDL_VIDEODRIVER").unwrap_or_else(|_| "dummy".to_string());

    let port = env::var("ROCKBOX_PORT").unwrap_or_else(|_| "6061".to_string());
    let ui_port = env::var("ROCKBOX_UI_PORT").unwrap_or_else(|_| "6062".to_string());
    let http_port = env::var("ROCKBOX_HTTP_PORT").unwrap_or_else(|_| "6063".to_string());

    match wait_for_rockboxd(port.parse()?, Some(1)) {
        Ok(_) => {}
        Err(_) => {
            thread::spawn(move || {
                let mut child = Command::new("rockboxd")
                    .env("SDL_VIDEODRIVER", video_driver)
                    .env("ROCKBOX_PORT", port)
                    .env("ROCKBOX_GRAPHQL_PORT", ui_port)
                    .env("ROCKBOX_TCP_PORT", http_port)
                    .env("SDL_AUDIODRIVER", "dummy")
                    .spawn()?;

                child.wait()?;
                Ok::<(), Error>(())
            });

            thread::sleep(std::time::Duration::from_secs(5));

            match rockbox_audio::read_audio_socket() {
                Ok(_) => {}
                Err(e) => eprintln!("Error reading audio socket: {}", e),
            }
        }
    };
    Ok(())
}
