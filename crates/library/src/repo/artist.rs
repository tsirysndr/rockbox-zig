use crate::entity::artist::Artist;
use sqlx::{Error, Pool, Sqlite};

pub async fn save(pool: Pool<Sqlite>, artist: Artist) -> Result<(), Error> {
    match sqlx::query(
        r#"
        INSERT INTO artist (
          id, 
          name,
          bio,
          image
        )
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(&artist.id)
    .bind(&artist.name)
    .bind(&artist.bio)
    .bind(&artist.image)
    .execute(&pool)
    .await
    {
        Ok(_) => {}
        Err(_e) => {
            // eprintln!("Error saving artist: {:?}", e);
        }
    }
    Ok(())
}

pub async fn find(pool: Pool<Sqlite>, id: &str) -> Result<Option<Artist>, Error> {
    match sqlx::query_as::<_, Artist>(
        r#"
        SELECT * FROM artist WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    {
        Ok(artist) => Ok(artist),
        Err(e) => {
            eprintln!("Error finding artist: {:?}", e);
            Err(e)
        }
    }
}

pub async fn all(pool: Pool<Sqlite>) -> Result<Vec<Artist>, Error> {
    match sqlx::query_as::<_, Artist>(
        r#"
        SELECT * FROM artist
        "#,
    )
    .fetch_all(&pool)
    .await
    {
        Ok(artists) => Ok(artists),
        Err(e) => {
            eprintln!("Error finding artists: {:?}", e);
            Err(e)
        }
    }
}
