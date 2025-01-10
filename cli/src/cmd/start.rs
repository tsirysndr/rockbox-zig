use std::{env, process::Command};

use anyhow::Error;
use rockbox::{install_rockboxd, wait_for_rockboxd};

pub fn start(with_ui: bool) -> Result<(), Error> {
    let video_driver = std::env::var("SDL_VIDEODRIVER").unwrap_or_else(|_| "dummy".to_string());

    let port = env::var("ROCKBOX_PORT").unwrap_or_else(|_| "6061".to_string());
    let ui_port = env::var("ROCKBOX_UI_PORT").unwrap_or_else(|_| "6062".to_string());
    let http_port = env::var("ROCKBOX_HTTP_PORT").unwrap_or_else(|_| "6063".to_string());
    let mpd_port = env::var("MPD_PORT").unwrap_or("6600".to_string());

    // try to connect to mpd_port to see if mpd server is already running
    if wait_for_rockboxd(mpd_port.parse()?, Some(1)).is_ok() {
        if with_ui {
            rmpc::main_tui()?;
        }
        return Ok(());
    }

    install_rockboxd()?;

    match wait_for_rockboxd(port.parse()?, Some(1)) {
        Ok(_) => {}
        Err(_) => {
            let mut child = Command::new("rockboxd")
                .env("SDL_VIDEODRIVER", video_driver)
                .env("ROCKBOX_PORT", port)
                .env("ROCKBOX_GRAPHQL_PORT", ui_port)
                .env("ROCKBOX_TCP_PORT", http_port)
                .spawn()?;

            child.wait()?;
        }
    };
    Ok(())
}
