use std::ffi::{c_char, c_int, c_long, c_uchar, c_uint, c_ulong, c_void};

pub mod browse;
pub mod dir;
pub mod events;
pub mod file;
pub mod menu;
pub mod metadata;
pub mod misc;
pub mod playback;
pub mod playlist;
pub mod plugin;
pub mod settings;
pub mod sound;
pub mod system;
pub mod tagcache;

const MAX_PATH: usize = 260;
const ID3V2_BUF_SIZE: usize = 1800;
const MAX_PATHNAME: usize = 80;
const NB_SCREENS: usize = 2;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
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
pub type ProgressFunc = Option<extern "C" fn(x: c_int)>;
pub type ActionCb = Option<extern "C" fn(file_name: *const c_char) -> c_uchar>;

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

#[repr(C)]
#[derive(Debug)]
pub struct Dir {}

#[repr(C)]
#[derive(Debug)]
pub struct dirent {}

#[repr(C)]
#[derive(Debug)]
pub struct Dirent {
    pub attribute: c_uint,
    pub d_name: [c_char; MAX_PATH],
}

const TAG_COUNT: usize = 32;
const SEEK_LIST_SIZE: usize = 32;
const TAGCACHE_MAX_FILTERS: usize = 4;
const TAGCACHE_MAX_CLAUSES: usize = 32;

#[repr(C)]
#[derive(Debug)]
pub struct TagcacheSearch {
    /* For internal use only. */
    fd: c_int,
    masterfd: c_int,
    idxfd: [c_int; TAG_COUNT],
    seeklist: [TagcacheSeeklistEntry; SEEK_LIST_SIZE],
    seek_list_count: c_int,
    filter_tag: [i32; TAGCACHE_MAX_FILTERS],
    filter_seek: [i32; TAGCACHE_MAX_FILTERS],
    filter_count: c_int,
    clause: [*mut TagcacheSearchClause; TAGCACHE_MAX_CLAUSES],
    clause_count: c_int,
    list_position: c_int,
    seek_pos: c_int,
    position: c_long,
    entry_count: c_int,
    valid: bool,
    initialized: bool,
    unique_list: *mut u32,
    unique_list_capacity: c_int,
    unique_list_count: c_int,

    /* Exported variables. */
    ramsearch: bool,     /* Is ram copy of the tagcache being used. */
    ramresult: bool,     /* False if result is not static, and must be copied. */
    r#type: c_int,       /* The tag type to be searched. */
    result: *mut c_char, /* The result data for all tags. */
    result_len: c_int,   /* Length of the result including \0 */
    result_seek: i32,    /* Current position in the tag data. */
    idx_id: i32,         /* Entry number in the master index. */
}

#[repr(C)]
#[derive(Debug)]
pub struct TagcacheSeeklistEntry {
    seek: i32,
    flag: i32,
    idx_id: i32,
}

#[repr(C)]
#[derive(Debug)]
pub struct TagcacheSearchClause {
    tag: c_int,
    r#type: c_int,
    numeric: bool,
    source: c_int,
    numeric_data: c_long,
    str: *mut c_char,
}

#[repr(C)]
#[derive(Debug)]
pub struct TagcacheStat {
    db_path: [c_char; MAX_PATHNAME + 1], // Path to DB root directory

    initialized: bool,       // Is tagcache currently busy?
    readyvalid: bool,        // Has tagcache ready status been ascertained?
    ready: bool,             // Is tagcache ready to be used?
    ramcache: bool,          // Is tagcache loaded in RAM?
    commit_delayed: bool,    // Has commit been delayed until next reboot?
    econ: bool,              // Is endianess correction enabled?
    syncscreen: bool,        // Synchronous operation with debug screen?
    curentry: *const c_char, // Path of the current entry being scanned

    commit_step: c_int,        // Commit progress
    ramcache_allocated: c_int, // Has RAM been allocated for ramcache?
    ramcache_used: c_int,      // How much RAM has been really used?
    progress: c_int,           // Current progress of disk scan
    processed_entries: c_int,  // Scanned disk entries so far
    total_entries: c_int,      // Total entries in tagcache
    queue_length: c_int,       // Command queue length
}

