use crate::entity::folder::Folder;
use sqlx::{types::chrono, Pool, Sqlite};

pub async fn save(pool: Pool<Sqlite>, folder: Folder) -> Result<String, sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO folder (
            id,
            name,
            created_at,
            updated_at
        )
        VALUES ($1, $2, $3, $4)
    "#,
    )
    .bind(&folder.id)
    .bind(&folder.name)
    .bind(folder.created_at)
    .bind(folder.updated_at)
    .execute(&pool)
    .await?;
    Ok(folder.id)
}

pub async fn find(pool: Pool<Sqlite>, id: &str) -> Result<Option<Folder>, sqlx::Error> {
    sqlx::query_as::<_, Folder>(r#"SELECT * FROM folder WHERE id = $1"#)
        .bind(id)
        .fetch_optional(&pool)
        .await
}

pub async fn find_by_parent(
    pool: Pool<Sqlite>,
    parent_id: Option<String>,
) -> Result<Vec<Folder>, sqlx::Error> {
    sqlx::query_as::<_, Folder>(r#"SELECT * FROM folder WHERE parent_id IS $1 ORDER BY name ASC"#)
        .bind(parent_id)
        .fetch_all(&pool)
        .await
}

pub async fn all(pool: Pool<Sqlite>) -> Result<Vec<Folder>, sqlx::Error> {
    sqlx::query_as::<_, Folder>(r#"SELECT * FROM folder ORDER BY name ASC"#)
        .fetch_all(&pool)
        .await
}

pub async fn delete(pool: Pool<Sqlite>, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query(r#"DELETE FROM folder WHERE id = $1"#)
        .bind(id)
        .execute(&pool)
        .await?;
    Ok(())
}

pub async fn update(pool: Pool<Sqlite>, folder: Folder) -> Result<(), sqlx::Error> {
    let name = match folder.name.is_empty() {
        true => None,
        false => Some(&folder.name),
    };
    sqlx::query(
        r#"
            UPDATE folder SET
                name = $2,
                updated_at = $3,
                parent_id = $4
            WHERE id = $1
        "#,
    )
    .bind(&folder.id)
    .bind(name)
    .bind(chrono::Utc::now())
    .bind(&folder.parent_id)
    .execute(&pool)
    .await?;
    Ok(())
}
