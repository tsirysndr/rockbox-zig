pub mod dsp;
pub mod mixer;
pub mod pcm;

pub fn adjust_volume() {
    unsafe {
        crate::adjust_volume();
    }
}

pub fn set() {
    unsafe {
        crate::sound_set();
    }
}

pub fn current() {
    unsafe {
        crate::sound_current();
    }
}

pub fn default() {
    unsafe {
        crate::sound_default();
    }
}

pub fn min() {
    unsafe {
        crate::sound_min();
    }
}

pub fn max() {
    unsafe {
        crate::sound_max();
    }
}

pub fn unit() {
    unsafe {
        crate::sound_unit();
    }
}

pub fn val2phys() {
    unsafe {
        crate::sound_val2phys();
    }
}

pub fn get_pitch() {
    unsafe {
        crate::sound_get_pitch();
    }
}

pub fn set_pitch() {
    unsafe {
        crate::sound_set_pitch();
    }
}

pub fn audio_master_sampr_list() {
    unsafe {
        crate::audio_master_sampr_list();
    }
}

pub fn beep_play() {
    unsafe {
        crate::beep_play();
    }
}

pub fn pcmbuf_fade() {
    unsafe {
        crate::pcmbuf_fade();
    }
}

pub fn pcmbuf_set_low_latency() {
    unsafe {
        crate::pcmbuf_set_low_latency();
    }
}

pub fn system_sound_play() {
    unsafe {
        crate::system_sound_play();
    }
}

pub fn keyclick_click() {
    unsafe {
        crate::keyclick_click();
    }
}
