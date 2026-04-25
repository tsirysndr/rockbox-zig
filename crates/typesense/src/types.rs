use rockbox_library::entity;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Track {
    pub id: String,
    pub path: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub album_artist: String,
    pub bitrate: i64,
    pub composer: String,
    pub disc_number: i64,
    pub filesize: i64,
    pub frequency: i64,
    pub length: i64,
    pub track_number: i64,
    pub year: i32,
    pub year_string: String,
    pub genre: String,
    pub md5: String,
    pub album_art: Option<String>,
    pub artist_id: Option<String>,
    pub album_id: Option<String>,
    pub genre_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
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
            bitrate: track.bitrate as i64,
            composer: track.composer,
            disc_number: track.disc_number as i64,
            filesize: track.filesize as i64,
            frequency: track.frequency as i64,
            length: track.length as i64,
            track_number: track.track_number.unwrap_or_default() as i64,
            year: track.year.unwrap_or_default() as i32,
            year_string: track.year_string.unwrap_or_default(),
            genre: track.genre.unwrap_or_default(),
            md5: track.md5,
            album_art: track.album_art,
            artist_id: Some(track.artist_id),
            album_id: Some(track.album_id),
            genre_id: Some(track.genre_id),
            created_at: track.created_at.to_rfc3339(),
            updated_at: track.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Album {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub year: i32,
    pub year_string: String,
    pub album_art: Option<String>,
    pub md5: String,
    pub artist_id: String,
    pub label: Option<String>,
}

impl From<entity::album::Album> for Album {
    fn from(album: entity::album::Album) -> Self {
        Self {
            id: album.id,
            title: album.title,
            artist: album.artist,
            year: album.year as i32,
            year_string: album.year_string,
            album_art: album.album_art,
            md5: album.md5,
            artist_id: album.artist_id,
            label: album.label,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Artist {
    pub id: String,
    pub name: String,
    pub bio: Option<String>,
    pub image: Option<String>,
}

impl From<entity::artist::Artist> for Artist {
    fn from(artist: entity::artist::Artist) -> Self {
        Self {
            id: artist.id,
            name: artist.name,
            bio: artist.bio,
            image: artist.image,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TrackHit {
    pub document: Track,
    pub highlight: Option<serde_json::Value>,
    pub highlights: Vec<serde_json::Value>,
    pub text_match: i64,
    pub text_match_info: serde_json::Value,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TrackResult {
    pub facet_counts: Vec<serde_json::Value>,
    pub found: i64,
    pub hits: Vec<TrackHit>,
    pub out_of: i64,
    pub page: i64,
    pub request_params: serde_json::Value,
    pub search_cutoff: bool,
    pub search_time_ms: i64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AlbumHit {
    pub document: Album,
    pub highlight: Option<serde_json::Value>,
    pub highlights: Vec<serde_json::Value>,
    pub text_match: i64,
    pub text_match_info: serde_json::Value,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AlbumResult {
    pub facet_counts: Vec<serde_json::Value>,
    pub found: i64,
    pub hits: Vec<AlbumHit>,
    pub out_of: i64,
    pub page: i64,
    pub request_params: serde_json::Value,
    pub search_cutoff: bool,
    pub search_time_ms: i64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ArtistHit {
    pub document: Artist,
    pub highlight: Option<serde_json::Value>,
    pub highlights: Vec<serde_json::Value>,
    pub text_match: i64,
    pub text_match_info: serde_json::Value,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ArtistResult {
    pub facet_counts: Vec<serde_json::Value>,
    pub found: i64,
    pub hits: Vec<ArtistHit>,
    pub out_of: i64,
    pub page: i64,
    pub request_params: serde_json::Value,
    pub search_cutoff: bool,
    pub search_time_ms: i64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub image: Option<String>,
    pub is_smart: bool,
    pub track_count: i64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PlaylistHit {
    pub document: Playlist,
    pub highlight: Option<serde_json::Value>,
    pub highlights: Vec<serde_json::Value>,
    pub text_match: i64,
    pub text_match_info: serde_json::Value,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PlaylistResult {
    pub facet_counts: Vec<serde_json::Value>,
    pub found: i64,
    pub hits: Vec<PlaylistHit>,
    pub out_of: i64,
    pub page: i64,
    pub request_params: serde_json::Value,
    pub search_cutoff: bool,
    pub search_time_ms: i64,
}
