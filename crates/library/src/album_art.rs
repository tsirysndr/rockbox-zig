use std::io::Write;

use anyhow::Error;
use lofty::{file::TaggedFileExt, probe::Probe, tag::Accessor};

pub fn extract_and_save_album_cover(track_path: &str) -> Result<Option<String>, Error> {
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

    let primary_tag = tagged_file.primary_tag();
    let tag = match primary_tag {
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

        if tag.album().is_none() {
            println!("No album found in file: {}", track_path);
            return Ok(None);
        }

        let album = md5::compute(tag.album().unwrap().as_bytes());
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
