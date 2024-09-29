use std::sync::{mpsc::Sender, Arc, Mutex};

use async_graphql::*;
use rockbox_sys::{
    events::RockboxCommand,
    types::{playlist_amount::PlaylistAmount, playlist_info::PlaylistInfo},
};

use crate::{rockbox_url, schema::objects::playlist::Playlist};

#[derive(Default)]
pub struct PlaylistQuery;

#[Object]
impl PlaylistQuery {
    async fn playlist_get_current(&self, _ctx: &Context<'_>) -> Result<Playlist, Error> {
        let client = reqwest::Client::new();
        let url = format!("{}/playlists/current", rockbox_url());
        let response = client.get(&url).send().await?;
        let response = response.json::<PlaylistInfo>().await?;
        Ok(Playlist {
            tracks: response.entries.into_iter().map(|t| t.into()).collect(),
        })
    }

    async fn get_resume_info(&self) -> String {
        "get resume info".to_string()
    }

    async fn get_track_info(&self) -> String {
        "get track info".to_string()
    }

    async fn get_first_index(&self) -> String {
        "get first index".to_string()
    }

    async fn get_display_index(&self) -> String {
        "get display index".to_string()
    }

    async fn playlist_amount(&self, _ctx: &Context<'_>) -> Result<i32, Error> {
        let client = reqwest::Client::new();
        let url = format!("{}/playlists/amount", rockbox_url());
        let response = client.get(&url).send().await?;
        let response = response.json::<PlaylistAmount>().await?;
        Ok(response.amount)
    }
}

#[derive(Default)]
pub struct PlaylistMutation;

#[Object]
impl PlaylistMutation {
    async fn playlist_resume(&self, ctx: &Context<'_>) -> Result<String, Error> {
        ctx.data::<Arc<Mutex<Sender<RockboxCommand>>>>()
            .unwrap()
            .lock()
            .unwrap()
            .send(RockboxCommand::PlaylistResume)?;
        Ok("playlist resume".to_string())
    }

    async fn resume_track(&self, ctx: &Context<'_>) -> Result<String, Error> {
        ctx.data::<Arc<Mutex<Sender<RockboxCommand>>>>()
            .unwrap()
            .lock()
            .unwrap()
            .send(RockboxCommand::PlaylistResumeTrack)?;
        Ok("resume track".to_string())
    }

    async fn playlist_set_modified(&self) -> String {
        "set modified".to_string()
    }

    async fn playlist_start(&self, ctx: &Context<'_>) -> Result<i32, Error> {
        let client = ctx.data::<reqwest::Client>().unwrap();
        let url = format!("{}/playlists/start", rockbox_url());
        client.put(&url).send().await?;
        Ok(0)
    }

    async fn playlist_sync(&self) -> String {
        "playlist sync".to_string()
    }

    async fn playlist_remove_all_tracks(&self) -> String {
        "playlist remove all tracks".to_string()
    }

    async fn playlist_create(
        &self,
        ctx: &Context<'_>,
        name: String,
        tracks: Vec<String>,
    ) -> Result<i32, Error> {
        let client = ctx.data::<reqwest::Client>().unwrap();
        let body = serde_json::json!({
            "name": name,
            "tracks": tracks,
        });

        let url = format!("{}/playlists", rockbox_url());
        let response = client.post(&url).json(&body).send().await?;
        let start_index = response.text().await?.parse()?;
        Ok(start_index)
    }

    async fn playlist_insert_track(&self) -> String {
        "playlist insert track".to_string()
    }

    async fn playlist_insert_directory(&self) -> String {
        "playlist insert directory".to_string()
    }

    async fn shuffle_playlist(&self, ctx: &Context<'_>) -> Result<i32, Error> {
        let client = ctx.data::<reqwest::Client>().unwrap();
        let url = format!("{}/playlists/shuffle", rockbox_url());
        let response = client.put(&url).send().await?;
        let ret = response.text().await?.parse()?;
        Ok(ret)
    }

    async fn warn_on_playlist_erase(&self) -> String {
        "warn on playlist erase".to_string()
    }
}
