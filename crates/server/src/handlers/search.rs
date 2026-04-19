use crate::http::{Context, Request, Response};
use anyhow::Error;
use rockbox_typesense::client::{search_albums, search_artists, search_tracks};
use serde::Serialize;

#[derive(Default, Serialize)]
struct SearchResponse {
    tracks: Vec<rockbox_typesense::types::Track>,
    albums: Vec<rockbox_typesense::types::Album>,
    artists: Vec<rockbox_typesense::types::Artist>,
}

pub async fn search(_ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
    let term = req
        .query_params
        .get("q")
        .and_then(|t| t.as_str())
        .unwrap_or_default();

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

    res.json(&SearchResponse {
        tracks,
        albums,
        artists,
    });

    Ok(())
}
