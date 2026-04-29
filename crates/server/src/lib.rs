use anyhow::Error;
use handlers::*;
use tracing::{error, warn};

use http::RockboxHttpServer;
use lazy_static::lazy_static;
use rockbox_graphql::{
    schema::objects::{self, audio_status::AudioStatus, track::Track},
    simplebroker::SimpleBroker,
};
use rockbox_library::repo;
use rockbox_mpd::MpdServer;
use rockbox_sys::events::RockboxCommand;
use rockbox_sys::{self as rb, types::mp3_entry::Mp3Entry};
use sqlx::{Pool, Sqlite};
use std::{
    collections::{HashMap, HashSet},
    ffi::{c_char, c_int},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
};

// Set by playlist mutation HTTP handlers so the broker emits a StreamPlaylist
// event on the next tick even when index and amount haven't changed (e.g. shuffle).
pub(crate) static PLAYLIST_DIRTY: AtomicBool = AtomicBool::new(false);

pub mod cache;
pub mod handlers;
pub mod http;
pub mod kv;
pub mod player_events;
pub mod scan;

// Force netstream FFI symbols into the staticlib output.
// These functions are called from C code (streamfd.c) but not from any Rust
// code, so rustc would otherwise drop the entire crate from librockbox_server.a.
#[allow(dead_code)]
mod _netstream {
    use rbnetstream::{rb_net_close, rb_net_len, rb_net_lseek, rb_net_open, rb_net_read};
    use std::ffi::{c_char, c_void};
    #[used]
    static FN_OPEN: unsafe extern "C" fn(*const c_char) -> i32 = rb_net_open;
    #[used]
    static FN_READ: unsafe extern "C" fn(i32, *mut c_void, usize) -> i64 = rb_net_read;
    #[used]
    static FN_LSEEK: extern "C" fn(i32, i64, i32) -> i64 = rb_net_lseek;
    #[used]
    static FN_LEN: extern "C" fn(i32) -> i64 = rb_net_len;
    #[used]
    static FN_CLOSE: extern "C" fn(i32) = rb_net_close;
}

pub const AUDIO_EXTENSIONS: [&str; 17] = [
    "mp3", "ogg", "flac", "m4a", "aac", "mp4", "alac", "wav", "wv", "mpc", "aiff", "ac3", "opus",
    "spx", "sid", "ape", "wma",
];

lazy_static! {
    pub static ref GLOBAL_MUTEX: Mutex<i32> = Mutex::new(0);
    pub static ref PLAYER_MUTEX: Mutex<i32> = Mutex::new(0);
}

#[no_mangle]
pub extern "C" fn debugfn(args: *const c_char, value: c_int) {
    let c_str = unsafe { std::ffi::CStr::from_ptr(args) };
    let str_slice = c_str.to_str().unwrap();
    tracing::debug!("{} {}", str_slice, value);
}

