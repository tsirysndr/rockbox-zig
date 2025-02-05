use anyhow::Error;
use reqwest::multipart;
use reqwest::Client;
use rockbox_library::entity::album::Album;
use rockbox_library::entity::track::Track;
use std::fs::File;
use std::io::Read;

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
            }
        }))
        .send()
        .await?;
    println!("Scrobbled: {}", response.status());
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

    println!("Unliked: {}", response.status());

    Ok(())
}
