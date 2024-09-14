use async_graphql::*;
use rockbox_sys as rb;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct CompressorSettings {
    pub threshold: i32,
    pub makeup_gain: i32,
    pub ratio: i32,
    pub knee: i32,
    pub release_time: i32,
    pub attack_time: i32,
}

#[Object]
impl CompressorSettings {
    async fn threshold(&self) -> i32 {
        self.threshold
    }

    async fn makeup_gain(&self) -> i32 {
        self.makeup_gain
    }

    async fn ratio(&self) -> i32 {
        self.ratio
    }

    async fn knee(&self) -> i32 {
        self.knee
    }

    async fn release_time(&self) -> i32 {
        self.release_time
    }

    async fn attack_time(&self) -> i32 {
        self.attack_time
    }
}

impl From<rb::types::user_settings::CompressorSettings> for CompressorSettings {
    fn from(settings: rb::types::user_settings::CompressorSettings) -> Self {
        Self {
            threshold: settings.threshold,
            makeup_gain: settings.makeup_gain,
            ratio: settings.ratio,
            knee: settings.knee,
            release_time: settings.release_time,
            attack_time: settings.attack_time,
        }
    }
}