#[no_mangle]
pub extern "C" fn start_server() {
    match rockbox_settings::load_settings(None) {
        Ok(_) => {}
        Err(e) => {
            warn!("Warning loading settings: {}", e);
        }
    }

    let mut app = RockboxHttpServer::new();

    app.get("/albums", get_albums);
    app.get("/albums/:id", get_album);
    app.get("/albums/:id/tracks", get_album_tracks);

    app.get("/artists", get_artists);
    app.get("/artists/:id", get_artist);
    app.get("/artists/:id/albums", get_artist_albums);
    app.get("/artists/:id/tracks", get_artist_tracks);

    app.get("/browse/tree-entries", get_tree_entries);

    app.get("/player", get_current_player);
    app.put("/player/load", load);
    app.put("/player/play", play);
    app.put("/player/pause", pause);
    app.put("/player/resume", resume);
    app.put("/player/ff-rewind", ff_rewind);
    app.get("/player/status", status);
    app.get("/player/current-track", current_track);
    app.get("/player/next-track", next_track);
    app.put("/player/flush-and-reload-tracks", flush_and_reload_tracks);
    app.put("/player/next", next);
    app.put("/player/previous", previous);
    app.put("/player/stop", stop);
    app.get("/player/file-position", get_file_position);
    app.get("/player/volume", get_volume);
    app.put("/player/volume", adjust_volume);

    app.post("/playlists", create_playlist);
    app.put("/playlists/start", start_playlist);
    app.put("/playlists/shuffle", shuffle_playlist);
    app.get("/playlists/amount", get_playlist_amount);
    app.put("/playlists/resume", resume_playlist);
    app.put("/playlists/resume-track", resume_track);
    app.get("/playlists/:id/tracks", get_playlist_tracks);
    app.post("/playlists/:id/tracks", insert_tracks);
    app.delete("/playlists/:id/tracks", remove_tracks);
    app.get("/playlists/:id", get_playlist);

    app.get("/saved-playlists/folders", list_playlist_folders);
    app.post("/saved-playlists/folders", create_playlist_folder);
    app.delete("/saved-playlists/folders/:id", delete_playlist_folder);
    app.get("/saved-playlists", list_saved_playlists);
    app.post("/saved-playlists", create_saved_playlist);
    app.get("/saved-playlists/:id/tracks", get_saved_playlist_tracks);
    app.get(
        "/saved-playlists/:id/track-ids",
        get_saved_playlist_track_ids,
    );
    app.post("/saved-playlists/:id/tracks", add_tracks_to_saved_playlist);
    app.delete(
        "/saved-playlists/:id/tracks/:track_id",
        remove_track_from_saved_playlist,
    );
    app.post("/saved-playlists/:id/play", play_saved_playlist);
    app.get("/saved-playlists/:id", get_saved_playlist);
    app.put("/saved-playlists/:id", update_saved_playlist);
    app.delete("/saved-playlists/:id", delete_saved_playlist);

    app.get("/smart-playlists", list_smart_playlists);
    app.post("/smart-playlists", create_smart_playlist);
    app.get("/smart-playlists/:id/tracks", get_smart_playlist_tracks);
    app.post("/smart-playlists/:id/play", play_smart_playlist);
    app.get("/smart-playlists/:id", get_smart_playlist);
    app.put("/smart-playlists/:id", update_smart_playlist);
    app.delete("/smart-playlists/:id", delete_smart_playlist);

    app.post("/track-stats/:id/played", record_track_played);
    app.post("/track-stats/:id/skipped", record_track_skipped);
    app.get("/track-stats/:id", get_track_stats);

    app.get("/tracks", get_tracks);
    app.get("/tracks/:id", get_track);
    app.put("/tracks/stream-metadata", save_stream_track_metadata);

    app.get("/version", get_rockbox_version);
    app.get("/status", get_status);
    app.get("/settings", get_global_settings);
    app.put("/settings", update_global_settings);
    app.put("/scan-library", scan_library);
    app.get("/search", search);

    app.get("/devices", get_devices);
    app.get("/devices/:id", get_device);
    app.put("/devices/:id/connect", connect);
    app.put("/devices/:id/disconnect", disconnect);

    app.get("/", index);
    app.get("/operations/:id", index);
    app.get("/schemas/:id", index);
    app.get("/openapi.json", get_openapi);

    // Pre-initialize the UPnP tokio runtime before any HTTP handler runs.
    // If initialized lazily inside a handler's block_on context, tokio 1.27+
    // panics with "Cannot start a runtime from within a runtime."
    rockbox_upnp::init();

    match app.listen() {
        Ok(_) => {}
        Err(e) => {
            error!("Error starting server: {}", e);
        }
    }
}

