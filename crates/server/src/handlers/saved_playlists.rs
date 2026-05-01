use actix_web::{error::ErrorInternalServerError, web, HttpResponse};
use rockbox_library::repo;
use rockbox_sys::{self as rb};
use serde::Deserialize;

use crate::{http::AppState, PLAYER_MUTEX};

type HandlerResult = actix_web::Result<HttpResponse>;

#[derive(Deserialize)]
pub struct CreatePlaylistBody {
    name: String,
    description: Option<String>,
    image: Option<String>,
    folder_id: Option<String>,
    track_ids: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct UpdatePlaylistBody {
    name: String,
    description: Option<String>,
    image: Option<String>,
    folder_id: Option<String>,
}

#[derive(Deserialize)]
pub struct AddTracksBody {
    track_ids: Vec<String>,
}

#[derive(Deserialize)]
pub struct CreateFolderBody {
    name: String,
}

#[derive(Deserialize)]
pub struct ListQuery {
    folder_id: Option<String>,
}

pub async fn list_saved_playlists(
    state: web::Data<AppState>,
    query: web::Query<ListQuery>,
) -> HandlerResult {
    let playlists = match query.folder_id.as_deref() {
        Some(fid) if !fid.is_empty() => state
            .playlist_store
            .list_by_folder(fid)
            .await
            .map_err(ErrorInternalServerError)?,
        _ => state
            .playlist_store
            .list()
            .await
            .map_err(ErrorInternalServerError)?,
    };
    Ok(HttpResponse::Ok().json(playlists))
}

pub async fn get_saved_playlist(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HandlerResult {
    let id = path.into_inner();
    match state
        .playlist_store
        .get(&id)
        .await
        .map_err(ErrorInternalServerError)?
    {
        Some(p) => Ok(HttpResponse::Ok().json(p)),
        None => Ok(HttpResponse::NotFound().finish()),
    }
}

pub async fn create_saved_playlist(
    state: web::Data<AppState>,
    body: web::Json<CreatePlaylistBody>,
) -> HandlerResult {
    let payload = body.into_inner();
    if payload.name.is_empty() {
        return Ok(HttpResponse::BadRequest().finish());
    }

    let playlist = state
        .playlist_store
        .create(
            &payload.name,
            payload.description.as_deref(),
            payload.image.as_deref(),
            payload.folder_id.as_deref(),
        )
        .await
        .map_err(ErrorInternalServerError)?;
    if let Some(ids) = payload.track_ids {
        if !ids.is_empty() {
            state
                .playlist_store
                .add_tracks(&playlist.id, &ids)
                .await
                .map_err(ErrorInternalServerError)?;
        }
    }
    let playlist = state
        .playlist_store
        .get(&playlist.id)
        .await
        .map_err(ErrorInternalServerError)?
        .unwrap_or(playlist);
    Ok(HttpResponse::Created().json(playlist))
}

pub async fn update_saved_playlist(
    state: web::Data<AppState>,
    path: web::Path<String>,
    body: web::Json<UpdatePlaylistBody>,
) -> HandlerResult {
    let id = path.into_inner();
    let payload = body.into_inner();
    state
        .playlist_store
        .update(
            &id,
            &payload.name,
            payload.description.as_deref(),
            payload.image.as_deref(),
            payload.folder_id.as_deref(),
        )
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::NoContent().finish())
}

pub async fn delete_saved_playlist(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HandlerResult {
    let id = path.into_inner();
    state
        .playlist_store
        .delete(&id)
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::NoContent().finish())
}

pub async fn get_saved_playlist_tracks(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HandlerResult {
    let playlist_id = path.into_inner();
    let track_ids = state
        .playlist_store
        .get_track_ids(&playlist_id)
        .await
        .map_err(ErrorInternalServerError)?;
    let mut tracks = Vec::with_capacity(track_ids.len());
    for id in &track_ids {
        if let Some(track) = repo::track::find(state.pool.clone(), id)
            .await
            .map_err(ErrorInternalServerError)?
        {
            tracks.push(track);
        }
    }
    Ok(HttpResponse::Ok().json(tracks))
}

pub async fn get_saved_playlist_track_ids(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HandlerResult {
    let playlist_id = path.into_inner();
    let track_ids = state
        .playlist_store
        .get_track_ids(&playlist_id)
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(track_ids))
}

pub async fn add_tracks_to_saved_playlist(
    state: web::Data<AppState>,
    path: web::Path<String>,
    body: web::Json<AddTracksBody>,
) -> HandlerResult {
    let playlist_id = path.into_inner();
    let payload = body.into_inner();
    state
        .playlist_store
        .add_tracks(&playlist_id, &payload.track_ids)
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::NoContent().finish())
}

pub async fn remove_track_from_saved_playlist(
    state: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> HandlerResult {
    let (playlist_id, track_id) = path.into_inner();
    state
        .playlist_store
        .remove_track(&playlist_id, &track_id)
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::NoContent().finish())
}

pub async fn play_saved_playlist(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HandlerResult {
    let playlist_id = path.into_inner();
    let track_ids = state
        .playlist_store
        .get_track_ids(&playlist_id)
        .await
        .map_err(ErrorInternalServerError)?;

    if track_ids.is_empty() {
        return Ok(HttpResponse::UnprocessableEntity().finish());
    }

    let mut paths = Vec::with_capacity(track_ids.len());
    for id in &track_ids {
        if let Some(track) = repo::track::find(state.pool.clone(), id)
            .await
            .map_err(ErrorInternalServerError)?
        {
            paths.push(track.path);
        }
    }

    if paths.is_empty() {
        return Ok(HttpResponse::UnprocessableEntity().finish());
    }

    let _player_mutex = PLAYER_MUTEX.lock().unwrap();
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

    Ok(HttpResponse::NoContent().finish())
}

pub async fn list_playlist_folders(state: web::Data<AppState>) -> HandlerResult {
    let folders = state
        .playlist_store
        .list_folders()
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(folders))
}

pub async fn create_playlist_folder(
    state: web::Data<AppState>,
    body: web::Json<CreateFolderBody>,
) -> HandlerResult {
    let payload = body.into_inner();
    if payload.name.is_empty() {
        return Ok(HttpResponse::BadRequest().finish());
    }
    let folder = state
        .playlist_store
        .create_folder(&payload.name)
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Created().json(folder))
}

pub async fn delete_playlist_folder(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HandlerResult {
    let id = path.into_inner();
    state
        .playlist_store
        .delete_folder(&id)
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::NoContent().finish())
}
