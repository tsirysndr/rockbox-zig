use crate::entity::playlist::Playlist;
use sqlx::{Pool, Sqlite};

pub async fn save(pool: Pool<Sqlite>, playlist: Playlist) -> Result<String, sqlx::Error> {
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
    Ok(playlist.id)
}

pub async fn find(pool: Pool<Sqlite>, id: &str) -> Result<Option<Playlist>, sqlx::Error> {
    sqlx::query_as::<_, Playlist>(r#"SELECT * FROM playlist WHERE id = $1"#)
        .bind(id)
        .fetch_optional(&pool)
        .await
}

pub async fn find_by_folder(
    pool: Pool<Sqlite>,
    folder_id: Option<String>,
) -> Result<Vec<Playlist>, sqlx::Error> {
    sqlx::query_as::<_, Playlist>(
        r#"
            SELECT * FROM playlist WHERE folder_id IS $1 ORDER BY name ASC   
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
    let name = match playlist.name.is_empty() {
        true => None,
        false => Some(&playlist.name),
    };
    sqlx::query(
        r#"
        UPDATE playlist SET
            name = $2,
            description = $3,
            image = $4,
            updated_at = $5,
            folder_id = $6
        WHERE id = $1
    "#,
    )
    .bind(&playlist.id)
    .bind(name)
    .bind(playlist.description)
    .bind(playlist.image)
    .bind(chrono::Utc::now())
    .bind(&playlist.folder_id)
    .execute(&pool)
    .await?;
    Ok(())
}
