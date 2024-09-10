use crate::Mp3Entry;

pub fn pause() {
    unsafe { crate::audio_pause() }
}

pub fn play(elapsed: i64, offset: i64) {
    unsafe { crate::audio_play(elapsed, offset) }
}

pub fn resume() {
    unsafe { crate::audio_resume() }
}

pub fn next() {
    unsafe { crate::audio_next() }
}

pub fn prev() {
    unsafe { crate::audio_prev() }
}

pub fn ff_rewind(newtime: i32) {
    unsafe { crate::audio_ff_rewind(newtime) }
}

pub fn next_track() -> *mut Mp3Entry {
    unsafe { crate::audio_next_track() }
}

pub fn status() -> i32 {
    unsafe { crate::audio_status() }
}

pub fn current_track() -> *mut Mp3Entry {
    unsafe { crate::audio_current_track() }
}

pub fn flush_and_reload_tracks() {
    unsafe { crate::audio_flush_and_reload_tracks() }
}

pub fn get_file_pos() -> i32 {
    unsafe { crate::audio_get_file_pos() }
}

pub fn hard_stop() {
    unsafe { crate::audio_hard_stop() }
}
