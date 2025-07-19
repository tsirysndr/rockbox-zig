use async_graphql::*;
use rockbox_sys as rb;
use serde::{Deserialize, Serialize};

use super::{
    compressor_settings::CompressorSettings, eq_band_setting::EqBandSetting,
    replaygain_settings::ReplaygainSettings,
};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub music_dir: String,
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

#[Object]
impl UserSettings {
    async fn music_dir(&self) -> &str {
        &self.music_dir
    }

    async fn volume(&self) -> i32 {
        self.volume
    }

    async fn balance(&self) -> i32 {
        self.balance
    }

    async fn bass(&self) -> i32 {
        self.bass
    }

    async fn treble(&self) -> i32 {
        self.treble
    }

    async fn channel_config(&self) -> i32 {
        self.channel_config
    }

    async fn stereo_width(&self) -> i32 {
        self.stereo_width
    }

    async fn bass_cutoff(&self) -> i32 {
        self.bass_cutoff
    }

    async fn treble_cutoff(&self) -> i32 {
        self.treble_cutoff
    }

    async fn crossfade(&self) -> i32 {
        self.crossfade
    }

    async fn crossfade_fade_in_delay(&self) -> i32 {
        self.crossfade_fade_in_delay
    }

    async fn crossfade_fade_out_delay(&self) -> i32 {
        self.crossfade_fade_out_delay
    }

    async fn crossfade_fade_in_duration(&self) -> i32 {
        self.crossfade_fade_in_duration
    }

    async fn crossfade_fade_out_duration(&self) -> i32 {
        self.crossfade_fade_out_duration
    }

    async fn crossfade_fade_out_mixmode(&self) -> i32 {
        self.crossfade_fade_out_mixmode
    }

    async fn replaygain_settings(&self) -> ReplaygainSettings {
        self.replaygain_settings.clone()
    }

    async fn crossfeed(&self) -> i32 {
        self.crossfeed
    }

    async fn crossfeed_direct_gain(&self) -> u32 {
        self.crossfeed_direct_gain
    }

    async fn crossfeed_cross_gain(&self) -> u32 {
        self.crossfeed_cross_gain
    }

    async fn crossfeed_hf_attenuation(&self) -> u32 {
        self.crossfeed_hf_attenuation
    }

    async fn crossfeed_hf_cutoff(&self) -> u32 {
        self.crossfeed_hf_cutoff
    }

    async fn eq_enabled(&self) -> bool {
        self.eq_enabled
    }

    async fn eq_precut(&self) -> u32 {
        self.eq_precut
    }

    async fn eq_band_settings(&self) -> Vec<EqBandSetting> {
        self.eq_band_settings.clone()
    }

    async fn beep(&self) -> i32 {
        self.beep
    }

    async fn keyclick(&self) -> i32 {
        self.keyclick
    }

    async fn keyclick_repeats(&self) -> i32 {
        self.keyclick_repeats
    }

    async fn dithering_enabled(&self) -> bool {
        self.dithering_enabled
    }

    async fn timestretch_enabled(&self) -> bool {
        self.timestretch_enabled
    }

    async fn list_accel_start_delay(&self) -> i32 {
        self.list_accel_start_delay
    }

    async fn list_accel_wait(&self) -> i32 {
        self.list_accel_wait
    }

    async fn touchpad_sensitivity(&self) -> i32 {
        self.touchpad_sensitivity
    }

    async fn touchpad_deadzone(&self) -> i32 {
        self.touchpad_deadzone
    }

    async fn pause_rewind(&self) -> i32 {
        self.pause_rewind
    }

    async fn unplug_mode(&self) -> i32 {
        self.unplug_mode
    }

    async fn unplug_autoresume(&self) -> bool {
        self.unplug_autoresume
    }

    async fn timeformat(&self) -> i32 {
        self.timeformat
    }

    async fn disk_spindown(&self) -> i32 {
        self.disk_spindown
    }

    async fn buffer_margin(&self) -> i32 {
        self.buffer_margin
    }

    async fn dirfilter(&self) -> i32 {
        self.dirfilter
    }

