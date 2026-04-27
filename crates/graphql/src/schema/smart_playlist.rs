use async_graphql::*;
use rockbox_library::entity::track::Track as LibraryTrack;
use rockbox_playlists::SmartPlaylist as RsSmartPlaylist;

use crate::{
    rockbox_url,
    schema::objects::{
        smart_playlist::{SmartPlaylist, TrackStats},
        track::Track,
    },
};

#[derive(Default)]
pub struct SmartPlaylistQuery;

#[Object]
impl SmartPlaylistQuery {
    async fn smart_playlists(&self, ctx: &Context<'_>) -> Result<Vec<SmartPlaylist>, Error> {
        let client = ctx.data::<reqwest::Client>()?;
        let url = format!("{}/smart-playlists", rockbox_url());
        let playlists = client
            .get(&url)
            .send()
            .await?
            .json::<Vec<RsSmartPlaylist>>()
            .await?;
        Ok(playlists.into_iter().map(SmartPlaylist::from).collect())
    }

    async fn smart_playlist(
        &self,
        ctx: &Context<'_>,
        id: String,
    ) -> Result<Option<SmartPlaylist>, Error> {
        let client = ctx.data::<reqwest::Client>()?;
        let url = format!("{}/smart-playlists/{}", rockbox_url(), id);
        let resp = client.get(&url).send().await?;
        if resp.status().as_u16() == 404 {
            return Ok(None);
        }
        Ok(Some(SmartPlaylist::from(
            resp.json::<RsSmartPlaylist>().await?,
        )))
    }

    async fn smart_playlist_track_ids(
        &self,
        ctx: &Context<'_>,
        id: String,
    ) -> Result<Vec<String>, Error> {
        let client = ctx.data::<reqwest::Client>()?;
        let url = format!("{}/smart-playlists/{}/tracks", rockbox_url(), id);
        let tracks = client
            .get(&url)
            .send()
            .await?
            .json::<Vec<serde_json::Value>>()
            .await?;
        Ok(tracks
            .into_iter()
            .filter_map(|t| t.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()))
            .collect())
    }

    async fn smart_playlist_tracks(
        &self,
        ctx: &Context<'_>,
        id: String,
    ) -> Result<Vec<Track>, Error> {
        let client = ctx.data::<reqwest::Client>()?;
        let url = format!("{}/smart-playlists/{}/tracks", rockbox_url(), id);
        let tracks = client
            .get(&url)
            .send()
            .await?
            .json::<Vec<LibraryTrack>>()
            .await?;
        Ok(tracks.into_iter().map(Track::from).collect())
    }

    async fn track_stats(
        &self,
        ctx: &Context<'_>,
        track_id: String,
    ) -> Result<Option<TrackStats>, Error> {
        let client = ctx.data::<reqwest::Client>()?;
        let url = format!("{}/track-stats/{}", rockbox_url(), track_id);
        let resp = client.get(&url).send().await?;
        if resp.status().as_u16() == 404 {
            return Ok(None);
        }
        Ok(Some(TrackStats::from(
            resp.json::<rockbox_playlists::TrackStats>().await?,
        )))
    }
}

#[derive(Default)]
pub struct SmartPlaylistMutation;

#[Object]
impl SmartPlaylistMutation {
    async fn create_smart_playlist(
        &self,
        ctx: &Context<'_>,
        name: String,
        description: Option<String>,
        image: Option<String>,
        folder_id: Option<String>,
        rules: String,
    ) -> Result<SmartPlaylist, Error> {
        let client = ctx.data::<reqwest::Client>()?;
        let rules_val: serde_json::Value = serde_json::from_str(&rules)?;
        let url = format!("{}/smart-playlists", rockbox_url());
        let playlist = client
            .post(&url)
            .json(&serde_json::json!({
                "name": name,
                "description": description,
                "image": image,
                "folder_id": folder_id,
                "rules": rules_val,
            }))
            .send()
            .await?
            .json::<RsSmartPlaylist>()
            .await?;
        Ok(SmartPlaylist::from(playlist))
    }

    async fn update_smart_playlist(
        &self,
        ctx: &Context<'_>,
        id: String,
        name: String,
        description: Option<String>,
        image: Option<String>,
        folder_id: Option<String>,
        rules: String,
    ) -> Result<bool, Error> {
        let client = ctx.data::<reqwest::Client>()?;
        let rules_val: serde_json::Value = serde_json::from_str(&rules)?;
        let url = format!("{}/smart-playlists/{}", rockbox_url(), id);
        client
            .put(&url)
            .json(&serde_json::json!({
                "name": name,
                "description": description,
                "image": image,
                "folder_id": folder_id,
                "rules": rules_val,
            }))
            .send()
            .await?;
        Ok(true)
    }

    async fn delete_smart_playlist(&self, ctx: &Context<'_>, id: String) -> Result<bool, Error> {
        let client = ctx.data::<reqwest::Client>()?;
        let url = format!("{}/smart-playlists/{}", rockbox_url(), id);
        client.delete(&url).send().await?;
        Ok(true)
    }

    async fn play_smart_playlist(&self, ctx: &Context<'_>, id: String) -> Result<bool, Error> {
        let client = ctx.data::<reqwest::Client>()?;
        let url = format!("{}/smart-playlists/{}/play", rockbox_url(), id);
        client.post(&url).send().await?;
        Ok(true)
    }

    async fn record_track_played(
        &self,
        ctx: &Context<'_>,
        track_id: String,
    ) -> Result<bool, Error> {
        let client = ctx.data::<reqwest::Client>()?;
        let url = format!("{}/track-stats/{}/played", rockbox_url(), track_id);
        client.post(&url).send().await?;
        Ok(true)
    }

    async fn record_track_skipped(
        &self,
        ctx: &Context<'_>,
        track_id: String,
    ) -> Result<bool, Error> {
        let client = ctx.data::<reqwest::Client>()?;
        let url = format!("{}/track-stats/{}/skipped", rockbox_url(), track_id);
        client.post(&url).send().await?;
        Ok(true)
    }
}
