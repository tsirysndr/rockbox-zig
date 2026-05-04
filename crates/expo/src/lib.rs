//! Rockbox mobile client — C-ABI wrapper around a tonic gRPC client.
//!
//! Designed to be linked into iOS (.xcframework) and Android (jniLibs) builds
//! and called from Expo modules' Swift / Kotlin glue.
//!
//! ## ABI
//!
//! - All entry points return either an `i32` status code (0 = ok, <0 = error)
//!   or a `*mut c_char` heap-owned C string. Callers MUST free returned strings
//!   via `rb_free_string`.
//! - Complex responses (status snapshot, current track, etc.) are returned as
//!   JSON to avoid struct marshaling pain across language boundaries.
//! - All synchronous fn names are prefixed `rb_*` and block on the embedded
//!   Tokio runtime — call from a background thread on the platform side.

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::RwLock;
use std::time::Duration;

use futures_util::pin_mut;
use futures_util::StreamExt;
use once_cell::sync::Lazy;
use serde::Serialize;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tokio::task::AbortHandle;

pub mod api {
    pub mod v1alpha1 {
        tonic::include_proto!("rockbox.v1alpha1");
    }
}

#[cfg(target_os = "android")]
mod jni_bridge;

// Embedded rockbox daemon. The `extern fn main_c()` reference inside this
// module is what keeps the C firmware code from being --gc-sections'd out
// of the cdylib link.
#[cfg(all(target_os = "android", feature = "embedded-daemon"))]
mod daemon;

use api::v1alpha1::{
    bluetooth_service_client::BluetoothServiceClient, browse_service_client::BrowseServiceClient,
    genre_service_client::GenreServiceClient, library_service_client::LibraryServiceClient,
    playback_service_client::PlaybackServiceClient,
    playlist_service_client::PlaylistServiceClient,
    saved_playlist_service_client::SavedPlaylistServiceClient,
    settings_service_client::SettingsServiceClient,
    smart_playlist_service_client::SmartPlaylistServiceClient,
    sound_service_client::SoundServiceClient, system_service_client::SystemServiceClient,
    AddTracksToSavedPlaylistRequest, AdjustVolumeRequest, ConnectBluetoothDeviceRequest,
    CreateSavedPlaylistRequest, CurrentTrackRequest, DeleteSavedPlaylistRequest,
    DisconnectBluetoothDeviceRequest, FastForwardRewindRequest, GetAlbumRequest, GetAlbumsRequest,
    GetArtistRequest, GetArtistsRequest, GetBluetoothDevicesRequest, GetCurrentRequest,
    GetGenreAlbumsRequest, GetGenreArtistsRequest, GetGenreRequest, GetGenreTracksRequest,
    GetGenresRequest, GetGlobalSettingsRequest, GetGlobalStatusRequest, GetLikedAlbumsRequest,
    GetLikedTracksRequest, GetSavedPlaylistTracksRequest, GetSavedPlaylistsRequest,
    GetSmartPlaylistTracksRequest, GetSmartPlaylistsRequest, GetTracksRequest,
    InsertDirectoryRequest, InsertTracksRequest, LikeTrackRequest, NextRequest, PauseRequest,
    PlayAlbumRequest, PlayAllTracksRequest, PlayArtistTracksRequest, PlayDirectoryRequest,
    PlayOrPauseRequest, PlaySavedPlaylistRequest, PlaySmartPlaylistRequest, PlayTrackRequest,
    PlaylistResumeRequest, PreviousRequest, RemoveTrackFromSavedPlaylistRequest,
    RemoveTracksRequest, ResumeRequest, ResumeTrackRequest, SaveSettingsRequest,
    ScanBluetoothRequest, SearchRequest, ShufflePlaylistRequest, SoundCurrentRequest, StartRequest,
    StatusRequest, StreamCurrentTrackRequest, StreamLibraryRequest, StreamPlaylistRequest,
    StreamStatusRequest, TreeGetEntriesRequest, UnlikeTrackRequest, UpdateSavedPlaylistRequest,
};

// ── Globals ──────────────────────────────────────────────────────────────────

/// One Tokio runtime per process. Multi-thread is fine on Android; iOS works
/// too — tokio uses pthread under the hood.
static RT: Lazy<Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(2)
        .thread_name("rockbox-rpc")
        .build()
        .expect("failed to build tokio runtime")
});

pub(crate) static SERVER_URL: Lazy<RwLock<String>> =
    Lazy::new(|| RwLock::new("http://127.0.0.1:6061".to_string()));

pub(crate) static HTTP_URL: Lazy<RwLock<String>> =
    Lazy::new(|| RwLock::new("http://127.0.0.1:6063".to_string()));

fn url() -> String {
    SERVER_URL.read().expect("server url poisoned").clone()
}

fn http_url() -> String {
    HTTP_URL.read().expect("http url poisoned").clone()
}

// ── String helpers ───────────────────────────────────────────────────────────

unsafe fn cstr_to_str<'a>(p: *const c_char) -> Option<&'a str> {
    if p.is_null() {
        return None;
    }
    CStr::from_ptr(p).to_str().ok()
}

fn ok_string<T: Serialize>(value: &T) -> *mut c_char {
    match serde_json::to_string(value) {
        Ok(s) => CString::new(s)
            .map(|c| c.into_raw())
            .unwrap_or(std::ptr::null_mut()),
        Err(_) => std::ptr::null_mut(),
    }
}

fn err_string(msg: impl AsRef<str>) -> *mut c_char {
    let payload = serde_json::json!({ "error": msg.as_ref() });
    CString::new(payload.to_string())
        .map(|c| c.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

/// Frees a string previously returned by any `rb_*_json` entry point.
/// Safe to call with a null pointer.
///
/// # Safety
/// `ptr` must be either null or a pointer returned by this library and must
/// not have been freed already.
#[no_mangle]
pub unsafe extern "C" fn rb_free_string(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    // Reclaim ownership and drop.
    drop(CString::from_raw(ptr));
}

// ── Init / config ────────────────────────────────────────────────────────────

/// Configure the gRPC server URL. Call once at startup before any other entry.
/// Returns 0 on success, -1 on invalid input.
///
/// # Safety
/// `url_ptr` must point to a valid NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn rb_set_server_url(url_ptr: *const c_char) -> c_int {
    let Some(s) = cstr_to_str(url_ptr) else {
        return -1;
    };
    match SERVER_URL.write() {
        Ok(mut g) => {
            *g = s.to_string();
            0
        }
        Err(_) => -2,
    }
}

/// Configure the rockboxd HTTP base URL (the port exposing `/devices` etc.).
/// Defaults to `http://127.0.0.1:6063` if never called.
///
/// # Safety
/// `url_ptr` must point to a valid NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn rb_set_http_url(url_ptr: *const c_char) -> c_int {
    let Some(s) = cstr_to_str(url_ptr) else {
        return -1;
    };
    match HTTP_URL.write() {
        Ok(mut g) => {
            *g = s.to_string();
            0
        }
        Err(_) => -2,
    }
}

