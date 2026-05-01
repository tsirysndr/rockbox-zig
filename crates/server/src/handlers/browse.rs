use std::{env, fs};

use actix_web::{error::ErrorInternalServerError, web, HttpResponse};
use rockbox_sys::types::tree::Entry;
use serde::Deserialize;

use crate::{cache::update_cache, http::AppState, AUDIO_EXTENSIONS};

type HandlerResult = actix_web::Result<HttpResponse>;

#[derive(Deserialize)]
pub struct TreeQuery {
    q: Option<String>,
    show_hidden: Option<String>,
}

pub async fn get_tree_entries(
    state: web::Data<AppState>,
    query: web::Query<TreeQuery>,
) -> HandlerResult {
    let home = env::var("HOME").map_err(ErrorInternalServerError)?;
    let music_library = env::var("ROCKBOX_LIBRARY").unwrap_or_else(|_| format!("{}/Music", home));

    let path = query.q.as_deref().unwrap_or(&music_library).to_string();
    let show_hidden = query.show_hidden.as_deref() == Some("true");

    if !fs::metadata(&path)
        .map_err(ErrorInternalServerError)?
        .is_dir()
    {
        return Ok(HttpResponse::InternalServerError().body("Path is not a directory"));
    }

    let mut fs_cache = state.fs_cache.lock().await;
    if let Some(entries) = fs_cache.get(&path) {
        let entries = entries.clone();
        drop(fs_cache);
        update_cache(state.fs_cache.clone(), &path, show_hidden);
        return Ok(HttpResponse::Ok().json(entries));
    }

    let mut entries: Vec<Entry> = vec![];

    for file in fs::read_dir(&path).map_err(ErrorInternalServerError)? {
        let file = file.map_err(ErrorInternalServerError)?;

        if file.metadata().map_err(ErrorInternalServerError)?.is_file()
            && !AUDIO_EXTENSIONS.iter().any(|ext| {
                file.path()
                    .to_string_lossy()
                    .ends_with(&format!(".{}", ext))
            })
        {
            continue;
        }

        if file.file_name().to_string_lossy().starts_with('.') && !show_hidden {
            continue;
        }

        entries.push(Entry {
            name: file.path().to_string_lossy().to_string(),
            time_write: file
                .metadata()
                .map_err(ErrorInternalServerError)?
                .modified()
                .map_err(ErrorInternalServerError)?
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map_err(ErrorInternalServerError)?
                .as_secs() as u32,
            attr: match file.metadata().map_err(ErrorInternalServerError)?.is_dir() {
                true => 0x10,
                false => 0,
            },
            ..Default::default()
        });
    }

    fs_cache.insert(path, entries.clone());

    Ok(HttpResponse::Ok().json(entries))
}
