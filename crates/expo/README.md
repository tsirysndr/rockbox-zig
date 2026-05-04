# `rockbox-expo`

The mobile-side Rust crate. Two builds in one workspace:

| Build | Output | Size | Purpose |
|---|---|---|---|
| **Default** (`cargo build -p rockbox-expo`) | `staticlib` + `cdylib` | ~6 MB | Thin tonic gRPC client — controls a remote rockboxd over LAN. iOS, web, and "remote-only" Android variants use this. |
| **`--features embedded-daemon`** (Android only) | `cdylib` | ~48 MB | Full in-process rockboxd: C firmware + 44 statically-linked codecs + all Rust sink crates + gRPC/HTTP/GraphQL/MPD servers + AAudio sink + mDNS discovery. The phone becomes a symmetric peer of any LAN rockboxd. |

The Expo Modules wrapper at [`expo/modules/rockbox-rpc/`](../../expo/modules/rockbox-rpc/)
loads the resulting library and forwards calls through React Native.

It is the **mobile counterpart** to the desktop client in
[`gpui/`](../../gpui/); the surface mirrors `gpui/src/client.rs` 1:1
wherever it makes sense.

---

## Why a separate crate?

`rockbox-rpc` (which the rockboxd server uses) pulls in `sqlx`, `typesense`,
`library`, `reqwest` with native TLS, `rocksky`, etc. — painful to
cross-compile to iOS / Android. The default build of `rockbox-expo` keeps
deps minimal:

- `tonic` (transport + codegen + prost), client only
- `tokio` runtime (multi-thread, 2 worker threads)
- `prost`, `serde`, `serde_json`, `once_cell`, `futures-util`
- `rockbox-discovery` for LAN mDNS / Bonjour scans

Proto bindings are generated from `proto/` (a symlink to `../rpc/proto`)
via [`tonic-build`] in `build.rs`, with `build_server(false)` and a
`type_attribute(".", "#[derive(serde::Serialize)]")` so any response can
be JSON-serialized in one line.

The `embedded-daemon` feature pulls the heavy deps in (rockbox-server,
rockbox-library with `fts5`, all the PCM sink crates, etc.) and links
the whole thing alongside the C firmware archives.

---

## Layout

```
crates/expo/
├── Cargo.toml              staticlib + cdylib, slim base + embedded-daemon feature
├── build.rs                tonic-build (client only) + firmware archive linking
├── proto -> ../rpc/proto   shared with the rest of the workspace
└── src/
    ├── lib.rs              gRPC client surface (rb_play, rb_pause, …)
    ├── daemon.rs           [embedded-daemon only] firmware boot + auto-scan
    └── jni_bridge.rs       per-rb_* JNI shims (Java_…_rb_1*)
```

---

## ABI conventions

- All entry points are prefixed `rb_*` and exported with `#[no_mangle]`.
- Unit operations return `i32` — `0` on success, negative on error.
- Reads return `*mut c_char` — heap-owned JSON. Caller **must** free via
  `rb_free_string`. Errors come back as `{ "error": "..." }` JSON objects;
  the platform glue checks for that key and throws.
- Strings flow in as `*const c_char` (NUL-terminated UTF-8); collections
  flow in as JSON-array C strings to keep the FFI narrow.
