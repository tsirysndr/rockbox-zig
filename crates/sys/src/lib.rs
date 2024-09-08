use std::ffi::{c_char, c_int, c_long, c_uchar, c_uint, c_ulong, c_void};

pub mod browse;
pub mod dir;
pub mod file;
pub mod metadata;
pub mod playback;
pub mod playlist;
pub mod settings;
pub mod sound;
pub mod system;
pub mod tagcache;

const MAX_PATH: usize = 260;
const ID3V2_BUF_SIZE: usize = 1800;

#[repr(C)]
#[derive(Debug)]
pub struct Mp3Entry {
    pub path: [c_char; MAX_PATH],           // char path[MAX_PATH]
    pub title: *mut c_char,                 // char* title
    pub artist: *mut c_char,                // char* artist
    pub album: *mut c_char,                 // char* album
    pub genre_string: *mut c_char,          // char* genre_string
    pub disc_string: *mut c_char,           // char* disc_string
    pub track_string: *mut c_char,          // char* track_string
    pub year_string: *mut c_char,           // char* year_string
    pub composer: *mut c_char,              // char* composer
    pub comment: *mut c_char,               // char* comment
    pub albumartist: *mut c_char,           // char* albumartist
    pub grouping: *mut c_char,              // char* grouping
    pub discnum: c_int,                     // int discnum
    pub tracknum: c_int,                    // int tracknum
    pub layer: c_int,                       // int layer
    pub year: c_int,                        // int year
    pub id3version: c_uchar,                // unsigned char id3version
    pub codectype: c_uint,                  // unsigned int codectype
    pub bitrate: c_uint,                    // unsigned int bitrate
    pub frequency: c_ulong,                 // unsigned long frequency
    pub id3v2len: c_ulong,                  // unsigned long id3v2len
    pub id3v1len: c_ulong,                  // unsigned long id3v1len
    pub first_frame_offset: c_ulong,        // unsigned long first_frame_offset
    pub filesize: c_ulong,                  // unsigned long filesize
    pub length: c_ulong,                    // unsigned long length
    pub elapsed: c_ulong,                   // unsigned long elapsed
    pub lead_trim: c_int,                   // int lead_trim
    pub tail_trim: c_int,                   // int tail_trim
    pub samples: u64,                       // uint64_t samples
    pub frame_count: c_ulong,               // unsigned long frame_count
    pub bytesperframe: c_ulong,             // unsigned long bytesperframe
    pub vbr: bool,                          // bool vbr
    pub has_toc: bool,                      // bool has_toc
    pub toc: [c_uchar; 100],                // unsigned char toc[100]
    pub needs_upsampling_correction: bool,  // bool needs_upsampling_correction
    pub id3v2buf: [c_char; ID3V2_BUF_SIZE], // char id3v2buf[ID3V2_BUF_SIZE]
    pub id3v1buf: [[c_char; 92]; 4],        // char id3v1buf[4][92]
    pub offset: c_ulong,                    // unsigned long offset
    pub index: c_int,                       // int index
    pub skip_resume_adjustments: bool,      // bool skip_resume_adjustments
    pub autoresumable: c_uchar,             // unsigned char autoresumable
    pub tagcache_idx: c_long,               // long tagcache_idx
    pub rating: c_int,                      // int rating
    pub score: c_int,                       // int score
    pub playcount: c_long,                  // long playcount
    pub lastplayed: c_long,                 // long lastplayed
    pub playtime: c_long,                   // long playtime
    pub track_level: c_long,                // long track_level
    pub album_level: c_long,                // long album_level
    pub track_gain: c_long,                 // long track_gain
    pub album_gain: c_long,                 // long album_gain
    pub track_peak: c_long,                 // long track_peak
    pub album_peak: c_long,                 // long album_peak
    pub has_embedded_albumart: bool,        // bool has_embedded_albumart
    pub albumart: *mut c_void,              // struct mp3_albumart albumart
    pub has_embedded_cuesheet: bool,        // bool has_embedded_cuesheet
    pub embedded_cuesheet: *mut c_void,     // struct embedded_cuesheet embedded_cuesheet
    pub cuesheet: *mut c_void,              // struct cuesheet* cuesheet
    pub mb_track_id: *mut c_char,           // char* mb_track_id
    pub is_asf_stream: bool,                // bool is_asf_stream
}

const PLAYLIST_CONTROL_FILE: &str = "./config/rockbox.org/.playlist_control";
const MAX_DIR_LEVELS: usize = 10;

