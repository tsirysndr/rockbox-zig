use crate::api::rockbox::v1alpha1::Track as RockboxTrack;

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

impl Into<RockboxTrack> for Track {
    fn into(self) -> RockboxTrack {
        RockboxTrack {
            id: self.id,
            title: self.title,
            artist: self.artist,
            album: self.album,
            length: self.duration as u32,
            track_number: self.track_number,
            album_artist: self.album_artist,
            album_art: self.album_art,
            ..Default::default()
        }
    }
}
