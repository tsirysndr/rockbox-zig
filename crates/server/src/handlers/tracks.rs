use anyhow::Error;
use rockbox_library::repo;

use crate::http::{Context, Request, Response};

pub async fn get_tracks(ctx: &Context, _req: &Request, res: &mut Response) -> Result<(), Error> {
    let tracks = repo::track::all(ctx.pool.clone()).await?;
    res.json(&tracks);
    Ok(())
}

pub async fn get_track(_ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
    res.text("get_track");
    Ok(())
}
