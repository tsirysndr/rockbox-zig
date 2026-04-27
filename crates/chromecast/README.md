# rockbox-chromecast

Chromecast output sink for Rockbox Zig. Streams live audio from the Rockbox
firmware to any Google Cast-compatible device on the LAN — Google Home, Chromecast
Audio, Chromecast with Google TV, Nest Hub, and third-party Cast receivers.

---

## Architecture overview

All Chromecast functionality is driven by `src/pcm.rs`. The `src/lib.rs` module
contains a `Player` trait implementation that is retained for internal use but is
**not** invoked from the server connect handler — the `cast_loop` thread in
`pcm.rs` owns the Cast session exclusively.

```
┌──────────────────────────────────────────────────────────────────────┐
│                          rockboxd process                            │
│                                                                      │
│  Rockbox firmware (C)                                                │
│  ┌────────────────────┐                                              │
│  │  PCM engine        │──pcm_chromecast_write()──►  BroadcastBuffer  │
│  │  (44.1 kHz S16LE)  │                               (ring buffer)  │
│  └────────────────────┘                                    │         │
│                                                            │         │
│  ┌─────────────────────────────────────────────┐           │         │
│  │  HTTP server  (port 7881)                   │◄──────────┘         │
│  │  GET /stream.wav  →  WAV header + PCM chunks│                     │
│  │  GET /now-playing/art  →  album art (JPEG…) │                     │
│  └─────────────────────┬───────────────────────┘                     │
│                         │  HTTP (WAV)                                │
│  ┌──────────────────────▼──────────────────────┐                     │
│  │  cast_loop thread  (src/pcm.rs)             │                     │
│  │  · connects to Cast device on port 8009     │                     │
│  │  · launches Rockbox app (ID 88DCBD57)       │                     │
│  │  · media.load(url=http://host:7881/…)       │                     │
│  │  · heartbeat.ping() every 500 ms            │                     │
│  │  · on track change → media.load() again     │                     │
│  │  · on RELOAD_REQUESTED → buffer.reset()     │                     │
│  │    + media.load() (pause/resume recovery)   │                     │
│  │  · exits when CAST_GENERATION changes       │                     │
│  └──────────────────────┬──────────────────────┘                     │
└─────────────────────────┼────────────────────────────────────────────┘
                          │  Cast protocol (TLS, port 8009)
                          │  + WAV stream (HTTP, port 7881)
                    ┌─────▼──────┐
                    │ Chromecast │
                    │  device    │
                    └────────────┘
```

---

## Module map

| File          | Responsibility                                                                          |
|---------------|-----------------------------------------------------------------------------------------|
| `src/pcm.rs`  | Primary: HTTP WAV server; `BroadcastBuffer`; `cast_loop`; full C FFI surface           |
| `src/lib.rs`  | Secondary: `Player` trait impl; Cast command dispatch (retained, not called by server) |
| `src/main.rs` | Example binary (connects to a hardcoded IP for manual testing)                          |

---

## Part 1 — WAV/HTTP broadcast + cast_loop (src/pcm.rs)

This is the **primary implementation**. The server connect handler arms the PCM
sink and the C firmware drives everything else through the FFI surface.

### BroadcastBuffer

```
firmware (writer)
     │  pcm_chromecast_write(buf, len)
     ▼
┌────────────────────────────────────────────┐
│  BroadcastBuffer  (max 4 MB)               │
│  - sequence counter (u64)                  │
│  - VecDeque of (seq, chunk) pairs          │
│  - Condvar to wake sleeping readers        │
│  - evict oldest chunks when full           │
│  - close() / reset() for session teardown  │
└────────────────────────────────────────────┘
     │  independent cursor per HTTP client
     ▼
HTTP client 1 (Chromecast), HTTP client 2, …
```

Each HTTP client gets its own `BroadcastReceiver` cursor. A lagging reader skips
forward to the current position so it never blocks the writer.

### WAV HTTP server

Listens on `chromecast_http_port` (default **7881**). Started once on the first
`pcm_chromecast_start()` call and kept alive for the process lifetime.

#### `GET /stream.wav`

1. Derives a WAV RIFF header from the current track:
   - `data_size = (duration_ms × byte_rate) / 1000`
   - `byte_rate = sample_rate × 4` (stereo 16-bit)
   - `Content-Length = 44 (header) + data_size`
   - If duration is unknown, `data_size = 0xFFFFFFFF` (Chromecast shows ∞).