/// Health check — round-trips a Status RPC. Returns 0 on success, -1 otherwise.
#[no_mangle]
pub extern "C" fn rb_ping() -> c_int {
    let res: Result<(), tonic::Status> = RT.block_on(async {
        let mut c = PlaybackServiceClient::connect(url())
            .await
            .map_err(|e| tonic::Status::unavailable(e.to_string()))?;
        c.status(StatusRequest {}).await?;
        Ok(())
    });
    match res {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

// ── Playback control ─────────────────────────────────────────────────────────

macro_rules! simple_call {
    ($fn_name:ident, $client:ident, $method:ident, $req:expr) => {
        #[no_mangle]
        pub extern "C" fn $fn_name() -> c_int {
            let res: Result<(), tonic::Status> = RT.block_on(async {
                let mut c = $client::connect(url())
                    .await
                    .map_err(|e| tonic::Status::unavailable(e.to_string()))?;
                c.$method($req).await?;
                Ok(())
            });
            match res {
                Ok(_) => 0,
                Err(_) => -1,
            }
        }
    };
}

simple_call!(rb_play, PlaybackServiceClient, resume, ResumeRequest {});
simple_call!(rb_pause, PlaybackServiceClient, pause, PauseRequest {});
simple_call!(
    rb_play_pause,
    PlaybackServiceClient,
    play_or_pause,
    PlayOrPauseRequest {}
);
simple_call!(rb_next, PlaybackServiceClient, next, NextRequest {});
simple_call!(rb_prev, PlaybackServiceClient, previous, PreviousRequest {});

/// Seek to `position_ms` milliseconds from the start of the current track.
#[no_mangle]
pub extern "C" fn rb_seek(position_ms: i32) -> c_int {
    let res: Result<(), tonic::Status> = RT.block_on(async {
        let mut c = PlaybackServiceClient::connect(url())
            .await
            .map_err(|e| tonic::Status::unavailable(e.to_string()))?;
        c.fast_forward_rewind(FastForwardRewindRequest {
            new_time: position_ms,
        })
        .await?;
        Ok(())
    });
    match res {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

// ── Read / status (returns JSON; caller must rb_free_string) ────────────────

#[derive(Serialize)]
struct StatusJson {
    /// Server-side playback status code: 0 stopped, 1 playing, 2 paused.
    status: i32,
}

/// Returns a heap-allocated JSON string with global playback status, or an
/// error JSON object on failure. Caller must free via `rb_free_string`.
#[no_mangle]
pub extern "C" fn rb_status_json() -> *mut c_char {
    let res: Result<StatusJson, tonic::Status> = RT.block_on(async {
        let mut c = PlaybackServiceClient::connect(url())
            .await
            .map_err(|e| tonic::Status::unavailable(e.to_string()))?;
        let resp = c.status(StatusRequest {}).await?.into_inner();
        Ok(StatusJson {
            status: resp.status,
        })
    });
    match res {
        Ok(v) => ok_string(&v),
        Err(e) => err_string(format!("status: {e}")),
    }
}

#[derive(Serialize, Default)]
struct TrackJson {
    id: String,
    path: String,
    title: String,
    artist: String,
    album: String,
    album_art: Option<String>,
    duration_ms: i64,
    elapsed_ms: i64,
}

/// Returns the current track as JSON (heap-allocated; free with `rb_free_string`).
/// JSON shape: `{ id, path, title, artist, album, album_art, duration_ms }`.
#[no_mangle]
pub extern "C" fn rb_current_track_json() -> *mut c_char {
    let res: Result<TrackJson, tonic::Status> = RT.block_on(async {
        let mut c = PlaybackServiceClient::connect(url())
            .await
            .map_err(|e| tonic::Status::unavailable(e.to_string()))?;
        let resp = c.current_track(CurrentTrackRequest {}).await?.into_inner();
        let album_art: Option<String> = resp.album_art.filter(|s: &String| !s.is_empty());
        Ok(TrackJson {
            id: resp.id,
            path: resp.path,
            title: resp.title,
            artist: resp.artist,
            album: resp.album,
            album_art,
            duration_ms: resp.length as i64,
            elapsed_ms: resp.elapsed as i64,
        })
    });
    match res {
        Ok(v) => ok_string(&v),
        Err(_e) => {
            // Many fresh installs return `unimplemented` until a track is loaded;
            // surface an empty JSON object rather than an error so the UI degrades
            // gracefully.
            ok_string(&TrackJson::default())
        }
    }
}

// ── Like / unlike ────────────────────────────────────────────────────────────

/// # Safety
/// `track_id_ptr` must point to a valid NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn rb_like_track(track_id_ptr: *const c_char) -> c_int {
    let Some(track_id) = cstr_to_str(track_id_ptr) else {
        return -1;
    };
    let track_id = track_id.to_string();
    let res: Result<(), tonic::Status> = RT.block_on(async {
        let mut c = LibraryServiceClient::connect(url())
            .await
            .map_err(|e| tonic::Status::unavailable(e.to_string()))?;
        c.like_track(LikeTrackRequest { id: track_id }).await?;
        Ok(())
    });
    if res.is_ok() {
        0
    } else {
        -1
    }
}

/// # Safety
/// `track_id_ptr` must point to a valid NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn rb_unlike_track(track_id_ptr: *const c_char) -> c_int {
    let Some(track_id) = cstr_to_str(track_id_ptr) else {
        return -1;
    };
    let track_id = track_id.to_string();
    let res: Result<(), tonic::Status> = RT.block_on(async {
        let mut c = LibraryServiceClient::connect(url())
            .await
            .map_err(|e| tonic::Status::unavailable(e.to_string()))?;
        c.unlike_track(UnlikeTrackRequest { id: track_id }).await?;
        Ok(())
    });
    if res.is_ok() {
        0
    } else {
        -1
    }
}

// ── Comprehensive RPC surface ───────────────────────────────────────────────
//
// Mirrors `gpui/src/client.rs`. JSON-returning entry points serialize the full
// proto response (every generated type derives `serde::Serialize`), so callers
// can `JSON.parse` and use the message structure directly. Unit ops return
// `i32` (0 = ok, <0 = error) and don't allocate.

/// Returns 0 if the gRPC call resolved Ok, -1 otherwise.
fn run_unit<F>(fut: F) -> c_int
where
    F: std::future::Future<Output = Result<(), tonic::Status>>,
{
    match RT.block_on(fut) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

/// Returns a heap JSON C string of `value`, or null on serialization failure.
fn json_response<T: Serialize>(value: T) -> *mut c_char {
    match serde_json::to_string(&value) {
        Ok(s) => CString::new(s)
            .map(|c| c.into_raw())
            .unwrap_or(std::ptr::null_mut()),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Take the inner of a tonic response if Ok, else build an `{ "error": ... }`
/// JSON payload.
fn unwrap_or_err_string<T: Serialize, E: std::fmt::Display>(res: Result<T, E>) -> *mut c_char {
    match res {
        Ok(v) => json_response(v),
        Err(e) => err_string(e.to_string()),
    }
}

/// Helper that maps tonic transport / RPC errors to a printable string.
async fn connect_err<T, F: std::future::Future<Output = Result<T, tonic::Status>>>(
    fut: F,
) -> Result<T, String> {
    fut.await.map_err(|e| e.to_string())
}

/// Boolean → bool conversion via i32 sentinel; 0 = false, anything else = true.
fn b(v: c_int) -> bool {
    v != 0
}

// ── Playback ────────────────────────────────────────────────────────────────

simple_call!(
    rb_resume_track,
    PlaylistServiceClient,
    resume_track,
    ResumeTrackRequest {
        start_index: 0,
        crc: 0,
        elapsed: 0,
        offset: 0,
    }
);
simple_call!(
    rb_playlist_resume,
    PlaylistServiceClient,
    playlist_resume,
    PlaylistResumeRequest {}
);
simple_call!(
    rb_play_all_tracks,
    PlaybackServiceClient,
    play_all_tracks,
    PlayAllTracksRequest {
        shuffle: Some(false),
        position: Some(0),
    }
);

/// # Safety
/// `path_ptr` must be a valid NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn rb_play_track(path_ptr: *const c_char) -> c_int {
    let Some(path) = cstr_to_str(path_ptr) else {
        return -1;
    };
    let path = path.to_string();
    run_unit(async move {
        let mut c = PlaybackServiceClient::connect(url())
            .await
            .map_err(|e| tonic::Status::unavailable(e.to_string()))?;
        c.play_track(PlayTrackRequest { path }).await?;
        Ok(())
    })
}

/// # Safety
/// `album_id_ptr` must be a valid NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn rb_play_album(album_id_ptr: *const c_char, shuffle: c_int) -> c_int {
    let Some(album_id) = cstr_to_str(album_id_ptr) else {
        return -1;
    };
    let album_id = album_id.to_string();
    let shuffle = b(shuffle);
    run_unit(async move {
        let mut c = PlaybackServiceClient::connect(url())
            .await
            .map_err(|e| tonic::Status::unavailable(e.to_string()))?;
        c.play_album(PlayAlbumRequest {
            album_id,
            shuffle: Some(shuffle),
            position: Some(0),
        })
        .await?;
        Ok(())
    })
}

/// # Safety
/// `artist_id_ptr` must be a valid NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn rb_play_artist_tracks(
    artist_id_ptr: *const c_char,
    shuffle: c_int,
) -> c_int {
    let Some(artist_id) = cstr_to_str(artist_id_ptr) else {
        return -1;
    };
    let artist_id = artist_id.to_string();
    let shuffle = b(shuffle);
    run_unit(async move {
        let mut c = PlaybackServiceClient::connect(url())
            .await
            .map_err(|e| tonic::Status::unavailable(e.to_string()))?;
        c.play_artist_tracks(PlayArtistTracksRequest {
            artist_id,
            shuffle: Some(shuffle),
            position: Some(0),
        })
        .await?;
        Ok(())
    })
}

/// # Safety
/// `path_ptr` must be a valid NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn rb_play_directory(
    path_ptr: *const c_char,
    shuffle: c_int,
    position: c_int,
) -> c_int {
    let Some(path) = cstr_to_str(path_ptr) else {
        return -1;
    };
    let path = path.to_string();
    let shuffle = b(shuffle);
    let pos = if position < 0 { None } else { Some(position) };
    run_unit(async move {
        let mut c = PlaybackServiceClient::connect(url())
            .await
            .map_err(|e| tonic::Status::unavailable(e.to_string()))?;
        c.play_directory(PlayDirectoryRequest {
            path,
            shuffle: Some(shuffle),
            recurse: Some(true),
            position: pos,
        })
        .await?;
        Ok(())
    })
}

// ── Playlist (queue) ────────────────────────────────────────────────────────

/// Jump to a queue position (calls `Start { start_index: pos }`).
#[no_mangle]
pub extern "C" fn rb_jump_to_queue_position(pos: c_int) -> c_int {
    run_unit(async move {
        let mut c = PlaylistServiceClient::connect(url())
            .await
            .map_err(|e| tonic::Status::unavailable(e.to_string()))?;
        c.start(StartRequest {
            start_index: Some(pos),
            elapsed: Some(0),
            offset: Some(0),
        })
        .await?;
        Ok(())
    })
}

simple_call!(
    rb_shuffle_playlist,
    PlaylistServiceClient,
    shuffle_playlist,
    ShufflePlaylistRequest { start_index: 0 }
);

/// `paths_json` must be a JSON array of strings.
/// `position` follows the rockbox INSERT_* constants.
///
/// # Safety
/// `paths_json_ptr` must be a valid NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn rb_insert_tracks(
    paths_json_ptr: *const c_char,
    position: c_int,
    shuffle: c_int,
) -> c_int {
    let Some(paths_json) = cstr_to_str(paths_json_ptr) else {
        return -1;
    };
    let Ok(paths) = serde_json::from_str::<Vec<String>>(paths_json) else {
        return -2;
    };
    let shuffle = b(shuffle);
    run_unit(async move {
        let mut c = PlaylistServiceClient::connect(url())
            .await
            .map_err(|e| tonic::Status::unavailable(e.to_string()))?;
        c.insert_tracks(InsertTracksRequest {
            playlist_id: None,
            position,
            tracks: paths,
            shuffle: Some(shuffle),
        })
        .await?;
        Ok(())
    })
}

