use sqlx::{sqlite::SqliteConnectOptions, Pool, Sqlite, SqlitePool};

#[derive(sqlx::FromRow, Debug)]
pub struct Track {
    pub id: String,
    pub path: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub album_id: String,
    pub artist_id: String,
    pub track_number: Option<i64>,
    pub length: i64,
    pub filesize: i64,
    pub album_art: Option<String>,
    pub genre: Option<String>,
    pub year: Option<i64>,
}

#[derive(sqlx::FromRow, Debug)]
pub struct Album {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album_art: Option<String>,
    pub track_count: i64,
}

#[derive(sqlx::FromRow, Debug)]
pub struct Artist {
    pub id: String,
    pub name: String,
    pub track_count: i64,
}

pub async fn open_pool() -> anyhow::Result<Pool<Sqlite>> {
    let home = std::env::var("HOME").unwrap_or_default();
    let db_path = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| format!("{}/.config/rockbox.org/rockbox-library.db", home));

    if !std::path::Path::new(&db_path).exists() {
        anyhow::bail!("library database not found at {}", db_path);
    }

    let options = SqliteConnectOptions::new()
        .filename(&db_path)
        .read_only(true);
    Ok(SqlitePool::connect_with(options).await?)
}

pub async fn all_tracks(pool: &Pool<Sqlite>) -> anyhow::Result<Vec<Track>> {
    let rows = sqlx::query_as::<_, Track>(
        "SELECT id, path, title, artist, album, album_id, artist_id, \
         track_number, length, filesize, album_art, genre, year FROM track \
         ORDER BY artist, album, track_number, title",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn track_by_path(pool: &Pool<Sqlite>, path: &str) -> anyhow::Result<Option<Track>> {
    let row = sqlx::query_as::<_, Track>(
        "SELECT id, path, title, artist, album, album_id, artist_id, \
         track_number, length, filesize, album_art, genre, year FROM track WHERE path = ?",
    )
    .bind(path)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn track_by_id(pool: &Pool<Sqlite>, id: &str) -> anyhow::Result<Option<Track>> {
    let row = sqlx::query_as::<_, Track>(
        "SELECT id, path, title, artist, album, album_id, artist_id, \
         track_number, length, filesize, album_art, genre, year FROM track WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn tracks_by_album(pool: &Pool<Sqlite>, album_id: &str) -> anyhow::Result<Vec<Track>> {
    let rows = sqlx::query_as::<_, Track>(
        "SELECT id, path, title, artist, album, album_id, artist_id, \
         track_number, length, filesize, album_art, genre, year FROM track \
         WHERE album_id = ? ORDER BY track_number, title",
    )
    .bind(album_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn tracks_by_artist(pool: &Pool<Sqlite>, artist_id: &str) -> anyhow::Result<Vec<Track>> {
    let rows = sqlx::query_as::<_, Track>(
        "SELECT id, path, title, artist, album, album_id, artist_id, \
         track_number, length, filesize, album_art, genre, year FROM track \
         WHERE artist_id = ? ORDER BY album, track_number, title",
    )
    .bind(artist_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn all_albums(pool: &Pool<Sqlite>) -> anyhow::Result<Vec<Album>> {
    let rows = sqlx::query_as::<_, Album>(
        "SELECT album_id AS id, album AS title, artist, album_art, COUNT(*) AS track_count \
         FROM track GROUP BY album_id ORDER BY artist, title",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn all_artists(pool: &Pool<Sqlite>) -> anyhow::Result<Vec<Artist>> {
    let rows = sqlx::query_as::<_, Artist>(
        "SELECT artist_id AS id, artist AS name, COUNT(*) AS track_count \
         FROM track GROUP BY artist_id ORDER BY name",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn count_tracks(pool: &Pool<Sqlite>) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM track")
        .fetch_one(pool)
        .await
        .unwrap_or(0)
}

pub async fn count_albums(pool: &Pool<Sqlite>) -> i64 {
    sqlx::query_scalar("SELECT COUNT(DISTINCT album_id) FROM track")
        .fetch_one(pool)
        .await
        .unwrap_or(0)
}

pub async fn count_artists(pool: &Pool<Sqlite>) -> i64 {
    sqlx::query_scalar("SELECT COUNT(DISTINCT artist_id) FROM track")
        .fetch_one(pool)
        .await
        .unwrap_or(0)
}
