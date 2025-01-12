use std::env;

use crate::http::{Context, Request, Response};
use crate::PLAYER_MUTEX;
use anyhow::Error;
use local_ip_addr::get_local_ip_address;
use rand::seq::SliceRandom;
use rockbox_graphql::read_files;
use rockbox_library::{entity, repo};
use rockbox_network::download_tracks;
use rockbox_sys::types::mp3_entry::Mp3Entry;
use rockbox_sys::{
    self as rb,
    types::{playlist_amount::PlaylistAmount, playlist_info::PlaylistInfo},
    PLAYLIST_INSERT_LAST, PLAYLIST_INSERT_LAST_SHUFFLED,
};
use rockbox_traits::types::track::Track;
use rockbox_types::{DeleteTracks, InsertTracks, NewPlaylist, PlaylistUpdate, StatusCode};
use serde_json::json;

pub async fn create_playlist(
    ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    if req.body.is_none() {
        res.set_status(400);
        return Ok(());
    }
    let body = req.body.as_ref().unwrap();
    let mut new_playlist: NewPlaylist = serde_json::from_str(body).unwrap();

    if let Some(playlist_name) = new_playlist.name.as_ref() {
        if playlist_name.is_empty() {
            res.set_status(400);
            return Ok(());
        }
        let playlist_id = repo::playlist::save(
            ctx.pool.clone(),
            entity::playlist::Playlist {
                id: cuid::cuid1()?,
                name: playlist_name.clone(),
                folder_id: new_playlist.folder_id,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                ..Default::default()
            },
        )
        .await?;

        let tracks = download_tracks(new_playlist.tracks).await?;
        for (position, track_path) in tracks.iter().enumerate() {
            let hash = format!("{:x}", md5::compute(track_path.as_bytes()));
            let track = repo::track::find_by_md5(ctx.pool.clone(), &hash).await?;
            if track.is_none() {
                continue;
            }
            repo::playlist_tracks::save(
                ctx.pool.clone(),
                entity::playlist_tracks::PlaylistTracks {
                    id: cuid::cuid1()?,
                    playlist_id: playlist_id.clone(),
                    track_id: track.unwrap().id,
                    position: position as u32,
                    created_at: chrono::Utc::now(),
                },
            )
            .await?;
        }

        res.set_status(200);
        res.text("0");
        return Ok(());
    }

    if new_playlist.tracks.is_empty() {
        return Ok(());
    }

    new_playlist.tracks = download_tracks(new_playlist.tracks).await?;

    let player_mutex = PLAYER_MUTEX.lock().unwrap();

    let dir = new_playlist.tracks[0].clone();
    let dir_parts: Vec<_> = dir.split('/').collect();
    let dir = dir_parts[0..dir_parts.len() - 1].join("/");
    let status = rb::playlist::create(&dir, None);
    if status == -1 {
        res.set_status(500);
        return Ok(());
    }
    let start_index = rb::playlist::build_playlist(
        new_playlist.tracks.iter().map(|t| t.as_str()).collect(),
        0,
        new_playlist.tracks.len() as i32,
    );
    res.text(&start_index.to_string());
    drop(player_mutex);
    Ok(())
}

pub async fn start_playlist(
    _ctx: &Context,
    req: &Request,
    _res: &mut Response,
) -> Result<(), Error> {
    let player_mutex = PLAYER_MUTEX.lock().unwrap();
    let start_index = match req.query_params.get("start_index") {
        Some(start_index) => start_index.as_str().unwrap_or("0").parse().unwrap_or(0),
        None => 0,
    };
    let elapsed = match req.query_params.get("elapsed") {
        Some(elapsed) => elapsed.as_str().unwrap_or("0").parse().unwrap_or(0),
        None => 0,
    };
    let offset = match req.query_params.get("offset") {
        Some(offset) => offset.as_str().unwrap_or("0").parse().unwrap_or(0),
        None => 0,
    };
    rb::playlist::start(start_index, elapsed, offset);
    drop(player_mutex);
    Ok(())
}

pub async fn shuffle_playlist(
    _ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let player_mutex = PLAYER_MUTEX.lock().unwrap();
    let start_index = match req.query_params.get("start_index") {
        Some(start_index) => start_index.as_str().unwrap_or("0").parse().unwrap_or(0),
        None => 0,
    };
    let seed = rb::system::current_tick();
    let ret = rb::playlist::shuffle(seed as i32, start_index as i32);
    res.text(&ret.to_string());
    drop(player_mutex);
    Ok(())
}

pub async fn get_playlist_amount(
    _ctx: &Context,
    _req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let player_mutex = PLAYER_MUTEX.lock().unwrap();
    let amount = rb::playlist::amount();
    res.json(&PlaylistAmount { amount });
    drop(player_mutex);
    Ok(())
}

