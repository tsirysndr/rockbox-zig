# CLAUDE.md — Rockbox Zig

## Project overview

Rockbox Zig is a modern wrapper around the [Rockbox](https://www.rockbox.org) open-source audio player firmware. It adds Rust/Zig services on top of the C firmware to expose gRPC, GraphQL, HTTP, and MPD APIs, a Typesense-backed search engine, Chromecast/AirPlay/Snapcast/Squeezelite output sinks, and a desktop/web UI.

The binary is called **`rockboxd`**. It is a single executable built by Zig that links:
- The Rockbox C firmware (compiled by Make into `build-lib/libfirmware.a` and friends)
- Rust crates (compiled by Cargo into `target/release/librockbox_cli.a` and `librockbox_server.a`)
- SDL2 for audio/event handling on the host platform

## Repository layout

```
firmware/          Rockbox C firmware (audio engine, codecs, DSP)
apps/              Rockbox application layer (playlist, database, plugins)
lib/               Codec libraries (rbcodec, fixedpoint, skin_parser, tlsf)
build-lib/         Out-of-tree Make build directory (generated; do not edit)
crates/            Rust workspace
  airplay/         ALAC encoder + RAOP/RTP sender (AirPlay 1 output)
  slim/            Slim Protocol + HTTP broadcast server (Squeezelite multi-room output)
  cli/             Entry point compiled to librockbox_cli.a (staticlib)
  server/          gRPC / HTTP server
  settings/        load_settings() — reads settings.toml, applies sinks
  sys/             FFI bindings to the C firmware (unsafe extern "C")
  library/         Audio file scanning and SQLite library management
  typesense/       Typesense client for fast music search
  netstream/       HTTP streaming (Range-request based fd multiplexing)
  chromecast/      Chromecast output
  rpc/             gRPC definitions / generated code
  graphql/         GraphQL schema and resolvers
  mpd/             MPD protocol server
  mpris/           MPRIS D-Bus integration
  tracklist/       Playlist / tracklist management
  types/           Shared Rust types
  traits/          Shared Rust traits
zig/               Zig build script and thin main.zig entry point
gpui/              Desktop client (GPUI / Rust) — reference UI for the mobile app
expo/              React Native / Expo mobile app (see `expo/CLAUDE.md` rules below)
```

## Build system

### Step 1 — C firmware (Make)
```sh
cd build-lib
make lib          # builds libfirmware.a, librockbox.a, codec libs
```
The `build-lib/` directory was pre-configured via Rockbox's `tools/configure` for the `sdlapp` target. Do **not** run `configure` again unless you know what you're doing — it regenerates the Makefile and overwrites any local edits.

### Step 2 — Rust crates (Cargo)
```sh
cargo build --release -p rockbox-cli     # produces target/release/librockbox_cli.a
cargo build --release -p rockbox-server  # produces target/release/librockbox_server.a
```
Both crates have `crate-type = ["staticlib"]`. All transitive rlib dependencies are bundled into the `.a`.

### Step 3 — Zig linker
```sh
cd zig
zig build          # links everything into zig-out/bin/rockboxd
```

### Quick full rebuild
```sh
cd build-lib && make lib && cd ..
cargo build --release -p rockbox-cli -p rockbox-server
cd zig && zig build
```

### Critical: stale binary pitfall
`zig build` only re-links if the `.a` files are newer than the binary. After changing C code, always run `make lib` first. After changing Rust code, run `cargo build --release`. If behavior doesn't match the code, check timestamps:
```sh
ls -la zig/zig-out/bin/rockboxd build-lib/libfirmware.a target/release/librockbox_cli.a
```

## Runtime configuration

Settings file: `~/.config/rockbox.org/settings.toml`

```toml
music_dir = "/path/to/Music"

# Audio output — pick one:
audio_output = "builtin"      # SDL audio (default)

audio_output = "fifo"
fifo_path = "/tmp/snapfifo"   # named FIFO for Snapcast; use "-" for stdout

audio_output = "airplay"
airplay_host = "192.168.1.x"  # RAOP receiver IP
airplay_port = 5000            # optional, default 5000

audio_output = "squeezelite"
squeezelite_port = 3483        # optional, Slim Protocol port (default 3483)
squeezelite_http_port = 9999   # optional, HTTP PCM stream port (default 9999)
```

Run one or more squeezelite clients pointing at rockboxd for multi-room:
```sh
squeezelite -s localhost -n "Living Room"
squeezelite -s localhost -n "Kitchen"
```

## PCM sink architecture

The audio output abstraction lives in `firmware/export/pcm_sink.h`. Each sink implements `struct pcm_sink_ops` (init / postinit / set_freq / lock / unlock / play / stop).

| Enum constant      | Value | Implementation file                         |
|--------------------|-------|---------------------------------------------|
| `PCM_SINK_BUILTIN`     | 0 | `firmware/target/hosted/sdl/pcm-sdl.c`        |
| `PCM_SINK_FIFO`        | 1 | `firmware/target/hosted/pcm-fifo.c`           |
| `PCM_SINK_AIRPLAY`     | 2 | `firmware/target/hosted/pcm-airplay.c`        |
| `PCM_SINK_SQUEEZELITE` | 3 | `firmware/target/hosted/pcm-squeezelite.c`    |

`crates/settings/src/lib.rs:load_settings()` reads `audio_output` and calls `pcm::switch_sink()`.

Rust constants + helpers live in `crates/sys/src/sound/pcm.rs`.

### FIFO sink (Snapcast)
- Pre-creates the named FIFO with `O_RDWR|O_NONBLOCK` in `pcm_fifo_set_path()` then clears `O_NONBLOCK`, so a permanent writer reference is held — readers never see premature EOF between tracks.
- `sink_dma_stop()` does NOT close the fd; it stays open across track transitions.
- **Startup order matters**: rockboxd must start before snapserver. If snapserver opens the FIFO first it may get EOF and stop reading.
- On macOS, snapserver v0.35.0 ignores the `-s` sample-format CLI flag; use `/usr/local/etc/snapserver.conf`:
  ```ini
  [stream]
  source = pipe:///tmp/snapfifo?name=default&sampleformat=44100:16:2
  ```

### AirPlay sink (RAOP)
- `crates/airplay/` implements the full RAOP stack in pure Rust (no tokio needed).
  - `alac.rs` — ALAC escape/verbatim frame encoder: 352 stereo S16LE samples → 1411-byte bitstream
  - `rtp.rs` — RTP/UDP packet sender; RTCP NTP sync packets sent every ~44 frames
  - `rtsp.rs` — synchronous RTSP client: ANNOUNCE (SDP) → SETUP → RECORD
- `pcm_airplay_connect()` is called once per `sink_dma_start()` (idempotent if already connected).
- The `rockbox-airplay` rlib must be force-included in `librockbox_cli.a` via the `use rockbox_airplay::_link_airplay as _` shim in `crates/cli/src/lib.rs`.

### Squeezelite sink (Slim Protocol + HTTP broadcast)
- `crates/slim/` implements a Slim Protocol TCP server and an HTTP PCM broadcast server, both in pure Rust.
  - `slimproto.rs` — accepts squeezelite connections; sends `STRM 's'` pointing at the HTTP port; replies to every `STMt` heartbeat with `audg` to prevent squeezelite's 36-second watchdog from firing.
  - `http.rs` — concurrent HTTP server (one thread per client); each client gets an independent `BroadcastReceiver` cursor into the shared buffer, enabling true multi-room playback.
  - `lib.rs` — `BroadcastBuffer`: sequence-numbered chunks, per-reader cursors, 4 MB cap with oldest-first eviction; lagging readers skip forward rather than blocking the writer.
- `firmware/target/hosted/pcm-squeezelite.c` paces the DMA loop to real time using `CLOCK_MONOTONIC`. **Use `int64_t` for the nanosecond diff** — unsigned subtraction wraps catastrophically when `tv_nsec` rolls over.
- The `rockbox-slim` rlib must be force-included via `use rockbox_slim::_link_slim as _` in `crates/cli/src/lib.rs`.
- **Slim Protocol framing**: client→server is `opcode[4] + u32_t length BE + payload`; server→client is `u16_t length BE + opcode[4] + payload` (length does NOT include the 2-byte length field itself).
- **ASCII-encoded PCM fields in STRM**: squeezelite subtracts `'0'` from `pcm_sample_size`, `pcm_sample_rate`, `pcm_channels`, `pcm_endianness`. Correct values: `'1'` (16-bit), `'3'` (44100 Hz), `'2'` (stereo), `'1'` (little-endian).

## Key cross-cutting concerns

### macOS SDL audio
`SDL_InitSubSystem(SDL_INIT_AUDIO)` must be called explicitly on macOS because the SDL event thread (which normally does it) is `#ifndef __APPLE__`. This is done in `firmware/target/hosted/sdl/system-sdl.c` in the `#else` branch of the event-thread guard.

### SIGTERM handling
`system-hosted.c` installs a SIGTERM handler that loops forever (waits for SDL quit event). `crates/cli/src/lib.rs` overrides SIGTERM/SIGINT with a handler that kills the typesense child and calls `_exit(0)`.

### Typesense subprocess
Spawned in `crates/cli/src/lib.rs` with `Stdio::piped()`. stdout/stderr lines are forwarded to `tracing::debug!`/`tracing::warn!` in background threads, keeping the PCM stdout stream clean in FIFO mode.

### Logging — use `tracing`, never `eprintln!`/`println!`
All Rust logging must use the `tracing` crate (`tracing::debug!`, `tracing::info!`, `tracing::warn!`, `tracing::error!`). **Never use `eprintln!` or `println!` for diagnostic output** in Rust code — they bypass the structured log filter, pollute stdout (breaking FIFO/pipe mode), and can't be silenced at runtime.

Severity guide:
- `tracing::error!` — unrecoverable failures (connection refused, missing config)
- `tracing::warn!` — recoverable issues (non-fatal fallbacks, unexpected-but-handled states)
- `tracing::info!` — notable lifecycle events (session established, device paired)
- `tracing::debug!` — per-packet/per-frame detail, protocol negotiation steps

`tracing` is declared as a workspace dependency in the root `Cargo.toml`; add `tracing = { workspace = true }` to any crate that needs it. Control verbosity at runtime with `RUST_LOG`, e.g. `RUST_LOG=debug rockboxd` or `RUST_LOG=rockbox_airplay=debug,info`.

### HTTP streaming
HTTP fds are encoded as values `<= STREAM_HTTP_FD_BASE (-1000)`. `stream_open/read/lseek/close` in `crates/netstream/` dispatch between HTTP and POSIX based on fd value. The global `STREAMS` map holds `Arc<Mutex<StreamState>>` per handle so concurrent reads don't serialize on a single lock.

## Adding a new PCM sink

1. Create `firmware/target/hosted/pcm-<name>.c` — model on `pcm-fifo.c`.
2. Add `PCM_SINK_<NAME>` to the enum in `firmware/export/pcm_sink.h`.
3. Register `&<name>_pcm_sink` in the `sinks[]` array in `firmware/pcm.c`.
4. Add `target/hosted/pcm-<name>.c` inside the `#if PLATFORM_HOSTED` block in `firmware/SOURCES`.
5. Add Rust constant `PCM_SINK_<NAME>: i32` in `crates/sys/src/sound/pcm.rs`.
6. Add a `set_<name>_*` wrapper if configuration is needed.
7. Handle in `crates/settings/src/lib.rs:load_settings()`.
8. If the sink has a Rust implementation in a new crate: add a `_link_<name>()` dummy fn and reference it from `crates/cli/src/lib.rs` to force inclusion in the staticlib.

## Mobile app (`expo/`)

A React Native client (Expo Router + NativeWind) lives in `expo/`. It mirrors
the GPUI desktop layout (`gpui/src/ui/`) — same dark palette, same
Spotify/Tidal-inspired information architecture: bottom-tab shell with a
persistent miniplayer, full-screen player modal, queue modal, and detail
screens for album / artist / playlist / genre. Most state is mock-only today;
real data should plug into the rockboxd gRPC / GraphQL client (`crates/server/`).

### Stack
- **Expo SDK 54** + **expo-router** for file-based routing (`app/`).
- **NativeWind 4** with Tailwind 3 — class-based styling against a custom palette
  declared in both `expo/tailwind.config.js` and `expo/constants/theme.ts`
  (keep the two in sync).
- `expo-image`, `expo-blur`, `expo-linear-gradient`, `@expo/vector-icons`
  (Ionicons + MaterialCommunityIcons), `react-native-safe-area-context`.

### Layout
```
expo/
├── app/
│   ├── _layout.tsx                 root stack, fonts, PlayerProvider, modals
│   ├── (tabs)/_layout.tsx          custom tab bar with merged miniplayer dock
│   ├── (tabs)/{index,search,library}.tsx
│   ├── player.tsx, queue.tsx, settings.tsx
│   ├── album/[id].tsx, artist/[id].tsx, playlist/[id].tsx, genre/[id].tsx
│   └── playlist/new.tsx            create regular OR smart (?mode=smart)
├── components/                     mini-player, action-sheet, track-context-menu, …
├── lib/
│   ├── player-context.tsx          single source of truth for playback state
│   ├── mock-data.ts                ALBUMS / ARTISTS / PLAYLISTS / GENRES + helpers
│   └── nativewind-setup.ts         cssInterop registrations (must be imported)
├── constants/theme.ts              `Colors` palette consumed by inline styles
├── tailwind.config.js              `Colors` palette mirrored as Tailwind tokens
├── babel.config.js                 babel-preset-expo + nativewind/babel
└── metro.config.js                 withNativeWind({ input: './global.css' })
```

### Styling rules — NativeWind only

- **Always use `className` for styling.** Inline `style={{...}}` is reserved for
  values className genuinely cannot express: `Animated.Value` bindings,
  runtime-computed widths (`` `${pct * 100}%` ``), per-instance shadow tokens,
  or colors derived from data (e.g. `genre.color`).
- **Never combine a function `style={(state) => ({...})}` with `className` on
  the same element.** The function `style` overrides NativeWind's class output
  and silently drops every utility on that element. Use arbitrary-value classes
  (`w-[48.5%]`, `h-[100px]`, `aspect-square`) and the `active:` variant for
  press feedback instead.
- Static `style={{...}}` objects (no callbacks) merge fine with `className`.
- Color tokens live in `tailwind.config.js` under `bg.*`, `accent.*`, `text.*`,
  `border`, `divider`, `slider.*`, `danger`. Reach for these (`bg-bg-card`,
  `text-text-secondary`, `bg-accent`) instead of hard-coded hex values.
- `expo-image`, `expo-blur`, `expo-linear-gradient`, and the safe-area
  `SafeAreaView` are wired up via `cssInterop` in `lib/nativewind-setup.ts` —
  any other third-party component needs to be registered there before it can
  accept `className`.
- Fonts: `font-sans` → SpaceGrotesk (UI), `font-mono` → JetBrainsMono
  (durations / numerics). The TTFs are bundled via the `expo-font` plugin in
  `app.json` and copied from `gpui/assets/fonts/`.

### Player state
`lib/player-context.tsx` holds queue, currentIdx, position (1 Hz tick),
isPlaying, shuffle, repeat, liked, userPlaylists, and the global track / entity
context-menu state. The mock advances `position` and auto-advances tracks; the
real implementation should replace the action handlers with rockboxd RPC calls
while keeping the same shape so the UI doesn't need to change.

### Useful commands
```sh
cd expo
bun install                        # or npm/yarn
bun run start                      # iOS / Android / web via expo-router
bunx tsc --noEmit                  # type check
bunx expo lint                     # lint
bunx expo export --platform web    # smoke-test the bundle (catches NativeWind transform issues)
```

## Useful commands

```sh
# Run the daemon
./zig/zig-out/bin/rockboxd

# Run with AirPlay debug logging
RUST_LOG=debug ./zig/zig-out/bin/rockboxd

# Test FIFO → stdout pipe
./zig/zig-out/bin/rockboxd | ffplay -f s16le -ar 44100 -ac 2 -

# Check binary vs library timestamps
ls -la zig/zig-out/bin/rockboxd build-lib/libfirmware.a target/release/librockbox_cli.a

# Verify AirPlay symbols are present
nm zig/zig-out/bin/rockboxd | grep pcm_airplay

# Verify squeezelite symbols are present
nm zig/zig-out/bin/rockboxd | grep pcm_squeezelite

# Verify a crate is in the staticlib
ar t target/release/librockbox_cli.a | grep airplay
ar t target/release/librockbox_cli.a | grep slim

# Multi-room squeezelite test
squeezelite -s localhost -n "Room 1"
squeezelite -s localhost -n "Room 2"
```
