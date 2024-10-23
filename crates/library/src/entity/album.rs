use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Default, Serialize, Deserialize, Clone)]
pub struct Album {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub year: u32,
    pub year_string: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album_art: Option<String>,
    pub md5: String,
    pub artist_id: String,
}

impl Into<rockbox_search::rockbox::search::v1alpha1::Album> for Album {
    fn into(self) -> rockbox_search::rockbox::search::v1alpha1::Album {
        rockbox_search::rockbox::search::v1alpha1::Album {
            id: self.id,
            title: self.title,
            artist: self.artist,
            year: self.year,
            year_string: self.year_string,
            album_art: self.album_art,
            md5: self.md5,
            artist_id: self.artist_id,
        }
    }
}

impl Into<rockbox_search::rockbox::search::v1alpha1::LikedAlbum> for Album {
    fn into(self) -> rockbox_search::rockbox::search::v1alpha1::LikedAlbum {
        rockbox_search::rockbox::search::v1alpha1::LikedAlbum {
            id: self.id,
            title: self.title,
            artist: self.artist,
            year: self.year,
            year_string: self.year_string,
            album_art: self.album_art,
            md5: self.md5,
            artist_id: self.artist_id,
        }
    }
}
