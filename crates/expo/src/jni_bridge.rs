//! Android JNI bridge for `expo.modules.rockboxrpc.RockboxRpcModule`.
//!
//! Kotlin `external fun rb_xxx(...)` declarations resolve to symbols of the
//! form `Java_<package>_<class>_<method>` (every `_` in the Kotlin name is
//! escaped to `_1` per the JNI spec). The C ABI exported from `lib.rs` uses
//! plain `rb_xxx` names, so we add this thin shim — only on Android — to
//! re-export each entry point under its JNI-mangled name and forward to the
//! existing C ABI implementation.
//!
//! All bridges are intentionally `unsafe extern "system"` and `#[no_mangle]`
//! so they show up as exported symbols in the resulting `.so`.

#![cfg(target_os = "android")]
#![allow(non_snake_case)]

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::ptr;

use jni::objects::{JClass, JString};
use jni::sys::{jint, jstring};
use jni::JNIEnv;

// ── String marshaling helpers ───────────────────────────────────────────────

/// Convert a (possibly null) `JString` into a `CString` we can pass over the
/// C ABI. Returns `None` if the input is null OR fails to read.
fn to_cstring(env: &mut JNIEnv, s: &JString) -> Option<CString> {
    if s.is_null() {
        return None;
    }
    let raw = env.get_string(s).ok()?;
    CString::new(raw.to_string_lossy().as_bytes()).ok()
}

/// Convert a heap `*mut c_char` returned by Rust into a Java `jstring`,
/// freeing the original Rust allocation in the process.
fn cstr_to_jstring(env: &mut JNIEnv, ptr: *mut c_char) -> jstring {
    if ptr.is_null() {
        return ptr::null_mut();
    }
    let s = unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() };
    unsafe { crate::rb_free_string(ptr) };
    match env.new_string(&s) {
        Ok(js) => js.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

// ── Macros for the common shapes ────────────────────────────────────────────

/// Bridge a `() -> i32` C ABI function under its JNI-mangled name.
macro_rules! bridge_unit {
    ($jni_name:ident, $rust_fn:ident) => {
        #[no_mangle]
        pub unsafe extern "system" fn $jni_name(_env: JNIEnv, _cls: JClass) -> jint {
            crate::$rust_fn() as jint
        }
    };
}

/// Bridge a `() -> *mut c_char` JSON-returning function.
macro_rules! bridge_json {
    ($jni_name:ident, $rust_fn:ident) => {
        #[no_mangle]
        pub unsafe extern "system" fn $jni_name(mut env: JNIEnv, _cls: JClass) -> jstring {
            let p = crate::$rust_fn();
            cstr_to_jstring(&mut env, p)
        }
    };
}

/// Bridge a `(*const c_char) -> i32` function (single string arg).
macro_rules! bridge_unit_str {
    ($jni_name:ident, $rust_fn:ident) => {
        #[no_mangle]
        pub unsafe extern "system" fn $jni_name(
            mut env: JNIEnv,
            _cls: JClass,
            arg: JString,
        ) -> jint {
            let Some(c) = to_cstring(&mut env, &arg) else {
                return -1;
            };
            crate::$rust_fn(c.as_ptr()) as jint
        }
    };
}

/// Bridge a `(*const c_char) -> *mut c_char` JSON function with a single
/// string arg.
macro_rules! bridge_json_str {
    ($jni_name:ident, $rust_fn:ident) => {
        #[no_mangle]
        pub unsafe extern "system" fn $jni_name(
            mut env: JNIEnv,
            _cls: JClass,
            arg: JString,
        ) -> jstring {
            let Some(c) = to_cstring(&mut env, &arg) else {
                return ptr::null_mut();
            };
            let p = crate::$rust_fn(c.as_ptr());
            cstr_to_jstring(&mut env, p)
        }
    };
}

/// Bridge a `(i32) -> i32` function.
macro_rules! bridge_unit_int {
    ($jni_name:ident, $rust_fn:ident) => {
        #[no_mangle]
        pub unsafe extern "system" fn $jni_name(_env: JNIEnv, _cls: JClass, arg: jint) -> jint {
            crate::$rust_fn(arg as c_int) as jint
        }
    };
}

/// Bridge a `(i32) -> *mut c_char` JSON function.
macro_rules! bridge_json_int {
    ($jni_name:ident, $rust_fn:ident) => {
        #[no_mangle]
        pub unsafe extern "system" fn $jni_name(
            mut env: JNIEnv,
            _cls: JClass,
            arg: jint,
        ) -> jstring {
            let p = crate::$rust_fn(arg as c_int);
            cstr_to_jstring(&mut env, p)
        }
    };
}

// ── Init / health ───────────────────────────────────────────────────────────

bridge_unit_str!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1set_1server_1url,
    rb_set_server_url
);
bridge_unit_str!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1set_1http_1url,
    rb_set_http_url
);
bridge_unit!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1ping,
    rb_ping
);

