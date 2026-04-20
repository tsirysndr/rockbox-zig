# CLAUDE.md — Rockbox Zig

## Project overview

Rockbox Zig is a modern wrapper around the [Rockbox](https://www.rockbox.org) open-source audio player firmware. It adds Rust/Zig services on top of the C firmware to expose gRPC, GraphQL, HTTP, and MPD APIs, a Typesense-backed search engine, Chromecast/AirPlay/Snapcast output sinks, and a desktop/web UI.

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
```

## PCM sink architecture

The audio output abstraction lives in `firmware/export/pcm_sink.h`. Each sink implements `struct pcm_sink_ops` (init / postinit / set_freq / lock / unlock / play / stop).

| Enum constant      | Value | Implementation file                         |
|--------------------|-------|---------------------------------------------|
| `PCM_SINK_BUILTIN` | 0     | `firmware/target/hosted/sdl/pcm-sdl.c`     |
| `PCM_SINK_FIFO`    | 1     | `firmware/target/hosted/pcm-fifo.c`        |
| `PCM_SINK_AIRPLAY` | 2     | `firmware/target/hosted/pcm-airplay.c`     |

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

# Verify a crate is in the staticlib
ar t target/release/librockbox_cli.a | grep airplay
```
