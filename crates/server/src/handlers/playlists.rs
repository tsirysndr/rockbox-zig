use crate::http::{Context, Request, Response};
use anyhow::Error;

pub async fn create_playlist(
    ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    Ok(())
}

pub async fn start_playlist(ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
    Ok(())
}

pub async fn shuffle_playlist(
    ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    Ok(())
}

pub async fn get_playlist_amount(
    ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    Ok(())
}

pub async fn resume_playlist(
    ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    Ok(())
}

pub async fn resume_track(ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
    Ok(())
}

pub async fn get_playlist_tracks(
    ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    Ok(())
}

pub async fn insert_tracks(ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
    Ok(())
}

pub async fn remove_tracks(ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
    Ok(())
}