/// Convenience: insert a single track at position INSERT_FIRST (-4).
///
/// # Safety
/// `path_ptr` must be a valid NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn rb_insert_track_next(path_ptr: *const c_char) -> c_int {
    let Some(p) = cstr_to_str(path_ptr) else {
        return -1;
    };
    let path = p.to_string();
    run_unit(async move {
        let mut c = PlaylistServiceClient::connect(url())
            .await
            .map_err(|e| tonic::Status::unavailable(e.to_string()))?;
        c.insert_tracks(InsertTracksRequest {
            playlist_id: None,
            position: -4, // INSERT_FIRST
            tracks: vec![path],
            shuffle: Some(false),
        })
        .await?;
        Ok(())
    })
}

/// Convenience: insert a single track at position INSERT_LAST (-3).
///
/// # Safety
/// `path_ptr` must be a valid NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn rb_insert_track_last(path_ptr: *const c_char) -> c_int {
    let Some(p) = cstr_to_str(path_ptr) else {
        return -1;
    };
    let path = p.to_string();
    run_unit(async move {
        let mut c = PlaylistServiceClient::connect(url())
            .await
            .map_err(|e| tonic::Status::unavailable(e.to_string()))?;
        c.insert_tracks(InsertTracksRequest {
            playlist_id: None,
            position: -3, // INSERT_LAST
            tracks: vec![path],
            shuffle: Some(false),
        })
        .await?;
        Ok(())
    })
}

