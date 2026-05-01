use std::{env, sync::atomic::Ordering, sync::Arc};

use actix_web::{error::ErrorInternalServerError, web, HttpResponse};
use futures_util::stream::{FuturesUnordered, StreamExt};
use local_ip_addr::get_local_ip_address;
use rand::seq::SliceRandom;
use rockbox_graphql::read_files_with_art;
use rockbox_library::audio_scan::save_audio_metadata;
use rockbox_library::repo;
use rockbox_sys::{
    self as rb,
    types::{playlist_amount::PlaylistAmount, playlist_info::PlaylistInfo},
    PLAYLIST_INSERT_LAST, PLAYLIST_INSERT_LAST_SHUFFLED,
};
use rockbox_traits::types::track::Track;
use rockbox_types::{DeleteTracks, InsertTracks, NewPlaylist, StatusCode};
use serde::Deserialize;
use tokio::sync::Semaphore;

use crate::{http::AppState, PLAYER_MUTEX, PLAYLIST_DIRTY};

type HandlerResult = actix_web::Result<HttpResponse>;

fn trim_path(s: String) -> String {
    let s = s.trim();
    s.split('#').next().unwrap_or(s).to_string()
}

pub async fn create_playlist(
    state: web::Data<AppState>,
    body: web::Json<NewPlaylist>,
) -> HandlerResult {
    let mut new_playlist = body.into_inner();
    new_playlist.tracks = new_playlist.tracks.into_iter().map(trim_path).collect();

    if new_playlist.tracks.is_empty() {
        return Ok(HttpResponse::Ok().finish());
    }

    let tracks_with_art: Vec<(String, Option<String>)> = new_playlist
        .tracks
        .iter()
        .map(|t| (t.clone(), None))
        .collect();
    persist_remote_track_metadata(state.pool.clone(), tracks_with_art).await;

    let _player_mutex = PLAYER_MUTEX.lock().unwrap();

    let current_is_http = rb::playback::current_track()
        .map(|t| t.path.starts_with("http://") || t.path.starts_with("https://"))
        .unwrap_or(false);
    let new_is_http = new_playlist.tracks[0].starts_with("http://")
        || new_playlist.tracks[0].starts_with("https://");
    if current_is_http || new_is_http {
        rb::playback::hard_stop();
    }

    let first = &new_playlist.tracks[0];
    let dir = if first.starts_with("http://") || first.starts_with("https://") {
        std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())
    } else {
        let parts: Vec<_> = first.split('/').collect();
        parts[..parts.len().saturating_sub(1)].join("/")
    };
    rb::playlist::create(&dir, None);

    let start_index = rb::playlist::build_playlist(
        new_playlist.tracks.iter().map(|t| t.as_str()).collect(),
        0,
        new_playlist.tracks.len() as i32,
    );
    PLAYLIST_DIRTY.store(true, Ordering::Relaxed);
    Ok(HttpResponse::Ok().body(start_index.to_string()))
}

#[derive(Deserialize)]
pub struct StartPlaylistQuery {
    start_index: Option<i32>,
    elapsed: Option<u64>,
    offset: Option<u64>,
}

pub async fn start_playlist(query: web::Query<StartPlaylistQuery>) -> HandlerResult {
    let start_index = query.start_index.unwrap_or(0);
    let elapsed = query.elapsed.unwrap_or(0);
    let offset = query.offset.unwrap_or(0);
    web::block(move || {
        let _player_mutex = PLAYER_MUTEX.lock().unwrap();
        rb::playlist::start(start_index, elapsed, offset);
        PLAYLIST_DIRTY.store(true, Ordering::Relaxed);
    })
    .await
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().finish())
}

#[derive(Deserialize)]
pub struct ShuffleQuery {
    start_index: Option<i32>,
}

pub async fn shuffle_playlist(query: web::Query<ShuffleQuery>) -> HandlerResult {
    let start_index = query.start_index.unwrap_or(0);
    let ret = web::block(move || {
        let _player_mutex = PLAYER_MUTEX.lock().unwrap();
        let seed = rb::system::current_tick();
        let ret = rb::playlist::shuffle(seed as i32, start_index);
        PLAYLIST_DIRTY.store(true, Ordering::Relaxed);
        ret
    })
    .await
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().body(ret.to_string()))
}

pub async fn get_playlist_amount() -> HandlerResult {
    let amount = web::block(|| {
        let _player_mutex = PLAYER_MUTEX.lock().unwrap();
        rb::playlist::amount()
    })
    .await
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(PlaylistAmount { amount }))
}

