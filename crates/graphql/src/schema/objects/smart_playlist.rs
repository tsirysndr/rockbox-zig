use async_graphql::*;
use rockbox_playlists::{SmartPlaylist as RsSmartPlaylist, TrackStats as RsTrackStats};
use serde::Serialize;

#[derive(Default, Clone, Serialize, SimpleObject)]
pub struct SmartPlaylist {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub image: Option<String>,
    pub folder_id: Option<String>,
    pub is_system: bool,
    pub rules: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Default, Clone, Serialize, SimpleObject)]
pub struct TrackStats {
    pub track_id: String,
    pub play_count: i64,
    pub skip_count: i64,
    pub last_played: Option<i64>,
    pub last_skipped: Option<i64>,
    pub updated_at: i64,
}

impl From<RsSmartPlaylist> for SmartPlaylist {
    fn from(p: RsSmartPlaylist) -> Self {
        Self {
            id: p.id,
            name: p.name,
            description: p.description,
            image: p.image,
            folder_id: p.folder_id,
            is_system: p.is_system,
            rules: serde_json::to_string(&p.rules).unwrap_or_default(),
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }
}

impl From<RsTrackStats> for TrackStats {
    fn from(s: RsTrackStats) -> Self {
        Self {
            track_id: s.track_id,
            play_count: s.play_count,
            skip_count: s.skip_count,
            last_played: s.last_played,
            last_skipped: s.last_skipped,
            updated_at: s.updated_at,
        }
    }
}
