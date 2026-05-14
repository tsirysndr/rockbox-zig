//! Rockbox WASM module — direct firmware control, no gRPC, no servers.
//!
//! JS calls these exports after loading the .wasm module:
//!   1. Set `Module.onPcmData = (ptr, bytes, sampleRate) => { ... }` for audio.
//!   2. Call `rb_daemon_start(configDir, musicDir)` to boot the firmware.
//!   3. Poll `rb_daemon_state()` until it returns 2 (running).
//!   4. Use `rb_play_url`, `rb_enqueue_url`, `rb_play`, `rb_pause`, etc.
//!
//! All media paths are HTTP URLs. The firmware's netstream layer handles them.

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_long};
use std::sync::atomic::{AtomicI32, Ordering};

// ── Firmware FFI ──────────────────────────────────────────────────────────────

extern "C" {
    fn main_c() -> c_int;

    // Firmware ready flag — set by wasm-bridge.c after audio_init() completes.
    fn rb_is_firmware_ready() -> c_int;

    // Command dispatcher (wasm-bridge.c).
    // audio_next/prev/pause/resume/stop/seek use id3_mutex (a Rockbox blocking
    // mutex) which requires the caller to be a valid Rockbox kernel thread.
    // Calling them from the Emscripten main JS thread corrupts the scheduler.
    // rb_wasm_cmd_post() uses only queue_post() (spinlock + condvar) — safe
    // from any OS thread.  The "wasm_cmd" Rockbox thread does the actual call.
    fn rb_wasm_cmd_post(id: c_long, data: isize);

    // All playlist-mutating calls go through the wasm_cmd thread now.
    fn rb_playlist_index() -> c_int;
    fn playlist_amount() -> c_int;

    // Sound — direct reads/writes to DSP state, no blocking mutex.
    fn adjust_volume(steps: c_int);
    fn sound_current(setting: c_int) -> c_int;

    // C-level JSON bridge helpers (wasm-bridge.c)
    fn rb_wasm_current_track_json() -> *mut c_char;
    fn rb_wasm_playlist_json() -> *mut c_char;
    fn rb_wasm_audio_status() -> c_int;
    fn rb_wasm_settings_json() -> *mut c_char;
}

// ── Command IDs — must match WASM_CMD_* in wasm-bridge.c ─────────────────────

const WASM_CMD_NEXT: c_long = 0;
const WASM_CMD_PREV: c_long = 1;
const WASM_CMD_PAUSE: c_long = 2;
const WASM_CMD_RESUME: c_long = 3;
const WASM_CMD_STOP: c_long = 4;
const WASM_CMD_SEEK: c_long = 5;
const WASM_CMD_PLAY_AT: c_long = 6;
const WASM_CMD_PLAY_URL: c_long = 7;
const WASM_CMD_ENQUEUE_URL: c_long = 8;
const WASM_CMD_CLEAR_QUEUE: c_long = 9;
const WASM_CMD_SHUFFLE: c_long = 10;
const WASM_CMD_SET_EQ_ENABLED: c_long = 11;
const WASM_CMD_SET_EQ_PRECUT: c_long = 12;
const WASM_CMD_SET_EQ_BAND: c_long = 13;
const WASM_CMD_SET_CROSSFADE: c_long = 14;
const WASM_CMD_SET_REPLAYGAIN: c_long = 15;
const WASM_CMD_SAVE_SETTINGS: c_long = 16;
const WASM_CMD_SET_BALANCE: c_long = 17;
const WASM_CMD_SET_CHANNEL_MODE: c_long = 18;
const WASM_CMD_SET_STEREO_WIDTH: c_long = 19;
const WASM_CMD_SET_CROSSFEED: c_long = 20;
const WASM_CMD_SET_SURROUND: c_long = 21;
const WASM_CMD_SET_BASS: c_long = 22;
const WASM_CMD_SET_TREBLE: c_long = 23;
const WASM_CMD_SET_DITHERING: c_long = 24;
const WASM_CMD_SET_AFR: c_long = 25;
const WASM_CMD_SET_PBE: c_long = 26;
const WASM_CMD_SET_TIMESTRETCH: c_long = 27;

