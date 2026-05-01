use actix_web::{error::ErrorInternalServerError, web, HttpResponse};
use rockbox_library::{audio_scan, repo};
use serde::Deserialize;

use crate::http::AppState;

type HandlerResult = actix_web::Result<HttpResponse>;

pub async fn get_tracks(state: web::Data<AppState>) -> HandlerResult {
    let tracks = repo::track::all(state.pool.clone())
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(tracks))
}

pub async fn get_track(state: web::Data<AppState>, path: web::Path<String>) -> HandlerResult {
    let track = repo::track::find(state.pool.clone(), &path.into_inner())
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(track))
}

#[derive(Deserialize)]
pub struct StreamMetadataBody {
    url: String,
    title: String,
    artist: String,
    album: String,
    duration_ms: u32,
}

pub async fn save_stream_track_metadata(
    state: web::Data<AppState>,
    body: web::Json<StreamMetadataBody>,
) -> HandlerResult {
    audio_scan::save_stream_metadata(
        state.pool.clone(),
        &body.url,
        &body.title,
        &body.artist,
        &body.album,
        body.duration_ms,
    )
    .await
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::NoContent().finish())
}
