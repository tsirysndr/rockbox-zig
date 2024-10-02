use anyhow::Error;

use crate::http::{Context, Request, Response};

pub async fn get_artists(ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
    Ok(())
}

pub async fn get_artist(ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
    Ok(())
}

pub async fn get_artist_albums(
    ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    Ok(())
}
