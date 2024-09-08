pub fn apply_settings() {
    unsafe {
        crate::pcm_apply_settings();
    }
}

pub fn play_data() {
    unsafe {
        crate::pcm_play_data();
    }
}

pub fn play_stop() {
    unsafe {
        crate::pcm_play_stop();
    }
}

pub fn set_frequency() {
    unsafe {
        crate::pcm_set_frequency();
    }
}

pub fn is_playing() {
    unsafe {
        crate::pcm_is_playing();
    }
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
