use crate::entity::playlist_tracks::PlaylistTracks;
use sqlx::{Pool, Sqlite};

pub async fn save(pool: Pool<Sqlite>, playlist_track: PlaylistTracks) {}

pub async fn find(pool: Pool<Sqlite>) {}
