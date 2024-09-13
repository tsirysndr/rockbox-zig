use serde::{Deserialize, Serialize};

pub mod mp3_entry;
pub mod system_status;
pub mod user_settings;

#[derive(Serialize, Deserialize)]
pub struct RockboxVersion {
    pub version: String,
}