/// # Safety
/// `path_ptr` must be a valid NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn rb_insert_directory(path_ptr: *const c_char, position: c_int) -> c_int {
    let Some(p) = cstr_to_str(path_ptr) else {
        return -1;
    };
    let directory = p.to_string();
    run_unit(async move {
        let mut c = PlaylistServiceClient::connect(url())
            .await
            .map_err(|e| tonic::Status::unavailable(e.to_string()))?;
        c.insert_directory(InsertDirectoryRequest {
            playlist_id: None,
            position,
            directory,
            recurse: Some(true),
            shuffle: Some(false),
        })
        .await?;
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn rb_remove_from_queue(position: c_int) -> c_int {
    run_unit(async move {
        let mut c = PlaylistServiceClient::connect(url())
            .await
            .map_err(|e| tonic::Status::unavailable(e.to_string()))?;
        c.remove_tracks(RemoveTracksRequest {
            positions: vec![position],
        })
        .await?;
        Ok(())
    })
}

/// JSON snapshot of `PlaylistService::GetCurrent` (queue position + tracks).
#[no_mangle]
pub extern "C" fn rb_get_playlist_current_json() -> *mut c_char {
    let res = RT.block_on(async {
        let mut c = PlaylistServiceClient::connect(url())
            .await
            .map_err(|e| e.to_string())?;
        connect_err(c.get_current(GetCurrentRequest {})).await
    });
    unwrap_or_err_string(res.map(|r| r.into_inner()))
}

// ── Library ─────────────────────────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn rb_get_tracks_json() -> *mut c_char {
    let res = RT.block_on(async {
        let mut c = LibraryServiceClient::connect(url())
            .await
            .map_err(|e| e.to_string())?;
        connect_err(c.get_tracks(GetTracksRequest {})).await
    });
    unwrap_or_err_string(res.map(|r| r.into_inner()))
}

#[no_mangle]
pub extern "C" fn rb_get_artists_json() -> *mut c_char {
    let res = RT.block_on(async {
        let mut c = LibraryServiceClient::connect(url())
            .await
            .map_err(|e| e.to_string())?;
        connect_err(c.get_artists(GetArtistsRequest {})).await
    });
    unwrap_or_err_string(res.map(|r| r.into_inner()))
}

#[no_mangle]
pub extern "C" fn rb_get_albums_json() -> *mut c_char {
    let res = RT.block_on(async {
        let mut c = LibraryServiceClient::connect(url())
            .await
            .map_err(|e| e.to_string())?;
        connect_err(c.get_albums(GetAlbumsRequest {})).await
    });
    unwrap_or_err_string(res.map(|r| r.into_inner()))
}

#[no_mangle]
pub extern "C" fn rb_get_liked_albums_json() -> *mut c_char {
    let res = RT.block_on(async {
        let mut c = LibraryServiceClient::connect(url())
            .await
            .map_err(|e| e.to_string())?;
        connect_err(c.get_liked_albums(GetLikedAlbumsRequest {})).await
    });
    unwrap_or_err_string(res.map(|r| r.into_inner()))
}

/// # Safety
/// `id_ptr` must be a valid NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn rb_get_artist_json(id_ptr: *const c_char) -> *mut c_char {
    let Some(id) = cstr_to_str(id_ptr) else {
        return err_string("missing id");
    };
    let id = id.to_string();
    let res = RT.block_on(async {
        let mut c = LibraryServiceClient::connect(url())
            .await
            .map_err(|e| e.to_string())?;
        connect_err(c.get_artist(GetArtistRequest { id })).await
    });
    unwrap_or_err_string(res.map(|r| r.into_inner()))
}

/// # Safety
/// `id_ptr` must be a valid NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn rb_get_album_json(id_ptr: *const c_char) -> *mut c_char {
    let Some(id) = cstr_to_str(id_ptr) else {
        return err_string("missing id");
    };
    let id = id.to_string();
    let res = RT.block_on(async {
        let mut c = LibraryServiceClient::connect(url())
            .await
            .map_err(|e| e.to_string())?;
        connect_err(c.get_album(GetAlbumRequest { id })).await
    });
    unwrap_or_err_string(res.map(|r| r.into_inner()))
}

#[no_mangle]
pub extern "C" fn rb_get_liked_tracks_json() -> *mut c_char {
    let res = RT.block_on(async {
        let mut c = LibraryServiceClient::connect(url())
            .await
            .map_err(|e| e.to_string())?;
        connect_err(c.get_liked_tracks(GetLikedTracksRequest {})).await
    });
    unwrap_or_err_string(res.map(|r| r.into_inner()))
}

/// # Safety
/// `term_ptr` must be a valid NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn rb_search_json(term_ptr: *const c_char) -> *mut c_char {
    let Some(term) = cstr_to_str(term_ptr) else {
        return err_string("missing term");
    };
    let term = term.to_string();
    let res = RT.block_on(async {
        let mut c = LibraryServiceClient::connect(url())
            .await
            .map_err(|e| e.to_string())?;
        connect_err(c.search(SearchRequest { term })).await
    });
    unwrap_or_err_string(res.map(|r| r.into_inner()))
}

// ── Genres ──────────────────────────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn rb_get_genres_json() -> *mut c_char {
    let res = RT.block_on(async {
        let mut c = GenreServiceClient::connect(url())
            .await
            .map_err(|e| e.to_string())?;
        connect_err(c.get_genres(GetGenresRequest {})).await
    });
    unwrap_or_err_string(res.map(|r| r.into_inner()))
}

/// # Safety
/// `id_ptr` must be a valid NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn rb_get_genre_json(id_ptr: *const c_char) -> *mut c_char {
    let Some(id) = cstr_to_str(id_ptr) else {
        return err_string("missing id");
    };
    let id = id.to_string();
    let res = RT.block_on(async {
        let mut c = GenreServiceClient::connect(url())
            .await
            .map_err(|e| e.to_string())?;
        connect_err(c.get_genre(GetGenreRequest { id })).await
    });
    unwrap_or_err_string(res.map(|r| r.into_inner()))
}

/// # Safety
/// `id_ptr` must be a valid NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn rb_get_genre_tracks_json(id_ptr: *const c_char) -> *mut c_char {
    let Some(id) = cstr_to_str(id_ptr) else {
        return err_string("missing id");
    };
    let id = id.to_string();
    let res = RT.block_on(async {
        let mut c = GenreServiceClient::connect(url())
            .await
            .map_err(|e| e.to_string())?;
        connect_err(c.get_genre_tracks(GetGenreTracksRequest { id })).await
    });
    unwrap_or_err_string(res.map(|r| r.into_inner()))
}

/// # Safety
/// `id_ptr` must be a valid NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn rb_get_genre_albums_json(id_ptr: *const c_char) -> *mut c_char {
    let Some(id) = cstr_to_str(id_ptr) else {
        return err_string("missing id");
    };
    let id = id.to_string();
    let res = RT.block_on(async {
        let mut c = GenreServiceClient::connect(url())
            .await
            .map_err(|e| e.to_string())?;
        connect_err(c.get_genre_albums(GetGenreAlbumsRequest { id })).await
    });
    unwrap_or_err_string(res.map(|r| r.into_inner()))
}

