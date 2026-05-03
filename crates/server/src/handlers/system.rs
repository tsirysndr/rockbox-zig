use std::env;

use actix_web::{error::ErrorInternalServerError, web, HttpResponse};
use rockbox_graphql::{simplebroker::SimpleBroker, types::ScanCompleted};
#[cfg(not(feature = "fts5"))]
use rockbox_library::repo;
use rockbox_library::{artists::update_metadata, audio_scan::scan_audio_files};
use rockbox_sys as rb;
#[cfg(not(feature = "fts5"))]
use rockbox_typesense::{client::*, types::*};
use serde::Deserialize;

use crate::http::AppState;

type HandlerResult = actix_web::Result<HttpResponse>;

pub async fn get_status() -> HandlerResult {
    let status = rb::system::get_global_status();
    Ok(HttpResponse::Ok().json(status))
}

pub async fn get_rockbox_version() -> HandlerResult {
    let version = rb::system::get_rockbox_version();
    Ok(HttpResponse::Ok().json(version))
}

#[derive(Deserialize)]
pub struct ScanQuery {
    path: Option<String>,
    rebuild_index: Option<String>,
}

pub async fn scan_library(
    state: web::Data<AppState>,
    query: web::Query<ScanQuery>,
) -> HandlerResult {
    let home = env::var("HOME").map_err(ErrorInternalServerError)?;
    let music_library = format!("{}/Music", home);

    let path = query.path.clone().unwrap_or_else(|| music_library.clone());

    scan_audio_files(state.pool.clone(), path.clone().into())
        .await
        .map_err(ErrorInternalServerError)?;

    let rebuild_index = query
        .rebuild_index
        .as_deref()
        .map(|s| s == "true" || s == "1")
        .unwrap_or(false);

    if path != music_library {
        SimpleBroker::publish(ScanCompleted);
        return Ok(HttpResponse::Ok().body("0"));
    }

    update_metadata(state.pool.clone())
        .await
        .map_err(ErrorInternalServerError)?;

    if !rebuild_index {
        SimpleBroker::publish(ScanCompleted);
        return Ok(HttpResponse::Ok().body("0"));
    }

    // FTS5 indexes are kept current by triggers on the track/album/artist
    // tables, so a rebuild after a scan is a no-op for that backend. Only
    // Typesense needs the explicit re-import.
    #[cfg(not(feature = "fts5"))]
    {
        let tracks = repo::track::all(state.pool.clone())
            .await
            .map_err(ErrorInternalServerError)?;
        let albums = repo::album::all(state.pool.clone())
            .await
            .map_err(ErrorInternalServerError)?;
        let artists = repo::artist::all(state.pool.clone())
            .await
            .map_err(ErrorInternalServerError)?;

        create_tracks_collection()
            .await
            .map_err(ErrorInternalServerError)?;
        create_albums_collection()
            .await
            .map_err(ErrorInternalServerError)?;
        create_artists_collection()
            .await
            .map_err(ErrorInternalServerError)?;

        insert_tracks(tracks.into_iter().map(Track::from).collect())
            .await
            .map_err(ErrorInternalServerError)?;
        insert_artists(artists.into_iter().map(Artist::from).collect())
            .await
            .map_err(ErrorInternalServerError)?;
        insert_albums(albums.into_iter().map(Album::from).collect())
            .await
            .map_err(ErrorInternalServerError)?;
    }

    SimpleBroker::publish(ScanCompleted);
    Ok(HttpResponse::Ok().body("0"))
}