pub async fn resume_playlist(
    _ctx: &Context,
    _req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let player_mutex = PLAYER_MUTEX.lock().unwrap();
    let status = rb::system::get_global_status();
    let playback_status = rb::playback::status();

    if status.resume_index == -1 || playback_status.status == 1 {
        res.json(&StatusCode { code: -1 });
        return Ok(());
    }

    let code = rb::playlist::resume();
    res.json(&StatusCode { code });
    drop(player_mutex);
    Ok(())
}

pub async fn resume_track(
    _ctx: &Context,
    _req: &Request,
    _res: &mut Response,
) -> Result<(), Error> {
    let player_mutex = PLAYER_MUTEX.lock().unwrap();
    let status = rb::system::get_global_status();
    rb::playlist::resume_track(
        status.resume_index,
        status.resume_crc32,
        status.resume_elapsed.into(),
        status.resume_offset.into(),
    );
    drop(player_mutex);
    Ok(())
}

pub async fn get_playlist_tracks(
    ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let playlist_id = &req.params[0];

    if playlist_id != "current" {
        let tracks = repo::playlist_tracks::find_by_playlist(ctx.pool.clone(), playlist_id).await?;
        let mut entries: Vec<Mp3Entry> = vec![];

        for track in tracks {
            entries.push(Mp3Entry {
                id: Some(track.id),
                path: track.path,
                title: track.title,
                artist: track.artist,
                album: track.album,
                length: track.length as u64,
                tracknum: track.track_number.unwrap_or_default() as i32,
                album_art: track.album_art,
                artist_id: Some(track.artist_id),
                album_id: Some(track.album_id),
                filesize: track.filesize as u64,
                frequency: track.frequency as u64,
                ..Default::default()
            });
        }
        res.json(&entries);
        return Ok(());
    }

    let player_mutex = PLAYER_MUTEX.lock().unwrap();
    let mut entries = vec![];
    let amount = rb::playlist::amount();

    for i in 0..amount {
        let info = rb::playlist::get_track_info(i);
        let entry = rb::metadata::get_metadata(-1, &info.filename);
        entries.push(entry);
    }

    res.json(&entries);

    drop(player_mutex);
    Ok(())
}

