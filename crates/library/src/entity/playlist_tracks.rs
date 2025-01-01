use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Default, Serialize, Deserialize)]
pub struct PlaylistTracks {
    pub id: String,
    pub playlist_id: String,
    pub track_id: String,
    pub position: u32,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}
