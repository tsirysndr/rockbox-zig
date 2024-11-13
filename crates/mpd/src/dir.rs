use std::{fs, thread};

use anyhow::Error;
use rockbox_settings::get_music_dir;

use crate::{consts::AUDIO_EXTENSIONS, Context};

pub fn read_dir(
    ctx: Context,
    path: String,
    recursive: bool,
    with_metadata: bool,
) -> Result<String, Error> {
    let music_dir = get_music_dir()?;
    let mut files = String::new();

    fn read_dir_recursive(
        ctx: &Context,
        path: String,
        music_dir: &str,
        recursive: bool,
        files: &mut String,
        with_metadata: bool,
    ) -> Result<(), Error> {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            let path_str = path.to_str().unwrap().to_string();

            if path.is_dir() {
                if recursive {
                    read_dir_recursive(
                        ctx,
                        path_str.clone(),
                        music_dir,
                        recursive,
                        files,
                        with_metadata,
                    )?;
                }
            }

            if AUDIO_EXTENSIONS.iter().any(|ext| path_str.ends_with(ext)) || path.is_dir() {
                let last_modified = entry.metadata()?.modified()?;
                let last_modified = chrono::DateTime::from_timestamp(
                    last_modified
                        .duration_since(std::time::UNIX_EPOCH)?
                        .as_secs() as i64,
                    0,
                )
                .unwrap();

                let last_modified = last_modified.format("%Y-%m-%dT%H:%M:%SZ").to_string();
                let entry_str = path_str.replace(music_dir, "");
                let entry_str = if entry_str.starts_with('/') {
                    &entry_str[1..]
                } else {
                    &entry_str
                };

                let mut metadata = format!(
                    "{}: {}\nLast-Modified: {}\n",
                    if path.is_dir() { "directory" } else { "file" },
                    entry_str,
                    last_modified
                );

                if !with_metadata {
                    files.push_str(&metadata);
                    continue;
                }

                let kv = ctx.kv.clone();
                let track = thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    let kv_clone = kv.clone();
                    let kv = rt.block_on(kv_clone.lock());
                    let track = kv.get(&path_str);
                    track.map(|x| x.clone())
                })
                .join()
                .unwrap();

                if let Some(track) = track {
                    metadata.push_str(&format!(
                        "Title: {}\nArtist: {}\nAlbum: {}\nTime: {}\nDuration: {}\n",
                        track.title,
                        track.artist,
                        track.album,
                        (track.length / 1000) as u32,
                        track.length / 1000
                    ));
                    if let Some(track_number) = track.track_number {
                        metadata.push_str(&format!("Track: {}\n", track_number));
                    }

                    if let Some(year) = track.year {
                        metadata.push_str(&format!("Date: {}\n", year));
                    }
                }
                files.push_str(&metadata);
            }
        }
        Ok(())
    }

    read_dir_recursive(&ctx, path, &music_dir, recursive, &mut files, with_metadata)?;
    Ok(files)
}
