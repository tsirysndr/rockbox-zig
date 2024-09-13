use std::ffi::c_void;

use crate::{PcmPlayCallbackType, PcmStatusCallbackType};

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
