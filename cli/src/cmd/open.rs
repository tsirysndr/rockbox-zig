use std::{env, path::Path, thread};

use anyhow::{anyhow, Error};
use rockbox::{
    api::rockbox::v1alpha1::{playback_service_client::PlaybackServiceClient, PlayTrackRequest},
    install_rockboxd, wait_for_rockboxd,
};

use super::start::start;

pub async fn open(path_or_url: &str) -> Result<(), Error> {
    install_rockboxd()?;

    let handle = thread::spawn(|| match start(false) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Failed to start Rockbox server: {}", e);
        }
    });

    let host = env::var("ROCKBOX_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("ROCKBOX_PORT").unwrap_or_else(|_| "6061".to_string());

    wait_for_rockboxd(port.parse()?, None)?;

    let mut client = PlaybackServiceClient::connect(format!("tcp://{}:{}", host, port)).await?;
    client
        .play_track(tonic::Request::new(PlayTrackRequest {
            path: normalize_path_or_url(path_or_url)?,
        }))
        .await?;

    drop(handle);
    Ok(())
}

fn normalize_path_or_url(path_or_url: &str) -> Result<String, Error> {
    if path_or_url.starts_with("http://") || path_or_url.starts_with("https://") {
        return Ok(path_or_url.to_string());
    }

    let path = Path::new(path_or_url);
    if !path.exists() {
        return Err(anyhow!("Path does not exist: {}", path_or_url));
    }

    if !path.is_file() {
        return Err(anyhow!("Path is not a file: {}", path_or_url));
    }

    Ok(path.canonicalize()?.to_string_lossy().to_string())
}
