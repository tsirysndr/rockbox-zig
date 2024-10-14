use async_graphql::*;
use rockbox_sys as rb;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct AudioStatus {
    pub status: i32,
}

#[Object]
impl AudioStatus {
    async fn status(&self) -> i32 {
        self.status
    }
}

impl From<rb::types::audio_status::AudioStatus> for AudioStatus {
    fn from(status: rb::types::audio_status::AudioStatus) -> Self {
        Self {
            status: status.status,
        }
    }
}
