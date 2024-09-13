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
