pub fn set_crossfeed_type() {
    unsafe {
        crate::dsp_set_crossfeed_type();
    }
}

pub fn eq_enable() {
    unsafe {
        crate::dsp_eq_enable();
    }
}

pub fn dither_enable() {
    unsafe {
        crate::dsp_dither_enable();
    }
}

pub fn get_timestretch() {
    unsafe {
        crate::dsp_get_timestretch();
    }
}

pub fn set_timestretch() {
    unsafe {
        crate::dsp_set_timestretch();
    }
}

pub fn timestretch_enable() {
    unsafe {
        crate::dsp_timestretch_enable();
    }
}

pub fn timestretch_available() {
    unsafe {
        crate::dsp_timestretch_available();
    }
}

pub fn configure() {
    unsafe {
        crate::dsp_configure();
    }
}

pub fn get_config() {
    unsafe {
        crate::dsp_get_config();
    }
}

pub fn process() {
    unsafe {
        crate::dsp_process();
    }
}
