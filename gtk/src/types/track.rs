#[derive(Debug, Default, Clone)]
pub struct Track {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: u64,
    pub track_number: u32,
    pub elapsed: u64,
    pub album_artist: String,
    pub album_art: Option<String>,
}
