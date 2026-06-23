use crate::audio_scan::save_audio_metadata;
use crate::repo;
use anyhow::Error;
use notify::event::{ModifyKind, RenameMode};
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use sqlx::{Pool, Sqlite};
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

const AUDIO_EXTENSIONS: [&str; 18] = [
    "mp3", "ogg", "flac", "m4a", "aac", "mp4", "alac", "wav", "wv", "mpc", "aiff", "aif", "ac3",
    "opus", "spx", "sid", "ape", "wma",
];

fn is_audio_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|ext| {
            let lower = ext.to_ascii_lowercase();
            AUDIO_EXTENSIONS.iter().any(|x| *x == lower.as_str())
        })
        .unwrap_or(false)
}

/// Start watching `music_dir` recursively. New audio files are inserted into
/// the database; removed audio files are deleted from it. The watcher handle
/// is leaked so it lives for the lifetime of the process — dropping the
/// `RecommendedWatcher` stops its background thread.
pub fn start_watcher(pool: Pool<Sqlite>, music_dir: PathBuf) -> Result<(), Error> {
    if !music_dir.exists() {
        warn!(
            "watcher: music_dir does not exist, skipping: {}",
            music_dir.display()
        );
        return Ok(());
    }

    info!(
        "watcher: starting library watcher on {}",
        music_dir.display()
    );

    let (tx, mut rx) = mpsc::unbounded_channel::<notify::Event>();

    let mut watcher: RecommendedWatcher =
        notify::recommended_watcher(move |res: notify::Result<notify::Event>| match res {
            Ok(event) => {
                let _ = tx.send(event);
            }
            Err(e) => warn!("watcher: notify error: {}", e),
        })?;

    watcher.watch(&music_dir, RecursiveMode::Recursive)?;
    // Keep the watcher alive for the lifetime of the process.
    Box::leak(Box::new(watcher));

    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            if let Err(e) = handle_event(pool.clone(), event).await {
                warn!("watcher: handler error: {}", e);
            }
        }
    });

    Ok(())
}

async fn handle_event(pool: Pool<Sqlite>, event: notify::Event) -> Result<(), Error> {
    match event.kind {
        EventKind::Create(_) => {
            for path in event.paths {
                add_path(&pool, &path).await;
            }
        }
        EventKind::Modify(ModifyKind::Data(_)) => {
            for path in event.paths {
                // Re-insert only if not already indexed; save_audio_metadata
                // is idempotent and will no-op for known paths.
                add_path(&pool, &path).await;
            }
        }
        EventKind::Modify(ModifyKind::Name(mode)) => {
            handle_rename(&pool, mode, event.paths).await;
        }
        EventKind::Remove(_) => {
            for path in event.paths {
                remove_path(&pool, &path).await;
            }
        }
        _ => {}
    }
    Ok(())
}

async fn add_path(pool: &Pool<Sqlite>, path: &Path) {
    if !is_audio_file(path) || !path.exists() {
        return;
    }
    let path_str = path.to_string_lossy().to_string();
    debug!("watcher: add {}", path_str);
    if let Err(e) = save_audio_metadata(pool.clone(), &path_str, None).await {
        warn!("watcher: failed to add {}: {}", path_str, e);
    }
}

async fn remove_path(pool: &Pool<Sqlite>, path: &Path) {
    if !is_audio_file(path) {
        return;
    }
    let path_str = path.to_string_lossy().to_string();
    match repo::track::delete_by_path(pool.clone(), &path_str).await {
        Ok(Some(track)) => info!("watcher: removed {} ({})", track.title, path_str),
        Ok(None) => debug!("watcher: remove for unknown path {}", path_str),
        Err(e) => warn!("watcher: failed to remove {}: {}", path_str, e),
    }
}

async fn handle_rename(pool: &Pool<Sqlite>, mode: RenameMode, paths: Vec<PathBuf>) {
    match mode {
        RenameMode::Both if paths.len() == 2 => {
            remove_path(pool, &paths[0]).await;
            add_path(pool, &paths[1]).await;
        }
        RenameMode::From => {
            for path in paths {
                remove_path(pool, &path).await;
            }
        }
        RenameMode::To => {
            for path in paths {
                add_path(pool, &path).await;
            }
        }
        _ => {}
    }
}
