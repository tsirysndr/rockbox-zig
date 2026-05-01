use actix_web::{error::ErrorInternalServerError, web, HttpResponse};
use rockbox_library::repo;

use crate::http::AppState;

type HandlerResult = actix_web::Result<HttpResponse>;

pub async fn get_artists(state: web::Data<AppState>) -> HandlerResult {
    let artists = repo::artist::all(state.pool.clone())
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(artists))
}

pub async fn get_artist(state: web::Data<AppState>, path: web::Path<String>) -> HandlerResult {
    let artist = repo::artist::find(state.pool.clone(), &path.into_inner())
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(artist))
}

pub async fn get_artist_albums(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HandlerResult {
    let albums = repo::album::find_by_artist(state.pool.clone(), &path.into_inner())
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(albums))
}

pub async fn get_artist_tracks(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HandlerResult {
    let tracks = repo::artist_tracks::find_by_artist(state.pool.clone(), &path.into_inner())
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(tracks))
}