    async fn show_filename_ext(&self) -> i32 {
        self.show_filename_ext
    }

    async fn default_codepage(&self) -> i32 {
        self.default_codepage
    }

    async fn hold_lr_for_scroll_in_list(&self) -> bool {
        self.hold_lr_for_scroll_in_list
    }

    async fn play_selected(&self) -> bool {
        self.play_selected
    }

    async fn single_mode(&self) -> i32 {
        self.single_mode
    }

    async fn party_mode(&self) -> bool {
        self.party_mode
    }

    async fn cuesheet(&self) -> bool {
        self.cuesheet
    }

    async fn car_adapter_mode(&self) -> bool {
        self.car_adapter_mode
    }

    async fn car_adapter_mode_delay(&self) -> i32 {
        self.car_adapter_mode_delay
    }

    async fn start_in_screen(&self) -> i32 {
        self.start_in_screen
    }

    async fn ff_rewind_min_step(&self) -> i32 {
        self.ff_rewind_min_step
    }

    async fn ff_rewind_accel(&self) -> i32 {
        self.ff_rewind_accel
    }

    async fn peak_meter_release(&self) -> i32 {
        self.peak_meter_release
    }

    async fn peak_meter_hold(&self) -> i32 {
        self.peak_meter_hold
    }

    async fn peak_meter_clip_hold(&self) -> i32 {
        self.peak_meter_clip_hold
    }

    async fn peak_meter_dbfs(&self) -> bool {
        self.peak_meter_dbfs
    }

    async fn peak_meter_min(&self) -> i32 {
        self.peak_meter_min
    }

    async fn peak_meter_max(&self) -> i32 {
        self.peak_meter_max
    }

    async fn wps_file(&self) -> &str {
        &self.wps_file
    }

    async fn sbs_file(&self) -> &str {
        &self.sbs_file
    }

    async fn lang_file(&self) -> &str {
        &self.lang_file
    }

    async fn playlist_catalog_dir(&self) -> &str {
        &self.playlist_catalog_dir
    }

    async fn skip_length(&self) -> i32 {
        self.skip_length
    }

    async fn max_files_in_dir(&self) -> i32 {
        self.max_files_in_dir
    }

    async fn max_files_in_playlist(&self) -> i32 {
        self.max_files_in_playlist
    }

    async fn volume_type(&self) -> i32 {
        self.volume_type
    }

    async fn battery_display(&self) -> i32 {
        self.battery_display
    }

    async fn show_icons(&self) -> bool {
        self.show_icons
    }

    async fn statusbar(&self) -> i32 {
        self.statusbar
    }

    async fn scrollbar(&self) -> i32 {
        self.scrollbar
    }

    async fn scrollbar_width(&self) -> i32 {
        self.scrollbar_width
    }

    async fn list_line_padding(&self) -> i32 {
        self.list_line_padding
    }

    async fn list_separator_height(&self) -> i32 {
        self.list_separator_height
    }

    async fn list_separator_color(&self) -> i32 {
        self.list_separator_color
    }

    async fn browse_current(&self) -> bool {
        self.browse_current
    }

    async fn scroll_paginated(&self) -> bool {
        self.scroll_paginated
    }

    async fn list_wraparound(&self) -> bool {
        self.list_wraparound
    }

    async fn list_order(&self) -> i32 {
        self.list_order
    }

    async fn scroll_speed(&self) -> i32 {
        self.scroll_speed
    }

    async fn bidir_limit(&self) -> i32 {
        self.bidir_limit
    }

    async fn scroll_delay(&self) -> i32 {
        self.scroll_delay
    }

    async fn scroll_step(&self) -> i32 {
        self.scroll_step
    }

    async fn autoloadbookmark(&self) -> i32 {
        self.autoloadbookmark
    }

    async fn autocreatebookmark(&self) -> i32 {
        self.autocreatebookmark
    }

    async fn autoupdatebookmark(&self) -> bool {
        self.autoupdatebookmark
    }

    async fn usemrb(&self) -> i32 {
        self.usemrb
    }

