use std::{env, thread};

use anyhow::Error;
use opener::open;
use owo_colors::OwoColorize;
use rockbox::{install_rockboxd, wait_for_rockboxd};

use super::start::*;

pub fn webui() -> Result<(), Error> {
    let port = env::var("ROCKBOX_UI_PORT").unwrap_or_else(|_| "6062".to_string());
    install_rockboxd()?;

    let handle = thread::spawn(|| match start() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Failed to start Rockbox server: {}", e);
        }
    });

    wait_for_rockboxd(port.parse()?, None)?;
    match open(format!("http://localhost:{}", port)) {
        Ok(_) => {}
        Err(_) => println!(
            "Open this link in your browser {}",
            format!("http://localhost:{}", port).purple()
        ),
    };
    handle.join().unwrap();
    Ok(())
}
