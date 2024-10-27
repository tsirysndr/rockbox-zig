use std::ffi::{c_char, c_int, c_uchar, c_void, CString};

use crate::{
    set_bool_setting, set_str_setting, set_value_setting,
    types::user_settings::{NewGlobalSettings, UserSettings},
    EqBandSetting, OptItems, SettingsList, Viewport, EQ_NUM_BANDS, NB_SCREENS,
};

pub fn get_global_settings() -> UserSettings {
    unsafe {
        crate::rb_get_crossfade_mode();
        crate::global_settings
    }
    .into()
}

pub fn get_settings_list(mut count: i32) -> SettingsList {
    unsafe { crate::get_settings_list(&mut count as *mut i32 as *mut c_int) }
}

pub fn find_setting(variable: *const c_void) -> SettingsList {
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

pub fn get_crossfade_mode() -> i32 {
    unsafe { crate::rb_get_crossfade_mode() }
}

pub fn save_settings(settings: NewGlobalSettings) {
    unsafe {
        set_bool_setting!(
            settings.playlist_shuffle,
            crate::global_settings.playlist_shuffle
        );
        set_value_setting!(settings.repeat_mode, crate::global_settings.repeat_mode);
        set_value_setting!(settings.bass, crate::global_settings.bass);
        set_value_setting!(settings.treble, crate::global_settings.treble);
        set_value_setting!(settings.bass_cutoff, crate::global_settings.bass_cutoff);
        set_value_setting!(settings.treble_cutoff, crate::global_settings.treble_cutoff);
        set_value_setting!(settings.crossfade, crate::global_settings.crossfade);
        set_bool_setting!(settings.fade_on_stop, crate::global_settings.fade_on_stop);
        set_value_setting!(
            settings.fade_in_delay,
            crate::global_settings.crossfade_fade_in_delay
        );
        set_value_setting!(
            settings.fade_in_duration,
            crate::global_settings.crossfade_fade_in_duration
        );
        set_value_setting!(
            settings.fade_out_delay,
            crate::global_settings.crossfade_fade_out_delay
        );
        set_value_setting!(
            settings.fade_out_duration,
            crate::global_settings.crossfade_fade_out_duration
        );
        set_value_setting!(settings.balance, crate::global_settings.balance);
        set_value_setting!(settings.stereo_width, crate::global_settings.stereo_width);
        set_value_setting!(settings.stereosw_mode, crate::global_settings.stereosw_mode);
        set_bool_setting!(
            settings.surround_enabled,
            crate::global_settings.surround_enabled
        );
        set_value_setting!(
            settings.surround_balance,
            crate::global_settings.surround_balance
        );
        set_value_setting!(settings.surround_fx1, crate::global_settings.surround_fx1);
        set_bool_setting!(settings.surround_fx2, crate::global_settings.surround_fx2);
        set_bool_setting!(settings.party_mode, crate::global_settings.party_mode);
        set_value_setting!(
            settings.channel_config,
            crate::global_settings.channel_config
        );
        set_str_setting!(settings.player_name, crate::global_settings.player_name, 64);
        set_bool_setting!(settings.eq_enabled, crate::global_settings.eq_enabled);

        if let Some(eq_band_settings) = settings.eq_band_settings {
            let mut array = [EqBandSetting {
                cutoff: 0,
                gain: 0,
                q: 0,
            }; EQ_NUM_BANDS];

            for (i, eq_band_setting) in eq_band_settings.into_iter().enumerate() {
                array[i] = eq_band_setting.into();
            }

            crate::global_settings.eq_band_settings = array;
        }

        if let Some(replaygain_settings) = settings.replaygain_settings {
            crate::global_settings.replaygain_settings = replaygain_settings.into();
        }

        crate::settings_save();
    }
}

pub fn apply_settings(read_disk: bool) {
    unsafe {
        let read_disk = if read_disk { 1 } else { 0 };
        crate::settings_apply(read_disk);
    }
}
