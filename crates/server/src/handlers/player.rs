use std::env;

use crate::http::{Context, Request, Response};
use anyhow::Error;
use local_ip_addr::get_local_ip_address;
use rand::seq::SliceRandom;
use rockbox_sys::{
    self as rb,
    types::{audio_status::AudioStatus, mp3_entry::Mp3Entry},
};
use rockbox_traits::types::track::Track;
use rockbox_types::{device::Device, LoadTracks, NewVolume};

pub async fn load(ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
    let mut player = ctx.player.lock().unwrap();
    if player.is_none() {
        res.set_status(404);
        return Ok(());
    }

    let player = player.as_deref_mut().unwrap();

    let req_body = req.body.as_ref().unwrap();
    let request: LoadTracks = serde_json::from_str(&req_body)?;

    let rockbox_addr = env::var("ROCKBOX_ADDR").unwrap_or_else(|_| get_local_ip_address().unwrap());
    let rockbox_port = env::var("ROCKBOX_GRAPHQL_PORT").unwrap_or_else(|_| "6062".to_string());
    let kv = ctx.kv.lock().unwrap();
    let mut tracks = request
        .tracks
        .iter()
        .filter(|t| kv.get(*t).is_some())
        .map(|t| {
            let track = kv.get(t).unwrap();
            Track {
                id: track.id.clone(),
                title: track.title.clone(),
                artist: track.artist.clone(),
                album: track.album.clone(),
                album_artist: Some(track.album_artist.clone()),
                artist_id: Some(track.artist_id.clone()),
                album_id: Some(track.album_id.clone()),
                album_cover: track.album_art.clone().map(|cover| {
                    format!("http://{}:{}/covers/{}", rockbox_addr, rockbox_port, cover)
                }),
                track_number: track.track_number,
                path: track.path.clone(),
                uri: format!(
                    "http://{}:{}/tracks/{}",
                    rockbox_addr, rockbox_port, track.id
                ),
                disc_number: track.disc_number,
                duration: Some(track.length as f32 / 1000.0),
                ..Default::default()
            }
        })
        .collect::<Vec<Track>>();

    if Some(true) == request.shuffle {
        tracks.shuffle(&mut rand::thread_rng());
    }

    player.load_tracks(tracks, None).await?;

    res.set_status(200);

    Ok(())
}

pub async fn play(ctx: &Context, req: &Request, _res: &mut Response) -> Result<(), Error> {
    let elapsed = match req.query_params.get("elapsed") {
        Some(elapsed) => elapsed.as_str().unwrap_or("0").parse().unwrap_or(0),
        None => 0,
    };
    let offset = match req.query_params.get("offset") {
        Some(offset) => offset.as_str().unwrap_or("0").parse().unwrap_or(0),
        None => 0,
    };
    let player = ctx.player.lock().unwrap();

    if player.is_none() {
        rb::playback::play(elapsed, offset);
    }

    Ok(())
}

pub async fn pause(ctx: &Context, _req: &Request, _res: &mut Response) -> Result<(), Error> {
    let player = ctx.player.lock().unwrap();

    match player.as_deref() {
        Some(player) => {
            player.pause().await?;
        }
        None => {
            rb::playback::pause();
        }
    }

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

pub async fn status(ctx: &Context, _req: &Request, res: &mut Response) -> Result<(), Error> {
    let mut player = ctx.player.lock().unwrap();

    if let Some(player) = player.as_deref_mut() {
        let current_playback = player.get_current_playback().await?;
        res.json(&AudioStatus {
            status: match current_playback.is_playing {
                true => 1,
                false => 0,
            },
        });
        return Ok(());
    }

    let status = rb::playback::status();
    res.json(&status);
    Ok(())
}

pub async fn current_track(ctx: &Context, _req: &Request, res: &mut Response) -> Result<(), Error> {
    let mut player = ctx.player.lock().unwrap();

    if let Some(player) = player.as_deref_mut() {
        let current_playback = player.get_current_playback().await?;
        let track: Option<Mp3Entry> = current_playback.current_track.map(|t| t.into());
        let track = track.map(|mut t| {
            t.elapsed = current_playback.position_ms as u64;
            t
        });
        res.json(&track);
        return Ok(());
    }

    let track = rb::playback::current_track();
    res.json(&track);
    Ok(())
}

pub async fn next_track(ctx: &Context, _req: &Request, res: &mut Response) -> Result<(), Error> {
    let player = ctx.player.lock().unwrap();

    if let Some(_player) = player.as_deref() {
        return Ok(());
    }

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

pub async fn resume(ctx: &Context, _req: &Request, _res: &mut Response) -> Result<(), Error> {
    let player = ctx.player.lock().unwrap();

    match player.as_deref() {
        Some(player) => {
            player.play().await?;
        }
        None => {
            rb::playback::resume();
        }
    }

    Ok(())
}

pub async fn next(ctx: &Context, _req: &Request, _res: &mut Response) -> Result<(), Error> {
    let player = ctx.player.lock().unwrap();

    match player.as_deref() {
        Some(player) => {
            player.next().await?;
        }
        None => {
            rb::playback::next();
        }
    }

    Ok(())
}

pub async fn previous(ctx: &Context, _req: &Request, _res: &mut Response) -> Result<(), Error> {
    let player = ctx.player.lock().unwrap();

    match player.as_deref() {
        Some(player) => {
            player.previous().await?;
        }
        None => {
            rb::playback::prev();
        }
    }

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

pub async fn get_current_player(
    ctx: &Context,
    _req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let device = ctx.current_device.lock().unwrap();

    if let Some(device) = device.as_ref() {
        res.json(device);
        return Ok(());
    }

    res.json(&Device {
        name: "Rockbox (Default Player)".to_string(),
        app: "default".to_string(),
        service: "rockbox".to_string(),
        ..Default::default()
    });
    Ok(())
}
