use anyhow::Error;
use rockbox_library::{audio_scan, repo};
use serde::Deserialize;

use crate::http::{Context, Request, Response};

pub async fn get_tracks(ctx: &Context, _req: &Request, res: &mut Response) -> Result<(), Error> {
    let tracks = repo::track::all(ctx.pool.clone()).await?;
    res.json(&tracks);
    Ok(())
}

pub async fn get_track(ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
    let track = repo::track::find(ctx.pool.clone(), &req.params[0]).await?;
    res.json(&track);
    Ok(())
}

#[derive(Deserialize)]
struct StreamMetadataBody {
    url: String,
    title: String,
    artist: String,
    album: String,
    duration_ms: u32,
}

pub async fn save_stream_track_metadata(
    ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let body = match req.body.as_ref() {
        Some(b) => b,
        None => {
            res.set_status(400);
            return Ok(());
        }
    };
    let params: StreamMetadataBody = serde_json::from_str(body)?;
    audio_scan::save_stream_metadata(
        ctx.pool.clone(),
        &params.url,
        &params.title,
        &params.artist,
        &params.album,
        params.duration_ms,
    )
    .await?;
    res.set_status(204);
    Ok(())
}
