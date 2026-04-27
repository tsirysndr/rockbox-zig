use std::ffi::{c_void, CString};

use crate::{PcmPlayCallbackType, PcmStatusCallbackType};

pub const PCM_SINK_BUILTIN: i32 = 0;
pub const PCM_SINK_FIFO: i32 = 1;
pub const PCM_SINK_AIRPLAY: i32 = 2;
pub const PCM_SINK_SQUEEZELITE: i32 = 3;
pub const PCM_SINK_UPNP: i32 = 4;
pub const PCM_SINK_CHROMECAST: i32 = 5;
pub const PCM_SINK_SNAPCAST_TCP: i32 = 6;

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

pub fn airplay_set_host(host: &str, port: u16) {
    let chost = CString::new(host).expect("host must not contain null bytes");
    unsafe { crate::pcm_airplay_set_host(chost.as_ptr(), port) }
    std::mem::forget(chost);
}

pub fn airplay_add_receiver(host: &str, port: u16) {
    let chost = CString::new(host).expect("host must not contain null bytes");
    unsafe { crate::pcm_airplay_add_receiver(chost.as_ptr(), port) }
    std::mem::forget(chost);
}

pub fn airplay_clear_receivers() {
    unsafe { crate::pcm_airplay_clear_receivers() }
}

pub fn fifo_set_path(path: &str) {
    let cpath = CString::new(path).expect("path must not contain null bytes");
    unsafe { crate::pcm_fifo_set_path(cpath.as_ptr()) }
    // Keep alive until C code finishes using it — it's only read during init,
    // so leaking is acceptable here for a startup-time config call.
    std::mem::forget(cpath);
}

pub fn squeezelite_set_slim_port(port: u16) {
    unsafe { crate::pcm_squeezelite_set_slim_port(port) }
}

pub fn squeezelite_set_http_port(port: u16) {
    unsafe { crate::pcm_squeezelite_set_http_port(port) }
}

pub fn upnp_set_http_port(port: u16) {
    unsafe { crate::pcm_upnp_set_http_port(port) }
}

pub fn upnp_set_renderer_url(url: &str) {
    let curl = std::ffi::CString::new(url).expect("url must not contain null bytes");
    unsafe { crate::pcm_upnp_set_renderer_url(curl.as_ptr()) }
    std::mem::forget(curl);
}

pub fn upnp_clear_renderer_url() {
    unsafe { crate::pcm_upnp_set_renderer_url(std::ptr::null()) }
}

/// Reset renderer-side state so the next play always sends SetAVTransportURI+Play.
/// Call this before switch_sink(PCM_SINK_UPNP) so output switching works live.
pub fn upnp_reset_renderer() {
    unsafe { crate::pcm_upnp_reset_renderer() }
}

pub fn chromecast_set_http_port(port: u16) {
    unsafe { crate::pcm_chromecast_set_http_port(port) }
}

pub fn chromecast_set_device_host(host: &str) {
    let chost = std::ffi::CString::new(host).expect("host must not contain null bytes");
    unsafe { crate::pcm_chromecast_set_device_host(chost.as_ptr()) }
    std::mem::forget(chost);
}

pub fn chromecast_set_device_port(port: u16) {
    unsafe { crate::pcm_chromecast_set_device_port(port) }
}

pub fn chromecast_teardown() {
    unsafe { crate::pcm_chromecast_teardown() }
}

pub fn tcp_set_host(host: &str) {
    let chost = CString::new(host).expect("host must not contain null bytes");
    unsafe { crate::pcm_tcp_set_host(chost.as_ptr()) }
    std::mem::forget(chost);
}

pub fn tcp_set_port(port: u16) {
    unsafe { crate::pcm_tcp_set_port(port) }
}
