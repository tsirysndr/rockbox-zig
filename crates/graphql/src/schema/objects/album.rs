use async_graphql::*;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Album {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub year: u32,
    pub year_string: String,
    pub album_art: Option<String>,
    pub md5: String,
}

#[Object]
impl Album {
    async fn id(&self) -> &str {
        &self.id
    }

    async fn title(&self) -> &str {
        &self.title
    }

    async fn artist(&self) -> &str {
        &self.artist
    }

    async fn year(&self) -> i32 {
        self.year as i32
    }

    async fn year_string(&self) -> &str {
        &self.year_string
    }

    async fn album_art(&self) -> Option<&str> {
        self.album_art.as_deref()
    }

    async fn md5(&self) -> &str {
        &self.md5
    }
}

impl From<rockbox_library::entity::album::Album> for Album {
    fn from(album: rockbox_library::entity::album::Album) -> Self {
        Self {
            id: album.id,
            title: album.title,
            artist: album.artist,
            year: album.year,
            year_string: album.year_string,
            album_art: album.album_art,
            md5: album.md5,
        }
    }
}