/// # Safety
/// `id_ptr` must be a valid NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn rb_get_genre_artists_json(id_ptr: *const c_char) -> *mut c_char {
    let Some(id) = cstr_to_str(id_ptr) else {
        return err_string("missing id");
    };
    let id = id.to_string();
    let res = RT.block_on(async {
        let mut c = GenreServiceClient::connect(url())
            .await
            .map_err(|e| e.to_string())?;
        connect_err(c.get_genre_artists(GetGenreArtistsRequest { id })).await
    });
    unwrap_or_err_string(res.map(|r| r.into_inner()))
}

// ── Sound ───────────────────────────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn rb_adjust_volume(steps: c_int) -> c_int {
    run_unit(async move {
        let mut c = SoundServiceClient::connect(url())
            .await
            .map_err(|e| tonic::Status::unavailable(e.to_string()))?;
        c.adjust_volume(AdjustVolumeRequest { steps }).await?;
        Ok(())
    })
}

/// `setting` follows the rockbox SOUND_* enum (0 = volume, etc.).
#[no_mangle]
pub extern "C" fn rb_sound_current_json(setting: c_int) -> *mut c_char {
    let res = RT.block_on(async {
        let mut c = SoundServiceClient::connect(url())
            .await
            .map_err(|e| e.to_string())?;
        connect_err(c.sound_current(SoundCurrentRequest { setting })).await
    });
    unwrap_or_err_string(res.map(|r| r.into_inner()))
}

// ── Settings ────────────────────────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn rb_save_shuffle(enabled: c_int) -> c_int {
    let enabled = b(enabled);
    run_unit(async move {
        let mut c = SettingsServiceClient::connect(url())
            .await
            .map_err(|e| tonic::Status::unavailable(e.to_string()))?;
        c.save_settings(SaveSettingsRequest {
            playlist_shuffle: Some(enabled),
            ..Default::default()
        })
        .await?;
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn rb_save_repeat(repeat_mode: c_int) -> c_int {
    run_unit(async move {
        let mut c = SettingsServiceClient::connect(url())
            .await
            .map_err(|e| tonic::Status::unavailable(e.to_string()))?;
        c.save_settings(SaveSettingsRequest {
            repeat_mode: Some(repeat_mode),
            ..Default::default()
        })
        .await?;
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn rb_get_global_settings_json() -> *mut c_char {
    let res = RT.block_on(async {
        let mut c = SettingsServiceClient::connect(url())
            .await
            .map_err(|e| e.to_string())?;
        connect_err(c.get_global_settings(GetGlobalSettingsRequest {})).await
    });
    unwrap_or_err_string(res.map(|r| r.into_inner()))
}

// ── System ──────────────────────────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn rb_get_global_status_json() -> *mut c_char {
    let res = RT.block_on(async {
        let mut c = SystemServiceClient::connect(url())
            .await
            .map_err(|e| e.to_string())?;
        connect_err(c.get_global_status(GetGlobalStatusRequest {})).await
    });
    unwrap_or_err_string(res.map(|r| r.into_inner()))
}

// ── Browse ──────────────────────────────────────────────────────────────────

/// `path_ptr` may be null to fetch the music root.
///
/// # Safety
/// If non-null, `path_ptr` must be a valid NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn rb_tree_get_entries_json(path_ptr: *const c_char) -> *mut c_char {
    let path = if path_ptr.is_null() {
        None
    } else {
        cstr_to_str(path_ptr).map(|s| s.to_string())
    };
    let res = RT.block_on(async {
        let mut c = BrowseServiceClient::connect(url())
            .await
            .map_err(|e| e.to_string())?;
        connect_err(c.tree_get_entries(TreeGetEntriesRequest { path })).await
    });
    unwrap_or_err_string(res.map(|r| r.into_inner()))
}

// ── Saved playlists ─────────────────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn rb_get_saved_playlists_json() -> *mut c_char {
    let res = RT.block_on(async {
        let mut c = SavedPlaylistServiceClient::connect(url())
            .await
            .map_err(|e| e.to_string())?;
        connect_err(c.get_saved_playlists(GetSavedPlaylistsRequest { folder_id: None })).await
    });
    unwrap_or_err_string(res.map(|r| r.into_inner()))
}

/// # Safety
/// `name_ptr` must be a valid UTF-8 NUL-terminated string. `description_ptr`
/// may be null. `track_ids_json_ptr` must be a JSON array of strings (may be
/// `[]` for an empty playlist).
#[no_mangle]
pub unsafe extern "C" fn rb_create_saved_playlist(
    name_ptr: *const c_char,
    description_ptr: *const c_char,
    track_ids_json_ptr: *const c_char,
) -> c_int {
    let Some(name) = cstr_to_str(name_ptr) else {
        return -1;
    };
    let name = name.to_string();
    let description = if description_ptr.is_null() {
        None
    } else {
        cstr_to_str(description_ptr).map(|s| s.to_string())
    };
    let track_ids: Vec<String> = if track_ids_json_ptr.is_null() {
        Vec::new()
    } else {
        let Some(s) = cstr_to_str(track_ids_json_ptr) else {
            return -2;
        };
        match serde_json::from_str(s) {
            Ok(v) => v,
            Err(_) => return -3,
        }
    };
    run_unit(async move {
        let mut c = SavedPlaylistServiceClient::connect(url())
            .await
            .map_err(|e| tonic::Status::unavailable(e.to_string()))?;
        c.create_saved_playlist(CreateSavedPlaylistRequest {
            name,
            description,
            image: None,
            folder_id: None,
            track_ids,
        })
        .await?;
        Ok(())
    })
}

/// # Safety
/// `id_ptr` and `name_ptr` must be valid UTF-8 NUL-terminated strings.
/// `description_ptr` may be null.
#[no_mangle]
pub unsafe extern "C" fn rb_update_saved_playlist(
    id_ptr: *const c_char,
    name_ptr: *const c_char,
    description_ptr: *const c_char,
) -> c_int {
    let Some(id) = cstr_to_str(id_ptr) else {
        return -1;
    };
    let Some(name) = cstr_to_str(name_ptr) else {
        return -1;
    };
    let id = id.to_string();
    let name = name.to_string();
    let description = if description_ptr.is_null() {
        None
    } else {
        cstr_to_str(description_ptr).map(|s| s.to_string())
    };
    run_unit(async move {
        let mut c = SavedPlaylistServiceClient::connect(url())
            .await
            .map_err(|e| tonic::Status::unavailable(e.to_string()))?;
        c.update_saved_playlist(UpdateSavedPlaylistRequest {
            id,
            name,
            description,
            image: None,
            folder_id: None,
        })
        .await?;
        Ok(())
    })
}

/// # Safety
/// `id_ptr` must be a valid NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn rb_delete_saved_playlist(id_ptr: *const c_char) -> c_int {
    let Some(id) = cstr_to_str(id_ptr) else {
        return -1;
    };
    let id = id.to_string();
    run_unit(async move {
        let mut c = SavedPlaylistServiceClient::connect(url())
            .await
            .map_err(|e| tonic::Status::unavailable(e.to_string()))?;
        c.delete_saved_playlist(DeleteSavedPlaylistRequest { id })
            .await?;
        Ok(())
    })
}

/// # Safety
/// Both pointers must be valid NUL-terminated UTF-8 strings.
#[no_mangle]
pub unsafe extern "C" fn rb_add_track_to_playlist(
    playlist_id_ptr: *const c_char,
    track_id_ptr: *const c_char,
) -> c_int {
    let Some(playlist_id) = cstr_to_str(playlist_id_ptr) else {
        return -1;
    };
    let Some(track_id) = cstr_to_str(track_id_ptr) else {
        return -1;
    };
    let playlist_id = playlist_id.to_string();
    let track_id = track_id.to_string();
    run_unit(async move {
        let mut c = SavedPlaylistServiceClient::connect(url())
            .await
            .map_err(|e| tonic::Status::unavailable(e.to_string()))?;
        c.add_tracks_to_saved_playlist(AddTracksToSavedPlaylistRequest {
            playlist_id,
            track_ids: vec![track_id],
        })
        .await?;
        Ok(())
    })
}

/// # Safety
/// Both pointers must be valid NUL-terminated UTF-8 strings.
#[no_mangle]
pub unsafe extern "C" fn rb_remove_track_from_playlist(
    playlist_id_ptr: *const c_char,
    track_id_ptr: *const c_char,
) -> c_int {
    let Some(playlist_id) = cstr_to_str(playlist_id_ptr) else {
        return -1;
    };
    let Some(track_id) = cstr_to_str(track_id_ptr) else {
        return -1;
    };
    let playlist_id = playlist_id.to_string();
    let track_id = track_id.to_string();
    run_unit(async move {
        let mut c = SavedPlaylistServiceClient::connect(url())
            .await
            .map_err(|e| tonic::Status::unavailable(e.to_string()))?;
        c.remove_track_from_saved_playlist(RemoveTrackFromSavedPlaylistRequest {
            playlist_id,
            track_id,
        })
        .await?;
        Ok(())
    })
}

/// # Safety
/// `id_ptr` must be a valid NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn rb_get_saved_playlist_tracks_json(id_ptr: *const c_char) -> *mut c_char {
    let Some(id) = cstr_to_str(id_ptr) else {
        return err_string("missing id");
    };
    let playlist_id = id.to_string();
    let res = RT.block_on(async {
        let mut c = SavedPlaylistServiceClient::connect(url())
            .await
            .map_err(|e| e.to_string())?;
        connect_err(c.get_saved_playlist_tracks(GetSavedPlaylistTracksRequest { playlist_id }))
            .await
    });
    unwrap_or_err_string(res.map(|r| r.into_inner()))
}

