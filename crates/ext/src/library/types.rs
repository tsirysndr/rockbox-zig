use chrono::{DateTime, Utc};
use rockbox_library::entity;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct Album {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub year: u32,
    pub year_string: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album_art: Option<String>,
    pub md5: String,
    pub artist_id: String,
    pub tracks: Vec<Track>,
}

impl From<entity::album::Album> for Album {
    fn from(album: entity::album::Album) -> Self {
        Self {
            id: album.id,
            title: album.title,
            artist: album.artist,
            year: album.year,
            year_string: album.year_string,
            album_art: album.album_art,
            md5: album.md5,
            artist_id: album.artist_id,
            tracks: vec![],
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct Artist {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    pub tracks: Vec<Track>,
    pub albums: Vec<Album>,
}

impl From<entity::artist::Artist> for Artist {
    fn from(artist: entity::artist::Artist) -> Self {
        Self {
            id: artist.id,
            name: artist.name,
            bio: artist.bio,
            image: artist.image,
            tracks: vec![],
            albums: vec![],
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
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

impl From<entity::track::Track> for Track {
    fn from(track: entity::track::Track) -> Self {
        Self {
            id: track.id,
            path: track.path,
            title: track.title,
            artist: track.artist,
            album: track.album,
            album_artist: track.album_artist,
            bitrate: track.bitrate,
            composer: track.composer,
            disc_number: track.disc_number,
            filesize: track.filesize,
            frequency: track.frequency,
            length: track.length,
            track_number: track.track_number,
            year: track.year,
            year_string: track.year_string,
            genre: track.genre,
            md5: track.md5,
            album_art: track.album_art,
            artist_id: track.artist_id,
            album_id: track.album_id,
            genre_id: track.genre_id,
            created_at: track.created_at,
            updated_at: track.updated_at,
        }
    }
}
