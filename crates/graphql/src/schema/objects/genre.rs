use super::{album::Album, artist::Artist, track::Track};
use async_graphql::*;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Genre {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub image: Option<String>,
    pub tracks: Vec<Track>,
    pub albums: Vec<Album>,
    pub artists: Vec<Artist>,
    pub track_count: i64,
}

#[Object]
impl Genre {
    async fn id(&self) -> &str {
        &self.id
    }

    async fn name(&self) -> &str {
        &self.name
    }

    async fn description(&self) -> Option<&str> {
        self.description.as_deref()
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

    async fn artists(&self) -> Vec<Artist> {
        self.artists.clone()
    }

    async fn track_count(&self) -> i64 {
        self.track_count
    }
}

impl From<rockbox_library::entity::genre::Genre> for Genre {
    fn from(g: rockbox_library::entity::genre::Genre) -> Self {
        Self {
            id: g.id,
            name: g.name,
            description: g.description,
            image: g.image,
            tracks: vec![],
            albums: vec![],
            artists: vec![],
            track_count: 0,
        }
    }
}
