use crate::http::{Context, Request, Response};
use anyhow::Error;
use rockbox_sys as rb;
use rockbox_types::NewVolume;

pub async fn play(_ctx: &Context, req: &Request, _res: &mut Response) -> Result<(), Error> {
    let elapsed = match req.query_params.get("elapsed") {
        Some(elapsed) => elapsed.as_str().unwrap_or("0").parse().unwrap_or(0),
        None => 0,
    };
    let offset = match req.query_params.get("offset") {
        Some(offset) => offset.as_str().unwrap_or("0").parse().unwrap_or(0),
        None => 0,
    };
    rb::playback::play(elapsed, offset);
    Ok(())
}

pub async fn pause(_ctx: &Context, _req: &Request, _res: &mut Response) -> Result<(), Error> {
    rb::playback::pause();
    Ok(())
}

pub async fn ff_rewind(_ctx: &Context, req: &Request, _res: &mut Response) -> Result<(), Error> {
    let newtime = match req.query_params.get("newtime") {
        Some(newtime) => newtime.as_str().unwrap_or("0").parse().unwrap_or(0),
        None => 0,
    };
    rb::playback::ff_rewind(newtime);
    Ok(())
}

pub async fn status(_ctx: &Context, _req: &Request, res: &mut Response) -> Result<(), Error> {
    let status = rb::playback::status();
    res.json(&status);
    Ok(())
}

pub async fn current_track(
    _ctx: &Context,
    _req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
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

pub async fn get_file_position(
    _ctx: &Context,
    _req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let position = rb::playback::get_file_pos();
    res.json(&position);
    Ok(())
}

pub async fn adjust_volume(_ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
    let req_body = req.body.as_ref().unwrap();
    let new_volume: NewVolume = serde_json::from_str(&req_body).unwrap();

    rb::sound::adjust_volume(new_volume.steps);
    res.json(&new_volume);
    Ok(())
}
