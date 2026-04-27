use async_graphql::*;
use rockbox_library::repo;
use rockbox_playlists::{rules::Candidate, PlaylistStore};
use sqlx::{Pool, Sqlite};
use std::collections::HashMap;

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

    let all_tracks = repo::track::all(pool.clone()).await?;

    let stats_map: HashMap<String, rockbox_playlists::TrackStats> = store
        .get_all_track_stats()
        .await?
        .into_iter()
        .map(|s| (s.track_id.clone(), s))
        .collect();

    let liked_ids: std::collections::HashSet<String> =
        repo::favourites::all_tracks(pool.clone())
            .await?
            .into_iter()
            .map(|t| t.id)
            .collect();

    let candidates: Vec<Candidate> = all_tracks
        .iter()
        .map(|t| {
            let stats = stats_map.get(&t.id);
            Candidate {
                id: t.id.clone(),
                title: t.title.clone(),
                artist: t.artist.clone(),
                album: t.album.clone(),
                year: t.year.map(|y| y as i64),
                genre: t.genre.clone(),
                duration_ms: t.length as i64 * 1000,
                bitrate: t.bitrate as i64,
                date_added_ts: t.created_at.timestamp(),
                play_count: stats.map(|s| s.play_count).unwrap_or(0),
                skip_count: stats.map(|s| s.skip_count).unwrap_or(0),
                last_played: stats.and_then(|s| s.last_played),
                last_skipped: stats.and_then(|s| s.last_skipped),
                is_liked: liked_ids.contains(&t.id),
            }
        })
        .collect();

    let resolved = rockbox_playlists::rules::resolve(&criteria, candidates);

    let track_map: HashMap<&str, &rockbox_library::entity::track::Track> =
        all_tracks.iter().map(|t| (t.id.as_str(), t)).collect();

    Ok(resolved
        .iter()
        .filter_map(|c| track_map.get(c.id.as_str()).map(|t| Track::from((*t).clone())))
        .collect())
}

#[derive(Default)]
pub struct SmartPlaylistQuery;

#[Object]
impl SmartPlaylistQuery {
    async fn smart_playlists(&self, ctx: &Context<'_>) -> Result<Vec<SmartPlaylist>, Error> {
        let store = ctx.data::<PlaylistStore>()?;
        let playlists = store.list_smart_playlists().await?;
        Ok(playlists.into_iter().map(SmartPlaylist::from).collect())
    }

    async fn smart_playlist(
        &self,
        ctx: &Context<'_>,
        id: String,
    ) -> Result<Option<SmartPlaylist>, Error> {
        let store = ctx.data::<PlaylistStore>()?;
        Ok(store.get_smart_playlist(&id).await?.map(SmartPlaylist::from))
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

    async fn delete_smart_playlist(
        &self,
        ctx: &Context<'_>,
        id: String,
    ) -> Result<bool, Error> {
        let store = ctx.data::<PlaylistStore>()?;
        store.delete_smart_playlist(&id).await.map_err(|e| async_graphql::Error::new(e.to_string()))
    }

    async fn play_smart_playlist(
        &self,
        ctx: &Context<'_>,
        id: String,
    ) -> Result<bool, Error> {
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
