use std::env;

use sqlx::{sqlite::SqliteConnectOptions, Error, Executor, Pool, Sqlite, SqlitePool};

pub mod album_art;
pub mod audio_scan;
pub mod entity;
pub mod repo;

pub async fn create_connection_pool() -> Result<Pool<Sqlite>, Error> {
    let home = env::var("HOME").unwrap();
    let rockbox_dir = format!("{}/.config/rockbox.org", home);
    std::fs::create_dir_all(&rockbox_dir).unwrap();
    let rockbox_db_path = format!("{}/rockbox-library.db", rockbox_dir);
    let db_url = env::var("DATABASE_URL").unwrap_or(rockbox_db_path);
    println!("db url {}", db_url);
    env::set_var("DATABASE_URL", &db_url);
    let options = SqliteConnectOptions::new()
        .filename(db_url)
        .create_if_missing(true);
    let pool = SqlitePool::connect_with(options).await?;
    pool.execute(include_str!(
        "../migrations/20240923093823_create_tables.sql"
    ))
    .await?;
    Ok(pool)
}
