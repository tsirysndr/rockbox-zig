use std::env;

use sqlx::{sqlite::SqliteConnectOptions, Error, Executor, Pool, Sqlite, SqlitePool};

pub mod album_art;
pub mod artists;
pub mod audio_scan;
pub mod copyright_message;
pub mod entity;
pub mod label;
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
    match pool
        .execute(include_str!(
            "../migrations/20241011011557_add_artist_id_column.sql"
        ))
        .await
    {
        Ok(_) => {}
        Err(_) => println!("artist_id column already exists"),
    }

    match pool
        .execute(include_str!(
            "../migrations/20241020125757_add-album_id-column.sql"
        ))
        .await
    {
        Ok(_) => {}
        Err(_) => println!("album_id column already exists"),
    }
    match pool
        .execute(include_str!(
            "../migrations/20251218042124_add_album_label.sql"
        ))
        .await
    {
        Ok(_) => {}
        Err(_) => println!("label column already exists"),
    }

    match pool
        .execute(include_str!(
            "../migrations/20251218044147_add_album_copyright_message.sql"
        ))
        .await
    {
        Ok(_) => {}
        Err(_) => println!("copyright_message column already exists"),
    }

    match pool
        .execute(include_str!(
            "../migrations/20251218173111_add_artist_genres.sql"
        ))
        .await
    {
        Ok(_) => {}
        Err(_) => println!("genres column already exists"),
    }

    sqlx::query("PRAGMA journal_mode=WAL")
        .execute(&pool)
        .await?;
    Ok(pool)
}
