use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct NewPlaylist {
    pub name: String,
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
pub struct DeleteTracks {
    pub positions: Vec<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusCode {
    pub code: i32,
}