// ── Payload structs for complex settings commands ─────────────────────────────
// Layouts must match the C typedefs in wasm-bridge.c.

#[repr(C)]
struct WasmEqBandCmd {
    band: c_int,
    cutoff: c_int,
    q: c_int,
    gain: c_int,
}

#[repr(C)]
struct WasmCrossfadeCmd {
    mode: c_int,
    fi_delay: c_int,
    fo_delay: c_int,
    fi_dur: c_int,
    fo_dur: c_int,
    mixmode: c_int,
}

#[repr(C)]
struct WasmReplaygainCmd {
    noclip: c_int,
    type_: c_int,
    preamp: c_int,
}

#[repr(C)]
struct WasmPbeCmd {
    pbe: c_int,
    precut: c_int,
}

#[repr(C)]
struct WasmCrossfeedDspCmd {
    type_: c_int,
    direct_gain: c_int,
    cross_gain: c_int,
    hf_attenuation: c_int,
    hf_cutoff: c_int,
}

#[repr(C)]
struct WasmSurroundCmd {
    enabled: c_int,
    balance: c_int,
    fx1: c_int,
    fx2: c_int,
    method2: c_int,
    mix: c_int,
}

// ── Daemon state ──────────────────────────────────────────────────────────────

/// 0 = stopped, 1 = starting, 2 = running
static DAEMON_STATE: AtomicI32 = AtomicI32::new(0);

// ── Helpers ───────────────────────────────────────────────────────────────────

unsafe fn cstr_to_string(p: *const c_char) -> Option<String> {
    if p.is_null() {
        return None;
    }
    CStr::from_ptr(p).to_str().ok().map(|s| s.to_owned())
}

fn json_str(s: impl AsRef<str>) -> *mut c_char {
    CString::new(s.as_ref())
        .map(|c| c.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

// ── Lifecycle ─────────────────────────────────────────────────────────────────

/// Boot the Rockbox firmware engine in a background thread.
///
/// `config_dir_ptr` — path used as HOME (Rockbox writes settings here).
/// `music_dir_ptr`  — base path for library scans (unused by WASM but kept for
///                    API symmetry with the Android cdylib).
///
/// Returns 0 on success, -1 if already started.
///
/// # Safety
/// Both pointers must be valid NUL-terminated UTF-8 C strings.
#[no_mangle]
pub unsafe extern "C" fn rb_daemon_start(
    config_dir_ptr: *const c_char,
    music_dir_ptr: *const c_char,
) -> c_int {
    if DAEMON_STATE
        .compare_exchange(0, 1, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return -1; // already started or starting
    }

    let config_dir = cstr_to_string(config_dir_ptr).unwrap_or_else(|| "/".into());
    let music_dir = cstr_to_string(music_dir_ptr).unwrap_or_else(|| "/music".into());

    // Set env vars that the firmware and settings crate read at boot.
    std::env::set_var("HOME", &config_dir);
    std::env::set_var("ROCKBOX_LIBRARY", &music_dir);

    std::thread::spawn(move || {
        DAEMON_STATE.store(2, Ordering::SeqCst);
        let _ = main_c(); // blocks for the firmware lifetime
        DAEMON_STATE.store(0, Ordering::SeqCst);
    });

    0
}

/// Returns daemon state: 0 = stopped, 1 = starting, 2 = running.
/// State 2 is only returned once rb_signal_firmware_ready() has been called
/// by the C firmware (after audio_init()), preventing JS from polling
/// firmware functions before they are initialised.
#[no_mangle]
pub extern "C" fn rb_daemon_state() -> c_int {
    let s = DAEMON_STATE.load(Ordering::SeqCst);
    if s == 2 && unsafe { rb_is_firmware_ready() } == 0 {
        return 1; // thread started but firmware not yet initialised
    }
    s
}

/// Frees a C string returned by any `rb_*_json` function.
///
/// # Safety
/// `ptr` must be null or a pointer returned by this library that has not been freed.
#[no_mangle]
pub unsafe extern "C" fn rb_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        drop(CString::from_raw(ptr));
    }
}

