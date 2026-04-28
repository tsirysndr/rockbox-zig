use std::io::Write;

use anyhow::Error;
use lofty::{file::TaggedFileExt, probe::Probe, tag::Accessor};

/// Read the first embedded picture from an audio file's tags and return the raw bytes + MIME type.
/// Returns None if the file has no embedded art or cannot be read.
pub fn read_album_art_bytes(track_path: &str) -> Option<(Vec<u8>, &'static str)> {
    let tagged_file = Probe::open(track_path).ok()?.read().ok()?;
    let tag = tagged_file
        .primary_tag()
        .filter(|t| !t.pictures().is_empty())
        .or_else(|| tagged_file.tags().iter().find(|t| !t.pictures().is_empty()))?;
    let picture = tag.pictures().first()?;
    let mime = match picture.mime_type() {
        Some(lofty::picture::MimeType::Png) => "image/png",
        _ => "image/jpeg",
    };
    Some((picture.data().to_vec(), mime))
}

pub fn extract_and_save_album_cover(track_path: &str) -> Result<Option<String>, Error> {
    extract_and_save_album_cover_with_key(track_path, None)
}

pub fn extract_and_save_album_cover_with_key(
    track_path: &str,
    album_key: Option<&str>,
) -> Result<Option<String>, Error> {
    let tagged_file = match Probe::open(track_path)
        .expect("ERROR: Bad path provided!")
        .read()
    {
        Ok(tagged_file) => tagged_file,
        Err(e) => {
            println!("Error opening file: {}", e);
            return Ok(None);
        }
    };

    let tag = match tagged_file
        .primary_tag()
        .filter(|tag| !tag.pictures().is_empty())
        .or_else(|| {
            tagged_file
                .tags()
                .iter()
                .find(|tag| !tag.pictures().is_empty())
        }) {
        Some(tag) => tag,
        None => {
            println!("No tag found in file: {}", track_path);
            return Ok(None);
        }
    };

    let pictures = tag.pictures();
    if pictures.len() > 0 {
        let home = std::env::var("HOME")?;
        let covers_path = format!("{}/.config/rockbox.org/covers", home);
        std::fs::create_dir_all(&covers_path)?;
        let picture = &pictures[0];
        let album_key = tag
            .album()
            .filter(|album| !album.trim().is_empty())
            .map(|album| album.to_string())
            .or_else(|| {
                album_key
                    .filter(|album| !album.trim().is_empty())
                    .map(|album| album.to_string())
            })
            .unwrap_or_else(|| track_path.to_string());
        let album = md5::compute(album_key.as_bytes());
        let filename = format!("{}/{:x}", covers_path, album);
        match picture.mime_type() {
            Some(lofty::picture::MimeType::Jpeg) => {
                let filename = format!("{}.jpg", filename);
                let mut file = std::fs::File::create(filename)?;
                file.write_all(picture.data())?;
                Ok(Some(format!("{:x}.jpg", album)))
            }
            Some(lofty::picture::MimeType::Png) => {
                let filename = format!("{}.png", filename);
                let mut file = std::fs::File::create(filename)?;
                file.write_all(picture.data())?;
                Ok(Some(format!("{:x}.png", album)))
            }
            Some(lofty::picture::MimeType::Bmp) => {
                let filename = format!("{}.bmp", filename);
                let mut file = std::fs::File::create(filename)?;
                file.write_all(picture.data())?;
                Ok(Some(format!("{:x}.bmp", album)))
            }
            Some(lofty::picture::MimeType::Gif) => {
                let filename = format!("{}.gif", filename);
                let mut file = std::fs::File::create(filename)?;
                file.write_all(picture.data())?;
                Ok(Some(format!("{:x}.gif", album)))
            }
            Some(lofty::picture::MimeType::Tiff) => {
                let filename = format!("{}.tiff", filename);
                let mut file = std::fs::File::create(filename)?;
                file.write_all(picture.data())?;
                Ok(Some(format!("{:x}.tiff", album)))
            }
            _ => {
                println!("Unsupported picture format");
                Ok(None)
            }
        }
    } else {
        Ok(None)
    }
}
