use crate::http::{Context, Request, Response};
use anyhow::Error;
use rockbox_search::{search_entities, Searchable};
use rockbox_types::SearchResults;

pub async fn search(_ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
    let term = req
        .query_params
        .get("q")
        .map(|t| t.as_str())
        .unwrap_or_default();

    match term {
        None => {
            res.json(&SearchResults::default());
        }
        Some(term) => {
            let albums = vec![];
            let artists = vec![];
            let tracks = vec![];
            let files = vec![];
            let liked_tracks = vec![];
            let liked_albums = vec![];

            let results = SearchResults {
                albums,
                artists,
                tracks,
                files,
                liked_tracks,
                liked_albums,
            };

            res.json(&results);
        }
    }
    Ok(())
}
