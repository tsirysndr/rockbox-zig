use crate::entity::playlist::Playlist;
use sqlx::{Pool, Sqlite};

pub async fn save(pool: Pool<Sqlite>, playlist: Playlist) {}

pub async fn find(pool: Pool<Sqlite>) {}

pub async fn all(pool: Pool<Sqlite>) {}