bridge_json!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1get_1devices_1json,
    rb_get_devices_json
);
bridge_unit_str!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1connect_1device,
    rb_connect_device
);
bridge_unit_str!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1disconnect_1device,
    rb_disconnect_device
);

// ── Playback (no args) ──────────────────────────────────────────────────────

bridge_unit!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1play,
    rb_play
);
bridge_unit!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1pause,
    rb_pause
);
bridge_unit!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1play_1pause,
    rb_play_pause
);
bridge_unit!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1next,
    rb_next
);
bridge_unit!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1prev,
    rb_prev
);
bridge_unit!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1resume_1track,
    rb_resume_track
);
bridge_unit!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1playlist_1resume,
    rb_playlist_resume
);
bridge_unit!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1play_1all_1tracks,
    rb_play_all_tracks
);

// ── Playback (1 string) ─────────────────────────────────────────────────────

bridge_unit_str!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1play_1track,
    rb_play_track
);

// ── Seek ────────────────────────────────────────────────────────────────────

bridge_unit_int!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1seek,
    rb_seek
);

// ── Playback (string + int / int / int) ─────────────────────────────────────

#[no_mangle]
pub unsafe extern "system" fn Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1play_1album(
    mut env: JNIEnv,
    _cls: JClass,
    id: JString,
    shuffle: jint,
) -> jint {
    let Some(c) = to_cstring(&mut env, &id) else {
        return -1;
    };
    crate::rb_play_album(c.as_ptr(), shuffle as c_int) as jint
}

#[no_mangle]
pub unsafe extern "system" fn Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1play_1artist_1tracks(
    mut env: JNIEnv,
    _cls: JClass,
    id: JString,
    shuffle: jint,
) -> jint {
    let Some(c) = to_cstring(&mut env, &id) else {
        return -1;
    };
    crate::rb_play_artist_tracks(c.as_ptr(), shuffle as c_int) as jint
}

#[no_mangle]
pub unsafe extern "system" fn Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1play_1directory(
    mut env: JNIEnv,
    _cls: JClass,
    path: JString,
    shuffle: jint,
    position: jint,
) -> jint {
    let Some(c) = to_cstring(&mut env, &path) else {
        return -1;
    };
    crate::rb_play_directory(c.as_ptr(), shuffle as c_int, position as c_int) as jint
}

// ── Queue ───────────────────────────────────────────────────────────────────

bridge_unit_int!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1jump_1to_1queue_1position,
    rb_jump_to_queue_position
);
bridge_unit!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1shuffle_1playlist,
    rb_shuffle_playlist
);
bridge_unit_int!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1remove_1from_1queue,
    rb_remove_from_queue
);
bridge_unit_str!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1insert_1track_1next,
    rb_insert_track_next
);
bridge_unit_str!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1insert_1track_1last,
    rb_insert_track_last
);

