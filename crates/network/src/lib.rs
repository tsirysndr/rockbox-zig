use std::{fs, io::copy};

use anyhow::{anyhow, Error};
use rockbox_library::{audio_scan::save_audio_metadata, create_connection_pool};

pub async fn download(url: &str) -> Result<String, Error> {
    if !url.starts_with("http") {
        return Ok(url.to_string());
    }

    let home_dir = dirs::home_dir().unwrap();
    let cache_dir = format!("{}/.cache/rockbox", home_dir.to_str().unwrap());
    fs::create_dir_all(&cache_dir)?;

    let client = reqwest::Client::new();
    let response = client.head(url).send().await?;
    let mime = response
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();
    let extension = get_file_extension(mime)?;
    let hash = md5::compute(url.as_bytes());
    let file_path = format!("{}/{:x}.{}", cache_dir, hash, extension);

    if fs::metadata(&file_path).is_ok() {
        return Ok(file_path);
    }

    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;
    let mut file = fs::File::create(&file_path)?;
    let content = response.bytes().await?;

    copy(&mut content.as_ref(), &mut file)?;

    let pool = create_connection_pool().await?;
    save_audio_metadata(pool, &file_path).await?;

    Ok(file_path)
}

pub async fn download_tracks(urls: Vec<String>) -> Result<Vec<String>, Error> {
    let mut files = Vec::new();
    let urls: Vec<&str> = urls.iter().map(|url| url.as_str()).collect();
    for url in urls {
        match download(url).await {
            Ok(path) => files.push(path),
            Err(e) => eprintln!("Failed to download {}: {}", url, e),
        }
    }
    Ok(files)
}

fn get_file_extension(mime: &str) -> Result<&str, Error> {
    match mime {
        "audio/mpeg" => Ok("mp3"),
        "audio/ogg" => Ok("ogg"),
        "audio/flac" => Ok("flac"),
        "audio/x-m4a" => Ok("m4a"),
        "audio/aac" => Ok("aac"),
        "video/mp4" => Ok("mp4"),
        "audio/wav" => Ok("wav"),
        "audio/x-wav" => Ok("wav"),
        "audio/x-wavpack" => Ok("wv"),
        "audio/x-musepack" => Ok("mpc"),
        "audio/aiff" => Ok("aiff"),
        "audio/x-aiff" => Ok("aiff"),
        "audio/ac3" => Ok("ac3"),
        "audio/vnd.dolby.dd-raw" => Ok("ac3"),
        "audio/opus" => Ok("opus"),
        "audio/x-speex" => Ok("spx"),
        "audio/prs.sid" => Ok("sid"),
        "audio/ape" => Ok("ape"),
        "audio/x-monkeys-audio" => Ok("ape"),
        "audio/x-ms-wma" => Ok("wma"),
        _ => Err(anyhow!("Unsupported mime type: {}", mime)),
    }
}
