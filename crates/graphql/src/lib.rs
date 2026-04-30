use anyhow::Error;
use async_graphql::Schema;
use futures::{future::BoxFuture, stream::FuturesUnordered, StreamExt};
use schema::{Mutation, Query, Subscription};
use tokio::fs;

pub mod schema;
pub mod server;
pub mod simplebroker;
pub mod types;

pub type RockboxSchema = Schema<Query, Mutation, Subscription>;

pub const AUDIO_EXTENSIONS: [&str; 17] = [
    "mp3", "ogg", "flac", "m4a", "aac", "mp4", "alac", "wav", "wv", "mpc", "aiff", "ac3", "opus",
    "spx", "sid", "ape", "wma",
];

pub fn rockbox_url() -> String {
    let port = std::env::var("ROCKBOX_TCP_PORT").unwrap_or_else(|_| "6063".to_string());
    format!("http://127.0.0.1:{}", port)
}

pub fn read_files(path: String) -> BoxFuture<'static, Result<Vec<String>, Error>> {
    Box::pin(async move {
        if path.starts_with("upnp://") {
            return read_upnp_files(path).await;
        }
        let mut result = Vec::new();
        let mut dir = fs::read_dir(path).await?;
        let mut futures = FuturesUnordered::new();
        while let Some(entry) = dir.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                let dir_path = path.clone();
                futures.push(tokio::spawn(async move {
                    read_files(dir_path.to_str().unwrap().to_string()).await
                }));
            } else if path.is_file() {
                if !AUDIO_EXTENSIONS
                    .into_iter()
                    .any(|ext| path.to_str().unwrap().ends_with(&format!(".{}", ext)))
                {
                    continue;
                }
                result.push(path.to_str().unwrap().to_string());
            }
        }
        while let Some(Ok(future)) = futures.next().await {
            result.extend(future?);
        }
        Ok(result)
    })
}

/// Like `read_files` but also returns the `albumArtURI` for UPnP tracks.
/// Local files always have `None` for the art URI.
pub fn read_files_with_art(
    path: String,
) -> BoxFuture<'static, Result<Vec<(String, Option<String>)>, Error>> {
    Box::pin(async move {
        if path.starts_with("upnp://") {
            return read_upnp_entries(path).await;
        }
        let files = read_files(path).await?;
        Ok(files.into_iter().map(|f| (f, None)).collect())
    })
}

pub fn read_upnp_entries(
    path: String,
) -> BoxFuture<'static, Result<Vec<(String, Option<String>)>, Error>> {
    Box::pin(async move {
        use rockbox_upnp::control_point::{
            browse_content_directory, percent_decode, percent_encode,
        };
        let rest = path.trim_start_matches("upnp://");
        let (ctrl_encoded, object_id_raw) = match rest.find('/') {
            None => (rest, "0"),
            Some(i) => (&rest[..i], &rest[i + 1..]),
        };
        let object_id = if object_id_raw.is_empty() {
            "0".to_string()
        } else {
            percent_decode(object_id_raw)
        };
        let control_url = percent_decode(ctrl_encoded);
        let ctrl_encoded = ctrl_encoded.to_string();
        let entries = browse_content_directory(&control_url, &object_id).await;
        let mut result = Vec::new();
        let mut futures = FuturesUnordered::new();
        for entry in entries {
            if entry.is_container {
                let sub_path = format!("upnp://{}/{}", ctrl_encoded, percent_encode(&entry.id));
                futures.push(tokio::spawn(read_upnp_entries(sub_path)));
            } else if let Some(uri) = entry.uri {
                result.push((uri, entry.album_art_uri));
            }
        }
        while let Some(Ok(sub)) = futures.next().await {
            result.extend(sub?);
        }
        Ok(result)
    })
}

pub fn read_upnp_files(path: String) -> BoxFuture<'static, Result<Vec<String>, Error>> {
    Box::pin(async move {
        let entries = read_upnp_entries(path).await?;
        Ok(entries.into_iter().map(|(uri, _)| uri).collect())
    })
}
