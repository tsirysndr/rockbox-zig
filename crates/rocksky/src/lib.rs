use anyhow::anyhow;
use anyhow::Error;
use api::rockbox::v1alpha1::playback_service_client::PlaybackServiceClient;
use api::rockbox::v1alpha1::NextRequest;
use api::rockbox::v1alpha1::PauseRequest;
use api::rockbox::v1alpha1::PlayRequest;
use api::rockbox::v1alpha1::PreviousRequest;
use api::rockbox::v1alpha1::ResumeRequest;
use api::rockbox::v1alpha1::StreamCurrentTrackRequest;
use api::rockbox::v1alpha1::StreamStatusRequest;
use futures_util::SinkExt;
use futures_util::StreamExt;
use lofty::file::TaggedFileExt;
use reqwest::multipart;
use reqwest::Client;
use rockbox_library::entity::album::Album;
use rockbox_library::entity::track::Track;
use serde_json::json;
use serde_json::Value;
use std::env;
use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use std::thread;
use tokio::sync::Mutex;
use tokio_tungstenite::connect_async;

const AUDIO_EXTENSIONS: [&str; 18] = [
    "mp3", "ogg", "flac", "m4a", "aac", "mp4", "alac", "wav", "wv", "mpc", "aiff", "aif", "ac3",
    "opus", "spx", "sid", "ape", "wma",
];

// const ROCKSKY_WS: &str = "ws://localhost:8000/ws";

pub mod api {
    #[path = ""]
    pub mod rockbox {

        #[path = "rockbox.v1alpha1.rs"]
        pub mod v1alpha1;
    }
}

pub async fn run_ws_session(token: String) -> Result<(), Error> {
    let rocksky_ws =
        env::var("ROCKSKY_WS").unwrap_or_else(|_| "wss://api.rocksky.app/ws".to_string());
    let (ws_stream, _) = connect_async(&rocksky_ws).await?;
    println!("Connected to {}", rocksky_ws);

    let (mut write, mut read) = ws_stream.split();
    let device_id = Arc::new(Mutex::new(String::new()));

    write
        .send(
            json!({
                "type": "register",
                "clientName": "Rockbox",
                "token": token
            })
            .to_string()
            .into(),
        )
        .await?;

    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(32);

    // Spawn track stream
    let tx_clone = tx.clone();
    tokio::spawn(start_track_stream(tx_clone));

    // Spawn status stream
    tokio::spawn(start_status_stream(tx.clone()));

    // Spawn sender
    {
        let device_id = device_id.clone();
        let token = token.clone();
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                let id = device_id.lock().await.clone();
                if let Err(e) = write
                    .send(
                        json!({
                            "type": "message",
                            "data": serde_json::from_str::<Value>(&msg).unwrap(),
                            "device_id": id,
                            "token": token
                        })
                        .to_string()
                        .into(),
                    )
                    .await
                {
                    eprintln!("Send error: {}", e);
                    break;
                }
            }
        });
    }

    let host = env::var("ROCKBOX_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("ROCKBOX_PORT").unwrap_or_else(|_| "6061".to_string());
    let url = format!("tcp://{}:{}", host, port);
    let mut client = PlaybackServiceClient::connect(url).await?;

    while let Some(msg) = read.next().await {
        let msg = match msg {
            Ok(m) => m.to_string(),
            Err(e) => {
                eprintln!("Read error: {}", e);
                break;
            }
        };

        let msg: Value = serde_json::from_str(&msg)?;
        if let Some(id) = msg["deviceId"].as_str() {
            *device_id.lock().await = id.to_string();
        }

        if let Some("command") = msg["type"].as_str() {
            if let Some(cmd) = msg["action"].as_str() {
                match cmd {
                    "play" => {
                        client.resume(tonic::Request::new(ResumeRequest {})).await?;
                    }
                    "pause" => {
                        client
                            .pause(tonic::Request::new(PauseRequest::default()))
                            .await?;
                    }
                    "next" => {
                        client
                            .next(tonic::Request::new(NextRequest::default()))
                            .await?;
                    }
                    "previous" => {
                        client
                            .previous(tonic::Request::new(PreviousRequest::default()))
                            .await?;
                    }
                    "seek" => {
                        let pos = msg["args"]["position"].as_i64().unwrap_or(0);
                        client
                            .play(tonic::Request::new(PlayRequest {
                                offset: 0,
                                elapsed: pos,
                            }))
                            .await?;
                    }
                    _ => {
                        println!("Unknown command: {}", cmd);
                    }
                };
            }
        }
    }

    Err(anyhow!("Connection closed"))
}