- Negative return codes used for daemon ops:
  - `-22` invalid input (null / non-UTF-8)
  - `-38` not built (`-DEMBEDDED_DAEMON` not set — remote-only build)
  - `-110` timeout (gRPC didn't bind within deadline)
  - `-114` already starting / running (idempotent)

---

## Surface map

| Group | Examples |
|-------|----------|
| Init | `rb_set_server_url`, `rb_set_http_url`, `rb_ping` |
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
| **Embedded daemon** | `rb_daemon_start(configDir, musicDir, deviceName)`, `rb_daemon_port`, `rb_daemon_state`, `rb_rescan_library` |

---

## Streaming subscriptions

Server-streaming RPCs and the mDNS scan share one model:

```text
tonic / mdns-sd stream
  → tokio mpsc<String>             (one queue per subscription)
    → rb_poll_event(id, timeout_ms) -> *mut c_char
       → Swift dispatch_async / Kotlin Dispatchers.IO loop
          → sendEvent("rockbox.<topic>", payload)
```

`rb_subscribe_*` returns an opaque `i32` subscription id. Each event JSON
is the prost message for the topic (e.g. `StatusResponse`, `CurrentTrackResponse`,
`PlaylistResponse`) or a `DiscoveredService` snapshot for the mDNS topic.

Topics: `rockbox.status`, `rockbox.currentTrack`, `rockbox.playlist`,
`rockbox.library`, `rockbox.discovery`. Stream errors propagate as
`{ "error": "..." }` payloads on the same channel; the platform glue
re-emits them on `rockbox.error`.

---

## Embedded daemon (Android cdylib only)

When built with `--features embedded-daemon`, the .so contains the entire
rockboxd:

```
┌────────────────────── librockbox_expo.so ───────────────────────────┐
│                                                                     │
│   JNI bridges (Java_…_rb_1*)                                        │
│         │                                                           │
│   ┌─────▼──────┐    ┌──────────────┐    ┌────────────────────┐      │
│   │ tonic gRPC │    │ daemon.rs    │    │ rockbox-server     │      │
│   │ client     │───▶│ rb_daemon_*  │───▶│ start_servers()    │      │
│   │ (lib.rs)   │    │              │    │   • gRPC :6061     │      │
│   └────────────┘    │ main_c() ────┼──┐ │   • HTTP :6063     │      │
│                     │              │  │ │   • GraphQL :6062  │      │
│                     └──────────────┘  │ │   • MPD :6600      │      │
│                                       │ │   • mDNS advertise │      │
│                                       │ └────────────────────┘      │
│                                       ▼                             │
│                          ┌─────────────────────────────────┐        │
│                          │ C firmware (apps/, firmware/)   │        │
│                          │   • playback engine             │        │
│                          │   • metadata + buffering        │        │
│                          │   • 44 statically-linked codecs │        │
│                          │   • DSP + replay-gain           │        │
│                          │   • PCM sinks → AAudio          │        │
│                          └────────┬────────────────────────┘        │
│                                   ▼                                 │
│                       ┌─────────────────────────┐                   │
│                       │ Android system libs     │                   │
│                       │ libaaudio, liblog,      │                   │
│                       │ libandroid              │                   │
│                       └─────────────────────────┘                   │
└─────────────────────────────────────────────────────────────────────┘
```

### Boot sequence

1. App launches → Expo modules initialize → `RockboxRpcModule.OnCreate`
   fires (Kotlin).
2. Kotlin calls `rb_daemon_start(configDir, musicDir, deviceName)` via JNI:
   - `configDir` = `applicationContext.filesDir` (`/data/user/0/.../files`)
   - `musicDir`  = `Environment.getExternalStoragePublicDirectory(DIRECTORY_MUSIC)`
     (`/storage/emulated/0/Music`)
   - `deviceName` = `android.os.Build.MODEL`
3. `daemon.rs::rb_daemon_start`:
   - Installs `tracing-android` subscriber → tag `rockbox`
   - Sets env vars: `HOME`, `ROCKBOX_LIBRARY` (canonical, NOT `ROCKBOX_MUSIC_DIR`),
     `ROCKBOX_DEVICE_NAME`, `ROCKBOX_PORT/GRAPHQL_PORT/TCP_PORT/MPD_PORT`
   - Spawns `rockbox-engine` pthread (2 MB stack) which calls `main_c()`
4. `main_c()` — the firmware boot in `apps/main.c`. Initializes kernel,
   threads, audio engine. Spawns:
   - **server thread** → `crates/server::start_servers()` — binds gRPC :6061,
     HTTP :6063, GraphQL :6062, MPD :6600
   - **broker thread** → `crates/server::start_broker()` — internal command bus
5. `rb_daemon_start` waits up to 30 s for gRPC :6061 to bind (TCP probe
   to localhost), then:
   - Calls `rb_set_server_url("http://127.0.0.1:6061")` so the in-process
     tonic client targets our own daemon
   - Calls `rb_set_http_url("http://127.0.0.1:6063")` for the REST surface
   - Spawns the **library scan thread**: opens the SQLite DB, and if it's
     empty (or `ROCKBOX_UPDATE_LIBRARY=1`), runs
     `rockbox_library::audio_scan::scan_audio_files($ROCKBOX_LIBRARY)`
6. Returns the gRPC port (`6061`) to JNI; Kotlin logs it.

Subsequent `rb_daemon_start` calls return `-114` immediately (idempotent).

### Force keepalives

The cdylib link uses `--gc-sections`, and rustc dead-code-strips rlibs
that aren't visibly referenced. Each PCM sink crate (`rockbox-airplay`,
`rockbox-slim`, `rockbox-chromecast`, `rockbox-upnp`) and `rbnetstream`
ships C-ABI exports the firmware needs (`pcm_airplay_*`, `rb_net_open`,
etc.) but rustc would strip them along with their crate.

