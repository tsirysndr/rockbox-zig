use sqlx::{Pool, Sqlite};
use crate::entity::artist_tracks::ArtistTracks;

pub async fn save(pool: Pool<Sqlite>, artist_tracks: ArtistTracks) {}

pub async fn all(pool: Pool<Sqlite>) {}
