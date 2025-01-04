use std::net::TcpStream;
use std::time::Duration;
use std::{env, process::Command};

use anyhow::Error;
use rockbox::{install_rockboxd, wait_for_rockboxd};

pub fn start() -> Result<(), Error> {
    let video_driver = std::env::var("SDL_VIDEODRIVER").unwrap_or_else(|_| "dummy".to_string());

    let port = env::var("ROCKBOX_PORT").unwrap_or_else(|_| "6061".to_string());
    let ui_port = env::var("ROCKBOX_UI_PORT").unwrap_or_else(|_| "6062".to_string());
    let http_port = env::var("ROCKBOX_HTTP_PORT").unwrap_or_else(|_| "6063".to_string());

    // try to connect to http_port to see if ther server is already running
    let addr = format!("127.0.0.1:{}", http_port);
    match TcpStream::connect_timeout(&addr.parse()?, Duration::from_secs(5)) {
        Ok(_) => {
            rmpc::main_tui()?;
            return Ok(());
        }
        Err(_) => {}
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
