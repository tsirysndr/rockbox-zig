pub fn channel_status() {
    unsafe {
        crate::mixer_channel_status();
    }
}

pub fn channel_get_buffer() {
    unsafe {
        crate::mixer_channel_get_buffer();
    }
}

pub fn channel_calculate_peaks() {
    unsafe {
        crate::mixer_channel_calculate_peaks();
    }
}

pub fn channel_play_data() {
    unsafe {
        crate::mixer_channel_play_data();
    }
}

pub fn channel_play_pause() {
    unsafe {
        crate::mixer_channel_play_pause();
    }
}

pub fn channel_stop() {
    unsafe {
        crate::mixer_channel_stop();
    }
}

pub fn channel_set_amplitude() {
    unsafe {
        crate::mixer_channel_set_amplitude();
    }
}

pub fn channel_get_bytes_waiting() {
    unsafe {
        crate::mixer_channel_get_bytes_waiting();
    }
}

pub fn channel_set_buffer_hook() {
    unsafe {
        crate::mixer_channel_set_buffer_hook();
    }
}

pub fn set_frequency() {
    unsafe {
        crate::mixer_set_frequency();
    }
}

pub fn get_frequency() {
    unsafe {
        crate::mixer_get_frequency();
    }
}
