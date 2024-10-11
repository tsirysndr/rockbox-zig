use crate::entity::{artist_tracks::ArtistTracks, track::Track};
use sqlx::{Pool, Sqlite};

pub async fn save(pool: Pool<Sqlite>, artist_track: ArtistTracks) -> Result<(), sqlx::Error> {
    let results = sqlx::query(
        r#"
        SELECT * FROM artist_tracks WHERE artist_id = $1 AND track_id = $2
        "#,
    )
    .bind(&artist_track.artist_id)
    .bind(&artist_track.track_id)
    .fetch_optional(&pool)
    .await?;

    if results.is_some() {
        return Ok(());
    }

    match sqlx::query(
        r#"
        INSERT INTO artist_tracks (
          id,
          artist_id, 
          track_id
        )
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(&artist_track.id)
    .bind(&artist_track.artist_id)
    .bind(&artist_track.track_id)
    .execute(&pool)
    .await
    {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Error saving artist track: {:?}", e);
            Err(e)
        }
    }
}

pub async fn find_by_artist(
    pool: Pool<Sqlite>,
    artist_id: &str,
) -> Result<Vec<Track>, sqlx::Error> {
    match sqlx::query_as::<_, Track>(
        r#"
        SELECT * FROM artist_tracks
        LEFT JOIN track ON artist_tracks.track_id = track.id
        WHERE artist_tracks.artist_id = $1
        ORDER BY title ASC
        "#,
    )
    .bind(artist_id)
    .fetch_all(&pool)
    .await
    {
        Ok(artist_tracks) => Ok(artist_tracks),
        Err(e) => {
            eprintln!("Error finding artist tracks: {:?}", e);
            Err(e)
        }
    }
}
