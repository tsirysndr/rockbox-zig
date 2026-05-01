use rockbox_library::entity::track::Track;
use rockbox_playlists::PlaylistStore;
use rockbox_sys::types::{mp3_entry::Mp3Entry, tree::Entry};
use rockbox_traits::Player;
use rockbox_types::device::Device;
use sqlx::Sqlite;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::kv::KV;

pub struct AppState {
    pub pool: sqlx::Pool<Sqlite>,
    pub fs_cache: Arc<tokio::sync::Mutex<HashMap<String, Vec<Entry>>>>,
    pub metadata_cache: Arc<tokio::sync::Mutex<HashMap<String, Mp3Entry>>>,
    pub devices: Arc<Mutex<Vec<Device>>>,
    pub current_device: Arc<Mutex<Option<Device>>>,
    pub player: Arc<Mutex<Option<Box<dyn Player + Send>>>>,
    pub kv: Arc<Mutex<KV<Track>>>,
    pub playlist_store: PlaylistStore,
}
