use std::{env, ffi::CString};

use actix_web::{error::ErrorInternalServerError, web, HttpResponse};
use local_ip_addr::get_local_ip_address;
use rand::seq::SliceRandom;
use rockbox_chromecast::Chromecast;
use rockbox_library::repo;
use rockbox_sys::{
    self as rb,
    types::{audio_status::AudioStatus, mp3_entry::Mp3Entry},
};
use rockbox_traits::types::track::Track;
use rockbox_types::{device::Device, LoadTracks, NewVolume};
use serde::Deserialize;

use crate::{
    handlers::playlists::hydrate_entry_from_track, http::AppState, GLOBAL_MUTEX, PLAYER_MUTEX,
};

type HandlerResult = actix_web::Result<HttpResponse>;

unsafe extern "C" {
    fn save_remote_track_metadata(url: *const std::ffi::c_char) -> i32;
}

pub async fn load(state: web::Data<AppState>, body: web::Json<LoadTracks>) -> HandlerResult {
    let _player_mutex = PLAYER_MUTEX.lock().unwrap();
    let mut player = state.player.lock().unwrap();
    if player.is_none() {
        return Ok(HttpResponse::NotFound().finish());
    }

    let mut current_device = state.current_device.lock().unwrap();
    let devices = state.devices.lock().unwrap();
    let device = devices
        .iter()
        .find(|d| d.id == *current_device.as_ref().unwrap().id)
        .cloned();
    if let Some(device) = device {
        let mut mutex = GLOBAL_MUTEX.lock().unwrap();
        *mutex = 1;
        *player = Chromecast::connect(device.clone()).map_err(ErrorInternalServerError)?;
        *current_device = Some(device);
    }

    let player = player.as_deref_mut().unwrap();

    let request = body.into_inner();

    for path in &request.tracks {
        if path.starts_with("http://") || path.starts_with("https://") {
            ensure_remote_track_metadata(path.clone())
                .await
                .map_err(ErrorInternalServerError)?;
        }
    }

    let rockbox_addr = env::var("ROCKBOX_ADDR").unwrap_or_else(|_| get_local_ip_address().unwrap());
    let rockbox_port = env::var("ROCKBOX_GRAPHQL_PORT").unwrap_or_else(|_| "6062".to_string());
    let mut tracks = Vec::new();

    for requested_path in &request.tracks {
        let track = {
            let kv = state.kv.lock().unwrap();
            kv.get(requested_path).cloned()
        };

        let track = match track {
            Some(t) => Some(t),
            None => {
                let t = repo::track::find_by_path(state.pool.clone(), requested_path)
                    .await
                    .map_err(ErrorInternalServerError)?;
                if let Some(ref t) = t {
                    let mut kv = state.kv.lock().unwrap();
                    kv.set(requested_path, t.clone());
                }
                t
            }
        };

        if let Some(track) = track {
            tracks.push(Track {
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
            });
        }
    }

    if tracks.is_empty() {
        return Ok(HttpResponse::NotFound().body("No playable tracks found"));
    }

    let mut tracks = tracks;
    if Some(true) == request.shuffle {
        tracks.shuffle(&mut rand::thread_rng());
    }

    player
        .load_tracks(tracks, None)
        .await
        .map_err(ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().finish())
}

