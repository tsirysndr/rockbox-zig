#[derive(sqlx::FromRow, Default)]
pub struct AlbumTracks {
    pub id: String,
    pub album_id: String,
    pub track_id: String,
}
