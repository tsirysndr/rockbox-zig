use crate::entity::folder::Folder;
use sqlx::{Pool, Sqlite};

pub async fn save(pool: Pool<Sqlite>, folder: Folder) {}

pub async fn find(pool: Pool<Sqlite>) {}

pub async fn all(pool: Pool<Sqlite>) {}