#[derive(Deserialize)]
pub struct PlayQuery {
    pub elapsed: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn play(state: web::Data<AppState>, query: web::Query<PlayQuery>) -> HandlerResult {
    let elapsed = query.elapsed.unwrap_or(0);
    let offset = query.offset.unwrap_or(0);

    // Route through fw_bus so audio_play() runs on the broker (a real
    // Rockbox kernel thread), not on the actix worker. Calling firmware
    // kernel functions from a non-Rockbox pthread corrupts the global
    // `__cores[0].running` slot and crashes the scheduler — see
    // crates/server/src/fw_bus.rs.
    web::block(move || {
        let _player_mutex = PLAYER_MUTEX.lock().unwrap();
        if state.player.lock().unwrap().is_none() {
            crate::fw_bus::send_and_wait(|reply| crate::fw_bus::FwCmd::Play {
                elapsed,
                offset,
                reply: Some(reply),
            });
        }
    })
    .await
    .map_err(ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().finish())
}

pub async fn pause(state: web::Data<AppState>) -> HandlerResult {
    let has_external = state.player.lock().unwrap().is_some();

    if has_external {
        let mut player = state.player.lock().unwrap();
        if let Some(p) = player.as_deref_mut() {
            p.pause().await.map_err(ErrorInternalServerError)?;
        }
    } else {
        web::block(move || {
            let _player_mutex = PLAYER_MUTEX.lock().unwrap();
            crate::fw_bus::send_and_wait(|reply| crate::fw_bus::FwCmd::Pause {
                reply: Some(reply),
            });
        })
        .await
        .map_err(ErrorInternalServerError)?;
    }

    Ok(HttpResponse::Ok().finish())
}

#[derive(Deserialize)]
pub struct FfRewindQuery {
    pub newtime: Option<i32>,
}

pub async fn ff_rewind(query: web::Query<FfRewindQuery>) -> HandlerResult {
    let newtime = query.newtime.unwrap_or(0);
    web::block(move || {
        let _player_mutex = PLAYER_MUTEX.lock().unwrap();
        crate::fw_bus::send_and_wait(|reply| crate::fw_bus::FwCmd::FfRewind {
            newtime,
            reply: Some(reply),
        });
    })
    .await
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().finish())
}

pub async fn status(state: web::Data<AppState>) -> HandlerResult {
    let has_external = state.player.lock().unwrap().is_some();

    if has_external {
        let mut player = state.player.lock().unwrap();
        if let Some(p) = player.as_deref_mut() {
            let current_playback = p
                .get_current_playback()
                .await
                .map_err(ErrorInternalServerError)?;
            return Ok(HttpResponse::Ok().json(AudioStatus {
                status: if current_playback.is_playing { 1 } else { 0 },
            }));
        }
    }

    let status = web::block(|| {
        let _player_mutex = PLAYER_MUTEX.lock().unwrap();
        rb::playback::status()
    })
    .await
    .map_err(ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(status))
}

pub async fn current_track(state: web::Data<AppState>) -> HandlerResult {
    let has_external = state.player.lock().unwrap().is_some();

    if has_external {
        let mut player = state.player.lock().unwrap();
        if let Some(p) = player.as_deref_mut() {
            let current_playback = p
                .get_current_playback()
                .await
                .map_err(ErrorInternalServerError)?;
            let mut track: Option<Mp3Entry> = current_playback.current_track.map(|mut t| {
                if t.path.is_empty() {
                    t.path = t.uri.clone();
                }
                t.into()
            });

            if let Some(lookup_path) = track
                .as_ref()
                .map(|t| t.path.clone())
                .filter(|path| !path.is_empty())
            {
                let metadata = find_track_metadata(&state, &lookup_path)
                    .await
                    .map_err(ErrorInternalServerError)?;
                if let Some(metadata) = metadata {
                    let t = track.as_mut().unwrap();
                    hydrate_entry_from_track(t, &metadata);
                }
            }

            let track = track.map(|mut t| {
                t.elapsed = current_playback.position_ms as u64;
                t
            });
            return Ok(HttpResponse::Ok().json(track));
        }
    }

    // Builtin player: FFI calls on a blocking thread, then DB lookup async
    let (track, audio_path, playlist_path) = web::block(|| {
        let _player_mutex = PLAYER_MUTEX.lock().unwrap();
        let track = rb::playback::current_track();
        let audio_path: Option<String> = track.as_ref().map(|t| t.path.clone());
        let playlist_index = rb::playlist::index();
        let playlist_path = if playlist_index >= 0 {
            let info = rb::playlist::get_track_info(playlist_index);
            if !info.filename.is_empty() {
                Some(info.filename)
            } else {
                None
            }
        } else {
            None
        };
        (track, audio_path, playlist_path)
    })
    .await
    .map_err(ErrorInternalServerError)?;

    let mut track = track;
    let lookup_path = playlist_path.or(audio_path);
    if let Some(path) = lookup_path {
        if let Some(metadata) = find_track_metadata(&state, &path)
            .await
            .map_err(ErrorInternalServerError)?
        {
            if let Some(t) = track.as_mut() {
                hydrate_entry_from_track(t, &metadata);
            }
        }
    }
    Ok(HttpResponse::Ok().json(track))
}

