use std::sync::{mpsc::Sender, Arc, Mutex};

use async_graphql::*;
use futures_util::Stream;
use rockbox_sys::{
    events::RockboxCommand,
    types::{playlist_amount::PlaylistAmount, playlist_info::PlaylistInfo},
};

use crate::{
    rockbox_url, schema::objects::playlist::Playlist, simplebroker::SimpleBroker, types::StatusCode,
};

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
            amount: response.amount,
            index: response.index,
            max_playlist_size: response.max_playlist_size,
            first_index: response.first_index,
            last_insert_pos: response.last_insert_pos,
            seed: response.seed,
            last_shuffled_start: response.last_shuffled_start,
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
    async fn playlist_resume(&self, _ctx: &Context<'_>) -> Result<i32, Error> {
        let client = reqwest::Client::new();
        let url = format!("{}/playlists/resume", rockbox_url());
        let response = client.put(&url).send().await?;
        let response = response.json::<StatusCode>().await?;
        Ok(response.code)
    }

    async fn resume_track(&self, ctx: &Context<'_>) -> Result<String, Error> {
        ctx.data::<Arc<Mutex<Sender<RockboxCommand>>>>()
            .unwrap()
            .lock()
            .unwrap()
            .send(RockboxCommand::PlaylistResumeTrack)?;
        Ok("".to_string())
    }

    async fn playlist_set_modified(&self) -> String {
        "set modified".to_string()
    }

    async fn playlist_start(
        &self,
        ctx: &Context<'_>,
        start_index: Option<i32>,
        elapsed: Option<i32>,
        offset: Option<i32>,
    ) -> Result<i32, Error> {
        let client = ctx.data::<reqwest::Client>().unwrap();
        let mut url = format!("{}/playlists/start", rockbox_url());

        if let Some(start_index) = start_index {
            url = format!("{}?start_index={}", url, start_index);
        }

        if let Some(elapsed) = elapsed {
            url = match url.contains("?") {
                true => format!("{}&elapsed={}", url, elapsed),
                false => format!("{}?elapsed={}", url, elapsed),
            };
        }

        if let Some(offset) = offset {
            url = match url.contains("?") {
                true => format!("{}&offset={}", url, offset),
                false => format!("{}?offset={}", url, offset),
            };
        }

        client.put(&url).send().await?;
        Ok(0)
    }

    async fn playlist_remove_track(&self, ctx: &Context<'_>, index: i32) -> Result<i32, Error> {
        let client = ctx.data::<reqwest::Client>().unwrap();
        let url = format!("{}/playlists/current/tracks", rockbox_url());
        let body = serde_json::json!({
            "positions": vec![index],
        });
        client.delete(&url).json(&body).send().await?;
        Ok(0)
    }

    async fn playlist_sync(&self) -> String {
        "playlist sync".to_string()
    }

    async fn playlist_remove_all_tracks(&self, ctx: &Context<'_>) -> Result<i32, Error> {
        let client = ctx.data::<reqwest::Client>().unwrap();
        let body = serde_json::json!({
            "positions": [],
        });
        let url = format!("{}/playlists/current/tracks", rockbox_url());
        let response = client.delete(&url).json(&body).send().await?;
        let start_index = response.text().await?.parse()?;
        Ok(start_index)
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

    async fn playlist_insert_tracks(
        &self,
        ctx: &Context<'_>,
        position: i32,
        tracks: Vec<String>,
    ) -> Result<i32, Error> {
        let client = ctx.data::<reqwest::Client>().unwrap();
        let body = serde_json::json!({
            "position": position,
            "tracks": tracks,
        });
        let url = format!("{}/playlists/current/tracks", rockbox_url());
        let response = client.post(&url).json(&body).send().await?;
        let start_index = response.text().await?.parse()?;
        Ok(start_index)
    }

    async fn playlist_insert_directory(
        &self,
        ctx: &Context<'_>,
        position: i32,
        directory: String,
    ) -> Result<i32, Error> {
        let client = ctx.data::<reqwest::Client>().unwrap();
        let body = serde_json::json!({
            "position": position,
            "tracks": [],
            "directory": directory,
        });
        let url = format!("{}/playlists/current/tracks", rockbox_url());
        let response = client.post(&url).json(&body).send().await?;
        let start_index = response.text().await?.parse()?;
        Ok(start_index)
    }

    async fn insert_playlist(
        &self,
        _ctx: &Context<'_>,
        _position: i32,
        _playlist_id: String,
    ) -> String {
        "playlist insert playlist".to_string()
    }

    async fn shuffle_playlist(&self, ctx: &Context<'_>) -> Result<i32, Error> {
        let client = ctx.data::<reqwest::Client>().unwrap();
        let url = format!("{}/playlists/shuffle", rockbox_url());
        let response = client.put(&url).send().await?;
        let ret = response.text().await?.parse()?;
        Ok(ret)
    }
}

#[derive(Default)]
pub struct PlaylistSubscription;

#[Subscription]
impl PlaylistSubscription {
    async fn playlist_changed(&self) -> impl Stream<Item = Playlist> {
        SimpleBroker::<Playlist>::subscribe()
    }
}