    async fn dircache(&self) -> bool {
        self.dircache
    }

    async fn tagcache_ram(&self) -> i32 {
        self.tagcache_ram
    }

    async fn tagcache_autoupdate(&self) -> bool {
        self.tagcache_autoupdate
    }

    async fn autoresume_enable(&self) -> bool {
        self.autoresume_enable
    }

    async fn autoresume_automatic(&self) -> i32 {
        self.autoresume_automatic
    }

    async fn autoresume_paths(&self) -> &str {
        &self.autoresume_paths
    }

    async fn runtimedb(&self) -> bool {
        self.runtimedb
    }

    async fn tagcache_scan_paths(&self) -> &str {
        &self.tagcache_scan_paths
    }

    async fn tagcache_db_path(&self) -> &str {
        &self.tagcache_db_path
    }

    async fn backdrop_file(&self) -> &str {
        &self.backdrop_file
    }

    async fn bg_color(&self) -> i32 {
        self.bg_color
    }

    async fn fg_color(&self) -> i32 {
        self.fg_color
    }

    async fn lss_color(&self) -> i32 {
        self.lss_color
    }

    async fn lse_color(&self) -> i32 {
        self.lse_color
    }

    async fn lst_color(&self) -> i32 {
        self.lst_color
    }

    async fn colors_file(&self) -> &str {
        &self.colors_file
    }

    async fn browser_default(&self) -> i32 {
        self.browser_default
    }

    async fn repeat_mode(&self) -> i32 {
        self.repeat_mode
    }

    async fn next_folder(&self) -> i32 {
        self.next_folder
    }

    async fn constrain_next_folder(&self) -> bool {
        self.constrain_next_folder
    }

    async fn recursive_dir_insert(&self) -> i32 {
        self.recursive_dir_insert
    }

    async fn fade_on_stop(&self) -> bool {
        self.fade_on_stop
    }

    async fn playlist_shuffle(&self) -> bool {
        self.playlist_shuffle
    }

    async fn warnon_erase_dynplaylist(&self) -> bool {
        self.warnon_erase_dynplaylist
    }

    async fn keep_current_track_on_replace_playlist(&self) -> bool {
        self.keep_current_track_on_replace_playlist
    }

    async fn show_shuffled_adding_options(&self) -> bool {
        self.show_shuffled_adding_options
    }

    async fn show_queue_options(&self) -> i32 {
        self.show_queue_options
    }

    async fn album_art(&self) -> i32 {
        self.album_art
    }

    async fn rewind_across_tracks(&self) -> bool {
        self.rewind_across_tracks
    }

    async fn playlist_viewer_icons(&self) -> bool {
        self.playlist_viewer_icons
    }

    async fn playlist_viewer_indices(&self) -> bool {
        self.playlist_viewer_indices
    }

    async fn playlist_viewer_track_display(&self) -> i32 {
        self.playlist_viewer_track_display
    }

    async fn talk_menu(&self) -> bool {
        self.talk_menu
    }

    async fn talk_dir(&self) -> i32 {
        self.talk_dir
    }

    async fn talk_dir_clip(&self) -> bool {
        self.talk_dir_clip
    }

    async fn talk_file(&self) -> i32 {
        self.talk_file
    }

    async fn talk_file_clip(&self) -> bool {
        self.talk_file_clip
    }

    async fn talk_filetype(&self) -> bool {
        self.talk_filetype
    }

    async fn talk_battery_level(&self) -> bool {
        self.talk_battery_level
    }

    async fn talk_mixer_amp(&self) -> i32 {
        self.talk_mixer_amp
    }

    async fn sort_case(&self) -> bool {
        self.sort_case
    }

    async fn sort_dir(&self) -> i32 {
        self.sort_dir
    }

    async fn sort_file(&self) -> i32 {
        self.sort_file
    }

    async fn interpret_numbers(&self) -> i32 {
        self.interpret_numbers
    }

    async fn poweroff(&self) -> i32 {
        self.poweroff
    }

    async fn battery_capacity(&self) -> i32 {
        self.battery_capacity
    }

