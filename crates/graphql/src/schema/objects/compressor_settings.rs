use async_graphql::*;
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
