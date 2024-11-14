use anyhow::Error;
use async_trait::async_trait;

#[async_trait]
pub trait Player {
    async fn play(&self, url: &str) -> Result<(), Error>;
    async fn next(&self) -> Result<(), Error>;
    async fn previous(&self) -> Result<(), Error>;
    async fn stop(&self) -> Result<(), Error>;
    async fn pause(&self) -> Result<(), Error>;
    async fn resume(&self) -> Result<(), Error>;
    async fn seek(&self, seconds: i32) -> Result<(), Error>;
    async fn volume(&self, level: f32) -> Result<(), Error>;
    async fn load_tracks(&self) -> Result<(), Error>;
    async fn play_next(&self) -> Result<(), Error>;
    async fn load(&self) -> Result<(), Error>;
    async fn get_current_playback(&self) -> Result<(), Error>;
    async fn get_current_tracklist(&self) -> Result<(), Error>;
    async fn play_track_at(&self) -> Result<(), Error>;
    async fn remove_track_at(&self) -> Result<(), Error>;
    async fn disconnect(&self) -> Result<(), Error>;
}

#[async_trait]
pub trait MediaProvider {
    async fn browse(&self, path: &str) -> Result<(), Error>;
}
