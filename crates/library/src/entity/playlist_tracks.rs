use chrono::{DateTime, Utc};

#[derive(sqlx::FromRow, Default)]
pub struct PlaylistTracks {
    pub id: String,
    pub playlist_id: String,
    pub track_id: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}
