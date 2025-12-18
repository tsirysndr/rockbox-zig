#[derive(sqlx::FromRow, Default)]
pub struct ArtistGenres {
    pub id: String,
    pub artist_id: String,
    pub genre_id: String,
}
