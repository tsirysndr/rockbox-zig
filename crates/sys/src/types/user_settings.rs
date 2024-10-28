use std::ffi::CStr;

use crate::cast_ptr;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ReplaygainSettings {
    pub noclip: bool, // scale to prevent clips
    pub r#type: i32, // 0=track gain, 1=album gain, 2=track gain if shuffle is on, album gain otherwise, 4=off
    pub preamp: i32, // scale replaygained tracks by this
}

impl From<crate::ReplaygainSettings> for ReplaygainSettings {
    fn from(settings: crate::ReplaygainSettings) -> Self {
        let noclip = settings.noclip;
        Self {
            noclip,
            r#type: settings.r#type,
            preamp: settings.preamp,
        }
    }
}

impl Into<crate::ReplaygainSettings> for ReplaygainSettings {
    fn into(self) -> crate::ReplaygainSettings {
        crate::ReplaygainSettings {
            noclip: self.noclip,
            r#type: self.r#type,
            preamp: self.preamp,
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct EqBandSetting {
    pub cutoff: i32,
    pub q: i32,
    pub gain: i32,
}

impl From<crate::EqBandSetting> for EqBandSetting {
    fn from(setting: crate::EqBandSetting) -> Self {
        Self {
            cutoff: setting.cutoff,
            q: setting.q,
            gain: setting.gain,
        }
    }
}

impl Into<crate::EqBandSetting> for EqBandSetting {
    fn into(self) -> crate::EqBandSetting {
        crate::EqBandSetting {
            cutoff: self.cutoff,
            q: self.q,
            gain: self.gain,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct TouchscreenParameter {
    pub A: i32,
    pub B: i32,
    pub C: i32,
    pub D: i32,
    pub E: i32,
    pub F: i32,
    pub divider: i32,
}

impl From<crate::TouchscreenParameter> for TouchscreenParameter {
    fn from(parameter: crate::TouchscreenParameter) -> Self {
        Self {
            A: parameter.A,
            B: parameter.B,
            C: parameter.C,
            D: parameter.D,
            E: parameter.E,
            F: parameter.F,
            divider: parameter.divider,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct CompressorSettings {
    pub threshold: i32,
    pub makeup_gain: i32,
    pub ratio: i32,
    pub knee: i32,
    pub release_time: i32,
    pub attack_time: i32,
}

impl From<crate::CompressorSettings> for CompressorSettings {
    fn from(settings: crate::CompressorSettings) -> Self {
        Self {
            threshold: settings.threshold,
            makeup_gain: settings.makeup_gain,
            ratio: settings.ratio,
            knee: settings.knee,
            release_time: settings.release_time,
            attack_time: settings.attack_time,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct UserSettings {
    pub music_dir: String,

    // Audio settings
    pub volume: i32,
    pub balance: i32,
    pub bass: i32,
    pub treble: i32,
    pub channel_config: i32,
    pub stereo_width: i32,

    pub bass_cutoff: i32,
    pub treble_cutoff: i32,

    pub crossfade: i32,
    pub crossfade_fade_in_delay: i32,
    pub crossfade_fade_out_delay: i32,
    pub crossfade_fade_in_duration: i32,
    pub crossfade_fade_out_duration: i32,
    pub crossfade_fade_out_mixmode: i32,

    // Replaygain
    pub replaygain_settings: ReplaygainSettings,

    // Crossfeed
    pub crossfeed: i32,
    pub crossfeed_direct_gain: u32,
    pub crossfeed_cross_gain: u32,
    pub crossfeed_hf_attenuation: u32,
    pub crossfeed_hf_cutoff: u32,

    // EQ
    pub eq_enabled: bool,
    pub eq_precut: u32,
    pub eq_band_settings: Vec<EqBandSetting>,

    // Misc. swcodec
    pub beep: i32,
    pub keyclick: i32,
    pub keyclick_repeats: i32,
    pub dithering_enabled: bool,
    pub timestretch_enabled: bool,

    // Misc options
    pub list_accel_start_delay: i32,
    pub list_accel_wait: i32,

    pub touchpad_sensitivity: i32,
    pub touchpad_deadzone: i32,

    pub pause_rewind: i32,
    pub unplug_mode: i32,
    pub unplug_autoresume: bool,

    pub timeformat: i32,
    pub disk_spindown: i32,
    pub buffer_margin: i32,

    pub dirfilter: i32,
    pub show_filename_ext: i32,
    pub default_codepage: i32,
    pub hold_lr_for_scroll_in_list: bool,
    pub play_selected: bool,
    pub single_mode: i32,
    pub party_mode: bool,
    pub cuesheet: bool,
    pub car_adapter_mode: bool,
    pub car_adapter_mode_delay: i32,
    pub start_in_screen: i32,
    pub ff_rewind_min_step: i32,
    pub ff_rewind_accel: i32,

    pub peak_meter_release: i32,
    pub peak_meter_hold: i32,
    pub peak_meter_clip_hold: i32,
    pub peak_meter_dbfs: bool,
    pub peak_meter_min: i32,
    pub peak_meter_max: i32,

    pub wps_file: String,
    pub sbs_file: String,
    pub lang_file: String,
    pub playlist_catalog_dir: String,
    pub skip_length: i32,
    pub max_files_in_dir: i32,
    pub max_files_in_playlist: i32,
    pub volume_type: i32,
    pub battery_display: i32,
    pub show_icons: bool,
    pub statusbar: i32,

    pub scrollbar: i32,
    pub scrollbar_width: i32,

    pub list_line_padding: i32,
    pub list_separator_height: i32,
    pub list_separator_color: i32,

    pub browse_current: bool,
    pub scroll_paginated: bool,
    pub list_wraparound: bool,
    pub list_order: i32,
    pub scroll_speed: i32,
    pub bidir_limit: i32,
    pub scroll_delay: i32,
    pub scroll_step: i32,

    pub autoloadbookmark: i32,
    pub autocreatebookmark: i32,
    pub autoupdatebookmark: bool,
    pub usemrb: i32,

    pub dircache: bool,
    pub tagcache_ram: i32,
    pub tagcache_autoupdate: bool,
    pub autoresume_enable: bool,
    pub autoresume_automatic: i32,
    pub autoresume_paths: String,
    pub runtimedb: bool,
    pub tagcache_scan_paths: String,
    pub tagcache_db_path: String,
    pub backdrop_file: String,

    pub bg_color: i32,
    pub fg_color: i32,
    pub lss_color: i32,
    pub lse_color: i32,
    pub lst_color: i32,
    pub colors_file: String,

    pub browser_default: i32,

    pub repeat_mode: i32,
    pub next_folder: i32,
    pub constrain_next_folder: bool,
    pub recursive_dir_insert: i32,
    pub fade_on_stop: bool,
    pub playlist_shuffle: bool,
    pub warnon_erase_dynplaylist: bool,
    pub keep_current_track_on_replace_playlist: bool,
    pub show_shuffled_adding_options: bool,
    pub show_queue_options: i32,
    pub album_art: i32,
    pub rewind_across_tracks: bool,

    pub playlist_viewer_icons: bool,
    pub playlist_viewer_indices: bool,
    pub playlist_viewer_track_display: i32,

    pub talk_menu: bool,
    pub talk_dir: i32,
    pub talk_dir_clip: bool,
    pub talk_file: i32,
    pub talk_file_clip: bool,
    pub talk_filetype: bool,
    pub talk_battery_level: bool,
    pub talk_mixer_amp: i32,

    pub sort_case: bool,
    pub sort_dir: i32,
    pub sort_file: i32,
    pub interpret_numbers: i32,

    pub poweroff: i32,
    pub battery_capacity: i32,
    pub battery_type: i32,
    pub spdif_enable: bool,
    pub usb_charging: i32,

    pub contrast: i32,
    pub invert: bool,
    pub flip_display: bool,
    pub cursor_style: i32,
    pub screen_scroll_step: i32,
    pub show_path_in_browser: i32,
    pub offset_out_of_view: bool,
    pub disable_mainmenu_scrolling: bool,
    pub icon_file: String,
    pub viewers_icon_file: String,
    pub font_file: String,
    pub glyphs_to_cache: i32,
    pub kbd_file: String,
    pub backlight_timeout: i32,
    pub caption_backlight: bool,
    pub bl_filter_first_keypress: bool,
    pub backlight_timeout_plugged: i32,
    pub bt_selective_softlock_actions: bool,
    pub bt_selective_softlock_actions_mask: i32,
    pub bl_selective_actions: bool,
    pub bl_selective_actions_mask: i32,
    pub backlight_on_button_hold: i32,
    pub lcd_sleep_after_backlight_off: i32,
    pub brightness: i32,

    pub speaker_mode: i32,
    pub prevent_skip: bool,

    pub touch_mode: i32,
    pub ts_calibration_data: TouchscreenParameter,

    pub pitch_mode_semitone: bool,
    pub pitch_mode_timestretch: bool,

    pub usb_hid: bool,
    pub usb_keypad_mode: i32,

    pub usb_skip_first_drive: bool,

    pub player_name: String,

    pub compressor_settings: CompressorSettings,

    pub sleeptimer_duration: i32,
    pub sleeptimer_on_startup: bool,
    pub keypress_restarts_sleeptimer: bool,

    pub show_shutdown_message: bool,

    pub hotkey_wps: i32,
    pub hotkey_tree: i32,

    pub resume_rewind: i32,

    pub depth_3d: i32,

    pub roll_off: i32,

    pub power_mode: i32,

    pub keyclick_hardware: bool,

    pub start_directory: String,
    pub root_menu_customized: bool,
    pub shortcuts_replaces_qs: bool,

    pub play_frequency: i32,
    pub volume_limit: i32,

    pub volume_adjust_mode: i32,
    pub volume_adjust_norm_steps: i32,

    pub surround_enabled: i32,
    pub surround_balance: i32,
    pub surround_fx1: i32,
    pub surround_fx2: i32,
    pub surround_method2: bool,
    pub surround_mix: i32,

    pub pbe: i32,
    pub pbe_precut: i32,

    pub afr_enabled: i32,

    pub governor: i32,
    pub stereosw_mode: i32,
}

impl From<crate::UserSettings> for UserSettings {
    fn from(settings: crate::UserSettings) -> Self {
        let home = std::env::var("HOME").unwrap();
        Self {
            music_dir: std::env::var("ROCKBOX_LIBRARY")
                .unwrap_or_else(|_| format!("{}/Music", home)),
            volume: settings.volume,
            balance: settings.balance,
            bass: settings.bass,
            treble: settings.treble,
            channel_config: settings.channel_config,
            stereo_width: settings.stereo_width,
            bass_cutoff: settings.bass_cutoff,
            treble_cutoff: settings.treble_cutoff,
            crossfade: settings.crossfade,
            crossfade_fade_in_delay: settings.crossfade_fade_in_delay,
            crossfade_fade_out_delay: settings.crossfade_fade_out_delay,
            crossfade_fade_in_duration: settings.crossfade_fade_in_duration,
            crossfade_fade_out_duration: settings.crossfade_fade_out_duration,
            crossfade_fade_out_mixmode: settings.crossfade_fade_out_mixmode,
            replaygain_settings: ReplaygainSettings::from(settings.replaygain_settings),
            crossfeed: settings.crossfeed,
            crossfeed_direct_gain: settings.crossfeed_direct_gain as u32,
            crossfeed_cross_gain: settings.crossfeed_cross_gain as u32,
            crossfeed_hf_attenuation: settings.crossfeed_hf_attenuation as u32,
            crossfeed_hf_cutoff: settings.crossfeed_hf_cutoff as u32,
            eq_enabled: settings.eq_enabled,
            eq_precut: settings.eq_precut as u32,
            eq_band_settings: settings
                .eq_band_settings
                .into_iter()
                .map(EqBandSetting::from)
                .collect(),
            beep: settings.beep,
            keyclick: settings.keyclick,
            keyclick_repeats: settings.keyclick_repeats,
            dithering_enabled: settings.dithering_enabled,
            timestretch_enabled: settings.timestretch_enabled,
            list_accel_start_delay: settings.list_accel_start_delay,
            list_accel_wait: settings.list_accel_wait,
            touchpad_sensitivity: settings.touchpad_sensitivity,
            touchpad_deadzone: settings.touchpad_deadzone,
            pause_rewind: settings.pause_rewind,
            unplug_mode: settings.unplug_mode,
            unplug_autoresume: settings.unplug_autoresume,
            timeformat: settings.timeformat,
            disk_spindown: settings.disk_spindown,
            buffer_margin: settings.buffer_margin,
            dirfilter: settings.dirfilter,
            show_filename_ext: settings.show_filename_ext,
            default_codepage: settings.default_codepage,
            hold_lr_for_scroll_in_list: settings.hold_lr_for_scroll_in_list,
            play_selected: settings.play_selected,
            single_mode: settings.single_mode,
            party_mode: settings.party_mode,
            cuesheet: settings.cuesheet,
            car_adapter_mode: settings.car_adapter_mode,
            car_adapter_mode_delay: settings.car_adapter_mode_delay,
            start_in_screen: settings.start_in_screen,
            ff_rewind_min_step: settings.ff_rewind_min_step,
            ff_rewind_accel: settings.ff_rewind_accel,
            peak_meter_release: settings.peak_meter_release,
            peak_meter_hold: settings.peak_meter_hold,
            peak_meter_clip_hold: settings.peak_meter_clip_hold,
            peak_meter_dbfs: settings.peak_meter_dbfs,
            peak_meter_min: settings.peak_meter_min,
            peak_meter_max: settings.peak_meter_max,
            wps_file: unsafe {
                CStr::from_ptr(cast_ptr!(settings.wps_file.as_ptr()))
                    .to_string_lossy()
                    .into_owned()
            },
            sbs_file: unsafe {
                CStr::from_ptr(cast_ptr!(settings.sbs_file.as_ptr()))
                    .to_string_lossy()
                    .into_owned()
            },
            lang_file: unsafe {
                CStr::from_ptr(cast_ptr!(settings.lang_file.as_ptr()))
                    .to_string_lossy()
                    .into_owned()
            },
            playlist_catalog_dir: unsafe {
                CStr::from_ptr(cast_ptr!(settings.playlist_catalog_dir.as_ptr()))
                    .to_string_lossy()
                    .into_owned()
            },
            skip_length: settings.skip_length,
            max_files_in_dir: settings.max_files_in_dir,
            max_files_in_playlist: settings.max_files_in_playlist,
            volume_type: settings.volume_type,
            battery_display: settings.battery_display,
            show_icons: settings.show_icons,
            statusbar: settings.statusbar,
            scrollbar: settings.scrollbar,
            scrollbar_width: settings.scrollbar_width,
            list_line_padding: settings.list_line_padding,
            list_separator_height: settings.list_separator_height,
            list_separator_color: settings.list_separator_color,
            browse_current: settings.browse_current,
            scroll_paginated: settings.scroll_paginated,
            list_wraparound: settings.list_wraparound,
            list_order: settings.list_order,
            scroll_speed: settings.scroll_speed,
            bidir_limit: settings.bidir_limit,
            scroll_delay: settings.scroll_delay,
            scroll_step: settings.scroll_step,
            autoloadbookmark: settings.autoloadbookmark,
            autocreatebookmark: settings.autocreatebookmark,
            autoupdatebookmark: settings.autoupdatebookmark,
            usemrb: settings.usemrb,
            dircache: settings.dircache,
            tagcache_ram: settings.tagcache_ram,
            tagcache_autoupdate: settings.tagcache_autoupdate,
            autoresume_enable: settings.autoresume_enable,
            autoresume_automatic: settings.autoresume_automatic,
            autoresume_paths: unsafe {
                CStr::from_ptr(cast_ptr!(settings.autoresume_paths.as_ptr()))
                    .to_string_lossy()
                    .into_owned()
            },
            runtimedb: settings.runtimedb,
            tagcache_scan_paths: unsafe {
                CStr::from_ptr(cast_ptr!(settings.tagcache_scan_paths.as_ptr()))
                    .to_string_lossy()
                    .into_owned()
            },
            tagcache_db_path: unsafe {
                CStr::from_ptr(cast_ptr!(settings.tagcache_db_path.as_ptr()))
                    .to_string_lossy()
                    .into_owned()
            },
            backdrop_file: unsafe {
                CStr::from_ptr(cast_ptr!(settings.backdrop_file.as_ptr()))
                    .to_string_lossy()
                    .into_owned()
            },
            bg_color: settings.bg_color,
            fg_color: settings.fg_color,
            lss_color: settings.lss_color,
            lse_color: settings.lse_color,
            lst_color: settings.lst_color,
            colors_file: unsafe {
                CStr::from_ptr(cast_ptr!(settings.colors_file.as_ptr()))
                    .to_string_lossy()
                    .into_owned()
            },
            browser_default: settings.browser_default,
            repeat_mode: settings.repeat_mode,
            next_folder: settings.next_folder,
            constrain_next_folder: settings.constrain_next_folder,
            recursive_dir_insert: settings.recursive_dir_insert,
            fade_on_stop: settings.fade_on_stop,
            playlist_shuffle: settings.playlist_shuffle,
            warnon_erase_dynplaylist: settings.warnon_erase_dynplaylist,
            keep_current_track_on_replace_playlist: settings.keep_current_track_on_replace_playlist,
            show_shuffled_adding_options: settings.show_shuffled_adding_options,
            show_queue_options: settings.show_queue_options,
            album_art: settings.album_art,
            rewind_across_tracks: settings.rewind_across_tracks,
            playlist_viewer_icons: settings.playlist_viewer_icons,
            playlist_viewer_indices: settings.playlist_viewer_indices,
            playlist_viewer_track_display: settings.playlist_viewer_track_display,
            talk_menu: settings.talk_menu,
            talk_dir: settings.talk_dir,
            talk_dir_clip: settings.talk_dir_clip,
            talk_file: settings.talk_file,
            talk_file_clip: settings.talk_file_clip,
            talk_filetype: settings.talk_filetype,
            talk_battery_level: settings.talk_battery_level,
            talk_mixer_amp: settings.talk_mixer_amp,
            sort_case: settings.sort_case,
            sort_dir: settings.sort_dir,
            sort_file: settings.sort_file,
            interpret_numbers: settings.interpret_numbers,
            poweroff: settings.poweroff,
            battery_capacity: settings.battery_capacity,
            battery_type: settings.battery_type,
            spdif_enable: settings.spdif_enable,
            usb_charging: settings.usb_charging,
            contrast: settings.contrast,
            invert: settings.invert,
            flip_display: settings.flip_display,
            cursor_style: settings.cursor_style,
            screen_scroll_step: settings.screen_scroll_step,
            show_path_in_browser: settings.show_path_in_browser,
            offset_out_of_view: settings.offset_out_of_view,
            disable_mainmenu_scrolling: settings.disable_mainmenu_scrolling,
            icon_file: unsafe {
                CStr::from_ptr(cast_ptr!(settings.icon_file.as_ptr()))
                    .to_string_lossy()
                    .into_owned()
            },
            viewers_icon_file: unsafe {
                CStr::from_ptr(cast_ptr!(settings.viewers_icon_file.as_ptr()))
                    .to_string_lossy()
                    .into_owned()
            },
            font_file: unsafe {
                CStr::from_ptr(cast_ptr!(settings.font_file.as_ptr()))
                    .to_string_lossy()
                    .into_owned()
            },
            glyphs_to_cache: settings.glyphs_to_cache,
            kbd_file: unsafe {
                CStr::from_ptr(cast_ptr!(settings.kbd_file.as_ptr()))
                    .to_string_lossy()
                    .into_owned()
            },
            backlight_timeout: settings.backlight_timeout,
            caption_backlight: settings.caption_backlight,
            bl_filter_first_keypress: settings.bl_filter_first_keypress,
            backlight_timeout_plugged: settings.backlight_timeout_plugged,
            bt_selective_softlock_actions: settings.bt_selective_softlock_actions,
            bt_selective_softlock_actions_mask: settings.bt_selective_softlock_actions_mask,
            bl_selective_actions: settings.bl_selective_actions,
            bl_selective_actions_mask: settings.bl_selective_actions_mask,
            backlight_on_button_hold: settings.backlight_on_button_hold,
            lcd_sleep_after_backlight_off: settings.lcd_sleep_after_backlight_off,
            brightness: settings.brightness,
            speaker_mode: settings.speaker_mode,
            prevent_skip: settings.prevent_skip,
            touch_mode: settings.touch_mode,
            ts_calibration_data: TouchscreenParameter::from(settings.ts_calibration_data),
            pitch_mode_semitone: settings.pitch_mode_semitone,
            pitch_mode_timestretch: settings.pitch_mode_timestretch,
            usb_hid: settings.usb_hid,
            usb_keypad_mode: settings.usb_keypad_mode,
            usb_skip_first_drive: settings.usb_skip_first_drive,
            player_name: unsafe {
                CStr::from_ptr(cast_ptr!(settings.player_name.as_ptr()))
                    .to_string_lossy()
                    .into_owned()
            },
            compressor_settings: CompressorSettings::from(settings.compressor_settings),
            sleeptimer_duration: settings.sleeptimer_duration,
            sleeptimer_on_startup: settings.sleeptimer_on_startup,
            keypress_restarts_sleeptimer: settings.keypress_restarts_sleeptimer,
            show_shutdown_message: settings.show_shutdown_message,
            hotkey_wps: settings.hotkey_wps,
            hotkey_tree: settings.hotkey_tree,
            resume_rewind: settings.resume_rewind,
            depth_3d: settings.depth_3d,
            roll_off: settings.roll_off,
            power_mode: settings.power_mode,
            keyclick_hardware: settings.keyclick_hardware,
            start_directory: unsafe {
                CStr::from_ptr(cast_ptr!(settings.start_directory.as_ptr()))
                    .to_string_lossy()
                    .into_owned()
            },
            root_menu_customized: settings.root_menu_customized,
            shortcuts_replaces_qs: settings.shortcuts_replaces_qs,
            play_frequency: settings.play_frequency,
            volume_limit: settings.volume_limit,
            volume_adjust_mode: settings.volume_adjust_mode,
            volume_adjust_norm_steps: settings.volume_adjust_norm_steps,
            surround_enabled: settings.surround_enabled,
            surround_balance: settings.surround_balance,
            surround_fx1: settings.surround_fx1,
            surround_fx2: settings.surround_fx2,
            surround_method2: settings.surround_method2,
            surround_mix: settings.surround_mix,
            pbe: settings.pbe,
            pbe_precut: settings.pbe_precut,
            afr_enabled: settings.afr_enabled,
            governor: settings.governor,
            stereosw_mode: settings.stereosw_mode,
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct NewGlobalSettings {
    pub music_dir: Option<String>,
    pub playlist_shuffle: Option<bool>,
    pub repeat_mode: Option<i32>,
    pub bass: Option<i32>,
    pub treble: Option<i32>,
    pub bass_cutoff: Option<i32>,
    pub treble_cutoff: Option<i32>,
    pub crossfade: Option<i32>,
    pub fade_on_stop: Option<bool>,
    pub fade_in_delay: Option<i32>,
    pub fade_in_duration: Option<i32>,
    pub fade_out_delay: Option<i32>,
    pub fade_out_duration: Option<i32>,
    pub fade_out_mixmode: Option<i32>,
    pub balance: Option<i32>,
    pub stereo_width: Option<i32>,
    pub stereosw_mode: Option<i32>,
    pub surround_enabled: Option<i32>,
    pub surround_balance: Option<i32>,
    pub surround_fx1: Option<i32>,
    pub surround_fx2: Option<i32>,
    pub party_mode: Option<bool>,
    pub channel_config: Option<i32>,
    pub player_name: Option<String>,
    pub eq_enabled: Option<bool>,
    pub eq_band_settings: Option<Vec<EqBandSetting>>,
    pub replaygain_settings: Option<ReplaygainSettings>,
}

impl From<UserSettings> for NewGlobalSettings {
    fn from(settings: UserSettings) -> Self {
        Self {
            music_dir: None,
            playlist_shuffle: Some(settings.playlist_shuffle),
            repeat_mode: Some(settings.repeat_mode),
            bass: Some(settings.bass),
            treble: Some(settings.treble),
            bass_cutoff: Some(settings.bass_cutoff),
            treble_cutoff: Some(settings.treble_cutoff),
            crossfade: Some(settings.crossfade),
            fade_on_stop: Some(settings.fade_on_stop),
            fade_in_delay: Some(settings.crossfade_fade_in_delay),
            fade_in_duration: Some(settings.crossfade_fade_in_duration),
            fade_out_delay: Some(settings.crossfade_fade_out_delay),
            fade_out_duration: Some(settings.crossfade_fade_out_duration),
            fade_out_mixmode: Some(settings.crossfade_fade_out_mixmode),
            balance: Some(settings.balance),
            stereo_width: Some(settings.stereo_width),
            stereosw_mode: Some(settings.stereosw_mode),
            surround_enabled: Some(settings.surround_enabled),
            surround_balance: Some(settings.surround_balance),
            surround_fx1: Some(settings.surround_fx1),
            surround_fx2: Some(settings.surround_fx2),
            party_mode: Some(settings.party_mode),
            channel_config: Some(settings.channel_config),
            player_name: Some(settings.player_name),
            eq_enabled: Some(settings.eq_enabled),
            eq_band_settings: Some(settings.eq_band_settings),
            replaygain_settings: Some(settings.replaygain_settings),
        }
    }
}
