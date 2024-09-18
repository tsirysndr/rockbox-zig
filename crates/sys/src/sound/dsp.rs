use crate::{DspBuffer, DspConfig};

pub fn set_crossfeed_type(r#type: i32) {
    unsafe { crate::dsp_set_crossfeed_type(r#type) }
}

pub fn eq_enable(enable: bool) {
    let enable = if enable { 1 } else { 0 };
    unsafe { crate::dsp_eq_enable(enable) }
}

pub fn dither_enable(enable: bool) {
    let enable = if enable { 1 } else { 0 };
    unsafe { crate::dsp_dither_enable(enable) }
}

pub fn get_timestretch() -> i32 {
    unsafe { crate::dsp_get_timestretch() }
}

pub fn set_timestretch(percent: i32) {
    unsafe { crate::dsp_set_timestretch(percent) }
}

pub fn timestretch_enable(enabled: bool) {
    let enabled = if enabled { 1 } else { 0 };
    unsafe { crate::dsp_timestretch_enable(enabled) }
}

pub fn timestretch_available() -> bool {
    let ret = unsafe { crate::dsp_timestretch_available() };
    ret != 0
}

pub fn configure(dsp: *mut DspConfig, setting: u32, value: i64) -> i64 {
    unsafe { crate::dsp_configure(dsp, setting, value) }
}

pub fn get_config(dsp_id: i32) -> DspConfig {
    unsafe { crate::dsp_get_config(dsp_id) }
}

pub fn process(dsp: *mut DspConfig, src: *mut DspBuffer, dst: *mut *mut DspBuffer) {
    unsafe { crate::dsp_process(dsp, src, dst) }
}
