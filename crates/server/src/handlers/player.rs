use crate::http::{Context, Request, Response};
use anyhow::Error;
use rockbox_sys as rb;

pub async fn play(_ctx: &Context, req: &Request, _res: &mut Response) -> Result<(), Error> {
    let elapsed = req
        .query_params
        .get("elapsed")
        .unwrap()
        .as_i64()
        .unwrap_or(0);
    let offset = req
        .query_params
        .get("offset")
        .unwrap()
        .as_i64()
        .unwrap_or(0);
    rb::playback::play(elapsed, offset);
    Ok(())
}

pub async fn pause(_ctx: &Context, _req: &Request, _res: &mut Response) -> Result<(), Error> {
    rb::playback::pause();
    Ok(())
}

pub async fn ff_rewind(_ctx: &Context, req: &Request, _res: &mut Response) -> Result<(), Error> {
    let newtime = req
        .query_params
        .get("newtime")
        .unwrap()
        .as_i64()
        .unwrap_or(0);
    rb::playback::ff_rewind(newtime as i32);
    Ok(())
}

pub async fn status(_ctx: &Context, _req: &Request, res: &mut Response) -> Result<(), Error> {
    let status = rb::playback::status();
    res.json(&status);
    Ok(())
}

pub async fn current_track(_ctx: &Context, _req: &Request, res: &mut Response) -> Result<(), Error> {
    let track = rb::playback::current_track();
    res.json(&track);
    Ok(())
}

pub async fn next_track(_ctx: &Context, _req: &Request, res: &mut Response) -> Result<(), Error> {
    let track = rb::playback::next_track();
    res.json(&track);
    Ok(())
}

pub async fn flush_and_reload_tracks(
    _ctx: &Context,
    _req: &Request,
    _res: &mut Response,
) -> Result<(), Error> {
    rb::playback::flush_and_reload_tracks();
    Ok(())
}

pub async fn resume(_ctx: &Context, _req: &Request, _res: &mut Response) -> Result<(), Error> {
    rb::playback::resume();
    Ok(())
}

pub async fn next(_ctx: &Context, _req: &Request, _res: &mut Response) -> Result<(), Error> {
    rb::playback::next();
    Ok(())
}

pub async fn previous(_ctx: &Context, _req: &Request, _res: &mut Response) -> Result<(), Error> {
    rb::playback::prev();
    Ok(())
}

pub async fn stop(_ctx: &Context, _req: &Request, _res: &mut Response) -> Result<(), Error> {
    rb::playback::hard_stop();
    Ok(())
}