    async fn battery_type(&self) -> i32 {
        self.battery_type
    }

    async fn spdif_enable(&self) -> bool {
        self.spdif_enable
    }

    async fn usb_charging(&self) -> i32 {
        self.usb_charging
    }

    async fn contrast(&self) -> i32 {
        self.contrast
    }

    async fn invert(&self) -> bool {
        self.invert
    }

    async fn flip_display(&self) -> bool {
        self.flip_display
    }

    async fn cursor_style(&self) -> i32 {
        self.cursor_style
    }

    async fn screen_scroll_step(&self) -> i32 {
        self.screen_scroll_step
    }

    async fn show_path_in_browser(&self) -> i32 {
        self.show_path_in_browser
    }

    async fn offset_out_of_view(&self) -> bool {
        self.offset_out_of_view
    }

    async fn disable_mainmenu_scrolling(&self) -> bool {
        self.disable_mainmenu_scrolling
    }

    async fn icon_file(&self) -> &str {
        &self.icon_file
    }

    async fn viewers_icon_file(&self) -> &str {
        &self.viewers_icon_file
    }

    async fn font_file(&self) -> &str {
        &self.font_file
    }

    async fn glyphs_to_cache(&self) -> i32 {
        self.glyphs_to_cache
    }

    async fn kbd_file(&self) -> &str {
        &self.kbd_file
    }

    async fn backlight_timeout(&self) -> i32 {
        self.backlight_timeout
    }

    async fn caption_backlight(&self) -> bool {
        self.caption_backlight
    }

    async fn bl_filter_first_keypress(&self) -> bool {
        self.bl_filter_first_keypress
    }

    async fn backlight_timeout_plugged(&self) -> i32 {
        self.backlight_timeout_plugged
    }

    async fn bt_selective_softlock_actions(&self) -> bool {
        self.bt_selective_softlock_actions
    }

    async fn bt_selective_softlock_actions_mask(&self) -> i32 {
        self.bt_selective_softlock_actions_mask
    }

    async fn bl_selective_actions(&self) -> bool {
        self.bl_selective_actions
    }

    async fn bl_selective_actions_mask(&self) -> i32 {
        self.bl_selective_actions_mask
    }

    async fn backlight_on_button_hold(&self) -> i32 {
        self.backlight_on_button_hold
    }

    async fn lcd_sleep_after_backlight_off(&self) -> i32 {
        self.lcd_sleep_after_backlight_off
    }

    async fn brightness(&self) -> i32 {
        self.brightness
    }

    async fn speaker_mode(&self) -> i32 {
        self.speaker_mode
    }

    async fn prevent_skip(&self) -> bool {
        self.prevent_skip
    }

    async fn touch_mode(&self) -> i32 {
        self.touch_mode
    }

    async fn pitch_mode_semitone(&self) -> bool {
        self.pitch_mode_semitone
    }

    async fn pitch_mode_timestretch(&self) -> bool {
        self.pitch_mode_timestretch
    }

    async fn usb_hid(&self) -> bool {
        self.usb_hid
    }

    async fn usb_keypad_mode(&self) -> i32 {
        self.usb_keypad_mode
    }

    async fn usb_skip_first_drive(&self) -> bool {
        self.usb_skip_first_drive
    }

    async fn player_name(&self) -> &str {
        &self.player_name
    }

    async fn compressor_settings(&self) -> CompressorSettings {
        self.compressor_settings.clone()
    }

    async fn sleeptimer_duration(&self) -> i32 {
        self.sleeptimer_duration
    }

    async fn sleeptimer_on_startup(&self) -> bool {
        self.sleeptimer_on_startup
    }

    async fn keypress_restarts_sleeptimer(&self) -> bool {
        self.keypress_restarts_sleeptimer
    }

    async fn show_shutdown_message(&self) -> bool {
        self.show_shutdown_message
    }

    async fn hotkey_wps(&self) -> i32 {
        self.hotkey_wps
    }

    async fn hotkey_tree(&self) -> i32 {
        self.hotkey_tree
    }