/// # Safety
/// `id_ptr` must be a valid NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn rb_play_saved_playlist(id_ptr: *const c_char) -> c_int {
    let Some(playlist_id) = cstr_to_str(id_ptr) else {
        return -1;
    };
    let playlist_id = playlist_id.to_string();
    run_unit(async move {
        let mut c = SavedPlaylistServiceClient::connect(url())
            .await
            .map_err(|e| tonic::Status::unavailable(e.to_string()))?;
        c.play_saved_playlist(PlaySavedPlaylistRequest { playlist_id })
            .await?;
        Ok(())
    })
}

// ── Smart playlists ─────────────────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn rb_get_smart_playlists_json() -> *mut c_char {
    let res = RT.block_on(async {
        let mut c = SmartPlaylistServiceClient::connect(url())
            .await
            .map_err(|e| e.to_string())?;
        connect_err(c.get_smart_playlists(GetSmartPlaylistsRequest {})).await
    });
    unwrap_or_err_string(res.map(|r| r.into_inner()))
}

/// # Safety
/// `id_ptr` must be a valid NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn rb_get_smart_playlist_tracks_json(id_ptr: *const c_char) -> *mut c_char {
    let Some(id_s) = cstr_to_str(id_ptr) else {
        return err_string("missing id");
    };
    let id = id_s.to_string();
    let res = RT.block_on(async {
        let mut c = SmartPlaylistServiceClient::connect(url())
            .await
            .map_err(|e| e.to_string())?;
        connect_err(c.get_smart_playlist_tracks(GetSmartPlaylistTracksRequest { id })).await
    });
    unwrap_or_err_string(res.map(|r| r.into_inner()))
}

/// # Safety
/// `id_ptr` must be a valid NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn rb_play_smart_playlist(id_ptr: *const c_char) -> c_int {
    let Some(id_s) = cstr_to_str(id_ptr) else {
        return -1;
    };
    let id = id_s.to_string();
    run_unit(async move {
        let mut c = SmartPlaylistServiceClient::connect(url())
            .await
            .map_err(|e| tonic::Status::unavailable(e.to_string()))?;
        c.play_smart_playlist(PlaySmartPlaylistRequest { id })
            .await?;
        Ok(())
    })
}

// ── Cast / AirPlay devices (HTTP REST, mirrors gpui/src/client.rs) ──────────
//
// rockboxd exposes a small REST surface on its `http_port` (default 6063) for
// device picking — Chromecast / AirPlay / Snapcast / UPnP. The gRPC layer
// doesn't cover this, so we use a tiny `reqwest` client to talk plain HTTP.

#[no_mangle]
pub extern "C" fn rb_get_devices_json() -> *mut c_char {
    let res: Result<String, String> = RT.block_on(async {
        let url = format!("{}/devices", http_url());
        let body = reqwest::get(&url)
            .await
            .map_err(|e| format!("get_devices: {e}"))?
            .text()
            .await
            .map_err(|e| format!("get_devices body: {e}"))?;
        Ok(body)
    });
    match res {
        Ok(json) => CString::new(json)
            .map(|c| c.into_raw())
            .unwrap_or(std::ptr::null_mut()),
        Err(e) => err_string(e),
    }
}

/// # Safety
/// `id_ptr` must be a valid NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn rb_connect_device(id_ptr: *const c_char) -> c_int {
    let Some(id) = cstr_to_str(id_ptr) else {
        return -1;
    };
    let id = id.to_string();
    let res: Result<(), String> = RT.block_on(async {
        let url = format!("{}/devices/{id}/connect", http_url());
        reqwest::Client::new()
            .put(&url)
            .send()
            .await
            .map_err(|e| format!("connect_device: {e}"))?;
        Ok(())
    });
    if res.is_ok() {
        0
    } else {
        -1
    }
}

/// # Safety
/// `id_ptr` must be a valid NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn rb_disconnect_device(id_ptr: *const c_char) -> c_int {
    let Some(id) = cstr_to_str(id_ptr) else {
        return -1;
    };
    let id = id.to_string();
    let res: Result<(), String> = RT.block_on(async {
        let url = format!("{}/devices/{id}/disconnect", http_url());
        reqwest::Client::new()
            .put(&url)
            .send()
            .await
            .map_err(|e| format!("disconnect_device: {e}"))?;
        Ok(())
    });
    if res.is_ok() {
        0
    } else {
        -1
    }
}

// ── Bluetooth ───────────────────────────────────────────────────────────────

