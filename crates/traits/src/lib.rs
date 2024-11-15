use anyhow::Error;
use async_trait::async_trait;
use types::{playback::Playback, track::Track};

pub mod types;

#[async_trait]
pub trait Player {
    async fn play(&self) -> Result<(), Error>;
    async fn next(&self) -> Result<(), Error>;
    async fn previous(&self) -> Result<(), Error>;
    async fn stop(&self) -> Result<(), Error>;
    async fn pause(&self) -> Result<(), Error>;
    async fn resume(&self) -> Result<(), Error>;
    async fn seek(&self, seconds: i32) -> Result<(), Error>;
    async fn volume(&self, level: f32) -> Result<(), Error>;
    async fn load_tracks(&self, tracks: Vec<Track>, start_index: Option<i32>) -> Result<(), Error>;
    async fn play_next(&self, track: Track) -> Result<(), Error>;
    async fn load(&mut self, track: Track) -> Result<(), Error>;
    async fn get_current_playback(&mut self) -> Result<Playback, Error>;
    async fn get_current_tracklist(&self) -> Result<(Vec<Track>, Vec<Track>), Error>;
    async fn play_track_at(&self, position: u32) -> Result<(), Error>;
    async fn remove_track_at(&self, position: u32) -> Result<(), Error>;
    async fn disconnect(&self) -> Result<(), Error>;
}

#[async_trait]
pub trait MediaProvider {
    async fn browse(&self, path: &str) -> Result<(), Error>;
}
