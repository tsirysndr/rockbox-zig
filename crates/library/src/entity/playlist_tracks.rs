#[derive(sqlx::FromRow, Default)]
pub struct PlaylistTracks {
    pub id: String,
    pub playlist_id: String,
    pub track_id: String,
}