pub async fn start_track_stream(tx: tokio::sync::mpsc::Sender<String>) -> Result<(), Error> {
    let host = env::var("ROCKBOX_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("ROCKBOX_PORT").unwrap_or_else(|_| "6061".to_string());
    let url = format!("tcp://{}:{}", host, port);
    let mut client = PlaybackServiceClient::connect(url).await?;
    let mut stream = client
        .stream_current_track(tonic::Request::new(StreamCurrentTrackRequest {}))
        .await?
        .into_inner();

    while let Some(Ok(track)) = stream.next().await {
        tx.send(
            json!({
                "type": "track",
                "title": track.title,
                "artist": track.artist,
                "album_artist": track.album_artist,
                "album": track.album,
                "length": track.length,
                "elapsed": track.elapsed,
                "track_number": track.tracknum,
                "disc_number": track.discnum,
                "composer": track.composer,
                "album_art": track.album_art
            })
            .to_string(),
        )
        .await?;
    }

    Ok(())
}

async fn start_status_stream(tx: tokio::sync::mpsc::Sender<String>) -> Result<(), Error> {
    let host = env::var("ROCKBOX_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("ROCKBOX_PORT").unwrap_or_else(|_| "6061".to_string());
    let url = format!("tcp://{}:{}", host, port);
    let mut client = PlaybackServiceClient::connect(url).await?;
    let mut stream = client
        .stream_status(tonic::Request::new(StreamStatusRequest {}))
        .await?
        .into_inner();

    while let Some(Ok(status)) = stream.next().await {
        tx.send(
            json!({
                "type": "status",
                "status": status.status
            })
            .to_string(),
        )
        .await?;
    }

    Ok(())
}

pub fn register_rockbox() -> Result<(), Error> {
    let home = dirs::home_dir().unwrap();
    let token_file = home.join(".config").join("rockbox.org").join("token");

    if !token_file.exists() {
        return Ok(());
    }

    let token = std::fs::read_to_string(token_file)?;

    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            let delay = 3;

            loop {
                match run_ws_session(token.clone()).await {
                    Ok(_) => {
                        println!("WebSocket session ended cleanly");
                    }
                    Err(e) => {
                        eprintln!("WebSocket session error: {}", e);
                    }
                }

                println!("Reconnecting in {} seconds...", delay);
                tokio::time::sleep(std::time::Duration::from_secs(delay)).await;
            }
        })
    });

    Ok(())
}

pub async fn upload_album_cover(name: &str) -> Result<(), Error> {
    let home = dirs::home_dir().unwrap();
    let cover = home
        .join(".config")
        .join("rockbox.org")
        .join("covers")
        .join(name);

    let mut file = File::open(&cover)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let part = multipart::Part::bytes(buffer).file_name(cover.display().to_string());
    let form = multipart::Form::new().part("file", part);

    let token_file = home.join(".config").join("rockbox.org").join("token");

    if !token_file.exists() {
        return Ok(());
    }

    let token = std::fs::read_to_string(token_file)?;

    let client = Client::new();

    const URL: &str = "https://uploads.rocksky.app";

    let response = client
        .post(URL)
        .header("Authorization", format!("Bearer {}", token))
        .multipart(form)
        .send()
        .await?;

    println!("Cover uploaded: {}", response.status());

    Ok(())
}

pub async fn scrobble(track: Track, album: Album) -> Result<(), Error> {
    let home = dirs::home_dir().unwrap();
    let token_file = home.join(".config").join("rockbox.org").join("token");

    if !token_file.exists() {
        return Ok(());
    }

    let token = std::fs::read_to_string(token_file)?;

    if let Some(album_art) = track.album_art.clone() {
        match upload_album_cover(&album_art).await {
            Ok(_) => {}
            Err(r) => {
                eprintln!("Failed to upload album art: {}", r);
            }
        }
    }

    let (lyrics, copyright_message) = match parse_lyrics_and_copyright(&track.path) {
        Ok((lyrics, copyright_message)) => (lyrics, copyright_message),
        Err(_) => (None, None),
    };

    let client = Client::new();
    const URL: &str = "https://api.rocksky.app/now-playing";
    let response = client
        .post(URL)
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "title": track.title,
            "album": track.album,
            "artist": track.artist,
            "albumArtist": track.album_artist,
            "duration": track.length,
            "trackNumber": track.track_number,
            "releaseDate": match album.year_string.contains("-") {
                true => Some(album.year_string),
                false => None,
            },
            "year": album.year,
            "discNumber": track.disc_number,
            "composer": track.composer,
            "albumArt": match track.album_art.is_some() {
                true => Some(format!("https://cdn.rocksky.app/covers/{}", track.album_art.unwrap())),
                false => None
            },
            "lyrics": lyrics,
            "copyrightMessage": copyright_message,
        }))
        .send()
        .await?;
    println!("Scrobbled: {}", response.status());

    if !response.status().is_success() {
        println!("Failed to scrobble: {}", response.text().await?);
    }

    Ok(())
}

