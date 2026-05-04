use async_graphql::*;
use rockbox_playlists::{resolver, PlaylistStore};
use sqlx::{Pool, Sqlite};

use crate::{
    rockbox_url,
    schema::objects::{
        smart_playlist::{SmartPlaylist, TrackStats},
        track::Track,
    },
};

/// Resolve the tracks for a smart playlist directly against the SQLite DB,
/// replicating the logic from the server handler without going through HTTP.
async fn resolve_smart_playlist_tracks(
    store: &PlaylistStore,
    pool: &Pool<Sqlite>,
    id: &str,
) -> Result<Vec<Track>, Error> {
    let criteria = match store.get_smart_playlist(id).await? {
        Some(p) => p.rules,
        None => return Ok(vec![]),
    };
    let tracks = resolver::resolve_tracks(store, pool, &criteria).await?;
    Ok(tracks.into_iter().map(Track::from).collect())
}

#[derive(Default)]
pub struct SmartPlaylistQuery;

#[Object]
impl SmartPlaylistQuery {
    async fn smart_playlists(&self, ctx: &Context<'_>) -> Result<Vec<SmartPlaylist>, Error> {
        let store = ctx.data::<PlaylistStore>()?;
        let pool = ctx.data::<Pool<Sqlite>>()?;
        let playlists = store.list_smart_playlists().await?;

        let (candidates, _) = resolver::build_candidates(store, pool).await?;

        Ok(playlists
            .into_iter()
            .map(|p| {
                let track_count =
                    rockbox_playlists::rules::resolve(&p.rules, candidates.clone()).len() as i64;
                let mut sp: SmartPlaylist = p.into();
                sp.track_count = track_count;
                sp
            })
            .collect())
    }

    async fn smart_playlist(
        &self,
        ctx: &Context<'_>,
        id: String,
    ) -> Result<Option<SmartPlaylist>, Error> {
        let store = ctx.data::<PlaylistStore>()?;
        let pool = ctx.data::<Pool<Sqlite>>()?;
        let playlist = store.get_smart_playlist(&id).await?;
        let result = match playlist {
            Some(p) => {
                let track_count = resolver::count_tracks(store, pool, &p.rules).await?;
                let mut sp: SmartPlaylist = p.into();
                sp.track_count = track_count;
                Some(sp)
            }
            None => None,
        };
        Ok(result)
    }

    async fn smart_playlist_tracks(
        &self,
        ctx: &Context<'_>,
        id: String,
    ) -> Result<Vec<Track>, Error> {
        let store = ctx.data::<PlaylistStore>()?;
        let pool = ctx.data::<Pool<Sqlite>>()?;
        resolve_smart_playlist_tracks(store, pool, &id).await
    }

    async fn smart_playlist_track_ids(
        &self,
        ctx: &Context<'_>,
        id: String,
    ) -> Result<Vec<String>, Error> {
        let store = ctx.data::<PlaylistStore>()?;
        let pool = ctx.data::<Pool<Sqlite>>()?;
        let tracks = resolve_smart_playlist_tracks(store, pool, &id).await?;
        Ok(tracks.into_iter().filter_map(|t| t.id).collect())
    }

    async fn track_stats(
        &self,
        ctx: &Context<'_>,
        track_id: String,
    ) -> Result<Option<TrackStats>, Error> {
        let store = ctx.data::<PlaylistStore>()?;
        Ok(store
            .get_track_stats(&track_id)
            .await?
            .map(TrackStats::from))
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
        let store = ctx.data::<PlaylistStore>()?;
        let pool = ctx.data::<Pool<Sqlite>>()?;
        let criteria: rockbox_playlists::rules::RuleCriteria = serde_json::from_str(&rules)?;
        let playlist = store
            .create_smart_playlist(
                &name,
                description.as_deref(),
                image.as_deref(),
                folder_id.as_deref(),
                &criteria,
            )
            .await?;
        let track_count = resolver::count_tracks(store, pool, &playlist.rules).await?;
        let mut sp: SmartPlaylist = playlist.into();
        sp.track_count = track_count;
        Ok(sp)
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
        let store = ctx.data::<PlaylistStore>()?;
        let criteria: rockbox_playlists::rules::RuleCriteria = serde_json::from_str(&rules)?;
        store
            .update_smart_playlist(
                &id,
                &name,
                description.as_deref(),
                image.as_deref(),
                folder_id.as_deref(),
                &criteria,
            )
            .await?;
        Ok(true)
    }

    async fn delete_smart_playlist(&self, ctx: &Context<'_>, id: String) -> Result<bool, Error> {
        let store = ctx.data::<PlaylistStore>()?;
        store
            .delete_smart_playlist(&id)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))
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
        let store = ctx.data::<PlaylistStore>()?;
        store.record_play(&track_id).await?;
        Ok(true)
    }

    async fn record_track_skipped(
        &self,
        ctx: &Context<'_>,
        track_id: String,
    ) -> Result<bool, Error> {
        let store = ctx.data::<PlaylistStore>()?;
        store.record_skip(&track_id).await?;
        Ok(true)
    }
}