#[repr(C)]
pub union StorageType {
    int_val: c_int, // assuming it's an integer type, adjust according to the actual definition
                    // other possible types if storage_type is a union of different types
}

#[repr(C)]
pub struct SoundSetting {}

#[repr(C)]
pub struct BoolSetting {}

#[repr(C)]
pub struct FilenameSetting {}

#[repr(C)]
pub struct IntSetting {}

#[repr(C)]
pub struct ChoiceSetting {}

#[repr(C)]
pub struct TableSetting {}

#[repr(C)]
pub struct CustomSetting {}

#[repr(C)]
pub struct SettingsList {
    pub flags: c_uint,            // uint32_t -> c_uint
    pub setting: *mut c_void,     // pointer to void
    pub lang_id: c_int,           // int
    pub default_val: StorageType, // union storage_type
    pub cfg_name: *const c_char,  // const char*
    pub cfg_vals: *const c_char,  // const char*

    // union with different possible struct types
    pub setting_type: SettingsTypeUnion,
}

#[repr(C)]
pub union SettingsTypeUnion {
    pub RESERVED: *const c_void, // void pointer for the reserved field
    pub sound_setting: *const SoundSetting, // pointer to SoundSetting struct
    pub bool_setting: *const BoolSetting, // pointer to BoolSetting struct
    pub filename_setting: *const FilenameSetting, // pointer to FilenameSetting struct
    pub int_setting: *const IntSetting, // pointer to IntSetting struct
    pub choice_setting: *const ChoiceSetting, // pointer to ChoiceSetting struct
    pub table_setting: *const TableSetting, // pointer to TableSetting struct
    pub custom_setting: *const CustomSetting, // pointer to CustomSetting struct
}

#[repr(C)]
pub union FrameBufferData {
    pub data: *mut c_void,   // void* in C
    pub ch_ptr: *mut c_char, // char* in C
    pub fb_ptr: *mut c_char,
}

#[repr(C)]
pub struct FrameBuffer {
    pub buffer_data: FrameBufferData, // union data
    pub get_address_fn: Option<extern "C" fn(x: c_int, y: c_int) -> *mut c_void>, // Function pointer
    pub stride: isize,                                                            // ptrdiff_t in C
    pub elems: usize,                                                             // size_t in C
}

#[repr(C)]
pub struct Viewport {
    pub x: c_int,                 // int in C
    pub y: c_int,                 // int in C
    pub width: c_int,             // int in C
    pub height: c_int,            // int in C
    pub flags: c_int,             // int in C
    pub font: c_int,              // int in C
    pub drawmode: c_int,          // int in C
    pub buffer: *mut FrameBuffer, // pointer to FrameBuffer struct
    pub fg_pattern: c_uint,       // unsigned int in C
    pub bg_pattern: c_uint,       // unsigned int in C
}

#[repr(C)]
pub enum OptionType {
    RbInt = 0,
    RbBool = 1,
}

#[repr(C)]
pub struct OptItems {
    pub string: *const c_uchar, // const unsigned char*
    pub voice_id: c_int,        // int32_t
}

pub type PcmPlayCallbackType =
    Option<extern "C" fn(start: *const *const c_void, size: *mut c_ulong)>;

#[repr(C)]
pub enum PcmDmaStatus {
    PcmDmaStatusErrDma = -1, // PCM_DMAST_ERR_DMA in C
    PcmDmaStatusOk = 0,      // PCM_DMAST_OK in C
    PcmDmaStatusStarted = 1, // PCM_DMAST_STARTED in C
}

pub type PcmStatusCallbackType = Option<extern "C" fn(status: PcmDmaStatus) -> PcmDmaStatus>;

#[repr(C)]
pub struct SampleFormat {
    pub version: u8,
    pub num_channels: u8,
    pub frac_bits: u8,
    pub output_scale: u8,
    pub frequency: i32,
    pub codec_frequency: i32,
}

#[repr(C)]
pub struct DspBuffer {
    pub remcount: i32, // Samples in buffer (In, Int, Out)

    // Union for channel pointers
    pub pin: [*const c_void; 2], // Channel pointers (In)
    pub p32: [*mut i32; 2],      // Channel pointers (Int)
    pub p16out: *mut i16,        // DSP output buffer (Out)

