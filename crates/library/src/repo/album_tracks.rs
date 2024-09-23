use crate::entity::album_tracks::AlbumTracks;
use sqlx::{Pool, Sqlite};

pub async fn save(pool: Pool<Sqlite>, album_track: AlbumTracks) {}

pub async fn find(pool: Pool<Sqlite>) {}