pub async fn save_track(track: Track, album: Album) -> Result<(), Error> {
    let home = dirs::home_dir().unwrap();
    let token_file = home.join(".config").join("rockbox.org").join("token");

    if !token_file.exists() {
        return Ok(());
    }

    let token = std::fs::read_to_string(token_file)?;

    if let Some(album_art) = track.album_art.clone() {
        match upload_album_cover(&album_art).await {
            Ok(_) => {}
            Err(r) => {
                eprintln!("Failed to upload album art: {}", r);
            }
        }
    }

    let (lyrics, copyright_message) = match parse_lyrics_and_copyright(&track.path) {
        Ok((lyrics, copyright_message)) => (lyrics, copyright_message),
        Err(_) => (None, None),
    };

    let client = Client::new();
    const URL: &str = "https://api.rocksky.app/tracks";
    let response = client
        .post(URL)
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "title": track.title,
            "album": track.album,
            "artist": track.artist,
            "albumArtist": match track.album_artist.is_empty() {
                true => track.artist,
                false => track.album_artist,
            },
            "duration": track.length,
            "trackNumber": track.track_number,
            "releaseDate": match album.year_string.contains("-") {
                true => Some(album.year_string),
                false => None,
            },
            "year": album.year,
            "discNumber": track.disc_number,
            "composer": track.composer,
            "albumArt": match track.album_art.is_some() {
                true => Some(format!("https://cdn.rocksky.app/covers/{}", track.album_art.unwrap())),
                false => None
            },
            "lyrics": lyrics,
            "copyrightMessage": copyright_message,
        }))
        .send()
        .await?;
    println!("Track Saved: {} {}", track.path, response.status());

    if !response.status().is_success() {
        println!(
            "Failed to save Track: {} {}",
            track.path,
            response.text().await?
        );
    }

    Ok(())
}

pub async fn like(track: Track, album: Album) -> Result<(), Error> {
    let home = dirs::home_dir().unwrap();
    let token_file = home.join(".config").join("rockbox.org").join("token");

    if !token_file.exists() {
        return Ok(());
    }

    let token = std::fs::read_to_string(token_file)?;

    if let Some(album_art) = track.album_art.clone() {
        match upload_album_cover(&album_art).await {
            Ok(_) => {}
            Err(r) => {
                eprintln!("Failed to upload album art: {}", r);
            }
        }
    }

    let client = Client::new();
    const URL: &str = "https://api.rocksky.app/likes";
    let response = client
        .post(URL)
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "title": track.title,
            "album": track.album,
            "artist": track.artist,
            "albumArtist": track.album_artist,
            "duration": track.length,
            "trackNumber": track.track_number,
            "releaseDate": match album.year_string.contains("-") {
                true => Some(album.year_string),
                false => None,
            },
            "year": album.year,
            "discNumber": track.disc_number,
            "composer": track.composer,
            "albumArt": match track.album_art.is_some() {
                true => Some(format!("https://cdn.rocksky.app/covers/{}", track.album_art.unwrap())),
                false => None
            }
        }))
        .send()
        .await?;
    println!("Liked: {}", response.status());
    Ok(())
}

pub async fn unlike(track: Track) -> Result<(), Error> {
    let home = dirs::home_dir().unwrap();
    let token_file = home.join(".config").join("rockbox.org").join("token");

    if !token_file.exists() {
        return Ok(());
    }

    let token = std::fs::read_to_string(token_file)?;

    let hash = sha256::digest(
        format!("{} - {} - {}", track.title, track.artist, track.album).to_lowercase(),
    );

    let client = Client::new();
    let url: &str = &format!("https://api.rocksky.app/likes/{}", hash);
    let response = client
        .delete(url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    println!("Unliked: {} {}", response.status(), hash);

    Ok(())
}

fn parse_lyrics_and_copyright(path: &str) -> Result<(Option<String>, Option<String>), Error> {
    if !AUDIO_EXTENSIONS
        .into_iter()
        .any(|ext| path.ends_with(&format!(".{}", ext)))
    {
        return Ok((None, None));
    }

    let tagged_file = lofty::read_from_path(path)?;

    let tag = match tagged_file.primary_tag() {
        Some(primary_tag) => primary_tag,
        None => tagged_file.first_tag().expect("No tags found"),
    };

    let lyrics = tag
        .get_string(&lofty::tag::ItemKey::Lyrics)
        .map(|x| x.to_string());
    let copyright_message = tag
        .get_string(&lofty::tag::ItemKey::CopyrightMessage)
        .map(|x| x.to_string());

    Ok((lyrics, copyright_message))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_parse_metadata() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("fixtures");
        path.push("08 - Internet Money - Speak(Explicit).m4a");

        let result = parse_lyrics_and_copyright(path.to_str().unwrap());
        assert!(result.is_ok());

        let result = result.unwrap();

        assert!(result.0.is_some());
        assert!(result.1.is_some());
    }
}