// ── Playback ──────────────────────────────────────────────────────────────────

/// Clear the queue, enqueue `url`, and start playback from position 0.
///
/// The actual playlist operations run in the wasm_cmd Rockbox kernel thread
/// so they can safely acquire playlist mutexes.
///
/// # Safety
/// `url_ptr` must be a valid NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn rb_play_url(url_ptr: *const c_char) -> c_int {
    let Some(url) = cstr_to_string(url_ptr) else {
        return -1;
    };
    let cpath = match CString::new(url) {
        Ok(c) => c,
        Err(_) => return -2,
    };
    // Transfer ownership to the command thread; it will call free().
    let raw = cpath.into_raw();
    rb_wasm_cmd_post(WASM_CMD_PLAY_URL, raw as isize);
    0
}

/// Append `url` to the end of the current queue.
///
/// # Safety
/// `url_ptr` must be a valid NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn rb_enqueue_url(url_ptr: *const c_char) -> c_int {
    let Some(url) = cstr_to_string(url_ptr) else {
        return -1;
    };
    let cpath = match CString::new(url) {
        Ok(c) => c,
        Err(_) => return -2,
    };
    let raw = cpath.into_raw();
    rb_wasm_cmd_post(WASM_CMD_ENQUEUE_URL, raw as isize);
    0
}

#[no_mangle]
pub extern "C" fn rb_play() -> c_int {
    let status = unsafe { rb_wasm_audio_status() };
    if status == 0 {
        // Stopped — restart from current playlist position (or 0).
        let idx = unsafe { rb_playlist_index() };
        let amount = unsafe { playlist_amount() };
        if amount > 0 {
            let start = if idx >= 0 && idx < amount { idx } else { 0 };
            unsafe { rb_wasm_cmd_post(WASM_CMD_PLAY_AT, start as isize) };
        }
    } else {
        unsafe { rb_wasm_cmd_post(WASM_CMD_RESUME, 0) };
    }
    0
}

#[no_mangle]
pub extern "C" fn rb_pause() -> c_int {
    unsafe { rb_wasm_cmd_post(WASM_CMD_PAUSE, 0) };
    0
}

#[no_mangle]
pub extern "C" fn rb_play_pause() -> c_int {
    // AUDIO_STATUS_PLAY=1, AUDIO_STATUS_PAUSE=2
    let s = unsafe { rb_wasm_audio_status() };
    if s == 1 {
        unsafe { rb_wasm_cmd_post(WASM_CMD_PAUSE, 0) };
    } else if s == 2 {
        unsafe { rb_wasm_cmd_post(WASM_CMD_RESUME, 0) };
    } else {
        // Stopped — restart from current playlist position.
        let idx = unsafe { rb_playlist_index() };
        let amount = unsafe { playlist_amount() };
        if amount > 0 {
            let start = if idx >= 0 && idx < amount { idx } else { 0 };
            unsafe { rb_wasm_cmd_post(WASM_CMD_PLAY_AT, start as isize) };
        }
    }
    0
}

#[no_mangle]
pub extern "C" fn rb_next() -> c_int {
    if unsafe { rb_wasm_audio_status() } != 0 {
        unsafe { rb_wasm_cmd_post(WASM_CMD_NEXT, 0) };
    } else {
        // Stopped — advance index and start directly.
        let idx = unsafe { rb_playlist_index() } + 1;
        let amount = unsafe { playlist_amount() };
        if idx < amount {
            unsafe { rb_wasm_cmd_post(WASM_CMD_PLAY_AT, idx as isize) };
        }
    }
    0
}