pub async fn next_track(state: web::Data<AppState>) -> HandlerResult {
    if state.player.lock().unwrap().is_some() {
        return Ok(HttpResponse::Ok().json(Option::<Mp3Entry>::None));
    }

    let (track, audio_path, playlist_path) = web::block(|| {
        let _player_mutex = PLAYER_MUTEX.lock().unwrap();
        let track = rb::playback::next_track();
        let audio_path: Option<String> = track.as_ref().map(|t| t.path.clone());
        let current_index = rb::playlist::index();
        let next_index = current_index + 1;
        let playlist_path = if next_index >= 0 && next_index < rb::playlist::amount() {
            let info = rb::playlist::get_track_info(next_index);
            if !info.filename.is_empty() {
                Some(info.filename)
            } else {
                None
            }
        } else {
            None
        };
        (track, audio_path, playlist_path)
    })
    .await
    .map_err(ErrorInternalServerError)?;

    let mut track = track;
    let lookup_path = playlist_path.or(audio_path);
    if let Some(path) = lookup_path {
        let hash = format!("{:x}", md5::compute(path.as_bytes()));
        if let Some(metadata) = repo::track::find_by_md5(state.pool.clone(), &hash)
            .await
            .map_err(ErrorInternalServerError)?
        {
            if let Some(t) = track.as_mut() {
                hydrate_entry_from_track(t, &metadata);
            }
        }
    }
    Ok(HttpResponse::Ok().json(track))
}

pub async fn flush_and_reload_tracks() -> HandlerResult {
    web::block(|| {
        let _player_mutex = PLAYER_MUTEX.lock().unwrap();
        crate::fw_bus::send_and_wait(|reply| crate::fw_bus::FwCmd::FlushAndReloadTracks {
            reply: Some(reply),
        });
    })
    .await
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().finish())
}

pub async fn resume(state: web::Data<AppState>) -> HandlerResult {
    let has_external = state.player.lock().unwrap().is_some();

    if has_external {
        let mut player = state.player.lock().unwrap();
        if let Some(p) = player.as_deref_mut() {
            p.play().await.map_err(ErrorInternalServerError)?;
        }
    } else {
        web::block(|| {
            let _player_mutex = PLAYER_MUTEX.lock().unwrap();
            crate::fw_bus::send_and_wait(|reply| crate::fw_bus::FwCmd::Resume {
                reply: Some(reply),
            });
        })
        .await
        .map_err(ErrorInternalServerError)?;
    }

    Ok(HttpResponse::Ok().finish())
}

pub async fn next() -> HandlerResult {
    web::block(|| {
        let _player_mutex = PLAYER_MUTEX.lock().unwrap();
        crate::fw_bus::send_and_wait(|reply| crate::fw_bus::FwCmd::Next { reply: Some(reply) });
    })
    .await
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().finish())
}

pub async fn previous() -> HandlerResult {
    web::block(|| {
        let _player_mutex = PLAYER_MUTEX.lock().unwrap();
        crate::fw_bus::send_and_wait(|reply| crate::fw_bus::FwCmd::Prev { reply: Some(reply) });
    })
    .await
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().finish())
}

