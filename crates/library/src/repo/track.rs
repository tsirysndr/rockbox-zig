use crate::entity::track::Track;
use sqlx::{Error, Pool, Sqlite};

pub async fn save(pool: Pool<Sqlite>, track: Track) -> Result<(), Error> {
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
          album_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)
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
    .execute(&pool)
    .await {
        Ok(_) => {}
        Err(_e) => {
            // eprintln!("Error saving track: {:?}", e);
        }
    }
    Ok(())
}

pub async fn find(pool: Pool<Sqlite>, id: &str) -> Result<Option<Track>, Error> {
    let result: Option<Track> = sqlx::query_as("SELECT * FROM track WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await?;
    Ok(result)
}

pub async fn find_by_md5(pool: Pool<Sqlite>, md5: &str) -> Result<Option<Track>, Error> {
    let result: Option<Track> = sqlx::query_as("SELECT * FROM track WHERE md5 = $1")
        .bind(md5)
        .fetch_optional(&pool)
        .await?;
    Ok(result)
}

pub async fn all(pool: Pool<Sqlite>) -> Result<Vec<Track>, Error> {
    let result: Vec<Track> = sqlx::query_as("SELECT * FROM track ORDER BY title ASC")
        .fetch_all(&pool)
        .await?;
    Ok(result)
}