#[no_mangle]
pub extern "C" fn rb_prev() -> c_int {
    if unsafe { rb_wasm_audio_status() } != 0 {
        unsafe { rb_wasm_cmd_post(WASM_CMD_PREV, 0) };
    } else {
        // Stopped — go back and start directly.
        let idx = (unsafe { rb_playlist_index() } - 1).max(0);
        unsafe { rb_wasm_cmd_post(WASM_CMD_PLAY_AT, idx as isize) };
    }
    0
}

/// Seek to `position_ms` milliseconds from the start of the current track.
#[no_mangle]
pub extern "C" fn rb_seek(position_ms: c_int) -> c_int {
    unsafe { rb_wasm_cmd_post(WASM_CMD_SEEK, position_ms as isize) };
    0
}

#[no_mangle]
pub extern "C" fn rb_stop() -> c_int {
    unsafe { rb_wasm_cmd_post(WASM_CMD_STOP, 0) };
    0
}

// ── Queue management ──────────────────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn rb_clear_queue() -> c_int {
    unsafe { rb_wasm_cmd_post(WASM_CMD_CLEAR_QUEUE, 0) };
    0
}

#[no_mangle]
pub extern "C" fn rb_shuffle_queue() -> c_int {
    unsafe { rb_wasm_cmd_post(WASM_CMD_SHUFFLE, 0) };
    0
}

/// Jump to queue position `pos` (0-based).
#[no_mangle]
pub extern "C" fn rb_jump_to_queue_position(pos: c_int) -> c_int {
    unsafe { rb_wasm_cmd_post(WASM_CMD_PLAY_AT, pos as isize) };
    0
}

// ── Sound ─────────────────────────────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn rb_adjust_volume(steps: c_int) -> c_int {
    unsafe { adjust_volume(steps) };
    0
}

/// Returns the current value of `setting` (SOUND_VOLUME=0, etc.).
#[no_mangle]
pub extern "C" fn rb_sound_current(setting: c_int) -> c_int {
    unsafe { sound_current(setting) }
}

// ── Status / track info (JSON) ────────────────────────────────────────────────

/// Returns `{"status": 0|1|2}` — 0=stopped, 1=playing, 2=paused.
/// Caller must free with `rb_free_string`.
#[no_mangle]
pub extern "C" fn rb_status_json() -> *mut c_char {
    let state = unsafe { rb_wasm_audio_status() };
    json_str(format!("{{\"status\":{}}}", state))
}

/// Returns current track metadata as JSON.
/// Shape: `{title, artist, album, path, duration_ms, elapsed_ms}`.
/// Caller must free with `rb_free_string`.
#[no_mangle]
pub extern "C" fn rb_current_track_json() -> *mut c_char {
    unsafe { rb_wasm_current_track_json() }
}

/// Returns queue state as JSON: `{index, amount}`.
/// Caller must free with `rb_free_string`.
#[no_mangle]
pub extern "C" fn rb_playlist_json() -> *mut c_char {
    unsafe { rb_wasm_playlist_json() }
}

// ── Settings (JSON read + command-thread writes) ──────────────────────────────

/// Returns current settings as JSON.
/// Shape: `{eq: {enabled, precut, bands: [{cutoff,q,gain}×10]},
///          crossfade?: {...}, replaygain: {noclip, type, preamp}}`.
/// Caller must free with `rb_free_string`.
#[no_mangle]
pub extern "C" fn rb_settings_json() -> *mut c_char {
    unsafe { rb_wasm_settings_json() }
}

/// Enable or disable the equalizer.  `enabled` = 0 (off) or 1 (on).
/// Applies immediately and persists to the Rockbox config file.
#[no_mangle]
pub extern "C" fn rb_set_eq_enabled(enabled: c_int) -> c_int {
    unsafe { rb_wasm_cmd_post(WASM_CMD_SET_EQ_ENABLED, enabled as isize) };
    0
}

/// Set the EQ pre-cut (0–240, in tenths of a dB).
/// Applies immediately and persists.
#[no_mangle]
pub extern "C" fn rb_set_eq_precut(precut: c_int) -> c_int {
    unsafe { rb_wasm_cmd_post(WASM_CMD_SET_EQ_PRECUT, precut as isize) };
    0
}