pub async fn insert_tracks(ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
    let playlist_id = &req.params[0];
    if playlist_id != "current" {
        let req_body = req.body.as_ref().unwrap();
        let mut tracklist: InsertTracks = serde_json::from_str(&req_body)?;
        let tracks = download_tracks(tracklist.tracks).await?;

        if let Some(dir) = &tracklist.directory {
            tracklist.tracks = read_files(dir.clone()).await?;
        }

        let playlist = repo::playlist::find(ctx.pool.clone(), playlist_id).await?;
        if playlist.is_none() {
            res.set_status(404);
            return Ok(());
        }

        for track_path in tracks {
            let current_tracks =
                repo::playlist_tracks::find_by_playlist(ctx.pool.clone(), playlist_id).await?;
            let hash = format!("{:x}", md5::compute(track_path.as_bytes()));

            if let Some(track) = repo::track::find_by_md5(ctx.pool.clone(), &hash).await? {
                repo::playlist_tracks::save(
                    ctx.pool.clone(),
                    entity::playlist_tracks::PlaylistTracks {
                        id: cuid::cuid1()?,
                        playlist_id: playlist_id.clone(),
                        track_id: track.id,
                        position: current_tracks.len() as u32,
                        created_at: chrono::Utc::now(),
                    },
                )
                .await?;
            }
        }
        res.text("0");
        return Ok(());
    }

    let player_mutex = PLAYER_MUTEX.lock().unwrap();
    let req_body = req.body.as_ref().unwrap();
    let mut tracklist: InsertTracks = serde_json::from_str(&req_body).unwrap();
    let amount = rb::playlist::amount();

    let mut player = ctx.player.lock().unwrap();

    if let Some(player) = player.as_deref_mut() {
        let kv = ctx.kv.lock().unwrap();
        let rockbox_addr =
            env::var("ROCKBOX_ADDR").unwrap_or_else(|_| get_local_ip_address().unwrap());
        let rockbox_port = env::var("ROCKBOX_GRAPHQL_PORT").unwrap_or_else(|_| "6062".to_string());

        let tracks = tracklist
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

        for track in tracks {
            player.play_next(track).await?;
        }

        res.text("0");
        return Ok(());
    }

    if let Some(dir) = &tracklist.directory {
        tracklist.tracks = read_files(dir.clone()).await?;
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

    let mut tracks: Vec<&str> = tracklist.tracks.iter().map(|t| t.as_str()).collect();
    let position = match tracklist.position {
        PLAYLIST_INSERT_LAST_SHUFFLED => {
            tracks.shuffle(&mut rand::thread_rng());
            PLAYLIST_INSERT_LAST
        }
        _ => tracklist.position,
    };
    rb::playlist::insert_tracks(tracks, position, tracklist.tracks.len() as i32);

    res.text(&tracklist.position.to_string());

    drop(player_mutex);

    Ok(())
}

pub async fn remove_tracks(ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
    let playlist_id = &req.params[0];
    if playlist_id != "current" {
        let req_body = req.body.as_ref().unwrap();
        let params = serde_json::from_str::<DeleteTracks>(&req_body)?;

        if params.positions.is_empty() {
            res.text("0");
            return Ok(());
        }

        for position in params.positions {
            repo::playlist_tracks::delete_track_at(ctx.pool.clone(), playlist_id, position as u32)
                .await?;
        }
        res.text("0");
        return Ok(());
    }

    let player_mutex = PLAYER_MUTEX.lock().unwrap();
    let player = ctx.player.lock().unwrap();

    if let Some(_) = player.as_deref() {
        res.text("0");
        return Ok(());
    }

    let req_body = req.body.as_ref().unwrap();
    let params = serde_json::from_str::<DeleteTracks>(&req_body)?;
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
    drop(player_mutex);
    Ok(())
}

pub async fn current_playlist(
    ctx: &Context,
    _req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let player_mutex = PLAYER_MUTEX.lock().unwrap();
    let mut metadata_cache = ctx.metadata_cache.lock().await;
    let mut entries = vec![];
    let amount = rb::playlist::amount();

    for i in 0..amount {
        let info = rb::playlist::get_track_info(i);
        let mut entry = rb::metadata::get_metadata(-1, &info.filename);
        let hash = format!("{:x}", md5::compute(info.filename.as_bytes()));

        if let Some(entry) = metadata_cache.get(&hash) {
            entries.push(entry.clone());
            continue;
        }

        let track = repo::track::find_by_md5(ctx.pool.clone(), &hash).await?;

        if track.is_none() {
            entries.push(entry.clone());
            continue;
        }

        entry.album_art = track.as_ref().map(|t| t.album_art.clone()).flatten();
        entry.album_id = track.as_ref().map(|t| t.album_id.clone());
        entry.artist_id = track.as_ref().map(|t| t.artist_id.clone());
        entry.genre_id = track.as_ref().map(|t| t.genre_id.clone());
        entry.id = track.as_ref().map(|t| t.id.clone());

        metadata_cache.insert(hash, entry.clone());
        entries.push(entry);
    }

    res.json(&entries);

    drop(player_mutex);
    Ok(())
}

pub async fn get_playlist(ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
    let playlist_id = &req.params[0];
    if playlist_id != "current" {
        let playlist = repo::playlist::find(ctx.pool.clone(), playlist_id).await?;
        if playlist.is_none() {
            res.set_status(404);
            return Ok(());
        }

        let playlist = playlist.unwrap();

        let tracks = repo::playlist_tracks::find_by_playlist(ctx.pool.clone(), playlist_id).await?;
        let mut entries: Vec<Mp3Entry> = vec![];
        for track in tracks.clone() {
            let mut entry = rb::metadata::get_metadata(-1, &track.path);
            entry.album_art = track.album_art;
            entry.album_id = Some(track.album_id);
            entry.artist_id = Some(track.artist_id);
            entry.genre_id = Some(track.genre_id);
            entry.id = Some(track.id);
            entries.push(entry);
        }

        let result = PlaylistInfo {
            id: Some(playlist.id),
            amount: tracks.len() as i32,
            entries,
            name: Some(playlist.name),
            folder_id: playlist.folder_id,
            image: playlist.image,
            description: playlist.description,
            created_at: Some(playlist.created_at.to_rfc3339()),
            updated_at: Some(playlist.updated_at.to_rfc3339()),
            ..Default::default()
        };
        res.json(&result);
        return Ok(());
    }

    let player_mutex = PLAYER_MUTEX.lock().unwrap();
    let mut player = ctx.player.lock().unwrap();

    if let Some(player) = player.as_deref_mut() {
        let current_playback = player.get_current_playback().await?;
        let tracks = current_playback.items;
        let index = match tracks.len() >= 2 {
            true => tracks.len() - 2,
            false => 0,
        } as i32;

        let result = PlaylistInfo {
            amount: tracks.len() as i32,
            index,
            entries: tracks.into_iter().map(|(t, _)| t.into()).collect(),
            ..Default::default()
        };
        res.json(&result);
        return Ok(());
    }

    let mut metadata_cache = ctx.metadata_cache.lock().await;
    let mut result = rb::playlist::get_current();
    let mut entries = vec![];
    let amount = rb::playlist::amount();

    for i in 0..amount {
        let info = rb::playlist::get_track_info(i);
        let mut entry = rb::metadata::get_metadata(-1, &info.filename);
        let hash = format!("{:x}", md5::compute(info.filename.as_bytes()));

        if let Some(entry) = metadata_cache.get(&hash) {
            entries.push(entry.clone());
            continue;
        }

        let track = repo::track::find_by_md5(ctx.pool.clone(), &hash).await?;

        if track.is_none() {
            entries.push(entry.clone());
            continue;
        }

        entry.album_art = track.as_ref().map(|t| t.album_art.clone()).flatten();
        entry.album_id = track.as_ref().map(|t| t.album_id.clone());
        entry.artist_id = track.as_ref().map(|t| t.artist_id.clone());
        entry.genre_id = track.as_ref().map(|t| t.genre_id.clone());
        entry.id = track.as_ref().map(|t| t.id.clone());

        metadata_cache.insert(hash, entry.clone());
        entries.push(entry);
    }

    result.amount = amount;
    result.max_playlist_size = rb::playlist::max_playlist_size();
    result.index = rb::playlist::index();
    result.first_index = rb::playlist::first_index();
    result.last_insert_pos = rb::playlist::last_insert_pos();
    result.seed = rb::playlist::seed();
    result.last_shuffled_start = rb::playlist::last_shuffled_start();
    result.entries = entries;

    res.json(&result);
    drop(player_mutex);
    Ok(())
}

pub async fn delete_playlist(
    ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let playlist_id = &req.params[0];
    repo::playlist::delete(ctx.pool.clone(), playlist_id).await?;
    repo::playlist_tracks::delete_by_playlist(ctx.pool.clone(), playlist_id).await?;
    res.json(&json!({ "id": playlist_id }));
    Ok(())
}

pub async fn update_playlist(
    ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let playlist_id = &req.params[0];
    if req.body.is_none() {
        res.set_status(400);
        return Ok(());
    }

    let body = req.body.as_ref().unwrap();
    let playlist: PlaylistUpdate = serde_json::from_str(body)?;
    repo::playlist::update(
        ctx.pool.clone(),
        entity::playlist::Playlist {
            id: playlist_id.clone(),
            name: playlist.name.unwrap_or_default(),
            folder_id: playlist.folder_id,
            ..Default::default()
        },
    )
    .await?;

    res.json(&json!({ "id": playlist_id }));
    Ok(())
}

pub async fn get_playlists(ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
    let folder_id = req.query_params.get("folder_id");
    let folder_id = folder_id.map(|f| f.as_str().unwrap().to_string());
    let playlist = repo::playlist::find_by_folder(ctx.pool.clone(), folder_id).await?;
    res.json(&playlist);
    Ok(())
}

pub async fn play_playlist(ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
    let playlist_id = &req.params[0];
    let tracks = repo::playlist_tracks::find_by_playlist(ctx.pool.clone(), playlist_id).await?;

    if tracks.is_empty() {
        res.set_status(200);
        return Ok(());
    }

    let player_mutex = PLAYER_MUTEX.lock().unwrap();

    let dir = tracks[0].clone();
    let dir = dir.path.clone();
    let dir_parts: Vec<_> = dir.split('/').collect();
    let dir = dir_parts[0..dir_parts.len() - 1].join("/");
    let status = rb::playlist::create(&dir, None);
    if status == -1 {
        res.set_status(500);
        return Ok(());
    }
    rb::playlist::build_playlist(
        tracks.iter().map(|t| t.path.as_str()).collect(),
        0,
        tracks.len() as i32,
    );

    let shuffle = match req.query_params.get("shuffle") {
        Some(shuffle) => shuffle.as_str().unwrap_or("0").parse().unwrap_or(0),
        None => 0,
    };

    if shuffle == 1 {
        let seed = rb::system::current_tick();
        rb::playlist::shuffle(seed as i32, 0);
    }

    let start_index = match req.query_params.get("start_index") {
        Some(start_index) => start_index.as_str().unwrap_or("0").parse().unwrap_or(0),
        None => 0,
    };
    let elapsed = match req.query_params.get("elapsed") {
        Some(elapsed) => elapsed.as_str().unwrap_or("0").parse().unwrap_or(0),
        None => 0,
    };
    let offset = match req.query_params.get("offset") {
        Some(offset) => offset.as_str().unwrap_or("0").parse().unwrap_or(0),
        None => 0,
    };
    rb::playlist::start(start_index, elapsed, offset);

    res.set_status(200);
    drop(player_mutex);
    Ok(())
}
