use crate::album_art::extract_and_save_album_cover;
use crate::copyright_message::extract_copyright_message;
use crate::entity::album::Album;
use crate::entity::album_tracks::AlbumTracks;
use crate::entity::artist::Artist;
use crate::entity::artist_tracks::ArtistTracks;
use crate::label::extract_label;
use crate::{entity::track::Track, repo};
use anyhow::Error;
use chrono::Utc;
use futures::future::BoxFuture;
use futures::stream::{FuturesUnordered, StreamExt};
use owo_colors::OwoColorize;
use rockbox_sys as rb;
use sqlx::{Pool, Sqlite};
use std::path::PathBuf;
use tokio::fs;

const AUDIO_EXTENSIONS: [&str; 18] = [
    "mp3", "ogg", "flac", "m4a", "aac", "mp4", "alac", "wav", "wv", "mpc", "aiff", "aif", "ac3",
    "opus", "spx", "sid", "ape", "wma",
];

pub fn scan_audio_files(
    pool: Pool<Sqlite>,
    audio_dir: PathBuf,
) -> BoxFuture<'static, Result<Vec<PathBuf>, Error>> {
    Box::pin(async move {
        let mut result = Vec::new();
        let mut dir = fs::read_dir(audio_dir).await?;
        let mut futures = FuturesUnordered::new();
        while let Some(entry) = dir.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                println!("{} {:?}", "Scanning".bright_green(), path);
                let dir_path = path.clone();
                let cloned_pool = pool.clone();
                futures.push(tokio::spawn(async move {
                    scan_audio_files(cloned_pool, dir_path).await
                }));
            } else if path.is_file() {
                let path = path.to_str().unwrap();
                if !AUDIO_EXTENSIONS
                    .into_iter()
                    .any(|ext| path.ends_with(&format!(".{}", ext)))
                {
                    continue;
                }
                save_audio_metadata(pool.clone(), path).await?;

                let path = path.into();
                result.push(path);
            }
        }

        while let Some(Ok(sub_result)) = futures.next().await {
            match sub_result {
                Ok(paths) => result.extend(paths),
                Err(e) => return Err(e),
            }
        }

        Ok(result)
    })
}

pub async fn save_audio_metadata(pool: Pool<Sqlite>, path: &str) -> Result<(), Error> {
    if !AUDIO_EXTENSIONS
        .into_iter()
        .any(|ext| path.ends_with(&format!(".{}", ext)))
    {
        return Ok(());
    }

    let filename = path.split('/').last().unwrap();
    let dir = path.replace(filename, "");
    println!(
        "{} {}{}",
        "Found".bright_green(),
        dir,
        filename.bright_yellow()
    );
    let entry = rb::metadata::get_metadata(-1, path);

    let track_hash = format!("{:x}", md5::compute(entry.path.as_bytes()));
    let artist_id = cuid::cuid1()?;
    let album_id = cuid::cuid1()?;
    let album_md5 = format!(
        "{:x}",
        md5::compute(format!("{}{}{}", entry.albumartist, entry.album, entry.year).as_bytes())
    );
    let artist_id = repo::artist::save(
        pool.clone(),
        Artist {
            id: artist_id.clone(),
            name: match entry.albumartist.is_empty() {
                true => entry.artist.clone(),
                false => entry.albumartist.clone(),
            },
            bio: None,
            image: None,
            genres: None,
        },
    )
    .await?;

    let album_art = extract_and_save_album_cover(&entry.path)?;
    let album_id = repo::album::save(
        pool.clone(),
        Album {
            id: album_id,
            title: entry.album.clone(),
            artist: match entry.albumartist.is_empty() {
                true => entry.artist.clone(),
                false => entry.albumartist.clone(),
            },
            year: entry.year as u32,
            year_string: entry.year_string.clone(),
            album_art: album_art.clone(),
            md5: album_md5,
            artist_id: artist_id.clone(),
            label: extract_label(&entry.path)?,
            copyright_message: extract_copyright_message(&entry.path)?,
        },
    )
    .await?;

    let track_id = repo::track::save(
        pool.clone(),
        Track {
            id: cuid::cuid1()?,
            path: entry.path.clone(),
            title: entry.title,
            artist: entry.artist.clone(),
            album: entry.album,
            genre: match entry.genre_string.as_str() {
                "" => None,
                _ => Some(entry.genre_string),
            },
            year: Some(entry.year as u32),
            track_number: Some(entry.tracknum as u32),
            disc_number: entry.discnum as u32,
            year_string: Some(entry.year_string),
            composer: entry.composer,
            album_artist: entry.albumartist.clone(),
            bitrate: entry.bitrate,
            frequency: entry.frequency as u32,
            filesize: entry.filesize as u32,
            length: entry.length as u32,
            md5: track_hash,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            artist_id: artist_id.clone(),
            album_id: album_id.clone(),
            album_art,
            ..Default::default()
        },
    )
    .await?;

    repo::album_tracks::save(
        pool.clone(),
        AlbumTracks {
            id: cuid::cuid1()?,
            album_id,
            track_id: track_id.clone(),
        },
    )
    .await?;

    repo::artist_tracks::save(
        pool.clone(),
        ArtistTracks {
            id: cuid::cuid1()?,
            artist_id,
            track_id,
        },
    )
    .await?;

    Ok(())
}
