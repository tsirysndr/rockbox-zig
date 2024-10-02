use crate::http::{Context, Request, Response};
use anyhow::Error;

pub async fn play(ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
    Ok(())
}

pub async fn pause(ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
    Ok(())
}

pub async fn ff_rewind(ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
    Ok(())
}

pub async fn status(ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
    Ok(())
}

pub async fn current_track(ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
    Ok(())
}

pub async fn next_track(ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
    Ok(())
}

pub async fn flush_and_reload_tracks(ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
  Ok(())
}


pub async fn resume(ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
  Ok(())
}


pub async fn next(ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
  Ok(())
}


pub async fn previous(ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
  Ok(())
}


pub async fn stop(ctx: &Context, req: &Request, res: &mut Response)-> Result<(), Error> {
  Ok(())
}

