use std::env;

use crate::http::{Context, Request, Response};
use anyhow::Error;
use rockbox_library::{audio_scan::scan_audio_files, repo};
use rockbox_search::{index_entity, Indexable};
use rockbox_sys as rb;

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

pub async fn scan_library(ctx: &Context, _req: &Request, res: &mut Response) -> Result<(), Error> {
    let home = env::var("HOME")?;
    let path = env::var("ROCKBOX_LIBRARY").unwrap_or(format!("{}/Music", home));
    scan_audio_files(ctx.pool.clone(), path.into()).await?;
    let tracks = repo::track::all(ctx.pool.clone()).await?;
    let albums = repo::album::all(ctx.pool.clone()).await?;
    let artists = repo::artist::all(ctx.pool.clone()).await?;
    let liked_albums = repo::favourites::all_albums(ctx.pool.clone()).await?;
    let liked_tracks = repo::favourites::all_tracks(ctx.pool.clone()).await?;

    res.text("0");
    Ok(())
}
