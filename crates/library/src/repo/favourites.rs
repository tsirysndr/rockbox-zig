use crate::entity::favourites::Favourites;
use sqlx::{Pool, Sqlite};

pub async fn save(pool: Pool<Sqlite>, favourite: Favourites) {}

pub async fn all(pool: Pool<Sqlite>) {}
