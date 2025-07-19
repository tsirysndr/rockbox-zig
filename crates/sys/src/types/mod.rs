use serde::{Deserialize, Serialize};

pub mod audio_status;
pub mod file_position;
pub mod mp3_entry;
pub mod playlist_amount;
pub mod playlist_info;
pub mod playlist_track_info;
pub mod system_status;
pub mod tree;
pub mod user_settings;

#[derive(Serialize, Deserialize)]
pub struct RockboxVersion {
    pub version: String,
}
