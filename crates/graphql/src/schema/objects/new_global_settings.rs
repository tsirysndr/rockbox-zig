use async_graphql::*;
use serde::{Deserialize, Serialize};

use super::{eq_band_setting::EqBandSettingInput, replaygain_settings::ReplaygainSettingsInput};

#[derive(Default, Serialize, Deserialize, InputObject)]
pub struct NewGlobalSettings {
    pub music_dir: Option<String>,
    pub playlist_shuffle: Option<bool>,
    pub repeat_mode: Option<i32>,
    pub bass: Option<i32>,
    pub treble: Option<i32>,
    pub bass_cutoff: Option<i32>,
    pub treble_cutoff: Option<i32>,
    pub crossfade: Option<i32>,
    pub fade_on_stop: Option<bool>,
    pub fade_in_delay: Option<i32>,
    pub fade_in_duration: Option<i32>,
    pub fade_out_delay: Option<i32>,
    pub fade_out_duration: Option<i32>,
    pub fade_out_mixmode: Option<i32>,
    pub balance: Option<i32>,
    pub stereo_width: Option<i32>,
    pub stereosw_mode: Option<i32>,
    pub surround_enabled: Option<bool>,
    pub surround_balance: Option<i32>,
    pub surround_fx1: Option<i32>,
    pub surround_fx2: Option<i32>,
    pub party_mode: Option<bool>,
    pub channel_config: Option<i32>,
    pub player_name: Option<String>,
    pub eq_enabled: Option<bool>,
    pub eq_band_settings: Option<Vec<EqBandSettingInput>>,
    pub replaygain_settings: Option<ReplaygainSettingsInput>,
}