    // Union for buffer count and proc_mask
    pub proc_mask: u32, // In-place effects already applied
    pub bufcount: i32,  // Buffer length/dest buffer remaining

    pub format: SampleFormat, // Buffer format data
}

impl DspBuffer {
    pub fn new() -> Self {
        Self {
            remcount: 0,
            pin: [std::ptr::null(); 2],
            p32: [std::ptr::null_mut(); 2],
            p16out: std::ptr::null_mut(),
            proc_mask: 0,
            bufcount: 0,
            format: SampleFormat {
                version: 0,
                num_channels: 0,
                frac_bits: 0,
                output_scale: 0,
                frequency: 0,
                codec_frequency: 0,
            },
        }
    }
}

pub type SampleInputFnType = unsafe extern "C" fn(samples: *const u8, size: usize);

pub type SampleOutputFnType = unsafe extern "C" fn(samples: *const u8, size: usize);

#[repr(C)]
pub struct DspProcEntry {
    pub data: isize, // intptr_t in C
    pub process: Option<extern "C" fn(*mut DspProcEntry, *mut *mut DspBuffer)>,
}

impl DspProcEntry {
    pub fn new() -> Self {
        Self {
            data: 0,
            process: None,
        }
    }
}

#[repr(C)]
pub struct DspProcSlot {
    pub proc_entry: DspProcEntry, // Adjust the type if necessary
    pub next: *mut DspProcSlot,
    pub mask: u32,
    pub version: u8,
    pub db_index: u8,
}

#[repr(C)]
pub struct SampleIoData {
    pub outcount: i32,
    pub format: SampleFormat, // Replace with actual type
    pub sample_depth: i32,
    pub stereo_mode: i32,
    pub input_samples: SampleInputFnType,
    pub sample_buf: DspBuffer, // Replace with actual type
    pub sample_buf_p: [*mut i32; 2],
    pub output_samples: SampleOutputFnType,
    pub output_sampr: u32,
    pub format_dirty: u8,
    pub output_version: u8,
}

#[repr(C)]
pub struct DspConfig {
    pub io_data: SampleIoData, // Adjust the type if necessary
    pub slot_free_mask: u32,
    pub proc_mask_enabled: u32,
    pub proc_mask_active: u32,
    pub proc_slots: *mut DspProcSlot,
}

#[repr(C)]
pub enum PcmMixerChannel {
    Playback = 0,
    Voice,
    NumChannels,
}

impl PcmMixerChannel {
    // Optionally, add methods for convenience
    pub fn as_u32(self) -> u32 {
        self as u32
    }

    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(PcmMixerChannel::Playback),
            1 => Some(PcmMixerChannel::Voice),
            // Include this if HAVE_HARDWARE_BEEP is not defined
            // 2 => Some(PcmMixerChannel::Beep),
            _ => None,
        }
    }
}

#[repr(C)]
pub enum ChannelStatus {
    Stopped = 0,
    Playing,
    Paused,
}

impl ChannelStatus {
    // Optionally, add methods for convenience
    pub fn as_u32(self) -> u32 {
        self as u32
    }

    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(ChannelStatus::Stopped),
            1 => Some(ChannelStatus::Playing),
            2 => Some(ChannelStatus::Paused),
            _ => None,
        }
    }
}

#[repr(C)]
pub struct PcmPeaks {
    pub left: u32,   // Left peak value
    pub right: u32,  // Right peak value
    pub period: i64, // For tracking calling period
    pub tick: i64,   // Last tick called
}

pub type ChanBufferHookFnType = extern "C" fn(start: *const c_void, size: usize);

#[repr(C)]
pub enum SystemSound {
    KeyClick = 0,
    TrackSkip,
    TrackNoMore,
    ListEdgeBeepWrap,
    ListEdgeBeepNoWrap,
}

