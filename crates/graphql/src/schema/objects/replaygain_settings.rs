use async_graphql::*;
use rockbox_sys as rb;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct ReplaygainSettings {
    pub noclip: bool, // scale to prevent clips
    pub r#type: i32, // 0=track gain, 1=album gain, 2=track gain if shuffle is on, album gain otherwise, 4=off
    pub preamp: i32, // scale replaygained tracks by this
}

#[Object]
impl ReplaygainSettings {
    async fn noclip(&self) -> bool {
        self.noclip
    }

    async fn r#type(&self) -> i32 {
        self.r#type
    }

    async fn preamp(&self) -> i32 {
        self.preamp
    }
}

impl From<rb::types::user_settings::ReplaygainSettings> for ReplaygainSettings {
    fn from(settings: rb::types::user_settings::ReplaygainSettings) -> Self {
        Self {
            noclip: settings.noclip,
            r#type: settings.r#type,
            preamp: settings.preamp,
        }
    }
}