pub async fn stop() -> HandlerResult {
    web::block(|| {
        let _player_mutex = PLAYER_MUTEX.lock().unwrap();
        crate::fw_bus::send_and_wait(|reply| crate::fw_bus::FwCmd::Stop { reply: Some(reply) });
    })
    .await
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().finish())
}

pub async fn get_file_position() -> HandlerResult {
    let position = web::block(|| {
        let _player_mutex = PLAYER_MUTEX.lock().unwrap();
        rb::playback::get_file_pos()
    })
    .await
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(position))
}

pub async fn get_volume() -> HandlerResult {
    let (volume, min, max) = web::block(|| {
        const SOUND_VOLUME: i32 = 0;
        (
            rb::sound::current(SOUND_VOLUME),
            rb::sound::min(SOUND_VOLUME),
            rb::sound::max(SOUND_VOLUME),
        )
    })
    .await
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({ "volume": volume, "min": min, "max": max })))
}

pub async fn adjust_volume(body: web::Json<NewVolume>) -> HandlerResult {
    let new_volume = body.into_inner();
    let steps = new_volume.steps;
    web::block(move || rb::sound::adjust_volume(steps))
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(new_volume))
}

pub async fn get_current_player(state: web::Data<AppState>) -> HandlerResult {
    let device = state.current_device.lock().unwrap();

    if let Some(device) = device.as_ref() {
        return Ok(HttpResponse::Ok().json(device));
    }

    Ok(HttpResponse::Ok().json(Device {
        name: "Rockbox (Default Player)".to_string(),
        app: "default".to_string(),
        service: "rockbox".to_string(),
        ..Default::default()
    }))
}

async fn find_track_metadata(
    state: &AppState,
    path: &str,
) -> Result<Option<rockbox_library::entity::track::Track>, anyhow::Error> {
    let hash = format!("{:x}", md5::compute(path.as_bytes()));
    let mut metadata = repo::track::find_by_md5(state.pool.clone(), &hash).await?;
    let internal_track = find_internal_track_by_url(state, path).await?;

    if metadata
        .as_ref()
        .map(|track| track.album_art.is_none())
        .unwrap_or(true)
    {
        if let Some(track) = internal_track.clone() {
            metadata = Some(track);
        }
    }

    if path.starts_with("http://") || path.starts_with("https://") {
        if metadata
            .as_ref()
            .map(|track| track.album_art.is_none())
            .unwrap_or(true)
            && internal_track.is_none()
        {
            ensure_remote_track_metadata(path.to_string()).await?;
            metadata = repo::track::find_by_md5(state.pool.clone(), &hash).await?;
        }
    }

    Ok(metadata)
}

async fn ensure_remote_track_metadata(path: String) -> Result<(), anyhow::Error> {
    let status = tokio::task::spawn_blocking(move || -> Result<i32, anyhow::Error> {
        let path_cstr = CString::new(path.as_str())?;
        Ok(unsafe { save_remote_track_metadata(path_cstr.as_ptr()) })
    })
    .await??;

    if status != 0 {
        return Err(anyhow::anyhow!("failed to save remote metadata"));
    }

    Ok(())
}

async fn find_internal_track_by_url(
    state: &AppState,
    path: &str,
) -> Result<Option<rockbox_library::entity::track::Track>, anyhow::Error> {
    let url = match reqwest::Url::parse(path) {
        Ok(url) => url,
        Err(_) => return Ok(None),
    };

    let mut segments = match url.path_segments() {
        Some(segments) => segments,
        None => return Ok(None),
    };

    let Some("tracks") = segments.next() else {
        return Ok(None);
    };
    let Some(track_id) = segments.next() else {
        return Ok(None);
    };

    if segments.next().is_some() || track_id.is_empty() {
        return Ok(None);
    }

    repo::track::find(state.pool.clone(), track_id)
        .await
        .map_err(Into::into)
}
