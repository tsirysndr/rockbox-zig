use rockbox_search::artist::Artist;
use rockbox_search::file::File;
use rockbox_search::liked_album::LikedAlbum;
use rockbox_search::liked_track::LikedTrack;
use rockbox_search::{album::Album, track::Track};
use serde::{Deserialize, Serialize};

pub mod device;

#[derive(Debug, Serialize, Deserialize)]
pub struct NewPlaylist {
    pub name: Option<String>,
    pub tracks: Vec<String>,
    pub folder_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InsertTracks {
    pub position: i32,
    pub tracks: Vec<String>,
    pub directory: Option<String>,
    pub shuffle: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoadTracks {
    pub tracks: Vec<String>,
    pub directory: Option<String>,
    pub album_id: Option<String>,
    pub shuffle: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewVolume {
    pub steps: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteTracks {
    pub positions: Vec<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusCode {
    pub code: i32,
}

#[derive(Default, Serialize, Deserialize)]
pub struct SearchResults {
    pub artists: Vec<Artist>,
    pub albums: Vec<Album>,
    pub tracks: Vec<Track>,
    pub liked_tracks: Vec<LikedTrack>,
    pub liked_albums: Vec<LikedAlbum>,
    pub files: Vec<File>,
}

#[derive(Default, Serialize, Deserialize)]
pub struct EqBandSetting {
    pub cutoff: i32,
    pub q: i32,
    pub gain: i32,
}

#[derive(Default, Serialize, Deserialize)]
pub struct ReplaygainSettings {
    pub enabled: bool,
    pub preamp: i32,
    pub peak: i32,
    pub clip: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Folder {
    pub name: String,
    pub parent_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FolderUpdate {
    pub name: Option<String>,
    pub parent_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaylistUpdate {
    pub name: Option<String>,
    pub folder_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub folder_id: Option<String>,
    pub image: Option<String>,
    pub description: Option<String>,
}
