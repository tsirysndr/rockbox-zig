//! Thin wrapper around the C-side `pcm_external_write` helper.
//!
//! That helper just calls `sinks[cur_sink]->ops.play(addr, size)` so the HLS
//! player's decoded PCM streams through whatever audio output the user has
//! currently selected — cpal, SDL, AirPlay, Snapcast, CMAF, etc.

use std::os::raw::{c_uint, c_void};

extern "C" {
    fn pcm_external_write(addr: *const c_void, size: usize);
    fn pcm_external_set_freq(rate: c_uint);
}

/// Push interleaved S16LE stereo PCM into the active sink.
pub fn write_pcm(samples: &[i16]) {
    if samples.is_empty() {
        return;
    }
    let bytes = std::mem::size_of_val(samples);
    unsafe { pcm_external_write(samples.as_ptr() as *const c_void, bytes) }
}

/// Tell the sink chain what sample rate the upcoming PCM was decoded at.
pub fn set_sample_rate(rate: u32) {
    unsafe { pcm_external_set_freq(rate) }
}
