use crate::{
    http::{Context, Request, Response},
    types::{DeleteTracks, InsertTracks, NewPlaylist},
};
use anyhow::Error;
use rockbox_sys::{self as rb, types::playlist_amount::PlaylistAmount};

pub async fn create_playlist(
    _ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    if req.body.is_none() {
        res.set_status(400);
        return Ok(());
    }
    let body = req.body.as_ref().unwrap();
    let new_playslist: NewPlaylist = serde_json::from_str(&body).unwrap();

    if new_playslist.tracks.is_empty() {
        return Ok(());
    }

    let dir = new_playslist.tracks[0].clone();
    let dir_parts: Vec<_> = dir.split('/').collect();
    let dir = dir_parts[0..dir_parts.len() - 1].join("/");
    let status = rb::playlist::create(&dir, None);
    if status == -1 {
        res.set_status(500);
        return Ok(());
    }
    let start_index = rb::playlist::build_playlist(
        new_playslist.tracks.iter().map(|t| t.as_str()).collect(),
        0,
        new_playslist.tracks.len() as i32,
    );
    res.text(&start_index.to_string());
    Ok(())
}

pub async fn start_playlist(
    _ctx: &Context,
    req: &Request,
    _res: &mut Response,
) -> Result<(), Error> {
    let start_index = req
        .query_params
        .get("start_index")
        .unwrap()
        .as_i64()
        .unwrap_or(0);
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
    rb::playlist::start(start_index as i32, elapsed as u64, offset as u64);
    Ok(())
}

pub async fn shuffle_playlist(
    _ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let start_index = req
        .query_params
        .get("start_index")
        .unwrap()
        .as_i64()
        .unwrap_or(0);
    let seed = rb::system::current_tick();
    let ret = rb::playlist::shuffle(seed as i32, start_index as i32);
    res.text(&ret.to_string());
    Ok(())
}

pub async fn get_playlist_amount(
    _ctx: &Context,
    _req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let amount = rb::playlist::amount();
    res.json(&PlaylistAmount { amount });
    Ok(())
}

pub async fn resume_playlist(
    _ctx: &Context,
    _req: &Request,
    _res: &mut Response,
) -> Result<(), Error> {
    rb::playlist::resume();
    Ok(())
}

pub async fn resume_track(
    _ctx: &Context,
    _req: &Request,
    _res: &mut Response,
) -> Result<(), Error> {
    let status = rb::system::get_global_status();
    rb::playlist::resume_track(
        status.resume_index,
        status.resume_crc32,
        status.resume_elapsed.into(),
        status.resume_offset.into(),
    );
    Ok(())
}

pub async fn get_playlist_tracks(
    _ctx: &Context,
    _req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let mut entries = vec![];
    let amount = rb::playlist::amount();

    for i in 0..amount {
        let info = rb::playlist::get_track_info(i);
        let entry = rb::metadata::get_metadata(-1, &info.filename);
        entries.push(entry);
    }

    res.json(&entries);
    Ok(())
}

pub async fn insert_tracks(_ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
    let req_body = req.body.as_ref().unwrap();
    let tracklist: InsertTracks = serde_json::from_str(&req_body).unwrap();
    let amount = rb::playlist::amount();

    if let Some(dir) = &tracklist.directory {
        if amount == 0 {
            let status = rb::playlist::create(dir, None);
            if status == -1 {
                res.set_status(500);
                res.text("Failed to create playlist");
                return Ok(());
            }
        }
        rb::playlist::insert_directory(dir, tracklist.position, true, true);
        if tracklist.shuffle.unwrap_or(false) {
            let random_seed = rb::system::current_tick() as i32;
            rb::playlist::shuffle(random_seed, 0);
        }
    }

    if tracklist.tracks.is_empty() {
        res.text("0");
        return Ok(());
    }

    if amount == 0 {
        let dir = tracklist.tracks[0].clone();
        let dir_parts: Vec<_> = dir.split('/').collect();
        let dir = dir_parts[0..dir_parts.len() - 1].join("/");
        let status = rb::playlist::create(&dir, None);
        if status == -1 {
            res.set_status(500);
            res.text("Failed to create playlist");
            return Ok(());
        }
        let start_index = 0;
        let start_index = rb::playlist::build_playlist(
            tracklist.tracks.iter().map(|t| t.as_str()).collect(),
            start_index,
            tracklist.tracks.len() as i32,
        );
        res.text(&start_index.to_string());
        return Ok(());
    }

    for (_, track) in tracklist.tracks.iter().enumerate() {
        rb::playlist::insert_track(track, tracklist.position, true, false);
    }

    res.text(&tracklist.position.to_string());

    Ok(())
}

pub async fn remove_tracks(_ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
    let req_body = req.body.as_ref().unwrap();
    let params = serde_json::from_str::<DeleteTracks>(&req_body).unwrap();
    let mut ret = 0;

    for position in &params.positions {
        ret = rb::playlist::delete_track(position.clone());
    }

    if params.positions.is_empty() {
        ret = rb::playlist::remove_all_tracks();
        res.text(&ret.to_string());
        return Ok(());
    }

    res.text(&ret.to_string());
    Ok(())
}

pub async fn current_playlist(
    _ctx: &Context,
    _req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let mut entries = vec![];
    let amount = rb::playlist::amount();

    for i in 0..amount {
        let info = rb::playlist::get_track_info(i);
        let entry = rb::metadata::get_metadata(-1, &info.filename);
        entries.push(entry);
    }

    res.json(&entries);

    Ok(())
}