    async fn resume_rewind(&self) -> i32 {
        self.resume_rewind
    }

    async fn depth_3d(&self) -> i32 {
        self.depth_3d
    }

    async fn roll_off(&self) -> i32 {
        self.roll_off
    }

    async fn power_mode(&self) -> i32 {
        self.power_mode
    }

    async fn keyclick_hardware(&self) -> bool {
        self.keyclick_hardware
    }

    async fn start_directory(&self) -> &str {
        &self.start_directory
    }

    async fn root_menu_customized(&self) -> bool {
        self.root_menu_customized
    }

    async fn shortcuts_replaces_qs(&self) -> bool {
        self.shortcuts_replaces_qs
    }

    async fn play_frequency(&self) -> i32 {
        self.play_frequency
    }

    async fn volume_limit(&self) -> i32 {
        self.volume_limit
    }

    async fn volume_adjust_mode(&self) -> i32 {
        self.volume_adjust_mode
    }

    async fn volume_adjust_norm_steps(&self) -> i32 {
        self.volume_adjust_norm_steps
    }

    async fn surround_enabled(&self) -> i32 {
        self.surround_enabled
    }

    async fn surround_balance(&self) -> i32 {
        self.surround_balance
    }

    async fn surround_fx1(&self) -> i32 {
        self.surround_fx1
    }

    async fn surround_fx2(&self) -> i32 {
        self.surround_fx2
    }

    async fn surround_method2(&self) -> bool {
        self.surround_method2
    }

    async fn surround_mix(&self) -> i32 {
        self.surround_mix
    }

    async fn pbe(&self) -> i32 {
        self.pbe
    }

    async fn pbe_precut(&self) -> i32 {
        self.pbe_precut
    }

    async fn afr_enabled(&self) -> i32 {
        self.afr_enabled
    }

    async fn governor(&self) -> i32 {
        self.governor
    }

    async fn stereosw_mode(&self) -> i32 {
        self.stereosw_mode
    }
}

impl From<rb::types::user_settings::UserSettings> for UserSettings {
    fn from(settings: rb::types::user_settings::UserSettings) -> Self {
        Self {
            music_dir: settings.music_dir,
            volume: 0,
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
            crossfeed_direct_gain: settings.crossfeed_direct_gain,
            crossfeed_cross_gain: settings.crossfeed_cross_gain,
            crossfeed_hf_attenuation: settings.crossfeed_hf_attenuation,
            crossfeed_hf_cutoff: settings.crossfeed_hf_cutoff,
            eq_enabled: settings.eq_enabled,
            eq_precut: settings.eq_precut,
            eq_band_settings: settings
                .eq_band_settings
                .into_iter()
                .map(|band| band.into())
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
            wps_file: settings.wps_file,
            sbs_file: settings.sbs_file,
            lang_file: settings.lang_file,
            playlist_catalog_dir: settings.playlist_catalog_dir,
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
            autoresume_paths: settings.autoresume_paths,
            runtimedb: settings.runtimedb,
            tagcache_scan_paths: settings.tagcache_scan_paths,
            tagcache_db_path: settings.tagcache_db_path,
            backdrop_file: settings.backdrop_file,
            bg_color: settings.bg_color,
            fg_color: settings.fg_color,
            lss_color: settings.lss_color,
            lse_color: settings.lse_color,
            lst_color: settings.lst_color,
            colors_file: settings.colors_file,
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
            icon_file: settings.icon_file,
            viewers_icon_file: settings.viewers_icon_file,
            font_file: settings.font_file,
            glyphs_to_cache: settings.glyphs_to_cache,
            kbd_file: settings.kbd_file,
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
            pitch_mode_semitone: settings.pitch_mode_semitone,
            pitch_mode_timestretch: settings.pitch_mode_timestretch,
            usb_hid: settings.usb_hid,
            usb_keypad_mode: settings.usb_keypad_mode,
            usb_skip_first_drive: settings.usb_skip_first_drive,
            player_name: settings.player_name,
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
            start_directory: settings.start_directory,
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
