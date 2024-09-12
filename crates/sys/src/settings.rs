use std::ffi::{c_char, c_int, c_uchar, c_void, CString};

use crate::{OptItems, SettingsList, UserSettings, Viewport, NB_SCREENS};

pub fn get_global_settings() -> UserSettings {
    unsafe { crate::global_settings }
}

pub fn get_settings_list(mut count: i32) -> *mut SettingsList {
    unsafe { crate::get_settings_list(&mut count as *mut i32 as *mut c_int) }
}

pub fn find_setting(variable: *const c_void) -> *mut SettingsList {
    unsafe { crate::find_setting(variable) }
}

pub fn settings_save() -> i32 {
    unsafe { crate::settings_save() }
}

pub fn option_screen(
    setting: *mut SettingsList,
    parent: [Viewport; NB_SCREENS],
    use_temp_var: bool,
    option_title: *mut c_uchar,
) -> bool {
    let use_temp_var = if use_temp_var { 1 } else { 0 };
    let ret = unsafe { crate::option_screen(setting, parent, use_temp_var, option_title) };
    ret != 0
}

pub fn set_option(
    string: &str,
    options: *const OptItems,
    numoptions: i32,
    function: Option<extern "C" fn(x: c_int) -> c_uchar>,
) -> bool {
    let sttring = CString::new(string).unwrap();
    let ret = unsafe { crate::set_option(sttring.as_ptr(), options, numoptions, function) };
    ret != 0
}

pub fn set_bool_options(
    string: &str,
    variable: *const c_uchar,
    yes_str: &str,
    yes_voice: i32,
    no_str: &str,
    no_voice: i32,
    function: Option<extern "C" fn(x: c_int) -> c_uchar>,
) {
    let string = CString::new(string).unwrap();
    let yes_str = CString::new(yes_str).unwrap();
    let no_str = CString::new(no_str).unwrap();
    unsafe {
        crate::set_bool_options(
            string.as_ptr(),
            variable,
            yes_str.as_ptr(),
            yes_voice,
            no_str.as_ptr(),
            no_voice,
            function,
        )
    }
}

pub fn set_int(
    unit: &str,
    voice_unit: i32,
    variable: *const c_int,
    function: Option<extern "C" fn(c_int)>,
    step: c_int,
    min: c_int,
    max: c_int,
    formatter: Option<extern "C" fn(*mut c_char, usize, c_int, *const c_char) -> *const c_char>,
) {
    let unit = CString::new(unit).unwrap();
    unsafe {
        crate::set_int(
            unit.as_ptr(),
            voice_unit,
            variable,
            function,
            step,
            min,
            max,
            formatter,
        )
    }
}

pub fn set_int_ex(
    unit: &str,
    voice_unit: i32,
    variable: *const c_int,
    function: Option<extern "C" fn(c_int)>,
    step: i32,
    min: i32,
    max: i32,
    formatter: Option<extern "C" fn(*mut c_char, usize, c_int, *const c_char) -> *const c_char>,
    get_talk_id: Option<extern "C" fn(c_int, c_int) -> c_int>,
) {
    let unit = CString::new(unit).unwrap();
    unsafe {
        crate::set_int_ex(
            unit.as_ptr(),
            voice_unit,
            variable,
            function,
            step,
            min,
            max,
            formatter,
            get_talk_id,
        )
    }
}

pub fn set_bool(string: &str, variable: *const c_uchar) -> bool {
    let string = CString::new(string).unwrap();
    let ret = unsafe { crate::set_bool(string.as_ptr(), variable) };
    ret != 0
}