2. Streams the header then drains `BroadcastBuffer` chunks until `data_size`
   bytes are sent or the client disconnects.

#### `GET /now-playing/art`

Searches the current track's directory for common album art filenames and
returns the first match. Returns 404 if none found.

### cast_loop and session lifecycle

`cast_loop(host, port, http_port, gen)` runs in a background thread. `gen` is
the current `CAST_GENERATION` value at spawn time.

```
cast_loop(gen)
  │
  └─► cast_session(gen)
         · CastDevice::connect_without_host_verification
         · connection.connect("receiver-0")
         · heartbeat.ping()
         · receiver.launch_app("88DCBD57")   ← Rockbox Cast app
         · connection.connect(transport_id)
         · media.load(initial)
         │
         └─► monitor loop (every 500 ms)
               · if CAST_STOP || CAST_GENERATION != gen → stop_app, return true
               · heartbeat.ping()  (failure → return false → reconnect)
               · if track path changed → art_seq++ → media.load(new metadata)
               · if RELOAD_REQUESTED && !CAST_STOP → buffer.reset() + media.load()
```

If `cast_session` returns `false` (heartbeat lost), `cast_loop` retries after
3 seconds. If it returns `true` (graceful stop), `cast_loop` exits and sets
`CAST_PLAYING = false` — but only if the generation is still current.

### Generation counter

`CAST_GENERATION: AtomicU32` is the mechanism that allows a stale `cast_loop`
to exit cleanly even after `CAST_STOP` has been re-armed for a new session:

- `pcm_chromecast_teardown()` increments the generation.
- Each `cast_loop` captures the generation at spawn time and exits when it
  diverges, preventing two concurrent cast loops from fighting over the device.

### RELOAD_REQUESTED

Set by `pcm_chromecast_start()` when `CAST_PLAYING = true` and `CAST_STOP =
false` (i.e. the cast loop is running normally). This covers the pause/resume
case: after a pause the C sink calls `sink_dma_stop()` then `sink_dma_start()`,
and `RELOAD_REQUESTED` signals the monitor loop to reset the buffer and reload
media so the Chromecast reconnects to the fresh stream.

`RELOAD_REQUESTED` is **not** set when `CAST_STOP = true` (teardown is in
progress) — in that case `pcm_chromecast_start()` detects `CAST_PLAYING = false`
and spawns a new `cast_loop` from scratch instead.

### FFI surface

The C firmware calls these symbols (`#[cfg(feature = "ffi")]` in `pcm.rs`):

```c
void  pcm_chromecast_set_http_port(uint16_t port);
void  pcm_chromecast_set_device_host(const char *host);
void  pcm_chromecast_set_device_port(uint16_t port);
void  pcm_chromecast_set_sample_rate(uint32_t rate);

// Called from sink_dma_start(). Starts HTTP server on first call (idempotent),
// resets buffer on subsequent calls, spawns cast_loop if not running.
int   pcm_chromecast_start(void);

int   pcm_chromecast_write(const uint8_t *buf, size_t len);

// No-op — Cast session stays alive during pause for seamless resume.
void  pcm_chromecast_stop(void);

// Graceful teardown: increments CAST_GENERATION, sets CAST_STOP, clears
// CAST_PLAYING, closes buffer. HTTP server stays alive.
// Called by the server when switching away from or to the Chromecast sink.
void  pcm_chromecast_teardown(void);

// Full shutdown (process exit): also resets PCM_STARTED.
void  pcm_chromecast_close(void);
```

The C implementation lives in `firmware/target/hosted/pcm-chromecast.c`,
registered as `PCM_SINK_CHROMECAST` in `firmware/export/pcm_sink.h`.

---

## Part 2 — Player trait (src/lib.rs)

`Chromecast::connect(device)` and the `CastPlayerInternal` background task are
retained for completeness but are **not** called by the server connect handler.
The `cast_loop` in `pcm.rs` owns the Cast session exclusively.

If you need to drive the Cast protocol directly (e.g. from a test binary or a
future multi-session feature), `lib.rs` provides:

| Player method  | Cast action                          |
|----------------|--------------------------------------|
| `play()`       | `media.play()`                       |
| `pause()`      | `media.pause()`                      |
| `resume()`     | `media.play()`                       |
| `stop()`       | no-op                                |
| `disconnect()` | `receiver.stop_app(session_id)`      |

