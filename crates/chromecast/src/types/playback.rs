use crate::types::track::Track;

#[derive(Debug, Clone, Default)]
pub struct Playback {
    pub current_track: Option<Track>,
    pub index: u32,
    pub current_item_id: Option<i32>,
    pub position_ms: u32,
    pub is_playing: bool,
    pub items: Vec<(Track, i32)>,
}