#[no_mangle]
pub unsafe extern "system" fn Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1insert_1tracks(
    mut env: JNIEnv,
    _cls: JClass,
    paths_json: JString,
    position: jint,
    shuffle: jint,
) -> jint {
    let Some(c) = to_cstring(&mut env, &paths_json) else {
        return -1;
    };
    crate::rb_insert_tracks(c.as_ptr(), position as c_int, shuffle as c_int) as jint
}

#[no_mangle]
pub unsafe extern "system" fn Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1insert_1directory(
    mut env: JNIEnv,
    _cls: JClass,
    path: JString,
    position: jint,
) -> jint {
    let Some(c) = to_cstring(&mut env, &path) else {
        return -1;
    };
    crate::rb_insert_directory(c.as_ptr(), position as c_int) as jint
}

bridge_json!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1get_1playlist_1current_1json,
    rb_get_playlist_current_json
);

// ── Library / search ────────────────────────────────────────────────────────

bridge_json!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1get_1tracks_1json,
    rb_get_tracks_json
);
bridge_json!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1get_1artists_1json,
    rb_get_artists_json
);
bridge_json!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1get_1albums_1json,
    rb_get_albums_json
);
bridge_json!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1get_1liked_1albums_1json,
    rb_get_liked_albums_json
);
bridge_json_str!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1get_1artist_1json,
    rb_get_artist_json
);
bridge_json_str!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1get_1album_1json,
    rb_get_album_json
);
bridge_json!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1get_1liked_1tracks_1json,
    rb_get_liked_tracks_json
);
bridge_json_str!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1search_1json,
    rb_search_json
);
bridge_unit_str!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1like_1track,
    rb_like_track
);
bridge_unit_str!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1unlike_1track,
    rb_unlike_track
);

// ── Sound / status ──────────────────────────────────────────────────────────

bridge_json!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1status_1json,
    rb_status_json
);
bridge_json!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1current_1track_1json,
    rb_current_track_json
);
bridge_unit_int!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1adjust_1volume,
    rb_adjust_volume
);
bridge_json_int!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1sound_1current_1json,
    rb_sound_current_json
);

// ── Settings ────────────────────────────────────────────────────────────────

bridge_unit_int!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1save_1shuffle,
    rb_save_shuffle
);
bridge_unit_int!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1save_1repeat,
    rb_save_repeat
);
bridge_json!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1get_1global_1settings_1json,
    rb_get_global_settings_json
);
bridge_json!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1get_1global_1status_1json,
    rb_get_global_status_json
);

// ── Browse ──────────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "system" fn Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1tree_1get_1entries_1json(
    mut env: JNIEnv,
    _cls: JClass,
    path: JString,
) -> jstring {
    let owned = to_cstring(&mut env, &path);
    let ptr = owned.as_ref().map(|c| c.as_ptr()).unwrap_or(ptr::null());
    let p = crate::rb_tree_get_entries_json(ptr);
    cstr_to_jstring(&mut env, p)
}

// ── Saved playlists ─────────────────────────────────────────────────────────

bridge_json!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1get_1saved_1playlists_1json,
    rb_get_saved_playlists_json
);

#[no_mangle]
pub unsafe extern "system" fn Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1create_1saved_1playlist(
    mut env: JNIEnv,
    _cls: JClass,
    name: JString,
    description: JString,
    ids_json: JString,
) -> jint {
    let Some(name_c) = to_cstring(&mut env, &name) else {
        return -1;
    };
    let desc_c = to_cstring(&mut env, &description);
    let ids_c = to_cstring(&mut env, &ids_json);
    let desc_ptr = desc_c.as_ref().map(|c| c.as_ptr()).unwrap_or(ptr::null());
    let ids_ptr = ids_c.as_ref().map(|c| c.as_ptr()).unwrap_or(ptr::null());
    crate::rb_create_saved_playlist(name_c.as_ptr(), desc_ptr, ids_ptr) as jint
}