`daemon.rs` defends against this with `#[used] static _KEEPALIVE_*`
constants that take the address of one real C-ABI fn from each crate —
that's enough to pin the whole crate's `#[no_mangle]` set into the link.
There's also a similar shim for the rockbox-server crate and for
`start_server` / `start_servers` / `start_broker`.

If a sink stops working after a refactor, check for missing keepalives.

### C firmware artefacts

The `embedded-daemon` build links these archives produced by the
`build-android-arm64/` Make tree (driven by
`tools/configure --target=205`):

- `librockbox.a`, `firmware/libfirmware.a`, `lib/librbcodec.a`
- `lib/libfixedpoint.a`, `lib/libtlsf.a`, `lib/libskin_parser.a`
- 44 codec entry-point archives (`flac.a`, `mpa.a`, `opus.a`, …) —
  bare-named, linked via Cargo's `+verbatim` modifier
- ~30 codec helper libraries (`libfaad.a`, `libffmpegFLAC.a`, `libmad.a`, …)

Linker arg `-Wl,-z,muldefs` is set in `build.rs` to tolerate duplicate
ogg symbols across vorbis/opus/speex/tremor (each codec ships its own
copy of libogg).

The cdylib-specific firmware sources live under
`firmware/target/hosted/android/cdylib/`:

| File | Role |
|---|---|
| `system-android.c` | Headless system_init + power_off + stdout/stderr→logcat shim |
| `pcm-aaudio.c` | AAudio PCM sink (replaces SDL audio) |
| `lc-android.c` | `lc_open()` / `lc_get_header()` over the static `lc_static_table[]` |
| `rb_zig_compat.c` | C compat layer for the 18 `rb_*` symbols `crates/sys` would otherwise pull from `zig/src/main.zig` |
| `lcd-noop.c`, `button-noop.c`, `cpuinfo-noop.c`, `audiohw-noop.c` | Stubs — UI lives in React Native, not on the device LCD |

---

## Build commands

### Host-only sanity check (no firmware deps)

```sh
cargo check -p rockbox-expo
```

### iOS — remote-only client

Produces `expo/modules/rockbox-rpc/ios/RockboxExpo.xcframework`:

```sh
rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios
( cd expo/modules/rockbox-rpc && bun run build:ios )
```

iOS doesn't currently ship the embedded daemon (the firmware tree
isn't yet cross-compiled to Apple Silicon). The Swift module's daemon
externs return `-38` (ENOSYS).

### Android — embedded daemon (default)

Produces `expo/modules/rockbox-rpc/android/src/main/jniLibs/<abi>/librockbox_expo.so`.
Two stages: first build the C firmware archives into `build-android-arm64/`,
then cross-compile the Rust cdylib that links them together.

#### Prereqs (one-time per machine)

