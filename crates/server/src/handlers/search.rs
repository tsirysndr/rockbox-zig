use actix_web::{error::ErrorInternalServerError, web, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::http::AppState;

type HandlerResult = actix_web::Result<HttpResponse>;

#[derive(Deserialize)]
pub struct SearchQuery {
    q: Option<String>,
}

#[derive(Default, Serialize)]
struct SearchResponse {
    tracks: Vec<rockbox_typesense::types::Track>,
    albums: Vec<rockbox_typesense::types::Album>,
    artists: Vec<rockbox_typesense::types::Artist>,
}

#[cfg(not(feature = "fts5"))]
async fn run_search(_state: &AppState, term: &str) -> Result<SearchResponse, anyhow::Error> {
    use rockbox_typesense::client::{search_albums, search_artists, search_tracks};
    let tracks = search_tracks(term)
        .await?
        .map(|r| r.hits.into_iter().map(|h| h.document).collect())
        .unwrap_or_default();
    let albums = search_albums(term)
        .await?
        .map(|r| r.hits.into_iter().map(|h| h.document).collect())
        .unwrap_or_default();
    let artists = search_artists(term)
        .await?
        .map(|r| r.hits.into_iter().map(|h| h.document).collect())
        .unwrap_or_default();
    Ok(SearchResponse {
        tracks,
        albums,
        artists,
    })
}

#[cfg(feature = "fts5")]
async fn run_search(state: &AppState, term: &str) -> Result<SearchResponse, anyhow::Error> {
    use rockbox_fts5::{search_albums, search_artists, search_tracks};
    let tracks = search_tracks(state.pool.clone(), term)
        .await?
        .map(|r| r.hits.into_iter().map(|h| h.document).collect())
        .unwrap_or_default();
    let albums = search_albums(state.pool.clone(), term)
        .await?
        .map(|r| r.hits.into_iter().map(|h| h.document).collect())
        .unwrap_or_default();
    let artists = search_artists(state.pool.clone(), term)
        .await?
        .map(|r| r.hits.into_iter().map(|h| h.document).collect())
        .unwrap_or_default();
    Ok(SearchResponse {
        tracks,
        albums,
        artists,
    })
}

pub async fn search(state: web::Data<AppState>, query: web::Query<SearchQuery>) -> HandlerResult {
    let term = query.q.as_deref().unwrap_or_default();
    let response = run_search(&state, term)
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(response))
}
