use anyhow::Error;
use lofty::file::TaggedFileExt;
use reqwest::multipart;
use reqwest::Client;
use rockbox_library::entity::album::Album;
use rockbox_library::entity::track::Track;
use std::fs::File;
use std::io::Read;

const AUDIO_EXTENSIONS: [&str; 18] = [
    "mp3", "ogg", "flac", "m4a", "aac", "mp4", "alac", "wav", "wv", "mpc", "aiff", "aif", "ac3",
    "opus", "spx", "sid", "ape", "wma",
];

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