/// Set one EQ band (`band` 0–9, `cutoff` in Hz, `q` Q-factor, `gain` in dB).
/// Applies immediately and persists.
#[no_mangle]
pub extern "C" fn rb_set_eq_band(band: c_int, cutoff: c_int, q: c_int, gain: c_int) -> c_int {
    let cmd = Box::new(WasmEqBandCmd {
        band,
        cutoff,
        q,
        gain,
    });
    // Transfer ownership to the command thread; it calls free().
    unsafe { rb_wasm_cmd_post(WASM_CMD_SET_EQ_BAND, Box::into_raw(cmd) as isize) };
    0
}

/// Set crossfade parameters.
/// `mode`: 0=off, 1=auto-skip, 2=manual-skip, 3=shuffle, 4=shuffle+manual, 5=always.
/// All delay/duration values are in seconds (0–15).
/// `mixmode`: 0=crossfade, 1=mix.
/// Applies on the next track transition and persists.
#[no_mangle]
pub extern "C" fn rb_set_crossfade(
    mode: c_int,
    fi_delay: c_int,
    fo_delay: c_int,
    fi_dur: c_int,
    fo_dur: c_int,
    mixmode: c_int,
) -> c_int {
    let cmd = Box::new(WasmCrossfadeCmd {
        mode,
        fi_delay,
        fo_delay,
        fi_dur,
        fo_dur,
        mixmode,
    });
    unsafe { rb_wasm_cmd_post(WASM_CMD_SET_CROSSFADE, Box::into_raw(cmd) as isize) };
    0
}

/// Set replaygain parameters.
/// `noclip`: 0 = allow clipping, 1 = scale to prevent clipping.
/// `type_`: 0=track, 1=album, 2=shuffle (track if shuffle on), 3=off.
/// `preamp`: additional gain in tenths of a dB (-120 to 120).
/// Applies immediately and persists.
#[no_mangle]
pub extern "C" fn rb_set_replaygain(noclip: c_int, type_: c_int, preamp: c_int) -> c_int {
    let cmd = Box::new(WasmReplaygainCmd {
        noclip,
        type_,
        preamp,
    });
    unsafe { rb_wasm_cmd_post(WASM_CMD_SET_REPLAYGAIN, Box::into_raw(cmd) as isize) };
    0
}

/// Set balance (-100 = full left, 0 = centre, +100 = full right).
#[no_mangle]
pub extern "C" fn rb_set_balance(value: c_int) -> c_int {
    unsafe { rb_wasm_cmd_post(WASM_CMD_SET_BALANCE, value as isize) };
    0
}

/// Set channel mode. Values: 0=stereo, 1=mono, 2=custom, 3=mono-L, 4=mono-R,
/// 5=karaoke, 6=swap.
#[no_mangle]
pub extern "C" fn rb_set_channel_mode(value: c_int) -> c_int {
    unsafe { rb_wasm_cmd_post(WASM_CMD_SET_CHANNEL_MODE, value as isize) };
    0
}

/// Set stereo width (0–250, where 100 = normal stereo).
#[no_mangle]
pub extern "C" fn rb_set_stereo_width(value: c_int) -> c_int {
    unsafe { rb_wasm_cmd_post(WASM_CMD_SET_STEREO_WIDTH, value as isize) };
    0
}

/// Set crossfeed (headphone DSP) parameters.
/// `type_`: 0=off, 1=Meier, 2=custom.
/// `direct_gain`, `cross_gain`, `hf_attenuation`: in tenths of dB (e.g. -60 = -6.0 dB).
/// `hf_cutoff`: in Hz (500–2000).
#[no_mangle]
pub extern "C" fn rb_set_crossfeed(
    type_: c_int,
    direct_gain: c_int,
    cross_gain: c_int,
    hf_attenuation: c_int,
    hf_cutoff: c_int,
) -> c_int {
    let cmd = Box::new(WasmCrossfeedDspCmd {
        type_,
        direct_gain,
        cross_gain,
        hf_attenuation,
        hf_cutoff,
    });
    unsafe { rb_wasm_cmd_post(WASM_CMD_SET_CROSSFEED, Box::into_raw(cmd) as isize) };
    0
}

