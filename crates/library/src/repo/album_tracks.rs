use crate::entity::{album_tracks::AlbumTracks, track::Track};
use sqlx::{Pool, Sqlite};

pub async fn save(pool: Pool<Sqlite>, album_track: AlbumTracks) -> Result<(), sqlx::Error> {
    let results = sqlx::query(
        r#"
        SELECT * FROM album_tracks WHERE album_id = $1 AND track_id = $2
        "#,
    )
    .bind(&album_track.album_id)
    .bind(&album_track.track_id)
    .fetch_optional(&pool)
    .await?;

    if results.is_some() {
        return Ok(());
    }

    match sqlx::query(
        r#"
        INSERT INTO album_tracks (
          id,
          album_id, 
          track_id
        )
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(&album_track.id)
    .bind(&album_track.album_id)
    .bind(&album_track.track_id)
    .execute(&pool)
    .await
    {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Error saving album track: {:?}", e);
            Err(e)
        }
    }
}

pub async fn find_by_album(pool: Pool<Sqlite>, album_id: &str) -> Result<Vec<Track>, sqlx::Error> {
    match sqlx::query_as::<_, Track>(
        r#"
        SELECT * FROM album_tracks
        LEFT JOIN track ON album_tracks.track_id = track.id
        WHERE album_tracks.album_id = $1 
        ORDER BY disc_number, track_number ASC
        "#,
    )
    .bind(album_id)
    .fetch_all(&pool)
    .await
    {
        Ok(album_tracks) => Ok(album_tracks),
        Err(e) => {
            eprintln!("Error finding album tracks: {:?}", e);
            Err(e)
        }
    }
}
