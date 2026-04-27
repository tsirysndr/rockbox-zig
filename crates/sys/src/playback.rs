use crate::types::{audio_status::AudioStatus, file_position::FilePosition, mp3_entry::Mp3Entry};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

struct MetadataOverride {
    title: String,
    artist: String,
    album: String,
    /// Milliseconds; 0 means "don't override".
    length_ms: u64,
    /// Remote URL of the album art image, empty means "no override".
    album_art_url: String,
}

static METADATA_OVERRIDES: OnceLock<Mutex<HashMap<String, MetadataOverride>>> = OnceLock::new();

fn overrides() -> &'static Mutex<HashMap<String, MetadataOverride>> {
    METADATA_OVERRIDES.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Store metadata that will be overlaid on top of whatever the C codec parsed
/// whenever `current_track()` returns a track whose path matches `url`.
/// Call this before or just after starting playback so all callers (HTTP API,
/// gRPC, GraphQL, MPD) immediately see meaningful values.
pub fn set_metadata_override(
    url: &str,
    title: &str,
    artist: &str,
    album: &str,
    length_ms: u64,
    album_art_url: &str,
) {
    overrides().lock().unwrap().insert(
        url.to_string(),
        MetadataOverride {
            title: title.to_string(),
            artist: artist.to_string(),
            album: album.to_string(),
            length_ms,
            album_art_url: album_art_url.to_string(),
        },
    );
}

/// Remove a previously-registered override (e.g. when the stream stops).
pub fn clear_metadata_override(url: &str) {
    overrides().lock().unwrap().remove(url);
}

pub fn pause() {
    unsafe {
        crate::audio_pause();
    }
}

pub fn play(elapsed: i64, offset: i64) {
    unsafe {
        crate::audio_play(elapsed, offset);
    }
}

pub fn resume() {
    unsafe {
        crate::audio_resume();
    }
}

pub fn next() {
    unsafe {
        crate::audio_next();
    }
}

pub fn prev() {
    unsafe {
        crate::audio_prev();
    }
}

pub fn ff_rewind(newtime: i32) {
    unsafe {
        crate::audio_ff_rewind(newtime);
    }
}

pub fn next_track() -> Option<Mp3Entry> {
    let track = unsafe { crate::audio_next_track() };

    if track.is_null() {
        return None;
    }

    let track = unsafe { track.as_ref() };

    match track {
        Some(track) => Some((*track).into()),
        None => None,
    }
}

pub fn status() -> AudioStatus {
    let status = unsafe { crate::audio_status() };
    return AudioStatus { status };
}

pub fn current_track() -> Option<Mp3Entry> {
    let track = unsafe { crate::audio_current_track() };

    if track.is_null() {
        return None;
    }

    let track = unsafe { track.as_ref() };

    match track {
        Some(track) => {
            let mut entry: Mp3Entry = (*track).into();
            if let Ok(map) = overrides().lock() {
                if let Some(ov) = map.get(&entry.path) {
                    if !ov.title.is_empty() {
                        entry.title = ov.title.clone();
                    }
                    if !ov.artist.is_empty() {
                        entry.artist = ov.artist.clone();
                    }
                    if !ov.album.is_empty() {
                        entry.album = ov.album.clone();
                    }
                    if ov.length_ms > 0 {
                        entry.length = ov.length_ms;
                    }
                    if !ov.album_art_url.is_empty() {
                        entry.album_art = Some(ov.album_art_url.clone());
                    }
                }
            }
            Some(entry)
        }
        None => None,
    }
}

pub fn flush_and_reload_tracks() {
    unsafe {
        crate::audio_flush_and_reload_tracks();
    }
}

pub fn get_file_pos() -> FilePosition {
    let position = unsafe { crate::audio_get_file_pos() };
    FilePosition { position }
}

pub fn hard_stop() {
    unsafe {
        crate::audio_hard_stop();
    }
}

pub fn set_repeat_mode(mode: i32) {
    unsafe {
        crate::set_repeat_mode(mode);
    }
}
