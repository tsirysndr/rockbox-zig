use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Default, Serialize, Deserialize)]
pub struct Track {
    pub id: String,
    pub path: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub album_artist: String,
    pub bitrate: u32,
    pub composer: String,
    pub disc_number: u32,
    pub filesize: u32,
    pub frequency: u32,
    pub length: u32,
    pub track_number: Option<u32>,
    pub year: Option<u32>,
    pub year_string: Option<String>,
    pub genre: Option<String>,
    pub md5: String,
    pub album_art: Option<String>,
    pub artist_id: String,
    pub album_id: String,
    pub genre_id: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}