```sh
# Android NDK r25+ (r27.1 verified). Either install via Android Studio's
# SDK Manager → SDK Tools → NDK (Side by side), or download from
# https://developer.android.com/ndk/downloads
export ANDROID_NDK_HOME=$HOME/Library/Android/sdk/ndk/27.1.12297006   # macOS path
# or e.g. $HOME/Android/Sdk/ndk/27.1.12297006 on Linux

# Rust cross-compile target + cargo-ndk
rustup target add aarch64-linux-android
cargo install cargo-ndk
```

Optional but highly recommended: add the matching `armeabi-v7a` and
`x86_64` targets if you need to ship for those ABIs too. The build script
currently only builds `arm64-v8a` because that's the only `build-android-<abi>`
firmware tree that's pre-configured.

#### Stage 1: configure + build the firmware archives

The firmware uses Rockbox's autotools-style `configure` script. Run it
**once** to generate `build-android-arm64/Makefile`, then `make` for any
C-side edit afterwards.

```sh
# One-time: configure the firmware build dir for the Android cdylib target.
# Target 205 = androidcdylib (model name) = our headless cdylib target.
# Default ABI is arm64-v8a, default API level is 26 (required for AAudio).
mkdir -p build-android-arm64
cd build-android-arm64
../tools/configure \
  --target=androidcdylib \
  --type=N \
  --ram=256 \
  --rbdir=/.rockbox
cd ..
```

The configure script reads two env vars when it sees the androidcdylib
target — set them before running if you need non-default values:

| Var | Default | Purpose |
|---|---|---|
| `ANDROID_NDK_HOME` | _(none — required)_ | Path to NDK install root |
| `ANDROID_TARGET_ABI` | `arm64-v8a` | One of `arm64-v8a` / `armeabi-v7a` / `x86_64` |
| `ANDROID_API_LEVEL` | `26` | Minimum SDK; **don't go below 26** (AAudio requires it) |

For a 32-bit ARM build, e.g.:

```sh
mkdir -p build-android-armv7
cd build-android-armv7
ANDROID_TARGET_ABI=armeabi-v7a ANDROID_API_LEVEL=26 \
  ../tools/configure --target=androidcdylib --type=N --ram=256 --rbdir=/.rockbox
```

…then build the archives. **Re-run after any C-side change**:

```sh
( cd build-android-arm64 && make -j8 )
```

Outputs (consumed by `crates/expo/build.rs` at link time):

```
build-android-arm64/
├── librockbox.a               apps/ + most of firmware/
├── firmware/libfirmware.a     low-level firmware glue
├── lib/librbcodec.a           rbcodec (metadata + DSP)
├── lib/libfixedpoint.a        fixed-point math
├── lib/libtlsf.a              memory allocator
├── lib/libskin_parser.a       skin / theme parser (vestigial)
└── lib/rbcodec/codecs/
    ├── flac.a, mpa.a, opus.a, …    44 codec entry-point archives (bare-named)
    └── libffmpegFLAC.a, libfaad.a, libmad.a, …   ~30 codec helper libs
```

The Make build is **incremental** — touch one C file and `make` rebuilds
just that .o and re-archives the affected .a. If a header in
`firmware/export/config/androidcdylib.h` changes (or any other broadly-included
header), most of the tree recompiles.

#### Stage 2: build the cdylib

```sh
PROFILE=release bash expo/modules/rockbox-rpc/scripts/build-android.sh
```

What the script does:

```sh
cargo ndk \
  -t arm64-v8a \
  --platform 26 \                 # AAudio requires API 26
  -o expo/modules/rockbox-rpc/android/src/main/jniLibs \
  build -p rockbox-expo \
        --features embedded-daemon \
        --release
```

`--features embedded-daemon` is the critical flag — without it, the cdylib
is the 6 MB remote-only tonic client (no firmware linked in), and Kotlin
will log `embedded daemon not built into this .so (remote-only mode)` at
boot. Override the feature set if you want a remote-only build for fast
JS iteration:

```sh
FEATURES="" bash expo/modules/rockbox-rpc/scripts/build-android.sh
```

