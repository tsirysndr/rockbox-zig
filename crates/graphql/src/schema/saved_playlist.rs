use async_graphql::*;
use rockbox_library::entity::track::Track as LibraryTrack;
use rockbox_playlists::{Playlist, PlaylistFolder};

use crate::{
    rockbox_url,
    schema::objects::{
        saved_playlist::{SavedPlaylist, SavedPlaylistFolder},
        track::Track,
    },
};

#[derive(Default)]
pub struct SavedPlaylistQuery;

#[Object]
impl SavedPlaylistQuery {
    async fn saved_playlists(
        &self,
        ctx: &Context<'_>,
        folder_id: Option<String>,
    ) -> Result<Vec<SavedPlaylist>, Error> {
        let client = ctx.data::<reqwest::Client>()?;
        let mut url = format!("{}/saved-playlists", rockbox_url());
        if let Some(fid) = folder_id.as_deref() {
            if !fid.is_empty() {
                url = format!("{}?folder_id={}", url, fid);
            }
        }
        let playlists = client
            .get(&url)
            .send()
            .await?
            .json::<Vec<Playlist>>()
            .await?;
        Ok(playlists.into_iter().map(SavedPlaylist::from).collect())
    }

    async fn saved_playlist(
        &self,
        ctx: &Context<'_>,
        id: String,
    ) -> Result<Option<SavedPlaylist>, Error> {
        let client = ctx.data::<reqwest::Client>()?;
        let url = format!("{}/saved-playlists/{}", rockbox_url(), id);
        let resp = client.get(&url).send().await?;
        if resp.status().as_u16() == 404 {
            return Ok(None);
        }
        Ok(Some(SavedPlaylist::from(resp.json::<Playlist>().await?)))
    }

    async fn saved_playlist_track_ids(
        &self,
        ctx: &Context<'_>,
        playlist_id: String,
    ) -> Result<Vec<String>, Error> {
        let client = ctx.data::<reqwest::Client>()?;
        let url = format!(
            "{}/saved-playlists/{}/track-ids",
            rockbox_url(),
            playlist_id
        );
        Ok(client.get(&url).send().await?.json::<Vec<String>>().await?)
    }

    async fn saved_playlist_tracks(
        &self,
        ctx: &Context<'_>,
        playlist_id: String,
    ) -> Result<Vec<Track>, Error> {
        let client = ctx.data::<reqwest::Client>()?;
        let url = format!("{}/saved-playlists/{}/tracks", rockbox_url(), playlist_id);
        let tracks = client
            .get(&url)
            .send()
            .await?
            .json::<Vec<LibraryTrack>>()
            .await?;
        Ok(tracks.into_iter().map(Track::from).collect())
    }

    async fn playlist_folders(&self, ctx: &Context<'_>) -> Result<Vec<SavedPlaylistFolder>, Error> {
        let client = ctx.data::<reqwest::Client>()?;
        let url = format!("{}/saved-playlists/folders", rockbox_url());
        let folders = client
            .get(&url)
            .send()
            .await?
            .json::<Vec<PlaylistFolder>>()
            .await?;
        Ok(folders.into_iter().map(SavedPlaylistFolder::from).collect())
    }
}

#[derive(Default)]
pub struct SavedPlaylistMutation;

#[Object]
impl SavedPlaylistMutation {
    async fn create_playlist_folder(
        &self,
        ctx: &Context<'_>,
        name: String,
    ) -> Result<SavedPlaylistFolder, Error> {
        let client = ctx.data::<reqwest::Client>()?;
        let url = format!("{}/saved-playlists/folders", rockbox_url());
        let folder = client
            .post(&url)
            .json(&serde_json::json!({ "name": name }))
            .send()
            .await?
            .json::<PlaylistFolder>()
            .await?;
        Ok(SavedPlaylistFolder::from(folder))
    }

    async fn delete_playlist_folder(&self, ctx: &Context<'_>, id: String) -> Result<bool, Error> {
        let client = ctx.data::<reqwest::Client>()?;
        let url = format!("{}/saved-playlists/folders/{}", rockbox_url(), id);
        client.delete(&url).send().await?;
        Ok(true)
    }

    async fn create_saved_playlist(
        &self,
        ctx: &Context<'_>,
        name: String,
        description: Option<String>,
        image: Option<String>,
        folder_id: Option<String>,
        track_ids: Option<Vec<String>>,
    ) -> Result<SavedPlaylist, Error> {
        let client = ctx.data::<reqwest::Client>()?;
        let url = format!("{}/saved-playlists", rockbox_url());
        let playlist = client
            .post(&url)
            .json(&serde_json::json!({
                "name": name,
                "description": description,
                "image": image,
                "folder_id": folder_id,
                "track_ids": track_ids,
            }))
            .send()
            .await?
            .json::<Playlist>()
            .await?;
        Ok(SavedPlaylist::from(playlist))
    }

    async fn update_saved_playlist(
        &self,
        ctx: &Context<'_>,
        id: String,
        name: String,
        description: Option<String>,
        image: Option<String>,
        folder_id: Option<String>,
    ) -> Result<bool, Error> {
        let client = ctx.data::<reqwest::Client>()?;
        let url = format!("{}/saved-playlists/{}", rockbox_url(), id);
        client
            .put(&url)
            .json(&serde_json::json!({
                "name": name,
                "description": description,
                "image": image,
                "folder_id": folder_id,
            }))
            .send()
            .await?;
        Ok(true)
    }

    async fn delete_saved_playlist(&self, ctx: &Context<'_>, id: String) -> Result<bool, Error> {
        let client = ctx.data::<reqwest::Client>()?;
        let url = format!("{}/saved-playlists/{}", rockbox_url(), id);
        client.delete(&url).send().await?;
        Ok(true)
    }

    async fn add_tracks_to_saved_playlist(
        &self,
        ctx: &Context<'_>,
        playlist_id: String,
        track_ids: Vec<String>,
    ) -> Result<bool, Error> {
        let client = ctx.data::<reqwest::Client>()?;
        let url = format!("{}/saved-playlists/{}/tracks", rockbox_url(), playlist_id);
        client
            .post(&url)
            .json(&serde_json::json!({ "track_ids": track_ids }))
            .send()
            .await?;
        Ok(true)
    }

    async fn remove_track_from_saved_playlist(
        &self,
        ctx: &Context<'_>,
        playlist_id: String,
        track_id: String,
    ) -> Result<bool, Error> {
        let client = ctx.data::<reqwest::Client>()?;
        let url = format!(
            "{}/saved-playlists/{}/tracks/{}",
            rockbox_url(),
            playlist_id,
            track_id
        );
        client.delete(&url).send().await?;
        Ok(true)
    }

    async fn play_saved_playlist(
        &self,
        ctx: &Context<'_>,
        playlist_id: String,
    ) -> Result<bool, Error> {
        let client = ctx.data::<reqwest::Client>()?;
        let url = format!("{}/saved-playlists/{}/play", rockbox_url(), playlist_id);
        client.post(&url).send().await?;
        Ok(true)
    }
}
