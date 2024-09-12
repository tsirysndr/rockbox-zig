use async_graphql::*;
use serde::Serialize;

use super::{
    compressor_settings::CompressorSettings, eq_band_setting::EqBandSetting,
    replaygain_settings::ReplaygainSettings, settings_list::SettingsList,
};

#[derive(Default, Clone, Serialize)]
pub struct UserSettings {
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

    pub qs_items: Vec<SettingsList>,

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

    pub pitch_mode_semitone: bool,
    pub pitch_mode_timestretch: bool,

    pub usb_hid: bool,
    pub usb_keypad_mode: i32,

    pub usb_skip_first_drive: bool,

    pub ui_vp_config: String,
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
    pub surround_fx2: bool,
    pub surround_method2: bool,
    pub surround_mix: i32,

    pub pbe: i32,
    pub pbe_precut: i32,

    pub afr_enabled: i32,

    pub governor: i32,
    pub stereosw_mode: i32,
}
