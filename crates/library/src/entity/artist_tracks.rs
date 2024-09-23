#[derive(sqlx::FromRow, Default)]
pub struct ArtistTracks {
    pub id: String,
    pub artist_id: String,
    pub track_id: String,
}
