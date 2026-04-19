use std::env;

use crate::http::{Context, Request, Response};
use anyhow::Error;
use rockbox_library::{artists::update_metadata, audio_scan::scan_audio_files, repo};
use rockbox_sys as rb;
use rockbox_typesense::client::*;
use rockbox_typesense::types::*;

pub async fn get_status(_ctx: &Context, _req: &Request, res: &mut Response) -> Result<(), Error> {
    let status = rb::system::get_global_status();
    res.json(&status);
    Ok(())
}

pub async fn get_rockbox_version(
    _ctx: &Context,
    _req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let version = rb::system::get_rockbox_version();
    res.json(&version);
    Ok(())
}

pub async fn scan_library(ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
    let home = env::var("HOME")?;
    let music_library = format!("{}/Music", home);

    let path = match req.query_params.get("path") {
        Some(path) => path.as_str().unwrap_or(&music_library),
        None => &music_library,
    };

    scan_audio_files(ctx.pool.clone(), path.into()).await?;

    let rebuild_index = match req.query_params.get("rebuild_index") {
        Some(rebuild_index) => {
            let rebuild_index = rebuild_index.as_str().unwrap_or("false");
            rebuild_index == "true" || rebuild_index == "1"
        }
        None => false,
    };

    if path != music_library {
        res.text("0");
        return Ok(());
    }

    update_metadata(ctx.pool.clone())?;

    if !rebuild_index {
        res.text("0");
        return Ok(());
    }

    let tracks = repo::track::all(ctx.pool.clone()).await?;
    let albums = repo::album::all(ctx.pool.clone()).await?;
    let artists = repo::artist::all(ctx.pool.clone()).await?;

    create_tracks_collection().await?;
    create_albums_collection().await?;
    create_artists_collection().await?;

    insert_tracks(tracks.into_iter().map(Track::from).collect()).await?;
    insert_artists(artists.into_iter().map(Artist::from).collect()).await?;
    insert_albums(albums.into_iter().map(Album::from).collect()).await?;

    res.text("0");
    Ok(())
}
