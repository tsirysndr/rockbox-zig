use crate::entity::album::Album;
use sqlx::{Pool, Sqlite};

pub async fn save(pool: Pool<Sqlite>, album: Album) -> Result<(), sqlx::Error> {
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
          artist_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
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
    .execute(&pool)
    .await
    {
        Ok(_) => {}
        Err(_e) => {
            // eprintln!("Error saving album: {:?}", e);
        }
    }
    Ok(())
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
            eprintln!("Error finding album: {:?}", e);
            Err(e)
        }
    }
}

pub async fn all(pool: Pool<Sqlite>) -> Result<Vec<Album>, sqlx::Error> {
    match sqlx::query_as::<_, Album>(
        r#"
        SELECT * FROM album ORDER BY title ASC
        "#,
    )
    .fetch_all(&pool)
    .await
    {
        Ok(albums) => Ok(albums),
        Err(e) => {
            eprintln!("Error finding albums: {:?}", e);
            Err(e)
        }
    }
}