/// Trigger a Bluetooth scan on the daemon. Required before `get_devices`
/// returns anything — rockboxd doesn't continuously scan, the picker has to
/// kick it. Returns 0 on success.
#[no_mangle]
pub extern "C" fn rb_scan_bluetooth() -> c_int {
    let res: Result<(), tonic::Status> = RT.block_on(async {
        let mut c = BluetoothServiceClient::connect(url())
            .await
            .map_err(|e| tonic::Status::unavailable(e.to_string()))?;
        c.scan(ScanBluetoothRequest { timeout_secs: 8 }).await?;
        Ok(())
    });
    if res.is_ok() {
        0
    } else {
        -1
    }
}

/// 1 if the Bluetooth service is reachable and answers GetDevices, 0 otherwise.
#[no_mangle]
pub extern "C" fn rb_bluetooth_available() -> c_int {
    let ok = RT.block_on(async {
        let Ok(mut c) = BluetoothServiceClient::connect(url()).await else {
            return false;
        };
        c.get_devices(GetBluetoothDevicesRequest {}).await.is_ok()
    });
    if ok {
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn rb_get_bluetooth_devices_json() -> *mut c_char {
    let res = RT.block_on(async {
        let mut c = BluetoothServiceClient::connect(url())
            .await
            .map_err(|e| e.to_string())?;
        connect_err(c.get_devices(GetBluetoothDevicesRequest {})).await
    });
    unwrap_or_err_string(res.map(|r| r.into_inner()))
}

/// # Safety
/// `addr_ptr` must be a valid NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn rb_connect_bluetooth(addr_ptr: *const c_char) -> c_int {
    let Some(s) = cstr_to_str(addr_ptr) else {
        return -1;
    };
    let address = s.to_string();
    run_unit(async move {
        let mut c = BluetoothServiceClient::connect(url())
            .await
            .map_err(|e| tonic::Status::unavailable(e.to_string()))?;
        c.connect_device(ConnectBluetoothDeviceRequest { address })
            .await?;
        Ok(())
    })
}

/// # Safety
/// `addr_ptr` must be a valid NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn rb_disconnect_bluetooth(addr_ptr: *const c_char) -> c_int {
    let Some(s) = cstr_to_str(addr_ptr) else {
        return -1;
    };
    let address = s.to_string();
    run_unit(async move {
        let mut c = BluetoothServiceClient::connect(url())
            .await
            .map_err(|e| tonic::Status::unavailable(e.to_string()))?;
        c.disconnect(DisconnectBluetoothDeviceRequest { address })
            .await?;
        Ok(())
    })
}

// ── Streaming subscriptions ─────────────────────────────────────────────────
//
// Pattern (Option B from the design discussion): each `rb_subscribe_*` spawns
// a tonic streaming RPC on the runtime, drains it into a bounded mpsc, and
// returns an `i32` subscription id. Platform code (Swift / Kotlin) drives a
// background loop calling `rb_poll_event(id, timeout_ms)`, which blocks up to
// `timeout_ms` waiting for the next JSON payload, returns it as a heap
// C string (free with `rb_free_string`), or returns null on timeout / closed
// stream. `rb_unsubscribe` aborts the task and removes the entry.

const EVENT_BUFFER: usize = 64;

struct Subscription {
    rx: mpsc::Receiver<String>,
    abort: AbortHandle,
}

static SUBS: Lazy<RwLock<HashMap<i32, Subscription>>> = Lazy::new(|| RwLock::new(HashMap::new()));
static NEXT_SUB_ID: AtomicI32 = AtomicI32::new(1);

#[derive(Serialize)]
struct StreamErrorJson {
    error: String,
}

fn next_sub_id() -> i32 {
    NEXT_SUB_ID.fetch_add(1, Ordering::SeqCst)
}

fn register_sub(rx: mpsc::Receiver<String>, abort: AbortHandle) -> i32 {
    let id = next_sub_id();
    if let Ok(mut map) = SUBS.write() {
        map.insert(id, Subscription { rx, abort });
    }
    id
}

/// Spawns `task_factory` on the runtime, where it should drain a tonic stream
/// into the supplied sender. Returns the subscription id.
fn spawn_stream<F, Fut>(task_factory: F) -> i32
where
    F: FnOnce(mpsc::Sender<String>) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    let (tx, rx) = mpsc::channel::<String>(EVENT_BUFFER);
    let handle = RT.spawn(async move {
        task_factory(tx).await;
    });
    register_sub(rx, handle.abort_handle())
}

