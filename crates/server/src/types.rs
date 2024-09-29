use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct NewPlaylist {
    pub name: String,
    pub tracks: Vec<String>,
}
