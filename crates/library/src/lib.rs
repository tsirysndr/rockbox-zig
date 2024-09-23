use std::env;

use sqlx::{sqlite::SqliteConnectOptions, Error, Executor, Pool, Sqlite, SqlitePool};

pub mod audio_scan;
pub mod entity;
pub mod repo;

pub async fn create_connection_pool() -> Result<Pool<Sqlite>, Error> {
    let db_url = env::var("DATABASE_URL").unwrap_or(":memory:".to_string());
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
