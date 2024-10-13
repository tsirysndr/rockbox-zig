use std::{fs, thread};

use anyhow::Error;
use rockbox_sys::types::tree::Entry;

use crate::{http::Context, AUDIO_EXTENSIONS};

pub fn update_cache(ctx: &Context, path: &str, show_hidden: bool) {
    let fs_cache = ctx.fs_cache.clone();
    let path = path.to_string();
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut fs_cache = rt.block_on(fs_cache.lock());
        let mut entries = vec![];

        for file in fs::read_dir(&path)? {
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
                attr: match file.metadata()?.is_dir() {
                    true => 0x10,
                    false => 0,
                },
                ..Default::default()
            });
        }

        fs_cache.insert(path, entries.clone());
        Ok::<(), Error>(())
    });
}
