use anyhow::Error;
use rockbox_library::repo;

use crate::http::{Context, Request, Response};

pub async fn get_artists(ctx: &Context, _req: &Request, res: &mut Response) -> Result<(), Error> {
    let artists = repo::artist::all(ctx.pool.clone()).await?;
    res.json(&artists);
    Ok(())
}

pub async fn get_artist(ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
    let artist = repo::artist::find(ctx.pool.clone(), &req.params[0]).await?;
    res.json(&artist);
    Ok(())
}

pub async fn get_artist_albums(
    _ctx: &Context,
    _req: &Request,
    _res: &mut Response,
) -> Result<(), Error> {
    todo!("to be implemented");
}
