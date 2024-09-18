use serde::{Deserialize, Serialize};

pub mod audio_status;
pub mod mp3_entry;
pub mod system_status;
pub mod user_settings;
pub mod file_position;

#[derive(Serialize, Deserialize)]
pub struct RockboxVersion {
    pub version: String,
}
