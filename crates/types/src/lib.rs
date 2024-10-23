use rockbox_search::rockbox::search::v1alpha1::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct NewPlaylist {
    pub name: Option<String>,
    pub tracks: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InsertTracks {
    pub position: i32,
    pub tracks: Vec<String>,
    pub directory: Option<String>,
    pub shuffle: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewVolume {
    pub steps: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteTracks {
    pub positions: Vec<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusCode {
    pub code: i32,
}

#[derive(Default, Serialize, Deserialize)]
pub struct SearchResults {
    pub artists: Vec<Artist>,
    pub albums: Vec<Album>,
    pub tracks: Vec<Track>,
    pub liked_tracks: Vec<LikedTrack>,
    pub liked_albums: Vec<LikedAlbum>,
    pub files: Vec<File>,
}
