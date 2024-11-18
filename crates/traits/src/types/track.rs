#[derive(Debug, Clone, Default)]
pub struct Artist {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Default)]
pub struct Album {
    pub id: String,
    pub title: String,
    pub cover: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Track {
    pub id: String,
    pub uri: String,
    pub path: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub track_number: Option<u32>,
    pub disc_number: u32,
    pub duration: Option<f32>,
    pub album_artist: Option<String>,
    pub album_cover: Option<String>,
    pub album_id: Option<String>,
    pub artist_id: Option<String>,
}
