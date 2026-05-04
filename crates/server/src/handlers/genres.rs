use actix_web::{error::ErrorInternalServerError, web, HttpResponse};
use rockbox_library::repo;
use serde::Serialize;

use crate::http::AppState;

type HandlerResult = actix_web::Result<HttpResponse>;

#[derive(Serialize)]
struct GenreListItem {
    id: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<String>,
    track_count: i64,
}

pub async fn get_genres(state: web::Data<AppState>) -> HandlerResult {
    let genres = repo::genre::all(state.pool.clone())
        .await
        .map_err(ErrorInternalServerError)?;

    let mut out = Vec::with_capacity(genres.len());
    for g in genres {
        let track_count = repo::genre::find_tracks(state.pool.clone(), &g.id)
            .await
            .map(|t| t.len() as i64)
            .unwrap_or(0);
        out.push(GenreListItem {
            id: g.id,
            name: g.name,
            description: g.description,
            image: g.image,
            track_count,
        });
    }
    Ok(HttpResponse::Ok().json(out))
}

pub async fn get_genre(state: web::Data<AppState>, path: web::Path<String>) -> HandlerResult {
    let id = path.into_inner();
    let genre = repo::genre::find(state.pool.clone(), &id)
        .await
        .map_err(ErrorInternalServerError)?;

    let genre = match genre {
        Some(g) => g,
        None => return Ok(HttpResponse::NotFound().finish()),
    };

    let track_count = repo::genre::find_tracks(state.pool.clone(), &id)
        .await
        .map(|t| t.len() as i64)
        .unwrap_or(0);

    Ok(HttpResponse::Ok().json(GenreListItem {
        id: genre.id,
        name: genre.name,
        description: genre.description,
        image: genre.image,
        track_count,
    }))
}

pub async fn get_genre_tracks(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HandlerResult {
    let tracks = repo::genre::find_tracks(state.pool.clone(), &path.into_inner())
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(tracks))
}

pub async fn get_genre_albums(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HandlerResult {
    let albums = repo::genre::find_albums(state.pool.clone(), &path.into_inner())
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(albums))
}

pub async fn get_genre_artists(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HandlerResult {
    let artists = repo::genre::find_artists(state.pool.clone(), &path.into_inner())
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(artists))
}