pub async fn resume_playlist() -> HandlerResult {
    let code = web::block(|| {
        let _player_mutex = PLAYER_MUTEX.lock().unwrap();
        let status = rb::system::get_global_status();
        let playback_status = rb::playback::status();
        if status.resume_index == -1 || playback_status.status == 1 {
            return -1;
        }
        let code = rb::playlist::resume();
        PLAYLIST_DIRTY.store(true, Ordering::Relaxed);
        code
    })
    .await
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(StatusCode { code }))
}

pub async fn resume_track() -> HandlerResult {
    web::block(|| {
        let _player_mutex = PLAYER_MUTEX.lock().unwrap();
        let status = rb::system::get_global_status();
        if status.resume_index == -1 {
            return;
        }
        if rb::playlist::amount() == 0 {
            let ret = rb::playlist::resume();
            if ret == -1 {
                return;
            }
        }
        rb::playlist::resume_track(
            status.resume_index,
            status.resume_crc32,
            status.resume_elapsed.into(),
            status.resume_offset.into(),
        );
        PLAYLIST_DIRTY.store(true, Ordering::Relaxed);
    })
    .await
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().finish())
}

pub async fn get_playlist_tracks(_path: web::Path<String>) -> HandlerResult {
    let entries = web::block(|| {
        let _player_mutex = PLAYER_MUTEX.lock().unwrap();
        let amount = rb::playlist::amount();
        let mut entries = Vec::with_capacity(amount as usize);
        for i in 0..amount {
            let info = rb::playlist::get_track_info(i);
            entries.push(rb::metadata::get_metadata(-1, &info.filename));
        }
        entries
    })
    .await
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(entries))
}

pub async fn insert_tracks(
    state: web::Data<AppState>,
    _path: web::Path<String>,
    body: web::Json<InsertTracks>,
) -> HandlerResult {
    let mut tracklist = body.into_inner();
    tracklist.tracks = tracklist.tracks.into_iter().map(trim_path).collect();

    let mut tracks_with_art: Vec<(String, Option<String>)> =
        tracklist.tracks.iter().map(|t| (t.clone(), None)).collect();

    if let Some(dir) = &tracklist.directory {
        let entries = read_files_with_art(dir.clone())
            .await
            .map_err(ErrorInternalServerError)?;
        tracklist.tracks = entries.iter().map(|(uri, _)| uri.clone()).collect();
        tracks_with_art = entries;
    }

    if tracklist.tracks.is_empty() {
        return Ok(HttpResponse::Ok().body("0"));
    }

    persist_remote_track_metadata(state.pool.clone(), tracks_with_art).await;

    let _player_mutex = PLAYER_MUTEX.lock().unwrap();
    let amount = rb::playlist::amount();
    let mut player = state.player.lock().unwrap();

    if let Some(player) = player.as_deref_mut() {
        let kv = state.kv.lock().unwrap();
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
            player
                .play_next(track)
                .await
                .map_err(ErrorInternalServerError)?;
        }

        return Ok(HttpResponse::Ok().body("0"));
    }

    if amount == 0 {
        let first = &tracklist.tracks[0];
        let dir = if first.starts_with("http://") || first.starts_with("https://") {
            "/".to_string()
        } else {
            let dir_parts: Vec<_> = first.split('/').collect();
            dir_parts[0..dir_parts.len() - 1].join("/")
        };
        let status = rb::playlist::create(&dir, None);
        if status == -1 {
            return Ok(HttpResponse::InternalServerError().body("Failed to create playlist"));
        }
        let start_index = rb::playlist::build_playlist(
            tracklist.tracks.iter().map(|t| t.as_str()).collect(),
            0,
            tracklist.tracks.len() as i32,
        );
        PLAYLIST_DIRTY.store(true, Ordering::Relaxed);
        return Ok(HttpResponse::Ok().body(start_index.to_string()));
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

    PLAYLIST_DIRTY.store(true, Ordering::Relaxed);
    Ok(HttpResponse::Ok().body(tracklist.position.to_string()))
}

pub async fn remove_tracks(
    state: web::Data<AppState>,
    _path: web::Path<String>,
    body: web::Json<DeleteTracks>,
) -> HandlerResult {
    let _player_mutex = PLAYER_MUTEX.lock().unwrap();
    let player = state.player.lock().unwrap();

    if player.as_deref().is_some() {
        return Ok(HttpResponse::Ok().body("0"));
    }

    let params = body.into_inner();
    let mut ret = 0;

    for position in &params.positions {
        ret = rb::playlist::delete_track(*position);
    }

    if params.positions.is_empty() {
        ret = rb::playlist::remove_all_tracks();
        PLAYLIST_DIRTY.store(true, Ordering::Relaxed);
        return Ok(HttpResponse::Ok().body(ret.to_string()));
    }

    PLAYLIST_DIRTY.store(true, Ordering::Relaxed);
    Ok(HttpResponse::Ok().body(ret.to_string()))
}

