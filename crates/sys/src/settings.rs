use std::ffi::c_int;

pub fn get_settings_list(mut count: i32) {
    unsafe {
        crate::get_settings_list(&mut count as *mut i32 as *mut c_int);
    }
}

pub fn find_setting() {
    unsafe {
        crate::find_setting();
    }
}

pub fn settings_save() {
    unsafe {
        crate::settings_save();
    }
}

pub fn option_screen() {
    unsafe {
        crate::option_screen();
    }
}

pub fn set_option() {
    unsafe {
        crate::set_option();
    }
}

pub fn set_bool_options() {
    unsafe {
        crate::set_bool_options();
    }
}

pub fn set_int() {
    unsafe {
        crate::set_int();
    }
}

pub fn set_int_ex() {
    unsafe {
        crate::set_int_ex();
    }
}

pub fn set_bool() {
    unsafe {
        crate::set_bool();
    }
}

pub fn set_color() {
    unsafe {
        crate::set_color();
    }
}
