use crate::http::{Context, Request, Response};
use anyhow::Error;
use rockbox_search::search_entities;
use rockbox_types::SearchResults;

pub async fn search(ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
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
            let albums = search_entities(
                &ctx.indexes.albums,
                term,
                &rockbox_search::album::Album::default(),
            )?;
            let artists = search_entities(
                &ctx.indexes.artists,
                term,
                &rockbox_search::artist::Artist::default(),
            )?;
            let tracks = search_entities(
                &ctx.indexes.tracks,
                term,
                &rockbox_search::track::Track::default(),
            )?;
            let files = search_entities(
                &ctx.indexes.files,
                term,
                &rockbox_search::file::File::default(),
            )?;
            let liked_tracks = search_entities(
                &ctx.indexes.liked_tracks,
                term,
                &rockbox_search::liked_track::LikedTrack::default(),
            )?;
            let liked_albums = search_entities(
                &ctx.indexes.liked_albums,
                term,
                &rockbox_search::liked_album::LikedAlbum::default(),
            )?;

            let results = SearchResults {
                albums: albums.into_iter().map(|(_, x)| x.into()).collect(),
                artists: artists.into_iter().map(|(_, x)| x.into()).collect(),
                tracks: tracks.into_iter().map(|(_, x)| x.into()).collect(),
                files: files.into_iter().map(|(_, x)| x.into()).collect(),
                liked_tracks: liked_tracks.into_iter().map(|(_, x)| x.into()).collect(),
                liked_albums: liked_albums.into_iter().map(|(_, x)| x.into()).collect(),
            };

            res.json(&results);
        }
    }
    Ok(())
}
