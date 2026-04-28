use crate::entity::track::Track;
use sqlx::{Error, Pool, Sqlite};

pub async fn save(pool: Pool<Sqlite>, track: Track) -> Result<String, Error> {
    match sqlx::query(
        r#"
        INSERT INTO track (
          id, 
          path, 
          title, 
          artist,
          album,
          genre,
          year,
          track_number,
          disc_number,
          year_string,
          composer,
          album_artist,
          bitrate,
          frequency,
          filesize,
          length,
          md5,
          created_at,
          updated_at,
          artist_id,
          album_id,
          album_art,
          is_remote
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23)
        "#,
    )
    .bind(&track.id)
    .bind(&track.path)
    .bind(&track.title)
    .bind(&track.artist)
    .bind(&track.album)
    .bind(track.genre)
    .bind(track.year)
    .bind(track.track_number)
    .bind(track.disc_number)
    .bind(&track.year_string)
    .bind(&track.composer)
    .bind(&track.album_artist)
    .bind(track.bitrate)
    .bind(track.frequency)
    .bind(track.filesize)
    .bind(track.length)
    .bind(&track.md5)
    .bind(track.created_at)
    .bind(track.updated_at)
    .bind(&track.artist_id)
    .bind(&track.album_id)
    .bind(&track.album_art)
    .bind(track.is_remote)
    .execute(&pool)
    .await {
        Ok(_) => Ok(track.id.clone()),
        Err(_e) => {
            // eprintln!("Error saving track: {:?}", e);
            let track = find_by_md5(pool.clone(), &track.md5).await?;
            Ok(track.unwrap().id)
        }
    }
}

pub async fn find(pool: Pool<Sqlite>, id: &str) -> Result<Option<Track>, Error> {
    let result: Option<Track> = sqlx::query_as("SELECT * FROM track WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await?;
    Ok(result)
}

pub async fn filter(
    pool: Pool<Sqlite>,
    r#where: (String, Vec<String>),
) -> Result<Vec<Track>, Error> {
    let sql = format!("SELECT * FROM track WHERE is_remote = 0 AND {}", r#where.0);
    let mut query = sqlx::query_as(&sql);

    for value in r#where.1 {
        query = query.bind(value.clone());
    }

    let result = query.fetch_all(&pool).await?;
    Ok(result)
}

pub async fn find_by_md5(pool: Pool<Sqlite>, md5: &str) -> Result<Option<Track>, Error> {
    let result: Option<Track> = sqlx::query_as("SELECT * FROM track WHERE md5 = $1")
        .bind(md5)
        .fetch_optional(&pool)
        .await?;
    Ok(result)
}

pub async fn find_by_path(pool: Pool<Sqlite>, path: &str) -> Result<Option<Track>, Error> {
    let result: Option<Track> = sqlx::query_as("SELECT * FROM track WHERE path = $1")
        .bind(path)
        .fetch_optional(&pool)
        .await?;
    Ok(result)
}

pub async fn all(pool: Pool<Sqlite>) -> Result<Vec<Track>, Error> {
    let result: Vec<Track> =
        sqlx::query_as("SELECT * FROM track WHERE is_remote = 0 ORDER BY title ASC")
            .fetch_all(&pool)
            .await?;
    Ok(result)
}

pub async fn update_album_art(pool: Pool<Sqlite>, id: &str, album_art: &str) -> Result<(), Error> {
    sqlx::query("UPDATE track SET album_art = $2 WHERE id = $1")
        .bind(id)
        .bind(album_art)
        .execute(&pool)
        .await?;
    Ok(())
}

pub async fn find_by_artist(pool: Pool<Sqlite>, artist: &str) -> Result<Vec<Track>, Error> {
    let result: Vec<Track> = sqlx::query_as(
        "SELECT * FROM track WHERE is_remote = 0 AND artist = $1 ORDER BY title ASC",
    )
    .bind(artist)
    .fetch_all(&pool)
    .await?;
    Ok(result)
}

pub async fn find_by_album(pool: Pool<Sqlite>, album: &str) -> Result<Vec<Track>, Error> {
    let result: Vec<Track> =
        sqlx::query_as("SELECT * FROM track WHERE is_remote = 0 AND album = $1 ORDER BY title ASC")
            .bind(album)
            .fetch_all(&pool)
            .await?;
    Ok(result)
}

pub async fn find_by_title(pool: Pool<Sqlite>, title: &str) -> Result<Vec<Track>, Error> {
    let result: Vec<Track> =
        sqlx::query_as("SELECT * FROM track WHERE is_remote = 0 AND title = $1 ORDER BY title ASC")
            .bind(title)
            .fetch_all(&pool)
            .await?;
    Ok(result)
}

pub async fn find_by_filename(pool: Pool<Sqlite>, filename: &str) -> Result<Vec<Track>, Error> {
    let result: Vec<Track> =
        sqlx::query_as("SELECT * FROM track WHERE path = $1 ORDER BY title ASC")
            .bind(filename)
            .fetch_all(&pool)
            .await?;
    Ok(result)
}

pub async fn update_stream_metadata(
    pool: Pool<Sqlite>,
    md5: &str,
    title: &str,
    artist: &str,
    album: &str,
    length: u32,
) -> Result<(), Error> {
    sqlx::query(
        "UPDATE track SET title = $2, artist = $3, album = $4, length = $5, updated_at = $6 WHERE md5 = $1",
    )
    .bind(md5)
    .bind(title)
    .bind(artist)
    .bind(album)
    .bind(length)
    .bind(chrono::Utc::now())
    .execute(&pool)
    .await?;
    Ok(())
}

pub async fn find_by_artist_album_date(
    pool: Pool<Sqlite>,
    artist: &str,
    album: &str,
    date: &str,
) -> Result<Vec<Track>, Error> {
    let result: Vec<Track> = sqlx::query_as(
        "SELECT * FROM track WHERE is_remote = 0 AND artist = $1 AND album = $2 AND year_string = $3 ORDER BY title ASC",
    )
    .bind(artist)
    .bind(album)
    .bind(date)
    .fetch_all(&pool)
    .await?;
    Ok(result)
}
