use async_graphql::*;
use serde::{Deserialize, Serialize};

use super::{album::Album, track::Track};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Artist {
    pub id: String,
    pub name: String,
    pub bio: Option<String>,
    pub image: Option<String>,
    pub tracks: Vec<Track>,
    pub albums: Vec<Album>,
}

#[Object]
impl Artist {
    async fn id(&self) -> &str {
        &self.id
    }

    async fn name(&self) -> &str {
        &self.name
    }

    async fn bio(&self) -> Option<&str> {
        self.bio.as_deref()
    }

    async fn image(&self) -> Option<&str> {
        self.image.as_deref()
    }

    async fn tracks(&self) -> Vec<Track> {
        self.tracks.clone()
    }

    async fn albums(&self) -> Vec<Album> {
        self.albums.clone()
    }
}

impl From<rockbox_library::entity::artist::Artist> for Artist {
    fn from(artist: rockbox_library::entity::artist::Artist) -> Self {
        Self {
            id: artist.id,
            name: artist.name,
            bio: artist.bio,
            image: artist.image,
            tracks: vec![],
            albums: vec![],
        }
    }
}

impl From<rockbox_search::artist::Artist> for Artist {
    fn from(artist: rockbox_search::artist::Artist) -> Self {
        Self {
            id: artist.id,
            name: artist.name,
            bio: artist.bio,
            image: artist.image,
            tracks: vec![],
            albums: vec![],
        }
    }
}
