use actix_web::{error::ErrorInternalServerError, web, HttpResponse};
use rockbox_library::repo;
use rockbox_playlists::{resolver, rules::RuleCriteria};

use crate::http::AppState;

type HandlerResult = actix_web::Result<HttpResponse>;

pub async fn get_albums(state: web::Data<AppState>) -> HandlerResult {
    let albums = repo::album::all(state.pool.clone())
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(albums))
}

pub async fn get_album(state: web::Data<AppState>, path: web::Path<String>) -> HandlerResult {
    let album = repo::album::find(state.pool.clone(), &path.into_inner())
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(album))
}

pub async fn get_album_tracks(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HandlerResult {
    let tracks = repo::album_tracks::find_by_album(state.pool.clone(), &path.into_inner())
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(tracks))
}

/// Apply smart-playlist-style rules to the track library and return the
/// matching albums (the album of each matching track, deduplicated, in
/// resolver order).
///
/// Body: a JSON RuleCriteria — same shape used by smart playlists.
pub async fn filter_albums(
    state: web::Data<AppState>,
    body: web::Json<RuleCriteria>,
) -> HandlerResult {
    let criteria = body.into_inner();
    let albums = resolver::filter_albums(&state.playlist_store, &state.pool, &criteria)
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(albums))
}
