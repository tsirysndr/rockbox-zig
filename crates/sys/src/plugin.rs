pub fn plugin_open() {
    unsafe {
        crate::plugin_open();
    }
}

pub fn plugin_get_buffer() {
    unsafe {
        crate::plugin_get_buffer();
    }
}

pub fn plugin_get_current_filename() {
    unsafe {
        crate::plugin_get_current_filename();
    }
}

pub fn plugin_reserve_buffer() {
    unsafe {
        crate::plugin_reserve_buffer();
    }
}