The Rust crate's `build.rs` automatically picks up the firmware archives
from `build-android-arm64/` via the `cargo:rerun-if-changed=…/static-libs.stamp`
directive — touching the firmware causes the cdylib to relink on the next
cargo invocation. Override the firmware dir with
`ROCKBOX_FIRMWARE_DIR=/elsewhere`.

Verify the build:

```sh
ls -lh expo/modules/rockbox-rpc/android/src/main/jniLibs/arm64-v8a/librockbox_expo.so
# embedded-daemon: ~48 MB
# remote-only:     ~6 MB

# Spot-check that the daemon entry points are exported:
$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin/llvm-nm \
  expo/modules/rockbox-rpc/android/src/main/jniLibs/arm64-v8a/librockbox_expo.so \
  | grep -E " T (rb_daemon_start|main_c|server_init|start_servers)$"
```

Expected output:

```
... T main_c
... T rb_daemon_start
... T server_init
... T start_servers
```

If `main_c` is missing, the firmware archives didn't get linked — verify
`--features embedded-daemon` was actually passed and that
`build-android-arm64/static-libs.stamp` exists.

#### Quick full rebuild (after pulling)

```sh
# Refresh firmware first (incremental)
( cd build-android-arm64 && make -j8 )

# Then relink the cdylib
PROFILE=release bash expo/modules/rockbox-rpc/scripts/build-android.sh

# Bundle into the app
( cd expo && bunx expo run:android )
```

If `build-android-arm64/` doesn't exist yet (fresh clone), do the
`tools/configure` step from "Stage 1" first.

### Android — remote-only client (no firmware)

Override the script's default to build the lighter client (useful for
fast JS iteration when the daemon work isn't needed):

```sh
FEATURES="" bash expo/modules/rockbox-rpc/scripts/build-android.sh
```

### Bundle into the Expo app

```sh
( cd expo && bunx expo run:android )         # or run:ios
```

`bunx expo prebuild` regenerates `android/`. Avoid `prebuild --clean`
on Android — the manifest is hand-edited (`MANAGE_EXTERNAL_STORAGE`,
`xmlns:tools`) and `--clean` will wipe those edits.

---

## Adding a new RPC

1. Add a `rb_<name>` wrapper in `src/lib.rs`. For unit ops, use the
   `simple_call!` macro or write `run_unit(async move { ... })`. For reads,
   `unwrap_or_err_string(res.map(|r| r.into_inner()))` does the JSON wrap.
2. Add a JNI bridge in `src/jni_bridge.rs`:
   `Java_expo_modules_rockboxrpc_RockboxRpcModule_rb_1<name>`. Note the
   JNI mangling — `_` in Java method names becomes `_1` in the C symbol.
3. Add the matching extern + `Function` / `AsyncFunction` in both
   `expo/modules/rockbox-rpc/ios/RockboxRpcModule.swift` and
   `.../RockboxRpcModule.kt`.
4. Add the typed signature to `expo/modules/rockbox-rpc/src/index.ts` and a
   one-line forwarder on `RockboxClient` in `expo/lib/rockbox-client.ts`.
5. Rebuild the native libs (`build:ios` / `build:android`); Metro doesn't
   pick up native changes automatically.

For server-streaming RPCs, follow the `spawn_stream(...)` pattern, declare
the matching event topic in `Events(...)` on both platforms, register a
`Function("subscribe<Name>")` Function, and add a typed
`subscribe<Name>(cb, onError?)` helper to `expo/lib/rockbox-client.ts`.

---

## Logging & diagnostics

Tag map:

| logcat tag | Source |
|---|---|
| `rockbox` | Rust `tracing::*` calls (default level: per-crate `debug`, see `daemon.rs::install_logcat_subscriber`) |
| `Rockbox` | C firmware `printf`/`fprintf` and `DEBUGF`/`logf`/`panicf` (routed via `debug-android.c` and the stdout/stderr pipe in `system-android.c`) |
| `rockbox-engine` | `system-android.c` boot diagnostics (cgroup/SELinux denials, etc.) |
| `rb-system-android`, `rb-pcm-aaudio` | other cdylib C tags |
| `RockboxRpc` | Kotlin Log calls in `RockboxRpcModule.kt` |
| `RockboxNowPlaying` | Kotlin Log calls in `NowPlayingService.kt` |

