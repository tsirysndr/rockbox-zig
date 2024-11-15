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
    pub title: String,
    pub artists: Vec<Artist>,
    pub album: Option<Album>,
    pub track_number: Option<u32>,
    pub disc_number: u32,
    pub duration: Option<f32>,
}
