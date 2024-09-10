use std::ffi::{c_int, c_void};

use crate::{ChanBufferHookFnType, ChannelStatus, PcmMixerChannel, PcmPeaks, PcmPlayCallbackType};

pub fn channel_status(channel: PcmMixerChannel) -> ChannelStatus {
    unsafe { crate::mixer_channel_status(channel) }
}

pub fn channel_get_buffer(channel: PcmMixerChannel, count: *mut c_int) -> *mut c_void {
    unsafe { crate::mixer_channel_get_buffer(channel, count) }
}

pub fn channel_calculate_peaks(channel: PcmMixerChannel, peaks: *mut PcmPeaks) {
    unsafe {
        crate::mixer_channel_calculate_peaks(channel, peaks);
    }
}

pub fn channel_play_data(
    channel: PcmMixerChannel,
    get_more: PcmPlayCallbackType,
    start: *const *const c_void,
    size: usize,
) {
    unsafe { crate::mixer_channel_play_data(channel, get_more, start, size) }
}

pub fn channel_play_pause(channel: PcmMixerChannel, play: bool) {
    let play = if play { 1 } else { 0 };
    unsafe { crate::mixer_channel_play_pause(channel, play) }
}

pub fn channel_stop(channel: PcmMixerChannel) {
    unsafe {
        crate::mixer_channel_stop(channel);
    }
}

pub fn channel_set_amplitude(channel: PcmMixerChannel, amplitude: u32) {
    unsafe {
        crate::mixer_channel_set_amplitude(channel, amplitude);
    }
}

pub fn channel_get_bytes_waiting(channel: PcmMixerChannel) -> usize {
    unsafe { crate::mixer_channel_get_bytes_waiting(channel) }
}

pub fn channel_set_buffer_hook(channel: PcmMixerChannel, r#fn: ChanBufferHookFnType) {
    unsafe { crate::mixer_channel_set_buffer_hook(channel, r#fn) }
}

pub fn set_frequency(samplerate: u32) {
    unsafe {
        crate::mixer_set_frequency(samplerate);
    }
}

pub fn get_frequency() -> u32 {
    unsafe { crate::mixer_get_frequency() }
}