extern "C" {
    // Playback control
    fn audio_pause() -> c_void;
    fn audio_play(elapsed: c_long, offset: c_long) -> c_void;
    fn audio_resume() -> c_void;
    fn audio_next() -> c_void;
    fn audio_prev() -> c_void;
    fn audio_ff_rewind(newtime: c_int) -> c_void;
    fn audio_next_track() -> *mut Mp3Entry;
    fn audio_status() -> c_int;
    fn audio_current_track() -> *mut Mp3Entry;
    fn audio_flush_and_reload_tracks() -> c_void;
    fn audio_get_file_pos() -> c_int;
    fn audio_hard_stop() -> c_void;

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
    fn adjust_volume(steps: c_int);
    fn sound_set(setting: c_int, value: c_int);
    fn sound_current(setting: c_int) -> c_int;
    fn sound_default(setting: c_int) -> c_int;
    fn sound_min(setting: c_int) -> c_int;
    fn sound_max(setting: c_int) -> c_int;
    fn sound_unit(setting: c_int) -> *const c_char;
    fn sound_val2phys(setting: c_int, value: c_int) -> c_int;
    fn sound_get_pitch() -> c_int;
    fn sound_set_pitch(pitch: c_int);
    fn pcm_apply_settings();
    fn pcm_play_data(
        get_more: PcmPlayCallbackType,
        status_cb: PcmStatusCallbackType,
        start: *const *const c_void,
        size: usize,
    );
    fn pcm_play_stop();
    fn pcm_set_frequency(frequency: c_uint);
    fn pcm_is_playing() -> c_uchar;
    fn pcm_play_lock();
    fn pcm_play_unlock();
    fn beep_play(frequency: c_uint, duration: c_uint, amplitude: c_uint);
    fn dsp_set_crossfeed_type(r#type: c_int);
    fn dsp_eq_enable(enable: c_uchar);
    fn dsp_dither_enable(enable: c_uchar);
    fn dsp_get_timestretch() -> c_int;
    fn dsp_set_timestretch(percent: c_int);
    fn dsp_timestretch_enable(enabled: c_uchar);
    fn dsp_timestretch_available() -> c_uchar;
    fn dsp_configure(dsp: *mut DspConfig, setting: c_uint, value: c_long) -> c_long;
    fn dsp_get_config(dsp_id: c_int) -> *mut DspConfig;
    fn dsp_process(dsp: *mut DspConfig, src: *mut DspBuffer, dst: *mut *mut DspBuffer);
    fn mixer_channel_status(channel: PcmMixerChannel) -> ChannelStatus;
    fn mixer_channel_get_buffer(channel: PcmMixerChannel, count: *mut c_int) -> *mut c_void;
    fn mixer_channel_calculate_peaks(channel: PcmMixerChannel, peaks: *mut PcmPeaks);
    fn mixer_channel_play_data(
        channel: PcmMixerChannel,
        get_more: PcmPlayCallbackType,
        start: *const *const c_void,
        size: usize,
    );
    fn mixer_channel_play_pause(channel: PcmMixerChannel, play: c_uchar);
    fn mixer_channel_stop(channel: PcmMixerChannel);
    fn mixer_channel_set_amplitude(channel: PcmMixerChannel, amplitude: c_uint);
    fn mixer_channel_get_bytes_waiting(channel: PcmMixerChannel) -> usize;
    fn mixer_channel_set_buffer_hook(channel: PcmMixerChannel, r#fn: ChanBufferHookFnType);
    fn mixer_set_frequency(samplerate: c_uint);
    fn mixer_get_frequency() -> c_uint;
    fn pcmbuf_fade(fade: c_int, r#in: c_uchar);
    fn pcmbuf_set_low_latency(state: c_uchar);
    fn system_sound_play(sound: SystemSound);
    fn keyclick_click(rawbutton: c_uchar, action: c_int);

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
    fn opendir(dirname: *const c_char) -> *mut Dir;
    fn closedir(dirp: *mut Dir) -> c_int;
    fn readdir(dirp: *mut Dir) -> *mut dirent;
    fn mkdir(path: *const c_char) -> c_int;
    fn rmdir(path: *const c_char) -> c_int;
    fn dir_get_info(dirp: *mut Dir, entry: *mut dirent) -> *mut Dirent;

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
    fn fdprintf();
    fn read_line();
    fn settings_parseline();
    fn reload_directory();
    fn create_numbered_filename();
    fn strip_extension();
    fn crc_32();
    fn crc_32r();
    fn filetype_get_attr();
    fn filetype_get_plugin();

    // Metadata
    fn get_metadata(id3: *mut Mp3Entry, fd: c_int, trackname: *const c_char) -> c_uchar;
    fn get_codec_string(codectype: c_int) -> *const c_char;
    fn count_mp3_frames(
        fd: c_int,
        startpos: c_int,
        filesize: c_int,
        progressfunc: ProgressFunc,
        buf: *mut c_uchar,
        buflen: usize,
    ) -> c_int;
    fn create_xing_header(
        fd: c_int,
        startpos: c_long,
        filesize: c_long,
        buf: *mut c_uchar,
        num_frames: c_ulong,
        rec_time: c_ulong,
        header_template: c_ulong,
        progressfunc: ProgressFunc,
        generate_toc: c_uchar,
        tempbuf: *mut c_uchar,
        tembuf_len: usize,
    ) -> c_int;
    fn tagcache_search(tcs: *mut TagcacheSearch, tag: c_int) -> c_uchar;
    fn tagcache_search_set_uniqbuf(tcs: *mut TagcacheSearch, buffer: *mut c_void, length: c_long);
    fn tagcache_search_add_filter(tcs: *mut TagcacheSearch, tag: c_int, seek: c_int) -> c_uchar;
    fn tagcache_get_next(tcs: *mut TagcacheSearch, buf: *mut c_char, size: c_long) -> c_uchar;
    // fn tagcahe_retrieve(tcs: *mut TagcacheSearch, idxid: c_int, tag: c_int) -> c_uchar;
    // fn tagcache_search_finish(tcs: *mut TagcacheSearch);
    fn tagcache_get_numeric(tcs: *mut TagcacheSearch, tag: c_int) -> c_long;
    fn tagcache_get_stat() -> *mut TagcacheStat;
    fn tagcache_commit_finalize();
    fn tagtree_subentries_do_action(cb: ActionCb) -> c_uchar;
    fn search_albumart_files(
        id3: *mut Mp3Entry,
        size_string: *const c_char,
        buf: *mut c_char,
        buflen: c_int,
    ) -> c_uchar;

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
    fn mutex_init();
    fn mutex_lock();
    fn mutex_unlock();
    fn semaphore_init();
    fn semaphore_wait();
    fn semaphore_release();
    fn reset_poweroff_timer();
    fn set_sleeptimer_duration();
    fn get_sleep_timer();

    // Menu
    fn root_menu_get_options();
    fn do_menu();
    fn root_menu_set_default();
    fn root_menu_write_to_cfg();
    fn root_menu_load_from_cfg();

    // Settings
    fn get_settings_list(count: *mut c_int) -> *mut SettingsList;
    fn find_setting(variable: *const c_void) -> *mut SettingsList;
    fn settings_save() -> c_int;
    fn option_screen(
        setting: *mut SettingsList,
        parent: [Viewport; NB_SCREENS],
        use_temp_var: c_uchar,
        option_title: *mut c_uchar,
    ) -> c_uchar;
    fn set_option(
        string: *const c_char,
        options: *const OptItems,
        numoptions: c_int,
        function: Option<extern "C" fn(x: c_int) -> c_uchar>,
    ) -> c_uchar;
    fn set_bool_options(
        string: *const c_char,
        variable: *const c_uchar,
        yes_str: *const c_char,
        yes_voice: c_int,
        no_str: *const c_char,
        no_voice: c_int,
        function: Option<extern "C" fn(x: c_int) -> c_uchar>,
    );
    fn set_int(
        unit: *const c_char,
        voice_unit: c_int,
        variable: *const c_int,
        function: Option<extern "C" fn(c_int)>,
        step: c_int,
        min: c_int,
        max: c_int,
        formatter: Option<extern "C" fn(*mut c_char, usize, c_int, *const c_char) -> *const c_char>,
    );
    fn set_int_ex(
        unit: *const c_char,
        voice_unit: c_int,
        variable: *const c_int,
        function: Option<extern "C" fn(c_int)>,
        step: c_int,
        min: c_int,
        max: c_int,
        formatter: Option<extern "C" fn(*mut c_char, usize, c_int, *const c_char) -> *const c_char>,
        get_talk_id: Option<extern "C" fn(c_int, c_int) -> c_int>,
    );
    fn set_bool(string: *const c_char, variable: *const c_uchar) -> c_uchar;

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
    fn plugin_get_current_filename();
    fn plugin_reserve_buffer();
}
