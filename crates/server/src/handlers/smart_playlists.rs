use crate::http::{Context, Request, Response};
use crate::PLAYER_MUTEX;
use anyhow::Error;
use rockbox_library::repo;
use rockbox_playlists::rules::{Candidate, RuleCriteria};
use rockbox_sys::{self as rb};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
struct CreateSmartPlaylistBody {
    name: String,
    description: Option<String>,
    image: Option<String>,
    folder_id: Option<String>,
    rules: RuleCriteria,
}

#[derive(Deserialize)]
struct UpdateSmartPlaylistBody {
    name: String,
    description: Option<String>,
    image: Option<String>,
    folder_id: Option<String>,
    rules: RuleCriteria,
}

pub async fn list_smart_playlists(
    ctx: &Context,
    _req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let playlists = ctx.playlist_store.list_smart_playlists().await?;
    res.json(&playlists);
    Ok(())
}

pub async fn get_smart_playlist(
    ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let id = req.params.first().map(|s| s.as_str()).unwrap_or("");
    match ctx.playlist_store.get_smart_playlist(id).await? {
        Some(p) => res.json(&p),
        None => res.set_status(404),
    }
    Ok(())
}

pub async fn create_smart_playlist(
    ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let body = match req.body.as_ref() {
        Some(b) => b,
        None => {
            res.set_status(400);
            return Ok(());
        }
    };
    let payload: CreateSmartPlaylistBody = serde_json::from_str(body)?;
    if payload.name.is_empty() {
        res.set_status(400);
        return Ok(());
    }
    let playlist = ctx
        .playlist_store
        .create_smart_playlist(
            &payload.name,
            payload.description.as_deref(),
            payload.image.as_deref(),
            payload.folder_id.as_deref(),
            &payload.rules,
        )
        .await?;
    res.set_status(201);
    res.json(&playlist);
    Ok(())
}

pub async fn update_smart_playlist(
    ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let id = req.params.first().map(|s| s.as_str()).unwrap_or("");
    let body = match req.body.as_ref() {
        Some(b) => b,
        None => {
            res.set_status(400);
            return Ok(());
        }
    };
    let payload: UpdateSmartPlaylistBody = serde_json::from_str(body)?;
    match ctx
        .playlist_store
        .update_smart_playlist(
            id,
            &payload.name,
            payload.description.as_deref(),
            payload.image.as_deref(),
            payload.folder_id.as_deref(),
            &payload.rules,
        )
        .await
    {
        Ok(()) => res.set_status(204),
        Err(_) => res.set_status(404),
    }
    Ok(())
}

pub async fn delete_smart_playlist(
    ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let id = req.params.first().map(|s| s.as_str()).unwrap_or("");
    let deleted = ctx.playlist_store.delete_smart_playlist(id).await?;
    if deleted {
        res.set_status(204);
    } else {
        res.set_status(404);
    }
    Ok(())
}

async fn resolve_smart_playlist_tracks(
    ctx: &Context,
    id: &str,
) -> Result<Option<(RuleCriteria, Vec<rockbox_library::entity::track::Track>)>, Error> {
    let criteria = match ctx.playlist_store.get_smart_playlist(id).await? {
        Some(p) => p.rules,
        None => return Ok(None),
    };

    let all_tracks = repo::track::all(ctx.pool.clone()).await?;

    let stats_map: HashMap<String, rockbox_playlists::TrackStats> = ctx
        .playlist_store
        .get_all_track_stats()
        .await?
        .into_iter()
        .map(|s| (s.track_id.clone(), s))
        .collect();

    let liked_ids: std::collections::HashSet<String> =
        repo::favourites::all_tracks(ctx.pool.clone())
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

    let resolved_ids: Vec<&str> = resolved.iter().map(|c| c.id.as_str()).collect();
    let track_map: HashMap<&str, &rockbox_library::entity::track::Track> =
        all_tracks.iter().map(|t| (t.id.as_str(), t)).collect();

    let tracks: Vec<rockbox_library::entity::track::Track> = resolved_ids
        .iter()
        .filter_map(|id| track_map.get(id).map(|t| (*t).clone()))
        .collect();

    Ok(Some((criteria, tracks)))
}

pub async fn get_smart_playlist_tracks(
    ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let id = req.params.first().map(|s| s.as_str()).unwrap_or("");
    match resolve_smart_playlist_tracks(ctx, id).await? {
        Some((_, tracks)) => res.json(&tracks),
        None => res.set_status(404),
    }
    Ok(())
}

pub async fn play_smart_playlist(
    ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let id = req.params.first().map(|s| s.as_str()).unwrap_or("");
    let tracks = match resolve_smart_playlist_tracks(ctx, id).await? {
        Some((_, t)) => t,
        None => {
            res.set_status(404);
            return Ok(());
        }
    };

    if tracks.is_empty() {
        res.set_status(422);
        return Ok(());
    }

    let paths: Vec<String> = tracks.iter().map(|t| t.path.clone()).collect();
    let player_mutex = PLAYER_MUTEX.lock().unwrap();
    let first = &paths[0];
    let dir = {
        let parts: Vec<_> = first.split('/').collect();
        parts[..parts.len().saturating_sub(1)].join("/")
    };
    rb::playlist::create(&dir, None);
    rb::playlist::build_playlist(
        paths.iter().map(|p| p.as_str()).collect(),
        0,
        paths.len() as i32,
    );
    rb::playlist::start(0, 0, 0);
    drop(player_mutex);

    res.set_status(204);
    Ok(())
}

pub async fn record_track_played(
    ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let track_id = req.params.first().map(|s| s.as_str()).unwrap_or("");
    ctx.playlist_store.record_play(track_id).await?;
    res.set_status(204);
    Ok(())
}

pub async fn record_track_skipped(
    ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let track_id = req.params.first().map(|s| s.as_str()).unwrap_or("");
    ctx.playlist_store.record_skip(track_id).await?;
    res.set_status(204);
    Ok(())
}

pub async fn get_track_stats(
    ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let track_id = req.params.first().map(|s| s.as_str()).unwrap_or("");
    match ctx.playlist_store.get_track_stats(track_id).await? {
        Some(s) => res.json(&s),
        None => res.set_status(404),
    }
    Ok(())
}
