use actix_web::{error::ErrorInternalServerError, web, HttpResponse};
use rockbox_typesense::client::{search_albums, search_artists, search_tracks};
use serde::{Deserialize, Serialize};

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

pub async fn search(query: web::Query<SearchQuery>) -> HandlerResult {
    let term = query.q.as_deref().unwrap_or_default();

    let tracks = search_tracks(term)
        .await
        .map_err(ErrorInternalServerError)?
        .map(|r| r.hits.into_iter().map(|h| h.document).collect())
        .unwrap_or_default();
    let albums = search_albums(term)
        .await
        .map_err(ErrorInternalServerError)?
        .map(|r| r.hits.into_iter().map(|h| h.document).collect())
        .unwrap_or_default();
    let artists = search_artists(term)
        .await
        .map_err(ErrorInternalServerError)?
        .map(|r| r.hits.into_iter().map(|h| h.document).collect())
        .unwrap_or_default();

    Ok(HttpResponse::Ok().json(SearchResponse {
        tracks,
        albums,
        artists,
    }))
}