#[no_mangle]
pub extern "C" fn start_servers() {
    let (cmd_tx, cmd_rx) = std::sync::mpsc::channel::<RockboxCommand>();
    let cmd_tx = Arc::new(Mutex::new(cmd_tx));

    thread::spawn(move || {
        let port = std::env::var("ROCKBOX_TCP_PORT").unwrap_or_else(|_| "6063".to_string());
        let url = format!("http://127.0.0.1:{}", port);
        let client = reqwest::blocking::Client::new();

        while let Ok(event) = cmd_rx.recv() {
            match event {
                RockboxCommand::Play(elapsed, offset) => {
                    client
                        .put(&format!(
                            "{}/player/play?elapsed={}&offset={}",
                            url, elapsed, offset
                        ))
                        .send()
                        .unwrap();
                }
                RockboxCommand::Pause => {
                    client.put(&format!("{}/player/pause", url)).send().unwrap();
                }
                RockboxCommand::Resume => {
                    client
                        .put(&format!("{}/player/resume", url))
                        .send()
                        .unwrap();
                }
                RockboxCommand::Next => {
                    client.put(&format!("{}/player/next", url)).send().unwrap();
                }
                RockboxCommand::Prev => {
                    client
                        .put(&format!("{}/player/previous", url))
                        .send()
                        .unwrap();
                }
                RockboxCommand::FfRewind(newtime) => {
                    client
                        .put(&format!("{}/player/ff-rewind?newtime={}", url, newtime))
                        .send()
                        .unwrap();
                }
                RockboxCommand::FlushAndReloadTracks => {
                    client
                        .put(&format!("{}/player/flush-and-reload-tracks", url))
                        .send()
                        .unwrap();
                }
                RockboxCommand::Stop => {
                    client.put(&format!("{}/player/stop", url)).send().unwrap();
                }
                RockboxCommand::PlaylistResume => {
                    client
                        .put(&format!("{}/playlists/resume", url))
                        .send()
                        .unwrap();
                }
                RockboxCommand::PlaylistResumeTrack => {
                    client
                        .put(&format!("{}/playlists/resume-track", url))
                        .send()
                        .unwrap();
                }
            }
        }
    });

    let cloned_cmd_tx = cmd_tx.clone();

    thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        match runtime.block_on(rockbox_rpc::server::start(cmd_tx.clone())) {
            Ok(_) => {}
            Err(e) => {
                error!("Error starting server: {}", e);
            }
        }
    });

    thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        match runtime.block_on(rockbox_graphql::server::start(cloned_cmd_tx.clone())) {
            Ok(_) => {}
            Err(e) => {
                error!("Error starting server: {}", e);
            }
        }
    });

    // Wait for the rpc server to start
    thread::sleep(std::time::Duration::from_millis(500));

    #[cfg(target_os = "linux")]
    {
        use rockbox_mpris::MprisServer;
        thread::spawn(
            move || match async_std::task::block_on(MprisServer::start()) {
                Ok(_) => {}
                Err(e) => {
                    error!("Error starting mpris server: {}", e);
                }
            },
        );
    }

    thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        match runtime.block_on(MpdServer::start()) {
            Ok(_) => {}
            Err(e) => {
                error!("Error starting mpd server: {}", e);
            }
        }
    });
}

