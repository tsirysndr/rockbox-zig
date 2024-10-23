use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Default, Debug, Clone, Serialize, Deserialize)]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track_number: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year_string: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genre: Option<String>,
    pub md5: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album_art: Option<String>,
    pub artist_id: String,
    pub album_id: String,
    pub genre_id: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

impl Into<rockbox_search::rockbox::search::v1alpha1::Track> for Track {
    fn into(self) -> rockbox_search::rockbox::search::v1alpha1::Track {
        rockbox_search::rockbox::search::v1alpha1::Track {
            id: self.id,
            path: self.path,
            title: self.title,
            artist: self.artist,
            album: self.album,
            album_artist: self.album_artist,
            bitrate: self.bitrate,
            composer: self.composer,
            disc_number: self.disc_number,
            filesize: self.filesize,
            frequency: self.frequency,
            length: self.length,
            track_number: self.track_number.unwrap_or_default(),
            year: self.year.unwrap_or_default(),
            year_string: self.year_string.unwrap_or_default(),
            genre: self.genre.unwrap_or_default(),
            md5: self.md5,
            album_art: self.album_art,
            artist_id: Some(self.artist_id),
            album_id: Some(self.album_id),
            genre_id: Some(self.genre_id),
            created_at: self.created_at.to_rfc3339(),
            updated_at: self.updated_at.to_rfc3339(),
        }
    }
}

impl Into<rockbox_search::rockbox::search::v1alpha1::LikedTrack> for Track {
    fn into(self) -> rockbox_search::rockbox::search::v1alpha1::LikedTrack {
        rockbox_search::rockbox::search::v1alpha1::LikedTrack {
            id: self.id,
            path: self.path,
            title: self.title,
            artist: self.artist,
            album: self.album,
            album_artist: self.album_artist,
            bitrate: self.bitrate,
            composer: self.composer,
            disc_number: self.disc_number,
            filesize: self.filesize,
            frequency: self.frequency,
            length: self.length,
            track_number: self.track_number.unwrap_or_default(),
            year: self.year.unwrap_or_default(),
            year_string: self.year_string.unwrap_or_default(),
            genre: self.genre.unwrap_or_default(),
            md5: self.md5,
            album_art: self.album_art,
            artist_id: Some(self.artist_id),
            album_id: Some(self.album_id),
            genre_id: Some(self.genre_id),
            created_at: self.created_at.to_rfc3339(),
            updated_at: self.updated_at.to_rfc3339(),
        }
    }
}