/// Set surround (Haas effect) parameters.
/// `enabled`: 0=off, or delay in ms (5, 8, 10, 15, 30).
/// `balance`: 0–99 %.
/// `fx1`: low-pass cutoff Hz (600–8000).
/// `fx2`: high-pass cutoff Hz (40–400).
/// `method2`: 0=method1, 1=method2.
/// `mix`: 0–100 %.
#[no_mangle]
pub extern "C" fn rb_set_surround(
    enabled: c_int,
    balance: c_int,
    fx1: c_int,
    fx2: c_int,
    method2: c_int,
    mix: c_int,
) -> c_int {
    let cmd = Box::new(WasmSurroundCmd {
        enabled,
        balance,
        fx1,
        fx2,
        method2,
        mix,
    });
    unsafe { rb_wasm_cmd_post(WASM_CMD_SET_SURROUND, Box::into_raw(cmd) as isize) };
    0
}

/// Set bass tone control in whole dB (-24..+24).
/// Applies immediately via the software tone-control DSP and persists.
#[no_mangle]
pub extern "C" fn rb_set_bass(value: c_int) -> c_int {
    unsafe { rb_wasm_cmd_post(WASM_CMD_SET_BASS, value as isize) };
    0
}

/// Set treble tone control in whole dB (-24..+24).
#[no_mangle]
pub extern "C" fn rb_set_treble(value: c_int) -> c_int {
    unsafe { rb_wasm_cmd_post(WASM_CMD_SET_TREBLE, value as isize) };
    0
}

/// Enable or disable dithered output.  `enabled` = 0 (off) or 1 (on).
#[no_mangle]
pub extern "C" fn rb_set_dithering(enabled: c_int) -> c_int {
    unsafe { rb_wasm_cmd_post(WASM_CMD_SET_DITHERING, enabled as isize) };
    0
}

/// Set Adaptive Frequency Response (AFR) mode.  `value` = 0 (off) or 1–3.
#[no_mangle]
pub extern "C" fn rb_set_afr(value: c_int) -> c_int {
    unsafe { rb_wasm_cmd_post(WASM_CMD_SET_AFR, value as isize) };
    0
}

/// Set Perceptual Bass Enhancement (PBE).
/// `pbe` = 0 (off) or 1–3 (strength).
/// `precut` = pre-cut in tenths of dB (0–240).
#[no_mangle]
pub extern "C" fn rb_set_pbe(pbe: c_int, precut: c_int) -> c_int {
    let cmd = Box::new(WasmPbeCmd { pbe, precut });
    unsafe { rb_wasm_cmd_post(WASM_CMD_SET_PBE, Box::into_raw(cmd) as isize) };
    0
}

/// Enable/disable time-stretch and set stretch percentage.
/// `stretch_pct` = 0 to disable, or 35–250 (100 = normal speed).
/// Internally the firmware uses `stretch_pct * 100` as the precision value.
#[no_mangle]
pub extern "C" fn rb_set_timestretch(stretch_pct: c_int) -> c_int {
    let data = if stretch_pct <= 0 {
        0isize
    } else {
        (stretch_pct as isize) * 100 // × PITCH_SPEED_PRECISION
    };
    unsafe { rb_wasm_cmd_post(WASM_CMD_SET_TIMESTRETCH, data) };
    0
}

/// Flush all current settings to the Rockbox config file (MEMFS).
/// Call this explicitly if you want to ensure persistence without changing any
/// individual setting.
#[no_mangle]
pub extern "C" fn rb_save_settings() -> c_int {
    unsafe { rb_wasm_cmd_post(WASM_CMD_SAVE_SETTINGS, 0) };
    0
}
