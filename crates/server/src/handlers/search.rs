use crate::http::{Context, Request, Response};
use anyhow::Error;
use rockbox_search::{
    search_album, search_artist, search_file, search_liked_album, search_liked_track, search_track,
};
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
            let albums = search_album(term)?.albums;
            let artists = search_artist(term)?.artists;
            let tracks = search_track(term)?.tracks;
            let files = search_file(term)?.files;
            let liked_tracks = search_liked_track(term)?.tracks;
            let liked_albums = search_liked_album(term)?.albums;

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
