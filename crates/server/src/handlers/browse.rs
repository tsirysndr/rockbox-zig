use std::fs;

use crate::http::{Context, Request, Response};
use anyhow::Error;
use rockbox_sys::types::tree::Entry;

const AUDIO_EXTENSIONS: [&str; 17] = [
    "mp3", "ogg", "flac", "m4a", "aac", "mp4", "alac", "wav", "wv", "mpc", "aiff", "ac3", "opus",
    "spx", "sid", "ape", "wma",
];

pub async fn get_tree_entries(
    _ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let path = match req.query_params.get("q") {
        Some(path) => path.as_str().unwrap_or("/"),
        None => "/",
    };
    let show_hidden = match req.query_params.get("show_hidden") {
        Some(show_hidden) => show_hidden.as_str().unwrap_or("false") == "true",
        None => false,
    };

    if !fs::metadata(path)?.is_dir() {
        res.set_status(500);
        res.text("Path is not a directory");
        return Ok(());
    }

    let mut entries = vec![];

    for file in fs::read_dir(path)? {
        let file = file?;

        if file.metadata()?.is_file()
            && !AUDIO_EXTENSIONS.iter().any(|ext| {
                file.path()
                    .to_string_lossy()
                    .ends_with(&format!(".{}", ext))
            })
        {
            continue;
        }

        if file.file_name().to_string_lossy().starts_with(".") && !show_hidden {
            continue;
        }

        entries.push(Entry {
            name: file.path().to_string_lossy().to_string(),
            time_write: file
                .metadata()?
                .modified()?
                .duration_since(std::time::SystemTime::UNIX_EPOCH)?
                .as_secs() as u32,
            ..Default::default()
        });
    }

    res.json(&entries);
    Ok(())
}
