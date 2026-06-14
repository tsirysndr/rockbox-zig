use crate::{DspBuffer, DspConfig, EqBandSetting};
use std::ffi::c_long;

pub fn set_crossfeed_type(r#type: i32) {
    unsafe { crate::dsp_set_crossfeed_type(r#type) }
}

pub fn set_crossfeed_direct_gain(gain: i32) {
    unsafe { crate::dsp_set_crossfeed_direct_gain(gain) }
}

pub fn set_crossfeed_cross_params(lf_gain: i64, hf_gain: i64, cutoff: i64) {
    unsafe {
        crate::dsp_set_crossfeed_cross_params(
            lf_gain as c_long,
            hf_gain as c_long,
            cutoff as c_long,
        )
    }
}

pub fn eq_enable(enable: bool) {
    let enable = if enable { 1 } else { 0 };
    unsafe { crate::dsp_eq_enable(enable) }
}

pub fn set_eq_precut(precut: i32) {
    unsafe { crate::dsp_set_eq_precut(precut) }
}

pub fn set_eq_coefs(band: i32, setting: &EqBandSetting) {
    unsafe { crate::dsp_set_eq_coefs(band, setting as *const EqBandSetting) }
}

pub fn dither_enable(enable: bool) {
    let enable = if enable { 1 } else { 0 };
    unsafe { crate::dsp_dither_enable(enable) }
}

pub fn afr_enable(var: i32) {
    unsafe { crate::dsp_afr_enable(var) }
}

pub fn pbe_enable(var: i32) {
    unsafe { crate::dsp_pbe_enable(var) }
}

pub fn pbe_precut(var: i32) {
    unsafe { crate::dsp_pbe_precut(var) }
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
    unsafe { crate::dsp_configure(dsp, setting, value as c_long) as i64 }
}

pub fn get_config(dsp_id: i32) -> DspConfig {
    unsafe { crate::dsp_get_config(dsp_id) }
}

pub fn process(dsp: *mut DspConfig, src: *mut DspBuffer, dst: *mut *mut DspBuffer) {
    unsafe { crate::dsp_process(dsp, src, dst) }
}