Quick capture recipe:

```sh
PID=$(adb shell pidof com.tsirysndr.Rockbox)
adb logcat -c
adb logcat -v time --pid=$PID
```

Override Rust verbosity at runtime by setting `RUST_LOG` BEFORE the daemon
starts (e.g. `setprop log.tag.rockbox D` is consulted on next boot).

---

## Known pitfalls (also in `MEMORY.md`)

| Symptom | Cause | Fix |
|---|---|---|
| `embedded daemon not built into this .so (remote-only mode)` | Build script ran without `--features embedded-daemon` | Use `expo/modules/rockbox-rpc/scripts/build-android.sh` (defaults to enabled) |
| `dlopen failed: cannot locate symbol "server_init"` | `CONFIG_SERVER` not defined → `apps/SOURCES` skips `server_thread.c` compilation | Set both `ROCKBOX_SERVER` and `CONFIG_SERVER` in `androidcdylib.h` |
| `Codec: cannot read file` for every track | Codec naming uses Java-shell `libNAME.so` convention but `lc_static_table[]` has bare `<name>.codec` entries | Gate the `libNAME.so` override in `lib/rbcodec/metadata/metadata.h` on `!CODECS_STATIC` |
| SIGSEGV in `init_mad` (or any codec init) at small fault address | `ci` symbol collision: 264-byte struct (codecs.c) merged into 8-byte pointer storage (codec_crt0.c) | Firmware-side rename: `firmware_ci` for the struct, `ci` for the pointer (both 8 bytes, same type) |
| Audio plays at chipmunk speed (~9 % too fast) | `pcm_sink::set_freq` receives an INDEX into `hw_freq_sampr[]`, not Hz; AAudio gets opened at "4 Hz", silently falls back to 48 kHz | `pcm-aaudio.c::sink_set_freq` looks up `hw_freq_sampr[freq_index]` first |
| `ForegroundServiceStartNotAllowedException` on play | Android 14+ blocks `startForegroundService` from background process state (`uidState: SVC`) even with `mediaPlayback` type | `RockboxNowPlayingModule.startServiceCompat` and `NowPlayingService.refreshNotification` check `ActivityManager.getMyMemoryState().importance` before promoting |
| ENOENT when the GraphQL `treeGetEntries` resolver browses `Music` | Daemon set `ROCKBOX_MUSIC_DIR` but the resolvers read `ROCKBOX_LIBRARY` | Set `ROCKBOX_LIBRARY` in `daemon.rs::configure_environment` |
| Library DB stays empty even after browsing works | Embedded daemon doesn't run the desktop CLI's startup scan | `daemon.rs::spawn_library_scan` runs after gRPC binds; force re-scan via `RockboxClient.rescanLibrary()` |
| `Permission denied` reading `/storage/emulated/0/Music` on API 33+ | `READ_EXTERNAL_STORAGE` is ignored on `targetSdk=33+`; `READ_MEDIA_AUDIO` only grants MediaStore queries | `MANAGE_EXTERNAL_STORAGE` in manifest + `useAllFilesAccessPrompt()` opens system Settings |
| Daemon dies after the app backgrounds for a few minutes | App process killed for memory; daemon dies with it | NowPlayingService is a foreground service — keep it running via `RockboxNowPlaying.start()` at app launch (called from `_layout.tsx`) |

---

## Skipped vs. `gpui/src/client.rs`

The HTTP-REST device endpoints (`fetch_devices`, `connect_device`,
`disconnect_device`) are not gRPC and aren't covered by this crate. The
`run_*_sync` driver loops are also not exposed — the JS side can call the
underlying unary RPCs directly and orchestrate its own caching.

[`tonic`]: https://docs.rs/tonic
[`tonic-build`]: https://docs.rs/tonic-build
