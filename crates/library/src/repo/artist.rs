use crate::entity::artist::Artist;
use sqlx::{Error, Pool, Sqlite};

pub async fn save(pool: Pool<Sqlite>, artist: Artist) -> Result<String, Error> {
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
        Ok(_) => Ok(artist.id.clone()),
        Err(_e) => {
            // eprintln!("Error saving artist: {:?}", e);
            // get the artist by name and return the id
            let artist = find_by_name(pool.clone(), &artist.name).await?;
            Ok(artist.unwrap().id)
        }
    }
}

pub async fn find_by_name(pool: Pool<Sqlite>, name: &str) -> Result<Option<Artist>, Error> {
    match sqlx::query_as::<_, Artist>(
        r#"
        SELECT * FROM artist WHERE name = $1
        "#,
    )
    .bind(name)
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

pub async fn filter(
    pool: Pool<Sqlite>,
    r#where: (String, Vec<String>),
) -> Result<Vec<Artist>, Error> {
    let sql = format!("SELECT * FROM artist WHERE {}", r#where.0);
    let mut query = sqlx::query_as(&sql);

    for value in r#where.1 {
        query = query.bind(value.clone());
    }

    let result = query.fetch_all(&pool).await?;
    Ok(result)
}

pub async fn all(pool: Pool<Sqlite>) -> Result<Vec<Artist>, Error> {
    match sqlx::query_as::<_, Artist>(
        r#"
        SELECT * FROM artist ORDER BY name ASC
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
