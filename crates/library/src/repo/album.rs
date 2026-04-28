use crate::entity::album::Album;
use sqlx::{Pool, Sqlite};
use tracing::warn;

pub async fn save(pool: Pool<Sqlite>, album: Album) -> Result<String, sqlx::Error> {
    match sqlx::query(
        r#"
        INSERT INTO album (
          id,
          title,
          artist,
          year,
          year_string,
          album_art,
          md5,
          artist_id,
          label,
          copyright_message
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#,
    )
    .bind(&album.id)
    .bind(&album.title)
    .bind(&album.artist)
    .bind(album.year)
    .bind(&album.year_string)
    .bind(&album.album_art)
    .bind(&album.md5)
    .bind(&album.artist_id)
    .bind(album.label)
    .bind(album.copyright_message)
    .execute(&pool)
    .await
    {
        Ok(_) => Ok(album.id.clone()),
        Err(_e) => {
            // eprintln!("Error saving album: {:?}", e);
            let album = find_by_md5(pool.clone(), &album.md5).await?;
            Ok(album.unwrap().id)
        }
    }
}

pub async fn filter(
    pool: Pool<Sqlite>,
    r#where: (String, Vec<String>),
) -> Result<Vec<Album>, sqlx::Error> {
    let sql = format!("SELECT * FROM album WHERE {}", r#where.0);
    let mut query = sqlx::query_as(&sql);

    for value in r#where.1 {
        query = query.bind(value.clone());
    }

    let result = query.fetch_all(&pool).await?;
    Ok(result)
}

pub async fn find_by_md5(pool: Pool<Sqlite>, md5: &str) -> Result<Option<Album>, sqlx::Error> {
    match sqlx::query_as::<_, Album>(
        r#"
        SELECT * FROM album WHERE md5 = $1
        "#,
    )
    .bind(md5)
    .fetch_optional(&pool)
    .await
    {
        Ok(album) => Ok(album),
        Err(e) => {
            warn!("Error finding album: {:?}", e);
            Err(e)
        }
    }
}

pub async fn find_by_artist(
    pool: Pool<Sqlite>,
    artist_id: &str,
) -> Result<Vec<Album>, sqlx::Error> {
    match sqlx::query_as::<_, Album>(
        r#"
        SELECT * FROM album WHERE artist_id = $1 AND EXISTS (
          SELECT 1 FROM track WHERE track.album_id = album.id AND track.is_remote = 0
        ) ORDER BY title ASC
        "#,
    )
    .bind(artist_id)
    .fetch_all(&pool)
    .await
    {
        Ok(albums) => Ok(albums),
        Err(e) => {
            warn!("Error finding albums: {:?}", e);
            Err(e)
        }
    }
}

pub async fn find(pool: Pool<Sqlite>, id: &str) -> Result<Option<Album>, sqlx::Error> {
    match sqlx::query_as::<_, Album>(
        r#"
        SELECT * FROM album WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    {
        Ok(album) => Ok(album),
        Err(e) => {
            warn!("Error finding album: {:?}", e);
            Err(e)
        }
    }
}

pub async fn all(pool: Pool<Sqlite>) -> Result<Vec<Album>, sqlx::Error> {
    match sqlx::query_as::<_, Album>(
        r#"
        SELECT * FROM album WHERE EXISTS (
          SELECT 1 FROM track WHERE track.album_id = album.id AND track.is_remote = 0
        ) ORDER BY title ASC
        "#,
    )
    .fetch_all(&pool)
    .await
    {
        Ok(albums) => Ok(albums),
        Err(e) => {
            warn!("Error finding albums: {:?}", e);
            Err(e)
        }
    }
}

pub async fn update_album_art(
    pool: Pool<Sqlite>,
    id: &str,
    album_art: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE album SET album_art = $2 WHERE id = $1")
        .bind(id)
        .bind(album_art)
        .execute(&pool)
        .await?;
    Ok(())
}