#[repr(C)]
#[derive(Debug)]
pub struct PlaylistInfo {
    pub utf8: bool,                   // bool utf8
    pub control_created: bool,        // bool control_created
    pub flags: c_uint,                // unsigned int flags
    pub fd: c_int,                    // int fd
    pub control_fd: c_int,            // int control_fd
    pub max_playlist_size: c_int,     // int max_playlist_size
    pub indices: *mut c_ulong,        // unsigned long* indices
    pub index: c_int,                 // int index
    pub first_index: c_int,           // int first_index
    pub amount: c_int,                // int amount
    pub last_insert_pos: c_int,       // int last_insert_pos
    pub started: bool,                // bool started
    pub last_shuffled_start: c_int,   // int last_shuffled_start
    pub seed: c_int,                  // int seed
    pub mutex: *mut c_void,           // struct mutex (convert to a void pointer for FFI)
    pub dirlen: c_int,                // int dirlen
    pub filename: [c_char; MAX_PATH], // char filename[MAX_PATH]
    pub control_filename:
        [c_char; std::mem::size_of::<[u8; PLAYLIST_CONTROL_FILE.len() + 100 + 8]>()], // char control_filename[sizeof(PLAYLIST_CONTROL_FILE) + 8]
    pub dcfrefs_handle: c_int, // int dcfrefs_handle
}

