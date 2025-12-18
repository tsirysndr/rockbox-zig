use crate::entity::genre::Genre;
use sqlx::{Error, Pool, Sqlite};

pub async fn save(pool: &Pool<Sqlite>, id: &str, name: &str) -> Result<(), Error> {
    match sqlx::query(
        r#"
        INSERT OR IGNORE INTO genre (id, name)
        VALUES ($1, $2)
        "#,
    )
    .bind(id)
    .bind(name)
    .execute(pool)
    .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(e.into()),
    }
}

pub async fn all(pool: Pool<Sqlite>) -> Result<Vec<Genre>, Error> {
    match sqlx::query_as(
        r#"
        SELECT * FROM genre
        "#,
    )
    .fetch_all(&pool)
    .await
    {
        Ok(genres) => Ok(genres),
        Err(e) => Err(e.into()),
    }
}
