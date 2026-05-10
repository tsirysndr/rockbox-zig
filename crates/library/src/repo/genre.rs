use crate::entity::album::Album;
use crate::entity::artist::Artist;
use crate::entity::genre::Genre;
use crate::entity::track::Track;
use sqlx::{Error, Pool, Sqlite};

// Track-genre association is derived through the artist: a track belongs to
// every genre that its artist has been tagged with in `artist_genres`. The
// per-track `track.genre_id` column is intentionally unused here — most files
// don't ship a clean genre tag, and the artist-level enrichment from the
// Rocksky API gives much better coverage.

pub async fn save(pool: &Pool<Sqlite>, id: &str, name: &str) -> Result<String, Error> {
    sqlx::query(
        r#"
        INSERT INTO genre (id, name)
        VALUES ($1, $2)
        ON CONFLICT(name) DO NOTHING
        "#,
    )
    .bind(id)
    .bind(name)
    .execute(pool)
    .await?;

    let row: (String,) = sqlx::query_as("SELECT id FROM genre WHERE name = $1")
        .bind(name)
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}

pub async fn all(pool: Pool<Sqlite>) -> Result<Vec<Genre>, Error> {
    sqlx::query_as::<_, Genre>(
        r#"
        SELECT DISTINCT g.* FROM genre g
        INNER JOIN artist_genres ag ON ag.genre_id = g.id
        INNER JOIN track t ON t.artist_id = ag.artist_id
        WHERE t.is_remote = 0
        ORDER BY g.name ASC
        "#,
    )
    .fetch_all(&pool)
    .await
}

pub async fn find(pool: Pool<Sqlite>, id: &str) -> Result<Option<Genre>, Error> {
    sqlx::query_as::<_, Genre>(
        r#"
        SELECT * FROM genre WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
}

pub async fn find_by_name(pool: Pool<Sqlite>, name: &str) -> Result<Option<Genre>, Error> {
    sqlx::query_as::<_, Genre>(
        r#"
        SELECT * FROM genre WHERE name = $1
        "#,
    )
    .bind(name)
    .fetch_optional(&pool)
    .await
}

pub async fn find_tracks(pool: Pool<Sqlite>, genre_id: &str) -> Result<Vec<Track>, Error> {
    sqlx::query_as::<_, Track>(
        r#"
        SELECT DISTINCT t.* FROM track t
        INNER JOIN artist_genres ag ON ag.artist_id = t.artist_id
        WHERE ag.genre_id = $1 AND t.is_remote = 0
        ORDER BY t.title ASC
        "#,
    )
    .bind(genre_id)
    .fetch_all(&pool)
    .await
}

pub async fn find_albums(pool: Pool<Sqlite>, genre_id: &str) -> Result<Vec<Album>, Error> {
    sqlx::query_as::<_, Album>(
        r#"
        SELECT DISTINCT a.* FROM album a
        INNER JOIN track t ON t.album_id = a.id
        INNER JOIN artist_genres ag ON ag.artist_id = t.artist_id
        WHERE ag.genre_id = $1 AND t.is_remote = 0
        ORDER BY a.title ASC
        "#,
    )
    .bind(genre_id)
    .fetch_all(&pool)
    .await
}

pub async fn find_artists(pool: Pool<Sqlite>, genre_id: &str) -> Result<Vec<Artist>, Error> {
    sqlx::query_as::<_, Artist>(
        r#"
        SELECT DISTINCT ar.* FROM artist ar
        INNER JOIN artist_genres ag ON ag.artist_id = ar.id
        INNER JOIN track t ON t.artist_id = ar.id
        WHERE ag.genre_id = $1 AND t.is_remote = 0
        ORDER BY ar.name ASC
        "#,
    )
    .bind(genre_id)
    .fetch_all(&pool)
    .await
}

pub async fn update_picture(pool: &Pool<Sqlite>, id: &str, image: &str) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE genre SET image = $2 WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(image)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_description(
    pool: &Pool<Sqlite>,
    id: &str,
    description: &str,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE genre SET description = $2 WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(description)
    .execute(pool)
    .await?;
    Ok(())
}
