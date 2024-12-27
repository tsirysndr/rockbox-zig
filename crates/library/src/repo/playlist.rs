use crate::entity::playlist::Playlist;
use sqlx::{Pool, Sqlite};

pub async fn save(pool: Pool<Sqlite>, playlist: Playlist) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO playlist (
            id,
            name,
            image,
            description,
            folder_id,
            created_at,
            updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
    "#,
    )
    .bind(&playlist.id)
    .bind(&playlist.name)
    .bind(&playlist.image)
    .bind(&playlist.description)
    .bind(&playlist.folder_id)
    .bind(playlist.created_at)
    .bind(playlist.updated_at)
    .execute(&pool)
    .await?;
    Ok(())
}

pub async fn find(pool: Pool<Sqlite>, id: &str) -> Result<Option<Playlist>, sqlx::Error> {
    sqlx::query_as::<_, Playlist>(r#"SELECT * FROM playlist WHERE id = $1"#)
        .bind(id)
        .fetch_optional(&pool)
        .await
}

pub async fn find_by_folder(
    pool: Pool<Sqlite>,
    folder_id: &str,
) -> Result<Vec<Playlist>, sqlx::Error> {
    sqlx::query_as::<_, Playlist>(
        r#"
            SELECT * FROM playlist WHERE folder_id = $1 ORDER BY name ASC   
         "#,
    )
    .bind(folder_id)
    .fetch_all(&pool)
    .await
}

pub async fn all(pool: Pool<Sqlite>) -> Result<Vec<Playlist>, sqlx::Error> {
    sqlx::query_as::<_, Playlist>(r#"SELECT * FROM playlist ORDER BY name ASC"#)
        .fetch_all(&pool)
        .await
}

pub async fn delete(pool: Pool<Sqlite>, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query(r#"DELETE FROM playlist WHERE id = $1"#)
        .bind(id)
        .execute(&pool)
        .await?;
    Ok(())
}

pub async fn update(pool: Pool<Sqlite>, playlist: Playlist) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE playlist SET
            name = $2,
            updated_at = $3,
            folder_id = $4
        WHERE id = $1
    "#,
    )
    .bind(&playlist.id)
    .bind(&playlist.name)
    .bind(chrono::Utc::now())
    .bind(&playlist.folder_id)
    .execute(&pool)
    .await?;
    Ok(())
}
