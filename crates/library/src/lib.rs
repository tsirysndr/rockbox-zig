use std::env;

use sqlx::{sqlite::SqliteConnectOptions, Error, Executor, Pool, Sqlite, SqlitePool};
use tracing::{debug, warn};

pub mod album_art;
pub mod artists;
pub mod audio_scan;
pub mod copyright_message;
pub mod entity;
pub mod label;
pub mod repo;

pub async fn create_connection_pool() -> Result<Pool<Sqlite>, Error> {
    let home = env::var("HOME").unwrap_or_else(|_| ".".into());
    let rockbox_dir = format!("{}/.config/rockbox.org", home);
    let _ = std::fs::create_dir_all(&rockbox_dir);
    let rockbox_db_path = format!("{}/rockbox-library.db", rockbox_dir);
    let db_url = env::var("DATABASE_URL").unwrap_or(rockbox_db_path);
    debug!("db url {}", db_url);
    // Do NOT call env::set_var here — it races with other threads on macOS.
    let options = SqliteConnectOptions::new()
        .filename(db_url)
        .create_if_missing(true)
        .busy_timeout(std::time::Duration::from_secs(30));
    let pool = SqlitePool::connect_with(options).await?;
    pool.execute(include_str!(
        "../migrations/20240923093823_create_tables.sql"
    ))
    .await?;
    match pool
        .execute(include_str!(
            "../migrations/20241011011557_add_artist_id_column.sql"
        ))
        .await
    {
        Ok(_) => {}
        Err(_) => warn!("artist_id column already exists"),
    }

    match pool
        .execute(include_str!(
            "../migrations/20241020125757_add-album_id-column.sql"
        ))
        .await
    {
        Ok(_) => {}
        Err(_) => warn!("album_id column already exists"),
    }
    match pool
        .execute(include_str!(
            "../migrations/20251218042124_add_album_label.sql"
        ))
        .await
    {
        Ok(_) => {}
        Err(_) => warn!("label column already exists"),
    }

    match pool
        .execute(include_str!(
            "../migrations/20251218044147_add_album_copyright_message.sql"
        ))
        .await
    {
        Ok(_) => {}
        Err(_) => warn!("copyright_message column already exists"),
    }

    match pool
        .execute(include_str!(
            "../migrations/20251218173111_add_artist_genres.sql"
        ))
        .await
    {
        Ok(_) => {}
        Err(_) => warn!("genres column already exists"),
    }

    match pool
        .execute(include_str!(
            "../migrations/20260425000000_add_playlist_tables.sql"
        ))
        .await
    {
        Ok(_) => {}
        Err(_) => warn!("playlist tables already exist"),
    }

    match pool
        .execute(include_str!(
            "../migrations/20260428000000_add_is_remote_to_track.sql"
        ))
        .await
    {
        Ok(_) => {}
        Err(_) => warn!("is_remote column already exists"),
    }

    /*
    pool.execute(include_str!(
        "../migrations/20260501000000_fix_datetime_formats.sql"
    ))
    .await?;
    */

    match pool
        .execute(include_str!(
            "../migrations/20260503000000_add_fts5_search.sql"
        ))
        .await
    {
        Ok(_) => {}
        Err(e) => warn!("fts5 migration: {}", e),
    }

    match pool
        .execute(include_str!(
            "../migrations/20260504000000_dedupe_genres.sql"
        ))
        .await
    {
        Ok(_) => {}
        Err(e) => warn!("dedupe_genres migration: {}", e),
    }

    sqlx::query("PRAGMA journal_mode=WAL")
        .execute(&pool)
        .await?;
    Ok(pool)
}
