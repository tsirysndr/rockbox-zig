use crate::entity::{album::Album, favourites::Favourites, track::Track};
use sqlx::{Pool, Sqlite};

pub async fn save(pool: Pool<Sqlite>, favourite: Favourites) -> Result<(), sqlx::Error> {
    if favourite.track_id.is_none() && favourite.album_id.is_none() {
        return Err(sqlx::Error::RowNotFound);
    }

    let results = sqlx::query(
        r#"
    SELECT * FROM favourites WHERE track_id = $1 OR album_id = $2
    "#,
    )
    .bind(&favourite.track_id)
    .bind(&favourite.album_id)
    .fetch_optional(&pool)
    .await?;

    if results.is_some() {
        return Ok(());
    }

    sqlx::query(
        r#"
    INSERT INTO favourites (
      id,
      track_id, 
      album_id,
      created_at
    )
    VALUES ($1, $2, $3, $4)
    "#,
    )
    .bind(&favourite.id)
    .bind(&favourite.track_id)
    .bind(&favourite.album_id)
    .bind(&favourite.created_at)
    .execute(&pool)
    .await?;
    Ok(())
}

pub async fn all_tracks(pool: Pool<Sqlite>) -> Result<Vec<Track>, sqlx::Error> {
    match sqlx::query_as::<_, Track>(
        r#"
    SELECT * FROM favourites LEFT JOIN track ON favourites.track_id = track.id WHERE favourites.track_id IS NOT NULL
    ORDER BY created_at DESC
    "#,
    )
    .fetch_all(&pool)
    .await
    {
        Ok(favourites) => Ok(favourites),
        Err(e) => {
            eprintln!("Error fetching favourites: {:?}", e);
            Err(e)
        }
    }
}

pub async fn all_albums(pool: Pool<Sqlite>) -> Result<Vec<Album>, sqlx::Error> {
    match sqlx::query_as::<_, Album>(
        r#"
    SELECT * FROM favourites LEFT JOIN album ON favourites.album_id = album.id WHERE favourites.album_id IS NOT NULL ORDER BY created_at DESC
    "#,
    )
    .fetch_all(&pool)
    .await
    {
        Ok(favourites) => Ok(favourites),
        Err(e) => {
            eprintln!("Error fetching favourites: {:?}", e);
            Err(e)
        }
    }
}

pub async fn delete(pool: Pool<Sqlite>, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
    DELETE FROM favourites WHERE track_id = $1 OR album_id = $2
    "#,
    )
    .bind(id)
    .bind(id)
    .execute(&pool)
    .await?;
    Ok(())
}