#[no_mangle]
pub extern "C" fn start_broker() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let pool = rt
        .block_on(rockbox_library::create_connection_pool())
        .unwrap();

    let mut metadata_cache: HashMap<String, Mp3Entry> = HashMap::new();
    let mut last_playlist_index: i32 = i32::MIN;
    let mut last_playlist_amount: i32 = i32::MIN;
    let mut last_entries: Vec<Mp3Entry> = Vec::new();
    let mut did_initial_restore = false;

    let mut current_scrobble_track: Option<Track> = None; // The track we are monitoring for scrobble
    let mut scrobbled_tracks: HashSet<String> = HashSet::new(); // Simple unique ID to prevent duplicates (use track.id if available)

    // Track stats auto-recording: detect play/skip on track change
    let playlist_store = rockbox_playlists::PlaylistStore::new(pool.clone());
    let mut last_stats_track_id: Option<String> = None;
    let mut last_stats_elapsed: u64 = 0;
    let mut last_stats_length: u64 = 0;

    loop {
        let mutex = GLOBAL_MUTEX.lock().unwrap();
        if *mutex == 1 {
            drop(mutex);
            thread::sleep(std::time::Duration::from_millis(100));
            rb::system::sleep(rb::HZ);
            continue;
        }

        drop(mutex);

        // On the first tick after firmware is ready, restore the saved playlist
        // from the control file so the queue is visible before the user presses play.
        if !did_initial_restore {
            did_initial_restore = true;
            let _guard = PLAYER_MUTEX.lock().unwrap();
            let status = rb::system::get_global_status();
            if status.resume_index != -1 && rb::playlist::amount() == 0 {
                rb::playlist::resume();
            }
        }

        let player_mutex = PLAYER_MUTEX.lock().unwrap();

        let playback_status: AudioStatus = rb::playback::status().into();
        SimpleBroker::publish(playback_status);

        match rb::playback::current_track() {
            Some(current_track) => {
                // audio_current_track()->path can diverge from the playlist
                // filename for HTTP stream tracks.  Use the playlist entry's
                // filename (same key the playlist section uses) so DB lookups
                // hit the right record and album_art is resolved correctly.
                let playlist_index = rb::playlist::index();
                let lookup_path = if playlist_index >= 0 {
                    let info = rb::playlist::get_track_info(playlist_index);
                    if !info.filename.is_empty() {
                        info.filename
                    } else {
                        current_track.path.clone()
                    }
                } else {
                    current_track.path.clone()
                };

                let hash = format!("{:x}", md5::compute(lookup_path.as_bytes()));
                let db_metadata = rt
                    .block_on(repo::track::find_by_md5(pool.clone(), &hash))
                    .ok()
                    .flatten();

                let mut track: Track = current_track.into();

                if let Some(metadata) = db_metadata {
                    // When the URL-keyed record has no album_art (it was saved
                    // from the HTTP stream which has no embedded art), fall back
                    // to the local track identified by the UUID in the URL path.
                    let album_art = if metadata.album_art.is_some() {
                        metadata.album_art.clone()
                    } else {
                        let uuid = lookup_path.rsplit('/').next().unwrap_or("");
                        if !uuid.is_empty() {
                            rt.block_on(repo::track::find(pool.clone(), uuid))
                                .ok()
                                .flatten()
                                .and_then(|t| t.album_art)
                        } else {
                            None
                        }
                    };
                    track.id = Some(metadata.id.clone());
                    track.album_art = album_art;
                    track.album_id = Some(metadata.album_id);
                    track.artist_id = Some(metadata.artist_id);
                    // Only fall back to DB metadata when the live Mp3Entry fields
                    // are empty but the track is fully loaded (non-zero length).
                    // If length is 0 the audio engine hasn't finished initialising
                    // yet; leave the track unchanged so clients don't interpret
                    // an elapsed=0 / valid-title combination as "started from the
                    // beginning" and override the resume position.
                    if track.length > 0 {
                        if track.title.is_empty() {
                            track.title = metadata.title.clone();
                        }
                        if track.artist.is_empty() {
                            track.artist = metadata.artist.clone();
                        }
                        if track.album.is_empty() {
                            track.album = metadata.album.clone();
                        }
                        if track.album_artist.is_empty() {
                            track.album_artist = metadata.album_artist.clone();
                        }
                    }
                    SimpleBroker::publish(track.clone());

                    let track_changed = if let Some(ref current) = current_scrobble_track {
                        current.path != track.path
                    } else {
                        true
                    };

                    if track_changed {
                        // Auto-record play or skip for the previous track (direct DB write,
                        // no HTTP roundtrip — avoids blocking the broker loop).
                        if let Some(prev_id) = last_stats_track_id.take() {
                            if last_stats_length > 10_000 && last_stats_elapsed > 2_000 {
                                let ratio = last_stats_elapsed as f64 / last_stats_length as f64;
                                if ratio >= 0.40 {
                                    let _ = rt.block_on(playlist_store.record_play(&prev_id));
                                } else {
                                    let _ = rt.block_on(playlist_store.record_skip(&prev_id));
                                }
                            }
                        }
                        current_scrobble_track = Some(track.clone());
                    }

                    // Update tracking state for the current track
                    last_stats_track_id = Some(metadata.id.clone());
                    last_stats_elapsed = track.elapsed;
                    last_stats_length = track.length;

                    // Check progress for scrobbling (only if we have a track to monitor)
                    if let Some(ref monitored_track) = current_scrobble_track {
                        if monitored_track.path == track.path {
                            let elapsed_ms = track.elapsed;
                            let length_ms = track.length;

                            if length_ms > 30_000 &&  // optional: ignore very short tracks per Last.fm rules
                                    elapsed_ms as f64 / length_ms as f64 >= 0.40
                            {
                                if !scrobbled_tracks.contains(&metadata.id) {
                                    let cloned_pool = pool.clone();
                                    let cloned_track = monitored_track.clone();
                                    thread::spawn(move || {
                                        let rt = tokio::runtime::Builder::new_current_thread()
                                            .enable_all()
                                            .build()
                                            .unwrap();
                                        match rt
                                            .block_on(scrobble(cloned_track, cloned_pool.clone()))
                                        {
                                            Ok(_) => {}
                                            Err(e) => error!("{}", e),
                                        }
                                    });
                                    scrobbled_tracks.insert(metadata.id.clone());
                                }
                            } else {
                                scrobbled_tracks.clear();
                            }
                        }
                    }
                } else {
                    // Track not in DB (e.g. an HTTP stream URL not yet scanned).
                    // Still publish so clients receive elapsed/status updates.
                    SimpleBroker::publish(track);
                }
            }
            None => {
                current_scrobble_track = None; // reset on no track
            }
        };

        // Detect what changed.  Consuming DIRTY here covers mutations (shuffle,
        // insert, remove) that leave amount/index unchanged.
        let amount = rb::playlist::amount();
        let current_index = rb::playlist::index();
        let dirty = PLAYLIST_DIRTY.swap(false, Ordering::Relaxed);

        let index_changed = current_index != last_playlist_index;
        let content_changed = amount != last_playlist_amount || dirty;

        if !index_changed && !content_changed {
            drop(player_mutex);
            thread::sleep(std::time::Duration::from_millis(100));
            rb::system::sleep(rb::HZ);
            continue;
        }

        last_playlist_index = current_index;
        last_playlist_amount = amount;

        let entries: Vec<Mp3Entry> = if content_changed {
            // Collect filenames while still holding the mutex (no I/O).
            let mut track_infos: Vec<(String, String)> = Vec::with_capacity(amount as usize);
            for i in 0..amount {
                let info = rb::playlist::get_track_info(i);
                let hash = format!("{:x}", md5::compute(info.filename.as_bytes()));
                track_infos.push((hash, info.filename));
            }

            // Release the mutex before any slow I/O so player commands aren't blocked.
            drop(player_mutex);

            // Build entries, reading from disk only for files not yet in cache.
            let mut built: Vec<Mp3Entry> = Vec::with_capacity(track_infos.len());
            for (hash, filename) in &track_infos {
                if let Some(entry) = metadata_cache.get(hash) {
                    built.push(entry.clone());
                    continue;
                }

                let is_http = filename.starts_with("http://") || filename.starts_with("https://");
                let db_track = rt
                    .block_on(repo::track::find_by_md5(pool.clone(), hash))
                    .unwrap();

                let entry = if is_http {
                    // For HTTP stream URLs, do NOT call get_metadata(-1, url) — that
                    // opens a live HTTP connection for every queued track and blocks
                    // the broker loop. Instead, build the entry from the database
                    // using the saved track metadata keyed by the URL hash.
                    let mut e = Mp3Entry::default();
                    e.path = filename.clone();
                    if let Some(ref t) = db_track {
                        e.title = t.title.clone();
                        e.artist = t.artist.clone();
                        e.album = t.album.clone();
                        e.albumartist = t.album_artist.clone();
                        e.length = t.length as u64;
                        e.bitrate = t.bitrate;
                        e.frequency = t.frequency as u64;
                        // URL-keyed record may have no album_art (stream has no
                        // embedded art). Fall back to the local track by UUID.
                        e.album_art = if t.album_art.is_some() {
                            t.album_art.clone()
                        } else {
                            let uuid = filename.rsplit('/').next().unwrap_or("");
                            if !uuid.is_empty() {
                                rt.block_on(repo::track::find(pool.clone(), uuid))
                                    .ok()
                                    .flatten()
                                    .and_then(|local| local.album_art)
                            } else {
                                None
                            }
                        };
                        e.album_id = Some(t.album_id.clone());
                        e.artist_id = Some(t.artist_id.clone());
                        e.genre_id = Some(t.genre_id.clone());
                        e.id = Some(t.id.clone());
                    }
                    e
                } else {
                    let mut e = rb::metadata::get_metadata(-1, filename);
                    if db_track.is_some() {
                        e.album_art = db_track.as_ref().map(|t| t.album_art.clone()).flatten();
                        e.album_id = db_track.as_ref().map(|t| t.album_id.clone());
                        e.artist_id = db_track.as_ref().map(|t| t.artist_id.clone());
                        e.genre_id = db_track.as_ref().map(|t| t.genre_id.clone());
                    }
                    e
                };

                metadata_cache.insert(hash.clone(), entry.clone());
                built.push(entry);
            }
            last_entries = built.clone();
            built
        } else {
            // Only the current index changed — reuse the cached entry list.
            // No get_track_info loop, no disk reads, no DB queries.
            drop(player_mutex);
            last_entries.clone()
        };

        SimpleBroker::publish(objects::playlist::Playlist {
            amount,
            index: current_index,
            max_playlist_size: rb::playlist::max_playlist_size(),
            first_index: rb::playlist::first_index(),
            last_insert_pos: rb::playlist::last_insert_pos(),
            seed: rb::playlist::seed(),
            last_shuffled_start: rb::playlist::last_shuffled_start(),
            tracks: entries.into_iter().map(|t| t.into()).collect(),
        });

        thread::sleep(std::time::Duration::from_millis(100));
        rb::system::sleep(rb::HZ);
    }
}

async fn scrobble(track: Track, pool: Pool<Sqlite>) -> Result<(), Error> {
    let album_id = track.album_id.unwrap();
    let track = repo::track::find(pool.clone(), &track.id.unwrap()).await?;
    let album = repo::album::find(pool, &album_id).await?;

    if let Some(track) = track {
        if let Some(album) = album {
            match rockbox_rocksky::scrobble(track, album).await {
                Ok(_) => {}
                Err(e) => error!("Failed to scrobble {}", e),
            };
        }
    }

    Ok(())
}
