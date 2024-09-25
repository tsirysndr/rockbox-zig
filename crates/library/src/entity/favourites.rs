#[derive(sqlx::FromRow, Default)]
pub struct Favourites {
    pub id: String,
    pub track_id: String,
    pub created_at: i64,
}
