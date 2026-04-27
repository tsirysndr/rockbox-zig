use async_graphql::*;
use rockbox_library::repo;
use rockbox_playlists::PlaylistStore;
use sqlx::{Pool, Sqlite};

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
        let store = ctx.data::<PlaylistStore>()?;
        let playlists = match folder_id.as_deref() {
            Some(fid) if !fid.is_empty() => store.list_by_folder(fid).await?,
            _ => store.list().await?,
        };
        Ok(playlists.into_iter().map(SavedPlaylist::from).collect())
    }

    async fn saved_playlist(
        &self,
        ctx: &Context<'_>,
        id: String,
    ) -> Result<Option<SavedPlaylist>, Error> {
        let store = ctx.data::<PlaylistStore>()?;
        Ok(store.get(&id).await?.map(SavedPlaylist::from))
    }

    async fn saved_playlist_tracks(
        &self,
        ctx: &Context<'_>,
        playlist_id: String,
    ) -> Result<Vec<Track>, Error> {
        let store = ctx.data::<PlaylistStore>()?;
        let pool = ctx.data::<Pool<Sqlite>>()?;
        let track_ids = store.get_track_ids(&playlist_id).await?;
        let mut tracks = Vec::with_capacity(track_ids.len());
        for id in &track_ids {
            if let Some(t) = repo::track::find(pool.clone(), id).await? {
                tracks.push(Track::from(t));
            } else if let Some(t) = repo::track::find_by_path(pool.clone(), id).await? {
                tracks.push(Track::from(t));
            }
        }
        Ok(tracks)
    }

    async fn saved_playlist_track_ids(
        &self,
        ctx: &Context<'_>,
        playlist_id: String,
    ) -> Result<Vec<String>, Error> {
        let store = ctx.data::<PlaylistStore>()?;
        Ok(store.get_track_ids(&playlist_id).await?)
    }

    async fn playlist_folders(&self, ctx: &Context<'_>) -> Result<Vec<SavedPlaylistFolder>, Error> {
        let store = ctx.data::<PlaylistStore>()?;
        let folders = store.list_folders().await?;
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
        let store = ctx.data::<PlaylistStore>()?;
        let folder = store.create_folder(&name).await?;
        Ok(SavedPlaylistFolder::from(folder))
    }

    async fn delete_playlist_folder(&self, ctx: &Context<'_>, id: String) -> Result<bool, Error> {
        let store = ctx.data::<PlaylistStore>()?;
        store.delete_folder(&id).await?;
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
        let store = ctx.data::<PlaylistStore>()?;
        let playlist = store
            .create(
                &name,
                description.as_deref(),
                image.as_deref(),
                folder_id.as_deref(),
            )
            .await?;
        if let Some(ids) = track_ids.filter(|v| !v.is_empty()) {
            store.add_tracks(&playlist.id, &ids).await?;
        }
        // refetch to get the correct track_count
        let updated = store.get(&playlist.id).await?.unwrap_or(playlist);
        Ok(SavedPlaylist::from(updated))
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
        let store = ctx.data::<PlaylistStore>()?;
        store
            .update(
                &id,
                &name,
                description.as_deref(),
                image.as_deref(),
                folder_id.as_deref(),
            )
            .await?;
        Ok(true)
    }

    async fn delete_saved_playlist(&self, ctx: &Context<'_>, id: String) -> Result<bool, Error> {
        let store = ctx.data::<PlaylistStore>()?;
        store
            .delete(&id)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))
    }

    async fn add_tracks_to_saved_playlist(
        &self,
        ctx: &Context<'_>,
        playlist_id: String,
        track_ids: Vec<String>,
    ) -> Result<bool, Error> {
        let store = ctx.data::<PlaylistStore>()?;
        store.add_tracks(&playlist_id, &track_ids).await?;
        Ok(true)
    }

    async fn remove_track_from_saved_playlist(
        &self,
        ctx: &Context<'_>,
        playlist_id: String,
        track_id: String,
    ) -> Result<bool, Error> {
        let store = ctx.data::<PlaylistStore>()?;
        store
            .remove_track(&playlist_id, &track_id)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))
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
