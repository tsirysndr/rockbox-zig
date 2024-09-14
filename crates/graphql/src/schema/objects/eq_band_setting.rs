use async_graphql::*;
use rockbox_sys as rb;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct EqBandSetting {
    pub cutoff: i32, // Hz
    pub q: i32,
    pub gain: i32, // +/- dB
}

#[Object]
impl EqBandSetting {
    async fn cutoff(&self) -> i32 {
        self.cutoff
    }

    async fn q(&self) -> i32 {
        self.q
    }

    async fn gain(&self) -> i32 {
        self.gain
    }
}

impl From<rb::types::user_settings::EqBandSetting> for EqBandSetting {
    fn from(setting: rb::types::user_settings::EqBandSetting) -> Self {
        Self {
            cutoff: setting.cutoff,
            q: setting.q,
            gain: setting.gain,
        }
    }
}
