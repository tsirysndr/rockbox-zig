use std::ffi::{c_void, CString};

use crate::{PcmPlayCallbackType, PcmStatusCallbackType};

pub const PCM_SINK_BUILTIN: i32 = 0;
pub const PCM_SINK_FIFO: i32 = 1;

pub fn apply_settings() {
    unsafe {
        crate::pcm_apply_settings();
    }
}

pub fn play_data(
    get_more: PcmPlayCallbackType,
    status_cb: PcmStatusCallbackType,
    start: *const *const c_void,
    size: usize,
) {
    unsafe { crate::pcm_play_data(get_more, status_cb, start, size) }
}

pub fn play_stop() {
    unsafe {
        crate::pcm_play_stop();
    }
}

pub fn set_frequency(frequency: u32) {
    unsafe { crate::pcm_set_frequency(frequency) }
}

pub fn is_playing() -> bool {
    let ret = unsafe { crate::pcm_is_playing() };
    ret != 0
}

pub fn play_lock() {
    unsafe {
        crate::pcm_play_lock();
    }
}

pub fn play_unlock() {
    unsafe {
        crate::pcm_play_unlock();
    }
}

pub fn switch_sink(sink: i32) -> bool {
    unsafe { crate::pcm_switch_sink(sink) != 0 }
}

pub fn fifo_set_path(path: &str) {
    let cpath = CString::new(path).expect("path must not contain null bytes");
    unsafe { crate::pcm_fifo_set_path(cpath.as_ptr()) }
    // Keep alive until C code finishes using it — it's only read during init,
    // so leaking is acceptable here for a startup-time config call.
    std::mem::forget(cpath);
}
