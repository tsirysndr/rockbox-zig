use std::ffi::CStr;

use crate::SystemSound;

pub mod dsp;
pub mod mixer;
pub mod pcm;

pub fn adjust_volume(steps: i32) {
    unsafe { crate::adjust_volume(steps) }
}

pub fn set(setting: i32, value: i32) {
    unsafe { crate::sound_set(setting, value) }
}

pub fn current(setting: i32) -> i32 {
    unsafe { crate::sound_current(setting) }
}

pub fn default(setting: i32) -> i32 {
    unsafe { crate::sound_default(setting) }
}

pub fn min(setting: i32) -> i32 {
    unsafe { crate::sound_min(setting) }
}

pub fn max(setting: i32) -> i32 {
    unsafe { crate::sound_max(setting) }
}

pub fn unit(setting: i32) -> String {
    let ret = unsafe { crate::sound_unit(setting) };
    let unit = unsafe { CStr::from_ptr(ret) };
    unit.to_str().unwrap().to_string()
}

pub fn val2phys(setting: i32, value: i32) -> i32 {
    unsafe { crate::sound_val2phys(setting, value) }
}

pub fn get_pitch() -> i32 {
    unsafe { crate::sound_get_pitch() }
}

pub fn set_pitch(pitch: i32) {
    unsafe { crate::sound_set_pitch(pitch) }
}

pub fn beep_play(frequency: u32, duration: u32, amplitude: u32) {
    unsafe { crate::beep_play(frequency, duration, amplitude) }
}

pub fn pcmbuf_fade(fade: i32, r#in: bool) {
    let r#in = if r#in { 1 } else { 0 };
    unsafe {
        crate::pcmbuf_fade(fade, r#in);
    }
}

pub fn pcmbuf_set_low_latency(state: bool) {
    let state = if state { 1 } else { 0 };
    unsafe {
        crate::pcmbuf_set_low_latency(state);
    }
}

pub fn system_sound_play(sound: SystemSound) {
    unsafe { crate::system_sound_play(sound) }
}

pub fn keyclick_click(rawbutton: bool, action: i32) {
    let rawbutton = if rawbutton { 1 } else { 0 };
    unsafe { crate::keyclick_click(rawbutton, action) }
}

/// Stock crossfade setter — drives the firmware directly. **Don't call
/// while audio is actively playing**: the firmware reconfigures the PCM
/// buffer ring (pcmbuf_init / audio_remake_audio_buffers), and on hosted
/// pthread targets (Android cdylib's pcm-aaudio.c) the writer pthread
/// runs asynchronously and races into half-rebuilt state → SIGSEGV at
/// PC=0 inside pcmbuf_pcm_callback. Use [`audio_set_crossfade_safe`] for
/// any caller that might fire mid-playback.
pub fn audio_set_crossfade(crossfade: i32) {
    unsafe { crate::audio_set_crossfade(crossfade) }
}

/// Crossfade setter that is safe to call any time, including
/// mid-playback. If the engine has a track loaded (`AUDIO_STATUS_PLAY`),
/// the change is bracketed by `audio_pause()` / `audio_resume()` so the
/// codec/audio threads quiesce before pcmbuf_init runs. Adds ~80 ms of
/// audible interruption when toggled mid-track; no overhead when stopped.
///
/// Same wrapper applies to any other setting that triggers an
/// `audio_remake_audio_buffers` (replaygain mode, output sample rate,
/// sound enable). Add a call-site wrapper for those too if needed.
pub fn audio_set_crossfade_safe(crossfade: i32) {
    // AUDIO_STATUS_PLAY = 0x0001 (firmware/export/audio.h). Set whenever
    // a track is loaded — true for both actively-playing and paused
    // states. Both need the bracketing because aa_thread is still the
    // PCM consumer in both cases.
    const AUDIO_STATUS_PLAY: i32 = 0x0001;
    let was_loaded = (unsafe { crate::audio_status() } & AUDIO_STATUS_PLAY) != 0;

    if was_loaded {
        // Pause the engine — codec_thread parks, no new PCM is produced.
        unsafe { crate::audio_pause() };
        // Give aa_thread a beat to drain any in-flight pcmbuf callback
        // and park on its pcm_data wait. 50 ms is comfortable; AAudio's
        // burst is typically a few ms.
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    unsafe { crate::audio_set_crossfade(crossfade) };

    if was_loaded {
        unsafe { crate::audio_resume() };
    }
}
