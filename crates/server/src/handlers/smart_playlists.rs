use actix_web::{error::ErrorInternalServerError, web, HttpResponse};
use rockbox_library::repo;
use rockbox_playlists::rules::{Candidate, RuleCriteria};
use rockbox_sys::{self as rb};
use serde::Deserialize;
use std::collections::HashMap;

use crate::{http::AppState, PLAYER_MUTEX};

type HandlerResult = actix_web::Result<HttpResponse>;

#[derive(Deserialize)]
pub struct CreateSmartPlaylistBody {
    name: String,
    description: Option<String>,
    image: Option<String>,
    folder_id: Option<String>,
    rules: RuleCriteria,
}

#[derive(Deserialize)]
pub struct UpdateSmartPlaylistBody {
    name: String,
    description: Option<String>,
    image: Option<String>,
    folder_id: Option<String>,
    rules: RuleCriteria,
}

pub async fn list_smart_playlists(state: web::Data<AppState>) -> HandlerResult {
    let playlists = state
        .playlist_store
        .list_smart_playlists()
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(playlists))
}

pub async fn get_smart_playlist(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HandlerResult {
    let id = path.into_inner();
    match state
        .playlist_store
        .get_smart_playlist(&id)
        .await
        .map_err(ErrorInternalServerError)?
    {
        Some(p) => Ok(HttpResponse::Ok().json(p)),
        None => Ok(HttpResponse::NotFound().finish()),
    }
}

pub async fn create_smart_playlist(
    state: web::Data<AppState>,
    body: web::Json<CreateSmartPlaylistBody>,
) -> HandlerResult {
    let payload = body.into_inner();
    if payload.name.is_empty() {
        return Ok(HttpResponse::BadRequest().finish());
    }
    let playlist = state
        .playlist_store
        .create_smart_playlist(
            &payload.name,
            payload.description.as_deref(),
            payload.image.as_deref(),
            payload.folder_id.as_deref(),
            &payload.rules,
        )
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Created().json(playlist))
}

pub async fn update_smart_playlist(
    state: web::Data<AppState>,
    path: web::Path<String>,
    body: web::Json<UpdateSmartPlaylistBody>,
) -> HandlerResult {
    let id = path.into_inner();
    let payload = body.into_inner();
    match state
        .playlist_store
        .update_smart_playlist(
            &id,
            &payload.name,
            payload.description.as_deref(),
            payload.image.as_deref(),
            payload.folder_id.as_deref(),
            &payload.rules,
        )
        .await
    {
        Ok(()) => Ok(HttpResponse::NoContent().finish()),
        Err(_) => Ok(HttpResponse::NotFound().finish()),
    }
}

pub async fn delete_smart_playlist(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HandlerResult {
    let id = path.into_inner();
    let deleted = state
        .playlist_store
        .delete_smart_playlist(&id)
        .await
        .map_err(ErrorInternalServerError)?;
    if deleted {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}

async fn resolve_smart_playlist_tracks(
    state: &AppState,
    id: &str,
) -> Result<Option<(RuleCriteria, Vec<rockbox_library::entity::track::Track>)>, anyhow::Error> {
    let criteria = match state.playlist_store.get_smart_playlist(id).await? {
        Some(p) => p.rules,
        None => return Ok(None),
    };

    let all_tracks = repo::track::all(state.pool.clone()).await?;

    let stats_map: HashMap<String, rockbox_playlists::TrackStats> = state
        .playlist_store
        .get_all_track_stats()
        .await?
        .into_iter()
        .map(|s| (s.track_id.clone(), s))
        .collect();

    let liked_ids: std::collections::HashSet<String> =
        repo::favourites::all_tracks(state.pool.clone())
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
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HandlerResult {
    let id = path.into_inner();
    match resolve_smart_playlist_tracks(&state, &id)
        .await
        .map_err(ErrorInternalServerError)?
    {
        Some((_, tracks)) => Ok(HttpResponse::Ok().json(tracks)),
        None => Ok(HttpResponse::NotFound().finish()),
    }
}

pub async fn play_smart_playlist(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HandlerResult {
    let id = path.into_inner();
    let tracks = match resolve_smart_playlist_tracks(&state, &id)
        .await
        .map_err(ErrorInternalServerError)?
    {
        Some((_, t)) => t,
        None => return Ok(HttpResponse::NotFound().finish()),
    };

    if tracks.is_empty() {
        return Ok(HttpResponse::UnprocessableEntity().finish());
    }

    let paths: Vec<String> = tracks.iter().map(|t| t.path.clone()).collect();
    web::block(move || {
        let _player_mutex = PLAYER_MUTEX.lock().unwrap();
        // Same broker routing as saved_playlists::play_smart_playlist —
        // playlist_start hits the kernel scheduler.
        crate::fw_bus::run_on_broker(move || {
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
        });
    })
    .await
    .map_err(ErrorInternalServerError)?;

    Ok(HttpResponse::NoContent().finish())
}

pub async fn record_track_played(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HandlerResult {
    let track_id = path.into_inner();
    state
        .playlist_store
        .record_play(&track_id)
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::NoContent().finish())
}

pub async fn record_track_skipped(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HandlerResult {
    let track_id = path.into_inner();
    state
        .playlist_store
        .record_skip(&track_id)
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::NoContent().finish())
}

pub async fn get_track_stats(state: web::Data<AppState>, path: web::Path<String>) -> HandlerResult {
    let track_id = path.into_inner();
    match state
        .playlist_store
        .get_track_stats(&track_id)
        .await
        .map_err(ErrorInternalServerError)?
    {
        Some(s) => Ok(HttpResponse::Ok().json(s)),
        None => Ok(HttpResponse::NotFound().finish()),
    }
}
