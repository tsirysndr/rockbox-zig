use async_graphql::*;
use serde::{Deserialize, Serialize};

use super::{album::Album, artist::Artist, track::Track};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    pub artists: Vec<Artist>,
    pub albums: Vec<Album>,
    pub tracks: Vec<Track>,
    pub liked_tracks: Vec<Track>,
    pub liked_albums: Vec<Album>,
}

#[Object]
impl SearchResults {
    async fn artists(&self) -> Vec<Artist> {
        self.artists.clone()
    }

    async fn albums(&self) -> Vec<Album> {
        self.albums.clone()
    }

    async fn tracks(&self) -> Vec<Track> {
        self.tracks.clone()
    }

    async fn liked_tracks(&self) -> Vec<Track> {
        self.liked_tracks.clone()
    }

    async fn liked_albums(&self) -> Vec<Album> {
        self.liked_albums.clone()
    }
}

impl From<rockbox_types::SearchResults> for SearchResults {
    fn from(results: rockbox_types::SearchResults) -> Self {
        SearchResults {
            artists: results.artists.into_iter().map(Into::into).collect(),
            albums: results.albums.into_iter().map(Into::into).collect(),
            tracks: results.tracks.into_iter().map(Into::into).collect(),
            liked_tracks: results.liked_tracks.into_iter().map(Into::into).collect(),
            liked_albums: results.liked_albums.into_iter().map(Into::into).collect(),
        }
    }
}