Next / previous are **not** routed through `lib.rs` — the server always calls
`rb::playback::next()` / `rb::playback::prev()` directly, and the `cast_loop`
monitor detects the resulting track-path change and calls `media.load()`.

---

## Connect / disconnect flow

### Connecting to Chromecast

```
server: PUT /devices/:id/connect  (service = "chromecast")
  1. pcm::chromecast_teardown()        ← stops any running cast_loop cleanly
  2. pcm::chromecast_set_device_host / port / http_port
  3. pcm::switch_sink(PCM_SINK_CHROMECAST)
  4. settings saved, device marked active

  [firmware starts playing]
  5. sink_dma_start() → pcm_chromecast_start()
       · HTTP server starts (first time) or buffer is reset (subsequent)
       · CAST_PLAYING = false → spawn cast_loop(gen)
       · cast_loop: connect → launch app → media.load → monitor loop
```

### Switching away / disconnecting

```
server: PUT /devices/:id/connect  (service != "chromecast")
  — or —
server: PUT /devices/:id/disconnect

  1. pcm::chromecast_teardown()
       · CAST_GENERATION++            ← old cast_loop will see generation mismatch
       · CAST_STOP = true             ← cast_loop exits monitor loop, stop_app()
       · CAST_PLAYING = false
       · buffer.close()               ← WAV readers unblock and exit
  2. pcm::switch_sink(new sink)
```

### Reconnecting to Chromecast (e.g. after using built-in)

Because `teardown()` set `CAST_PLAYING = false`, the next `pcm_chromecast_start()`
call always spawns a **fresh** `cast_loop` with a new generation — no stale state
from the previous session.

---

## Device discovery

Chromecast devices are discovered via **mDNS** (`_googlecast._tcp.local.`).
`scan_chromecast_devices()` in `crates/server/src/scan.rs` browses the LAN and
sets `device.service = "chromecast"` on each result. Devices appear in the
GraphQL `devices` query and the UI device picker in real time.

> **Note**: the `Device::from(ServiceInfo)` conversion does not set the
> `service` field — `scan_chromecast_devices` overrides it explicitly to
> `"chromecast"` so the connect handler routes correctly.

---

## Configuration

Add to `~/.config/rockbox.org/settings.toml`:

```toml
music_dir            = "/path/to/Music"
audio_output         = "chromecast"

chromecast_host      = "192.168.1.60"  # IP of the target Chromecast
chromecast_port      = 8009            # optional, default 8009
chromecast_http_port = 7881            # optional, default 7881
```

If you prefer to select the device dynamically via the UI, use
`audio_output = "builtin"` at startup and connect through the device picker.
The WAV stream and Cast session are started on demand when audio plays.

### Port summary

| Port | Protocol  | Purpose                                             |
|------|-----------|-----------------------------------------------------|
| 8009 | TCP / TLS | Cast control channel (Protobuf)                     |
| 7881 | HTTP      | WAV audio stream + album art served **by rockboxd** |

> Port 7881 must be reachable from the Chromecast device. If rockboxd runs
> inside a VM or container, ensure the WAV HTTP port is forwarded to the host.

---

## Known limitations

| Feature                               | Status                                      |
|---------------------------------------|---------------------------------------------|
| Play / pause / resume                 | ✅ Implemented                               |
| Next / previous track                 | ✅ Via `rb::playback::next/prev` + cast_loop |
| Track metadata + album art display    | ✅ Implemented                               |
| Reconnect after output switch         | ✅ Via teardown + fresh cast_loop            |
| Volume control                        | ⏳ Not yet implemented                       |
| Seek within track                     | ⏳ Not yet implemented                       |
| Multi-device fan-out                  | ⏳ Not yet implemented (single device only)  |

---

## Dependencies

| Crate             | Version   | Purpose                                                   |
|-------------------|-----------|-----------------------------------------------------------|
| `chromecast`      | 0.18.2    | Cast protocol client (Protobuf/TLS)                       |
| `tokio`           | workspace | Async runtime for Cast background task                    |
| `async-trait`     | workspace | `Player` trait with async methods                         |
| `rockbox-traits`  | local     | `Player` trait definition                                 |
| `rockbox-types`   | local     | `Device`, `Track`, `Playback` types                       |
| `rockbox-sys`     | local     | FFI to Rockbox C firmware (current track, playback state) |
| `rockbox-library` | local     | SQLite library for track lookups                          |
| `md5`             | —         | Device ID hashing                                         |
| `tracing`         | workspace | Structured logging                                        |