/// Subscribes to `PlaybackService::StreamStatus`. Each event is a JSON
/// `{ "status": <int> }` payload (mirrors `rb_status_json`).
#[no_mangle]
pub extern "C" fn rb_subscribe_status() -> c_int {
    let server_url = url();
    spawn_stream(move |tx| async move {
        loop {
            let mut c = match PlaybackServiceClient::connect(server_url.clone()).await {
                Ok(c) => c,
                Err(_) => {
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    continue;
                }
            };
            let mut s = match c.stream_status(StreamStatusRequest {}).await {
                Ok(r) => r.into_inner(),
                Err(_) => {
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    continue;
                }
            };
            while let Ok(Some(msg)) = s.message().await {
                let payload = serde_json::to_string(&StatusJson { status: msg.status })
                    .unwrap_or_else(|_| "{}".into());
                if tx.send(payload).await.is_err() {
                    return;
                }
            }
            // Stream closed; reconnect after a short backoff.
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    })
}

/// Subscribes to `PlaybackService::StreamCurrentTrack`. Each event JSON
/// matches the `rb_current_track_json` shape.
#[no_mangle]
pub extern "C" fn rb_subscribe_current_track() -> c_int {
    let server_url = url();
    spawn_stream(move |tx| async move {
        loop {
            let mut c = match PlaybackServiceClient::connect(server_url.clone()).await {
                Ok(c) => c,
                Err(_) => {
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    continue;
                }
            };
            let mut s = match c.stream_current_track(StreamCurrentTrackRequest {}).await {
                Ok(r) => r.into_inner(),
                Err(_) => {
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    continue;
                }
            };
            while let Ok(Some(msg)) = s.message().await {
                let payload = serde_json::to_string(&TrackJson {
                    id: msg.id,
                    path: msg.path,
                    title: msg.title,
                    artist: msg.artist,
                    album: msg.album,
                    album_art: msg.album_art.filter(|a| !a.is_empty()),
                    duration_ms: msg.length as i64,
                    elapsed_ms: msg.elapsed as i64,
                })
                .unwrap_or_else(|_| "{}".into());
                if tx.send(payload).await.is_err() {
                    return;
                }
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    })
}

#[derive(Serialize)]
struct PlaylistEventJson {
    index: i32,
    amount: i32,
    tracks: Vec<TrackJson>,
}

/// Subscribes to `PlaybackService::StreamPlaylist`. Each event is a JSON
/// `{ index, amount, tracks: [...] }` snapshot of the queue.
#[no_mangle]
pub extern "C" fn rb_subscribe_playlist() -> c_int {
    let server_url = url();
    spawn_stream(move |tx| async move {
        loop {
            let mut c = match PlaybackServiceClient::connect(server_url.clone()).await {
                Ok(c) => c,
                Err(_) => {
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    continue;
                }
            };
            let mut s = match c.stream_playlist(StreamPlaylistRequest {}).await {
                Ok(r) => r.into_inner(),
                Err(_) => {
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    continue;
                }
            };
            while let Ok(Some(msg)) = s.message().await {
                let tracks: Vec<TrackJson> = msg
                    .tracks
                    .into_iter()
                    .map(|t| TrackJson {
                        id: t.id,
                        path: t.path,
                        title: t.title,
                        artist: t.artist,
                        album: t.album,
                        album_art: t.album_art.filter(|a| !a.is_empty()),
                        duration_ms: t.length as i64,
                        elapsed_ms: t.elapsed as i64,
                    })
                    .collect();
                let payload = serde_json::to_string(&PlaylistEventJson {
                    index: msg.index,
                    amount: msg.amount,
                    tracks,
                })
                .unwrap_or_else(|_| "{}".into());
                if tx.send(payload).await.is_err() {
                    return;
                }
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    })
}

// ── mDNS / Bonjour discovery ────────────────────────────────────────────────
//
// Wraps the `rockbox-discovery` crate so the mobile app can find rockboxd
// instances on the local network. Each event is one resolved service:
//   { "name", "fullname", "hostname", "port", "addresses": [...], "properties": {...} }
// Service names follow Bonjour conventions, e.g. "_rockbox._tcp.local.".

#[derive(Serialize)]
struct DiscoveryEventJson {
    name: String,
    fullname: String,
    hostname: String,
    port: u16,
    addresses: Vec<String>,
    properties: HashMap<String, String>,
}

/// Subscribes to mDNS / Bonjour discovery for `service_name` (e.g. the
/// constants exposed by `rockbox-discovery`: `_rockbox._tcp.local.`,
/// `_googlecast._tcp.local.`, etc.). Each resolved service triggers one
/// event payload.
///
/// # Safety
/// `service_name_ptr` must be a valid NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn rb_subscribe_discovery(service_name_ptr: *const c_char) -> c_int {
    let Some(service_name) = cstr_to_str(service_name_ptr) else {
        return -1;
    };
    let service_name = service_name.to_string();
    spawn_stream(move |tx| async move {
        let stream = rockbox_discovery::discover(&service_name);
        pin_mut!(stream);
        while let Some(info) = stream.next().await {
            let payload = DiscoveryEventJson {
                name: info.get_hostname().to_string(),
                fullname: info.get_fullname().to_string(),
                hostname: info.get_hostname().to_string(),
                port: info.get_port(),
                addresses: info.get_addresses().iter().map(|a| a.to_string()).collect(),
                properties: info
                    .get_properties()
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect(),
            };
            let json = serde_json::to_string(&payload).unwrap_or_else(|_| "{}".into());
            if tx.send(json).await.is_err() {
                break;
            }
        }
    })
}

/// Returns the well-known mDNS service name for the rockboxd gRPC daemon
/// (`_rockbox._tcp.local.`) as a heap C string. Free with `rb_free_string`.
#[no_mangle]
pub extern "C" fn rb_rockbox_service_name() -> *mut c_char {
    CString::new(rockbox_discovery::ROCKBOX_SERVICE_NAME)
        .map(|c| c.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

/// Returns the well-known mDNS service name for Chromecast devices.
#[no_mangle]
pub extern "C" fn rb_chromecast_service_name() -> *mut c_char {
    CString::new(rockbox_discovery::CHROMECAST_SERVICE_NAME)
        .map(|c| c.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

/// Subscribes to `LibraryService::StreamLibrary`. Each event is the full
/// (potentially large) library snapshot serialized as JSON.
#[no_mangle]
pub extern "C" fn rb_subscribe_library() -> c_int {
    let server_url = url();
    spawn_stream(move |tx| async move {
        match LibraryServiceClient::connect(server_url).await {
            Ok(mut c) => match c.stream_library(StreamLibraryRequest {}).await {
                Ok(resp) => {
                    let mut s = resp.into_inner();
                    while let Ok(Some(msg)) = s.message().await {
                        let payload = serde_json::to_string(&msg).unwrap_or_else(|_| "{}".into());
                        if tx.send(payload).await.is_err() {
                            break;
                        }
                    }
                }
                Err(e) => {
                    let _ = tx
                        .send(
                            serde_json::to_string(&StreamErrorJson {
                                error: format!("stream_library: {e}"),
                            })
                            .unwrap_or_default(),
                        )
                        .await;
                }
            },
            Err(e) => {
                let _ = tx
                    .send(
                        serde_json::to_string(&StreamErrorJson {
                            error: format!("connect: {e}"),
                        })
                        .unwrap_or_default(),
                    )
                    .await;
            }
        }
    })
}

/// Blocks up to `timeout_ms` waiting for the next event on subscription
/// `sub_id`. Returns a heap-owned JSON C string (free with `rb_free_string`),
/// or null on timeout / closed stream / unknown subscription.
#[no_mangle]
pub extern "C" fn rb_poll_event(sub_id: c_int, timeout_ms: c_int) -> *mut c_char {
    // Take the receiver out of the map briefly so we don't hold the lock
    // across the blocking await. Put it back unless it's been closed.
    let mut rx = {
        let mut map = match SUBS.write() {
            Ok(m) => m,
            Err(_) => return std::ptr::null_mut(),
        };
        match map.remove(&sub_id) {
            Some(s) => s,
            None => return std::ptr::null_mut(),
        }
    };

    let timeout = if timeout_ms < 0 {
        Duration::from_secs(60 * 60)
    } else {
        Duration::from_millis(timeout_ms as u64)
    };

    let received = RT.block_on(async { tokio::time::timeout(timeout, rx.rx.recv()).await });

    let payload = match received {
        Ok(Some(msg)) => Some(msg),
        Ok(None) => None,              // sender dropped → stream closed
        Err(_) => Some(String::new()), // marker: timeout, sub still alive
    };

    // Re-insert if the stream is still alive (any non-None payload).
    if payload.is_some() {
        if let Ok(mut map) = SUBS.write() {
            map.insert(sub_id, rx);
        }
    } else {
        // Stream closed — make sure the task is gone too.
        rx.abort.abort();
    }

    match payload {
        Some(s) if s.is_empty() => std::ptr::null_mut(),
        Some(s) => CString::new(s)
            .map(|c| c.into_raw())
            .unwrap_or(std::ptr::null_mut()),
        None => std::ptr::null_mut(),
    }
}

/// Cancels and removes a subscription. Returns 0 if found, -1 otherwise.
#[no_mangle]
pub extern "C" fn rb_unsubscribe(sub_id: c_int) -> c_int {
    let removed = match SUBS.write() {
        Ok(mut m) => m.remove(&sub_id),
        Err(_) => return -1,
    };
    match removed {
        Some(s) => {
            s.abort.abort();
            0
        }
        None => -1,
    }
}

// Reference values so the linker keeps the connect timeout default usable
// after stripping. (Defensive — some link configurations drop unused symbols.)
#[doc(hidden)]
pub fn _link_keepalive() -> Duration {
    Duration::from_secs(5)
}