pub async fn get_playlist(state: web::Data<AppState>, _path: web::Path<String>) -> HandlerResult {
    let _player_mutex = PLAYER_MUTEX.lock().unwrap();
    let mut player = state.player.lock().unwrap();

    if let Some(player) = player.as_deref_mut() {
        let current_playback = player
            .get_current_playback()
            .await
            .map_err(ErrorInternalServerError)?;
        let tracks = current_playback.items;
        let index = match tracks.len() >= 2 {
            true => tracks.len() - 2,
            false => 0,
        } as i32;

        let mut entries = Vec::with_capacity(tracks.len());
        for (mut track, _) in tracks {
            if track.path.is_empty() {
                track.path = track.uri.clone();
            }

            let mut entry: rockbox_sys::types::mp3_entry::Mp3Entry = track.into();
            if !entry.path.is_empty() {
                if let Some(metadata) = find_track_metadata(&state, &entry.path)
                    .await
                    .map_err(ErrorInternalServerError)?
                {
                    entry.id = Some(metadata.id);
                    entry.album_art = metadata.album_art.or(entry.album_art.clone());
                    entry.album_id = Some(metadata.album_id);
                    entry.artist_id = Some(metadata.artist_id);
                    entry.genre_id = Some(metadata.genre_id);
                }
            }

            entries.push(entry);
        }

        let result = PlaylistInfo {
            amount: entries.len() as i32,
            index,
            entries,
            ..Default::default()
        };
        return Ok(HttpResponse::Ok().json(result));
    }

    let mut metadata_cache = state.metadata_cache.lock().await;
    let mut result = rb::playlist::get_current();
    let mut entries = vec![];
    let amount = rb::playlist::amount();

    for i in 0..amount {
        let info = rb::playlist::get_track_info(i);
        let mut entry = rb::metadata::get_metadata(-1, &info.filename);
        let hash = format!("{:x}", md5::compute(info.filename.as_bytes()));

        if let Some(cached) = metadata_cache.get(&hash) {
            entries.push(cached.clone());
            continue;
        }

        let track = find_track_metadata(&state, &info.filename)
            .await
            .map_err(ErrorInternalServerError)?;

        if track.is_none() {
            entries.push(entry.clone());
            continue;
        }

        entry.album_art = track.as_ref().and_then(|t| t.album_art.clone());
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

    Ok(HttpResponse::Ok().json(result))
}

async fn persist_remote_track_metadata(
    pool: sqlx::Pool<sqlx::Sqlite>,
    tracks: Vec<(String, Option<String>)>,
) {
    let sem = Arc::new(Semaphore::new(8));
    let mut futs: FuturesUnordered<tokio::task::JoinHandle<()>> = FuturesUnordered::new();

    for (track, art_uri) in tracks {
        if !track.starts_with("http://") && !track.starts_with("https://") {
            continue;
        }
        if reqwest::Url::parse(&track)
            .map(|u| u.path() == "/stream.wav")
            .unwrap_or(false)
        {
            continue;
        }
        match find_internal_track_by_pool(&pool, &track).await {
            Ok(Some(_)) => continue,
            Ok(None) => {}
            Err(e) => {
                tracing::warn!("metadata db check failed for {}: {}", track, e);
                continue;
            }
        }
        let pool = pool.clone();
        let sem = sem.clone();
        futs.push(tokio::spawn(async move {
            let _permit = sem.acquire_owned().await.unwrap();
            if let Err(e) = save_audio_metadata(pool, &track, art_uri.as_deref()).await {
                tracing::warn!("save_audio_metadata failed for {}: {}", track, e);
            }
        }));
    }

    while futs.next().await.is_some() {}
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

    Ok(metadata)
}

async fn find_internal_track_by_url(
    state: &AppState,
    path: &str,
) -> Result<Option<rockbox_library::entity::track::Track>, anyhow::Error> {
    find_internal_track_by_pool(&state.pool, path).await
}

async fn find_internal_track_by_pool(
    pool: &sqlx::Pool<sqlx::Sqlite>,
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

    repo::track::find(pool.clone(), track_id)
        .await
        .map_err(Into::into)
}
