use crate::types::{audio_status::AudioStatus, file_position::FilePosition, mp3_entry::Mp3Entry};

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

    println!("current_track: {:?}", track);

    if track.is_null() {
        return None;
    }

    let track = unsafe { track.as_ref() };

    match track {
        Some(track) => Some((*track).into()),
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
