use actix_web::{error::ErrorInternalServerError, web, HttpResponse};
use rockbox_library::repo;

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
