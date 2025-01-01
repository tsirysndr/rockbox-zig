use crate::entity::{playlist_tracks::PlaylistTracks, track::Track};
use sqlx::{Pool, Sqlite};

pub async fn save(pool: Pool<Sqlite>, playlist_track: PlaylistTracks) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO playlist_tracks (
            id,
            playlist_id,
            track_id,
            position,
            created_at
        )
        VALUES ($1, $2, $3, $4, $5)
    "#,
    )
    .bind(&playlist_track.id)
    .bind(&playlist_track.playlist_id)
    .bind(&playlist_track.track_id)
    .bind(playlist_track.position)
    .bind(playlist_track.created_at)
    .execute(&pool)
    .await?;
    Ok(())
}

pub async fn find_by_playlist(
    pool: Pool<Sqlite>,
    playlist_id: &str,
) -> Result<Vec<Track>, sqlx::Error> {
    match sqlx::query_as::<_, Track>(
        r#"
        SELECT * FROM playlist_tracks
        LEFT JOIN track ON playlist_tracks.track_id = track.id
        WHERE playlist_tracks.playlist_id = $1
        ORDER BY playlist_tracks.position ASC
        "#,
    )
    .bind(playlist_id)
    .fetch_all(&pool)
    .await
    {
        Ok(playlist_tracks) => Ok(playlist_tracks),
        Err(e) => {
            eprintln!("Error finding playlist tracks: {:?}", e);
            Err(e)
        }
    }
}

pub async fn delete(pool: Pool<Sqlite>, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query(r#"DELETE FROM playlist_tracks WHERE id = $1"#)
        .bind(id)
        .execute(&pool)
        .await?;
    Ok(())
}

pub async fn delete_by_playlist(pool: Pool<Sqlite>, playlist_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query(r#"DELETE FROM playlist_tracks WHERE playlist_id = $1"#)
        .bind(playlist_id)
        .execute(&pool)
        .await?;
    Ok(())
}

pub async fn delete_track_at(
    pool: Pool<Sqlite>,
    playlist_id: &str,
    position: u32,
) -> Result<(), sqlx::Error> {
    sqlx::query(r#"DELETE FROM playlist_tracks WHERE playlist_id = $1 AND position = $2"#)
        .bind(playlist_id)
        .bind(position)
        .execute(&pool)
        .await?;

    let tracks = sqlx::query_as::<_, Track>(
        r#"
        SELECT * FROM playlist_tracks
        LEFT JOIN track ON playlist_tracks.track_id = track.id
        WHERE playlist_tracks.playlist_id = $1
        ORDER BY playlist_tracks.created_at ASC
        "#,
    )
    .bind(playlist_id)
    .fetch_all(&pool)
    .await?;

    for (i, track) in tracks.iter().enumerate() {
        sqlx::query(
            r#"UPDATE playlist_tracks SET position = $1 WHERE playlist_id = $2 AND track_id = $3"#,
        )
        .bind(i as u32)
        .bind(playlist_id)
        .bind(&track.id)
        .execute(&pool)
        .await?;
    }

    Ok(())
}
