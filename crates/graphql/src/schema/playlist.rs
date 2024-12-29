use std::sync::{mpsc::Sender, Arc, Mutex};

use async_graphql::*;
use futures_util::Stream;
use rockbox_library::repo;
use rockbox_sys::{
    events::RockboxCommand,
    types::{playlist_amount::PlaylistAmount, playlist_info::PlaylistInfo},
};

use crate::{
    rockbox_url, schema::objects::folder::Folder, schema::objects::playlist::Playlist,
    simplebroker::SimpleBroker, types::StatusCode,
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
            ..Default::default()
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

    async fn playlist(&self, _ctx: &Context<'_>, id: String) -> Result<Playlist, Error> {
        let url = format!("{}/playlists/{}", rockbox_url(), id);
        let client = reqwest::Client::new();
        let response = client.get(url).send().await?;
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
            name: response.name,
            description: response.description,
            image: response.image,
            created_at: response.created_at,
            updated_at: response.updated_at,
            id: response.id,
            ..Default::default()
        })
    }

    async fn playlists(
        &self,
        _ctx: &Context<'_>,
        folder_id: Option<String>,
    ) -> Result<Vec<Playlist>, Error> {
        let url = match folder_id {
            Some(folder_id) => format!("{}/playlists?folder_id={}", rockbox_url(), folder_id),
            None => format!("{}/playlists", rockbox_url()),
        };
        let client = reqwest::Client::new();
        let response = client.get(url).send().await?;
        let response = response.json::<Vec<rockbox_types::Playlist>>().await?;
        Ok(response
            .into_iter()
            .map(|p| Playlist {
                id: Some(p.id),
                name: Some(p.name),
                folder_id: p.folder_id,
                description: p.description,
                image: p.image,
                created_at: Some(
                    chrono::DateTime::from_timestamp(p.created_at as i64, 0)
                        .unwrap()
                        .to_rfc3339(),
                ),
                updated_at: Some(
                    chrono::DateTime::from_timestamp(p.updated_at as i64, 0)
                        .unwrap()
                        .to_rfc3339(),
                ),
                ..Default::default()
            })
            .collect())
    }

    async fn folder(&self, _ctx: &Context<'_>, id: String) -> Result<Folder, Error> {
        let url = format!("{}/folders/{}", rockbox_url(), id);
        let client = reqwest::Client::new();
        let response = client.get(url).send().await?;
        let response = response.json::<Folder>().await?;
        Ok(response)
    }

    async fn folders(
        &self,
        _ctx: &Context<'_>,
        parent_id: Option<String>,
    ) -> Result<Vec<Folder>, Error> {
        let url = match parent_id {
            Some(parent_id) => format!("{}/folders?parent_id={}", rockbox_url(), parent_id),
            None => format!("{}/folders", rockbox_url()),
        };
        let client = reqwest::Client::new();
        let response = client.get(url).send().await?;
        let response = response.json::<Vec<Folder>>().await?;
        Ok(response)
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

    async fn playlist_remove_track(
        &self,
        ctx: &Context<'_>,
        index: i32,
        playlist_id: Option<String>,
    ) -> Result<i32, Error> {
        let client = ctx.data::<reqwest::Client>().unwrap();
        let playlist_id = playlist_id.unwrap_or("current".to_string());
        let url = format!("{}/playlists/{}/tracks", rockbox_url(), playlist_id);
        let body = serde_json::json!({
            "positions": vec![index],
        });
        client.delete(&url).json(&body).send().await?;
        Ok(0)
    }

    async fn playlist_sync(&self) -> String {
        "playlist sync".to_string()
    }

    async fn playlist_remove_all_tracks(
        &self,
        ctx: &Context<'_>,
        playlist_id: Option<String>,
    ) -> Result<i32, Error> {
        let client = ctx.data::<reqwest::Client>().unwrap();
        let body = serde_json::json!({
            "positions": [],
        });
        let playlist_id = playlist_id.unwrap_or("current".to_string());
        let url = format!("{}/playlists/{}/tracks", rockbox_url(), playlist_id);
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

    async fn insert_tracks(
        &self,
        ctx: &Context<'_>,
        playlist_id: Option<String>,
        position: i32,
        tracks: Vec<String>,
    ) -> Result<i32, Error> {
        let client = ctx.data::<reqwest::Client>().unwrap();
        let body = serde_json::json!({
            "position": position,
            "tracks": tracks,
        });
        let playlist_id = playlist_id.unwrap_or("current".to_string());
        let url = format!("{}/playlists/{}/tracks", rockbox_url(), playlist_id);
        let response = client.post(&url).json(&body).send().await?;
        let start_index = response.text().await?.parse()?;
        Ok(start_index)
    }

    async fn insert_directory(
        &self,
        ctx: &Context<'_>,
        playlist_id: Option<String>,
        position: i32,
        directory: String,
    ) -> Result<i32, Error> {
        let client = ctx.data::<reqwest::Client>().unwrap();
        let body = serde_json::json!({
            "position": position,
            "tracks": [],
            "directory": directory,
        });
        let playlist_id = playlist_id.unwrap_or("current".to_string());
        let url = format!("{}/playlists/{}/tracks", rockbox_url(), playlist_id);
        let response = client.post(&url).json(&body).send().await?;
        let start_index = response.text().await?.parse()?;
        Ok(start_index)
    }

    async fn insert_playlist(
        &self,
        _ctx: &Context<'_>,
        _position: i32,
        _target_playlist_id: Option<String>,
        _playlist_id: String,
        _shuffle: Option<bool>,
    ) -> String {
        todo!()
    }

    async fn insert_album(
        &self,
        ctx: &Context<'_>,
        album_id: String,
        position: i32,
        playlist_id: Option<String>,
    ) -> Result<i32, Error> {
        let client = ctx.data::<reqwest::Client>().unwrap();
        let pool = ctx.data::<sqlx::Pool<sqlx::Sqlite>>()?;
        let tracks = repo::album_tracks::find_by_album(pool.clone(), &album_id).await?;
        let tracks: Vec<String> = tracks.into_iter().map(|t| t.path).collect();
        let body = serde_json::json!({
            "position": position,
            "tracks": tracks,
        });
        let playlist_id = playlist_id.unwrap_or("current".to_string());
        let url = format!("{}/playlists/{}/tracks", rockbox_url(), playlist_id);
        let response = client.post(&url).json(&body).send().await?;
        let start_index = response.text().await?.parse()?;
        Ok(start_index)
    }

    async fn shuffle_playlist(&self, ctx: &Context<'_>) -> Result<i32, Error> {
        let client = ctx.data::<reqwest::Client>().unwrap();
        let url = format!("{}/playlists/shuffle", rockbox_url());
        let response = client.put(&url).send().await?;
        let ret = response.text().await?.parse()?;
        Ok(ret)
    }

    async fn create_folder(
        &self,
        _ctx: &Context<'_>,
        name: String,
        parent_id: Option<String>,
    ) -> Result<Folder, Error> {
        let url = format!("{}/folders", rockbox_url());
        let body = match parent_id {
            Some(parent_id) => serde_json::json!({
            "name": name,
            "parent_id": parent_id,
            }),
            None => serde_json::json!({
                "name": name,
            }),
        };
        let client = reqwest::Client::new();
        let response = client.post(&url).json(&body).send().await?;
        let response = response.json::<Folder>().await?;
        Ok(response)
    }

    async fn remove_folder(&self, _ctx: &Context<'_>, id: String) -> Result<String, Error> {
        let client = reqwest::Client::new();
        let url = format!("{}/folders/{}", rockbox_url(), id);
        client.delete(&url).send().await?;
        Ok(id)
    }

    async fn remove_playlist(&self, _ctx: &Context<'_>, id: String) -> Result<String, Error> {
        let client = reqwest::Client::new();
        let url = format!("{}/playlists/{}", rockbox_url(), id);
        client.delete(&url).send().await?;
        Ok(id)
    }

    async fn rename_folder(
        &self,
        _ctx: &Context<'_>,
        id: String,
        name: String,
    ) -> Result<String, Error> {
        let client = reqwest::Client::new();
        let url = format!("{}/folders/{}", rockbox_url(), id);
        client
            .put(&url)
            .json(&serde_json::json!({"name": name}))
            .send()
            .await?;
        Ok(id)
    }

    async fn rename_playlist(
        &self,
        _ctx: &Context<'_>,
        id: String,
        name: String,
    ) -> Result<String, Error> {
        let client = reqwest::Client::new();
        let url = format!("{}/playlists/{}", rockbox_url(), id);
        client
            .put(&url)
            .json(&serde_json::json!({"name": name}))
            .send()
            .await?;
        Ok(id)
    }

    async fn move_folder(
        &self,
        _ctx: &Context<'_>,
        folder_id: String,
        destination: String,
    ) -> Result<String, Error> {
        let client = reqwest::Client::new();
        let url = format!("{}/folders/{}", rockbox_url(), folder_id);
        client
            .put(&url)
            .json(&serde_json::json!({"parent_id": destination}))
            .send()
            .await?;
        Ok(folder_id)
    }

    async fn move_playlist(
        &self,
        _ctx: &Context<'_>,
        playlist_id: String,
        destination: String,
    ) -> Result<String, Error> {
        let client = reqwest::Client::new();
        let url = format!("{}/playlists/{}", rockbox_url(), playlist_id);
        client
            .put(&url)
            .json(&serde_json::json!({"folder_id": destination}))
            .send()
            .await?;
        Ok(playlist_id)
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