#[repr(C)]
#[derive(Debug)]
pub struct PlaylistTrackInfo {
    pub filename: [c_char; MAX_PATH], // char filename[MAX_PATH]
    pub attr: c_int,
    pub index: c_int,
    pub display_index: c_int,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ThemableIcons {
    NoIcon = -1,
    IconNoIcon,              // Icon_NOICON = NOICON
    IconAudio,               // Icon_Audio
    IconFolder,              // Icon_Folder
    IconPlaylist,            // Icon_Playlist
    IconCursor,              // Icon_Cursor
    IconWps,                 // Icon_Wps
    IconFirmware,            // Icon_Firmware
    IconFont,                // Icon_Font
    IconLanguage,            // Icon_Language
    IconConfig,              // Icon_Config
    IconPlugin,              // Icon_Plugin
    IconBookmark,            // Icon_Bookmark
    IconPreset,              // Icon_Preset
    IconQueued,              // Icon_Queued
    IconMoving,              // Icon_Moving
    IconKeyboard,            // Icon_Keyboard
    IconReverseCursor,       // Icon_Reverse_Cursor
    IconQuestionmark,        // Icon_Questionmark
    IconMenuSetting,         // Icon_Menu_setting
    IconMenuFunctioncall,    // Icon_Menu_functioncall
    IconSubmenu,             // Icon_Submenu
    IconSubmenuEntered,      // Icon_Submenu_Entered
    IconRecording,           // Icon_Recording
    IconVoice,               // Icon_Voice
    IconGeneralSettingsMenu, // Icon_General_settings_menu
    IconSystemMenu,          // Icon_System_menu
    IconPlaybackMenu,        // Icon_Playback_menu
    IconDisplayMenu,         // Icon_Display_menu
    IconRemoteDisplayMenu,   // Icon_Remote_Display_menu
    IconRadioScreen,         // Icon_Radio_screen
    IconFileViewMenu,        // Icon_file_view_menu
    IconEQ,                  // Icon_EQ
    IconRockbox,             // Icon_Rockbox
    IconLastThemeable,       // Icon_Last_Themeable
}

#[repr(C)]
#[derive(Debug)]
pub struct TreeCache {
    pub entries_handle: c_int,     // int entries_handle
    pub name_buffer_handle: c_int, // int name_buffer_handle
    pub max_entries: c_int,        // int max_entries
    pub name_buffer_size: c_int,   // int name_buffer_size (in bytes)
}

#[repr(C)]
#[derive(Debug)]
pub struct TreeContext {
    pub currdir: [c_char; MAX_PATH], // char currdir[MAX_PATH]
    pub dirlevel: c_int,             // int dirlevel
    pub selected_item: c_int,        // int selected_item
    pub selected_item_history: [c_int; MAX_DIR_LEVELS], // int selected_item_history[MAX_DIR_LEVELS]
    pub dirfilter: *mut c_int,       // int* dirfilter
    pub filesindir: c_int,           // int filesindir
    pub dirsindir: c_int,            // int dirsindir
    pub dirlength: c_int,            // int dirlength
    pub currtable: c_int,            // int currtable (db use)
    pub currextra: c_int,            // int currextra (db use)
    pub sort_dir: c_int,             // int sort_dir
    pub out_of_tree: c_int,          // int out_of_tree
    pub cache: TreeCache,            // struct tree_cache cache
    pub dirfull: bool,               // bool dirfull
    pub is_browsing: bool,           // bool is_browsing
    pub browse: *mut BrowseContext,  // struct browse_context* browse
}

#[repr(C)]
#[derive(Debug)]
pub struct BrowseContext {
    pub dirfilter: c_int, // int dirfilter
    pub flags: c_uint,    // unsigned flags
    pub callback_show_item:
        Option<extern "C" fn(name: *mut c_char, attr: c_int, tc: *mut TreeContext) -> bool>, // bool (*callback_show_item)(...)
    pub title: *mut c_char,      // char* title
    pub icon: ThemableIcons,     // enum themable_icons icon
    pub root: *const c_char,     // const char* root
    pub selected: *const c_char, // const char* selected
    pub buf: *mut c_char,        // char* buf
    pub bufsize: usize,          // size_t bufsize
}

#[repr(C)]
#[derive(Debug)]
pub struct Entry {
    pub name: *mut c_char,   // char* name
    pub attr: c_int,         // int attr (FAT attributes + file type flags)
    pub time_write: c_uint,  // unsigned time_write (Last write time)
    pub customaction: c_int, // int customaction (db use)
}

pub type PlaylistInsertCb = Option<extern "C" fn()>;
pub type AddToPlCallback = Option<extern "C" fn()>;

#[repr(C)]
#[derive(Debug)]
pub struct Tm {
    pub tm_sec: c_int,          // Seconds. [0-60] (1 leap second)
    pub tm_min: c_int,          // Minutes. [0-59]
    pub tm_hour: c_int,         // Hours. [0-23]
    pub tm_mday: c_int,         // Day. [1-31]
    pub tm_mon: c_int,          // Month. [0-11]
    pub tm_year: c_int,         // Year - 1900
    pub tm_wday: c_int,         // Day of week. [0-6]
    pub tm_yday: c_int,         // Days in year. [0-365]
    pub tm_isdst: c_int,        // DST. [-1/0/1]
    pub tm_gmtoff: c_long,      // Seconds east of UTC
    pub tm_zone: *const c_char, // Timezone abbreviation
}

extern "C" {
    // Playback control
    fn audio_pause();
    fn audio_play(elapsed: c_long, offset: c_long);
    fn audio_resume();
    fn audio_next();
    fn audio_prev();
    fn audio_ff_rewind(newtime: c_int);
    fn audio_next_track() -> *mut Mp3Entry;
    fn audio_status() -> c_int;
    fn audio_current_track() -> *mut Mp3Entry;
    fn audio_flush_and_reload_tracks();
    fn audio_get_file_pos() -> c_int;
    fn audio_hard_stop();

    // Playlist control
    fn playlist_get_current() -> *mut PlaylistInfo;
    fn playlist_get_resume_info(resume_index: *mut c_int) -> c_int;
    fn playlist_get_track_info(
        playlist: *mut PlaylistInfo,
        index: c_int,
        info: *mut PlaylistTrackInfo,
    ) -> c_int;
    fn playlist_get_first_index(playlist: *mut PlaylistInfo) -> c_int;
    fn playlist_get_display_index() -> c_int;
    fn playlist_amount() -> c_int;
    fn playlist_resume() -> c_int;
    fn playlist_resume_track(start_index: c_int, crc: c_uint, elapsed: c_ulong, offset: c_ulong);
    fn playlist_set_modified(playlist: *mut PlaylistInfo, modified: c_uchar);
    fn playlist_start(start_index: c_int, elapsed: c_ulong, offset: c_ulong);
    fn playlist_sync(playlist: *mut PlaylistInfo);
    fn playlist_remove_all_tracks(playlist: *mut PlaylistInfo) -> c_int;
    fn playlist_create(dir: *const c_char, file: *const c_char) -> c_int;
    fn playlist_insert_track(
        playlist: *mut PlaylistInfo,
        filename: *const c_char,
        position: c_int,
        queue: c_uchar,
        sync: c_uchar,
    ) -> c_int;
    fn playlist_insert_directory(
        playlist: *mut PlaylistInfo,
        dir: *const c_char,
        position: c_int,
        queue: c_uchar,
        recurse: c_uchar,
    ) -> c_int;
    fn playlist_insert_playlist(
        playlist: *mut PlaylistInfo,
        filename: *const c_char,
        position: c_int,
        queue: c_uchar,
    ) -> c_int;
    fn playlist_shuffle(random_sed: c_int, start_index: c_int) -> c_int;
    fn warn_on_pl_erase() -> c_uchar;

    // Sound
    fn adjust_volume();
    fn sound_set();
    fn sound_current();
    fn sound_default();
    fn sound_min();
    fn sound_max();
    fn sound_unit();
    fn sound_val2phys();
    fn sound_enum_hw_eq_band_setting();
    fn sound_get_pitch();
    fn sound_set_pitch();
    fn audio_master_sampr_list();
    fn pcm_apply_settings();
    fn pcm_play_data();
    fn pcm_play_stop();
    fn pcm_set_frequency();
    fn pcm_is_playing();
    fn pcm_play_lock();
    fn pcm_play_unlock();
    fn beep_play();
    fn dsp_set_crossfeed_type();
    fn dsp_eq_enable();
    fn dsp_dither_enable();
    fn dsp_get_timestretch();
    fn dsp_set_timestretch();
    fn dsp_timestretch_enable();
    fn dsp_timestrech_available();
    fn dsp_configure();
    fn dsp_get_config();
    fn dsp_process();
    fn mixer_channel_status();
    fn mixer_channel_get_buffer();
    fn mixer_channel_calculate_peaks();
    fn mixer_channel_play_data();
    fn mixer_channel_play_pause();
    fn mixer_channel_stop();
    fn mixer_channel_set_amplitude();
    fn mixer_channel_get_bytes_waiting();
    fn mixer_channel_set_buffer_hook();
    fn mixer_set_frequency();
    fn mixer_get_frequency();
    fn pcmbuf_fade();
    fn pcmbuf_set_low_latency();
    fn system_sound_play();
    fn keyclick_click();

    // Browsing
    fn rockbox_browse(browse: *mut BrowseContext) -> c_int;
    fn tree_get_context() -> *mut TreeContext;
    fn tree_get_entries(t: *mut TreeContext) -> *mut Entry;
    fn tree_get_entry_at(t: *mut TreeContext, index: c_int) -> *mut Entry;
    fn set_current_file(path: *const c_char);
    fn set_dirfilter(l_dirfilter: c_int);
    fn onplay_show_playlist_menu(
        path: *const c_char,                  // const char* path
        attr: c_int,                          // int attr
        playlist_insert_cb: PlaylistInsertCb, // void (*playlist_insert_cb)()
    );
    fn onplay_show_playlist_cat_menu(
        track_name: *const c_char,
        attr: c_int,
        add_to_pl_cb: AddToPlCallback,
    );
    fn browse_id3(
        id3: *mut Mp3Entry,
        playlist_display_index: c_int,
        playlist_amount: c_int,
        modified: *mut Tm,
        track_ct: c_int,
    ) -> c_uchar;

    // Directory
    fn open_dir();
    fn close_dir();
    fn readdir();
    fn mkdir();
    fn rmdir();
    fn dir_exists();
    fn dir_get_info();

    // File
    fn open_utf8();
    fn open();
    fn creat();
    fn close();
    fn read();
    fn lseek();
    fn write();
    fn remove();
    fn rename();
    fn ftruncate();
    fn filesize();
    fn fdprintf();
    fn read_line();
    fn settings_parseline();
    fn storage_sleep();
    fn storage_spin();
    fn storage_spindown();
    fn reload_directory();
    fn create_numbered_filename();
    fn file_exists();
    fn strip_extension();
    fn crc_32();
    fn crc_32r();
    fn filetype_get_attr();
    fn filetype_get_plugin();

    // Metadata
    fn get_metadata();
    fn get_codec_string();
    fn count_mp3_frames();
    fn create_xing_header();
    fn tagcache_search();
    fn tagcache_search_set_uniqbuf();
    fn tagcache_search_add_filter();
    fn tagcache_get_next();
    fn tagcache_get_numeric();
    fn tagcache_get_stat();
    fn tagcache_commit_finalize();
    fn tagcache_is_in_ram();
    fn tagcache_fill_tags();
    fn tagtree_subentries_do_action();
    fn search_albumart_files();

    // Kernel / System
    fn sleep(ticks: c_uint);
    fn r#yield();
    fn current_tick();
    fn default_event_handler(event: c_long);
    fn create_thread();
    fn thread_self();
    fn thread_exit();
    fn thread_wait();
    fn thread_thaw();
    fn thread_set_priority(thread_id: c_uint, priority: c_int);
    fn mutext_init();
    fn mutex_lock();
    fn mutex_unlock();
    fn semaphore_init();
    fn semaphore_wait();
    fn semaphore_release();
    fn reset_poweroff_timer();
    fn set_sleeptimer_duration();
    fn get_sleep_timer();
    fn commit_dcache();
    fn commit_discard_dcache();
    fn commit_discard_idcache();

    // Menu
    fn root_menu_get_options();
    fn do_menu();
    fn root_menu_set_default();
    fn root_menu_write_to_cfg();
    fn root_menu_load_from_cfg();

    // Settings
    fn get_settings_list(count: *mut c_int);
    fn find_settings();
    fn settings_save();
    fn option_screen();
    fn set_option();
    fn set_bool_options();
    fn set_int();
    fn set_int_ex();
    fn set_bool();
    fn set_color();

    // Misc
    fn codec_load_file();
    fn codec_run_proc();
    fn codec_close();
    fn read_bmp_file();
    fn read_bmp_fd();
    fn read_jpeg_file();
    fn read_jpeg_fd();

    // Plugin
    fn plugin_open();
    fn plugin_get_buffer();
    fn plugin_get_audio_buffer();
    fn plugin_release_audio_buffer();
    fn plugin_get_current_filename();
    fn plugin_reserve_buffer();
}
