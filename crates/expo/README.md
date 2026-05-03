# `rockbox-expo` — C-ABI gRPC client for the Expo mobile app

This crate produces a `staticlib` + `cdylib` that wraps the rockboxd gRPC
client (built with [`tonic`]) and exposes a flat C ABI suitable for embedding
in iOS (`.xcframework`) and Android (`jniLibs/<abi>/librockbox_expo.so`)
builds. The companion [Expo Modules wrapper](../../expo/modules/rockbox-rpc/)
loads the resulting library and forwards calls through React Native.

It is the **mobile counterpart** to the desktop client in [`gpui/`](../../gpui/);
the surface mirrors `gpui/src/client.rs` 1:1 wherever it makes sense.

## Why a separate crate?

The `rockbox-rpc` crate (which the rockboxd server uses) pulls in heavy
dependencies — `sqlx`, `typesense`, `library`, `reqwest` with native TLS,
`rocksky`, etc. — that are painful to cross-compile to iOS / Android. This
crate keeps deps minimal:

- `tonic` (transport + codegen + prost), client only
- `tokio` runtime (multi-thread, 2 worker threads)
- `prost`, `serde`, `serde_json`, `once_cell`, `futures-util`
- `rockbox-discovery` for LAN mDNS / Bonjour scans

Proto bindings are generated from `proto/` (a symlink to
`../rpc/proto`) via [`tonic-build`] in `build.rs`, with
`build_server(false)` and a `type_attribute(".", "#[derive(serde::Serialize)]")`
configuration so any response can be JSON-serialized in one line.

## Layout

```
crates/expo/
├── Cargo.toml          staticlib + cdylib, slim deps
├── build.rs            tonic-build (client only) + serde derive on every proto
├── proto -> ../rpc/proto   shared with the rest of the workspace
└── src/lib.rs          runtime, FFI surface, subscriptions
```

## ABI conventions

- All entry points are prefixed `rb_*` and exported with `#[no_mangle]`.
- Unit operations return `i32` — `0` on success, negative on error.
- Reads return `*mut c_char` — heap-owned JSON. Caller **must** free via
  `rb_free_string`. Errors come back as `{ "error": "..." }` JSON objects;
  the platform glue checks for that key and throws.
- Strings flow in as `*const c_char` (NUL-terminated UTF-8); collections
  flow in as JSON-array C strings to keep the FFI narrow.

## Surface map

| Group | Examples |
|-------|----------|
| Init | `rb_set_server_url`, `rb_ping` |
| Playback | `rb_play / pause / play_pause / next / prev`, `rb_seek`, `rb_play_album / play_artist_tracks / play_track / play_directory` |
| Queue | `rb_jump_to_queue_position`, `rb_insert_tracks`, `rb_insert_track_next / last`, `rb_remove_from_queue`, `rb_shuffle_playlist`, `rb_get_playlist_current_json` |
| Library | `rb_get_tracks_json`, `rb_get_artists_json`, `rb_get_album_json`, `rb_search_json`, `rb_like_track / unlike_track`, `rb_get_liked_tracks_json` |
| Sound / Settings | `rb_adjust_volume`, `rb_sound_current_json`, `rb_save_shuffle / save_repeat`, `rb_get_global_settings_json`, `rb_get_global_status_json` |
| Browse | `rb_tree_get_entries_json` |
| Saved playlists | `rb_get_saved_playlists_json`, `rb_create_saved_playlist`, `rb_update_saved_playlist`, `rb_delete_saved_playlist`, `rb_add_track_to_playlist`, `rb_remove_track_from_playlist`, `rb_get_saved_playlist_tracks_json`, `rb_play_saved_playlist` |
| Smart playlists | `rb_get_smart_playlists_json`, `rb_get_smart_playlist_tracks_json`, `rb_play_smart_playlist` |
| Bluetooth | `rb_bluetooth_available`, `rb_get_bluetooth_devices_json`, `rb_connect_bluetooth`, `rb_disconnect_bluetooth` |
| Server-streaming | `rb_subscribe_status`, `rb_subscribe_current_track`, `rb_subscribe_playlist`, `rb_subscribe_library`, `rb_subscribe_discovery(serviceName)` |
| Stream pump | `rb_poll_event(subId, timeoutMs)`, `rb_unsubscribe(subId)` |
| Discovery constants | `rb_rockbox_service_name`, `rb_chromecast_service_name` |
| Memory | `rb_free_string` |

## Streaming subscriptions

Server-streaming RPCs and the mDNS scan share one model:

```text
tonic / mdns-sd stream
  → tokio mpsc<String>           (one queue per subscription)
    → rb_poll_event(id, timeout_ms) -> *mut c_char
       → Swift dispatch_async / Kotlin Dispatchers.IO loop
          → sendEvent("rockbox.<topic>", payload)
```

`rb_subscribe_*` returns an opaque `i32` subscription id. Each event JSON is
the prost message for the topic (e.g. `StatusResponse`, `CurrentTrackResponse`,
`PlaylistResponse`) or a `DiscoveredService` snapshot for the mDNS topic.

Topics: `rockbox.status`, `rockbox.currentTrack`, `rockbox.playlist`,
`rockbox.library`, `rockbox.discovery`. Stream errors propagate as
`{ "error": "..." }` payloads on the same channel; the platform glue
re-emits them on `rockbox.error`.

## Building

Host-only sanity check:

```sh
cargo check -p rockbox-expo
```

Cross-compile for mobile (driven by the [Expo module's build scripts](../../expo/modules/rockbox-rpc/scripts/)):

```sh
# iOS — produces ../../expo/modules/rockbox-rpc/ios/RockboxExpo.xcframework
rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios
( cd ../../expo/modules/rockbox-rpc && bun run build:ios )

# Android — produces ../../expo/modules/rockbox-rpc/android/src/main/jniLibs/<abi>/librockbox_expo.so
cargo install cargo-ndk
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
export ANDROID_NDK_HOME=...   # NDK r25+
( cd ../../expo/modules/rockbox-rpc && bun run build:android )
```

## Adding a new RPC

1. Add a `rb_<name>` wrapper in `src/lib.rs`. For unit ops, use the
   `simple_call!` macro or write `run_unit(async move { ... })`. For reads,
   `unwrap_or_err_string(res.map(|r| r.into_inner()))` does the JSON wrap.
2. Add the matching extern + `Function` / `AsyncFunction` in both
   `expo/modules/rockbox-rpc/ios/RockboxRpcModule.swift` and
   `.../RockboxRpcModule.kt`.
3. Add the typed signature to `expo/modules/rockbox-rpc/src/index.ts` and a
   one-line forwarder on `RockboxClient` in `expo/lib/rockbox-client.ts`.
4. Rebuild the native libs (`build:ios` / `build:android`); Metro doesn't
   pick up native changes automatically.

For server-streaming RPCs, follow the `spawn_stream(...)` pattern, declare
the matching event topic in `Events(...)` on both platforms, register a
`Function("subscribe<Name>")` Function, and add a typed
`subscribe<Name>(cb, onError?)` helper to `expo/lib/rockbox-client.ts`.

## Skipped vs. `gpui/src/client.rs`

The HTTP-REST device endpoints (`fetch_devices`, `connect_device`,
`disconnect_device`) are not gRPC and aren't covered by this crate. The
`run_*_sync` driver loops are also not exposed — the JS side can call the
underlying unary RPCs directly and orchestrate its own caching.

[`tonic`]: https://docs.rs/tonic
[`tonic-build`]: https://docs.rs/tonic-build
