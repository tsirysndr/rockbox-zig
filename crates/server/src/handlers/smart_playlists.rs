use actix_web::{error::ErrorInternalServerError, web, HttpResponse};
use rockbox_playlists::{resolver, rules::RuleCriteria, SmartPlaylist as RsSmartPlaylist};
use rockbox_sys::{self as rb};
use serde::{Deserialize, Serialize};

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

#[derive(Serialize)]
struct SmartPlaylistResponse {
    id: String,
    name: String,
    description: Option<String>,
    image: Option<String>,
    folder_id: Option<String>,
    is_system: bool,
    rules: RuleCriteria,
    created_at: i64,
    updated_at: i64,
    track_count: i64,
}

impl SmartPlaylistResponse {
    fn new(p: RsSmartPlaylist, track_count: i64) -> Self {
        Self {
            id: p.id,
            name: p.name,
            description: p.description,
            image: p.image,
            folder_id: p.folder_id,
            is_system: p.is_system,
            rules: p.rules,
            created_at: p.created_at,
            updated_at: p.updated_at,
            track_count,
        }
    }
}

pub async fn list_smart_playlists(state: web::Data<AppState>) -> HandlerResult {
    let playlists = state
        .playlist_store
        .list_smart_playlists()
        .await
        .map_err(ErrorInternalServerError)?;

    let (candidates, _) = resolver::build_candidates(&state.playlist_store, &state.pool)
        .await
        .map_err(ErrorInternalServerError)?;

    let response: Vec<SmartPlaylistResponse> = playlists
        .into_iter()
        .map(|p| {
            let track_count =
                rockbox_playlists::rules::resolve(&p.rules, candidates.clone()).len() as i64;
            SmartPlaylistResponse::new(p, track_count)
        })
        .collect();
    Ok(HttpResponse::Ok().json(response))
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
        Some(p) => {
            let track_count = resolver::count_tracks(&state.playlist_store, &state.pool, &p.rules)
                .await
                .map_err(ErrorInternalServerError)?;
            Ok(HttpResponse::Ok().json(SmartPlaylistResponse::new(p, track_count)))
        }
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
    let track_count = resolver::count_tracks(&state.playlist_store, &state.pool, &playlist.rules)
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Created().json(SmartPlaylistResponse::new(playlist, track_count)))
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
    let tracks = resolver::resolve_tracks(&state.playlist_store, &state.pool, &criteria).await?;
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
