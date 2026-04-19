use serde::{Deserialize, Serialize};

pub mod device;

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
pub struct LoadTracks {
    pub tracks: Vec<String>,
    pub directory: Option<String>,
    pub album_id: Option<String>,
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
    pub artists: Vec<rockbox_typesense::types::Artist>,
    pub albums: Vec<rockbox_typesense::types::Album>,
    pub tracks: Vec<rockbox_typesense::types::Track>,
    pub liked_tracks: Vec<rockbox_typesense::types::Track>,
    pub liked_albums: Vec<rockbox_typesense::types::Album>,
}

#[derive(Default, Serialize, Deserialize)]
pub struct EqBandSetting {
    pub cutoff: i32,
    pub q: i32,
    pub gain: i32,
}

#[derive(Default, Serialize, Deserialize)]
pub struct ReplaygainSettings {
    pub enabled: bool,
    pub preamp: i32,
    pub peak: i32,
    pub clip: i32,
}