#[no_mangle]
pub unsafe extern "system" fn Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1update_1saved_1playlist(
    mut env: JNIEnv,
    _cls: JClass,
    id: JString,
    name: JString,
    description: JString,
) -> jint {
    let Some(id_c) = to_cstring(&mut env, &id) else {
        return -1;
    };
    let Some(name_c) = to_cstring(&mut env, &name) else {
        return -1;
    };
    let desc_c = to_cstring(&mut env, &description);
    let desc_ptr = desc_c.as_ref().map(|c| c.as_ptr()).unwrap_or(ptr::null());
    crate::rb_update_saved_playlist(id_c.as_ptr(), name_c.as_ptr(), desc_ptr) as jint
}

bridge_unit_str!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1delete_1saved_1playlist,
    rb_delete_saved_playlist
);

#[no_mangle]
pub unsafe extern "system" fn Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1add_1track_1to_1playlist(
    mut env: JNIEnv,
    _cls: JClass,
    pid: JString,
    tid: JString,
) -> jint {
    let Some(p) = to_cstring(&mut env, &pid) else {
        return -1;
    };
    let Some(t) = to_cstring(&mut env, &tid) else {
        return -1;
    };
    crate::rb_add_track_to_playlist(p.as_ptr(), t.as_ptr()) as jint
}

#[no_mangle]
pub unsafe extern "system" fn Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1remove_1track_1from_1playlist(
    mut env: JNIEnv,
    _cls: JClass,
    pid: JString,
    tid: JString,
) -> jint {
    let Some(p) = to_cstring(&mut env, &pid) else {
        return -1;
    };
    let Some(t) = to_cstring(&mut env, &tid) else {
        return -1;
    };
    crate::rb_remove_track_from_playlist(p.as_ptr(), t.as_ptr()) as jint
}

bridge_json_str!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1get_1saved_1playlist_1tracks_1json,
    rb_get_saved_playlist_tracks_json
);
bridge_unit_str!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1play_1saved_1playlist,
    rb_play_saved_playlist
);

// ── Smart playlists ─────────────────────────────────────────────────────────

bridge_json!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1get_1smart_1playlists_1json,
    rb_get_smart_playlists_json
);
bridge_json_str!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1get_1smart_1playlist_1tracks_1json,
    rb_get_smart_playlist_tracks_json
);
bridge_unit_str!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1play_1smart_1playlist,
    rb_play_smart_playlist
);

// ── Bluetooth ───────────────────────────────────────────────────────────────

bridge_unit!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1bluetooth_1available,
    rb_bluetooth_available
);
bridge_unit!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1scan_1bluetooth,
    rb_scan_bluetooth
);
bridge_json!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1get_1bluetooth_1devices_1json,
    rb_get_bluetooth_devices_json
);
bridge_unit_str!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1connect_1bluetooth,
    rb_connect_bluetooth
);
bridge_unit_str!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1disconnect_1bluetooth,
    rb_disconnect_bluetooth
);

// ── Streaming ───────────────────────────────────────────────────────────────

bridge_unit!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1subscribe_1status,
    rb_subscribe_status
);
bridge_unit!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1subscribe_1current_1track,
    rb_subscribe_current_track
);
bridge_unit!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1subscribe_1playlist,
    rb_subscribe_playlist
);
bridge_unit!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1subscribe_1library,
    rb_subscribe_library
);
bridge_unit_str!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1subscribe_1discovery,
    rb_subscribe_discovery
);

#[no_mangle]
pub unsafe extern "system" fn Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1poll_1event(
    mut env: JNIEnv,
    _cls: JClass,
    sub_id: jint,
    timeout_ms: jint,
) -> jstring {
    let p = crate::rb_poll_event(sub_id as c_int, timeout_ms as c_int);
    cstr_to_jstring(&mut env, p)
}

bridge_unit_int!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1unsubscribe,
    rb_unsubscribe
);

// ── Discovery service-name constants ────────────────────────────────────────

bridge_json!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1rockbox_1service_1name,
    rb_rockbox_service_name
);
bridge_json!(
    Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1chromecast_1service_1name,
    rb_chromecast_service_name
);
