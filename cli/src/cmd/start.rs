use std::process::Command;

use anyhow::Error;

pub fn start() -> Result<(), Error> {
    let video_driver = std::env::var("SDL_VIDEODRIVER").unwrap_or_else(|_| "dummy".to_string());

    let mut child = Command::new("rockboxd")
        .env("SDL_VIDEODRIVER", video_driver)
        .spawn()?;

    child.wait()?;

    Ok(())
}
