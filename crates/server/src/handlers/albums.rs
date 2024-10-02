use anyhow::Error;

use crate::http::{Context, Request, Response};

pub async fn get_albums(ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
    Ok(())
}

pub async fn get_album(ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
    Ok(())
}

pub async fn get_album_tracks(
    ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    println!("{:?}", req);
    res.text("get_album_tracks");
    Ok(())
}
