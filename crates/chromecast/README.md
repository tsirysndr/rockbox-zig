# rockbox-chromecast

Chromecast output sink for Rockbox Zig. Streams live audio from the Rockbox
firmware to any Google Cast-compatible device on the LAN — Google Home, Chromecast
Audio, Chromecast with Google TV, Nest Hub, and third-party Cast receivers.

---

## Architecture overview

The Chromecast implementation has two independent, complementary halves:

```
┌──────────────────────────────────────────────────────────────────────┐
│                          rockboxd process                            │
│                                                                      │
│  Rockbox firmware (C)                                                │
│  ┌────────────────────┐                                              │
│  │  PCM engine        │──pcm_chromecast_write()──►  BroadcastBuffer │
│  │  (44.1 kHz S16LE)  │                               (ring buffer) │
│  └────────────────────┘                                    │         │
│                                                            │         │
│  ┌─────────────────────────────────────────────┐          │         │
│  │  HTTP server  (port 7881)                   │◄─────────┘         │
│  │  GET /stream.wav  →  WAV header + PCM chunks│                    │
│  │  GET /now-playing/art  →  album art (JPEG…) │                    │
│  └─────────────────────┬───────────────────────┘                    │
│                         │  HTTP (WAV)                                │
│  ┌──────────────────────▼──────────────────────┐                    │
│  │  Cast protocol thread                       │                    │
│  │  (Cast TCP 8009, TLS, Protobuf)             │                    │
│  │  · media.load(url=http://host:7881/…)       │                    │
│  │  · heartbeat.ping() every 500 ms            │                    │
│  │  · on track change → media.load() again     │                    │
│  └──────────────────────┬───────────────────────┘                   │
└─────────────────────────┼────────────────────────────────────────────┘
                          │  Cast protocol (TLS, port 8009)
                          │  + WAV stream (HTTP, port 7881)
                    ┌─────▼──────┐
                    │ Chromecast │
                    │  device    │
                    └────────────┘
```

The two halves communicate through a standard URL: the Cast protocol thread
tells the device to fetch `http://<rockboxd-host>:7881/stream.wav`, and the HTTP
server serves that stream from the broadcast buffer filled by the firmware.

---

## Module map

| File          | Responsibility                                                                     |
|---------------|------------------------------------------------------------------------------------|
| `src/lib.rs`  | `Player` trait implementation; Cast protocol control thread; MPSC command dispatch |
| `src/pcm.rs`  | HTTP WAV broadcast server; album art server; `BroadcastBuffer`; C FFI surface      |
| `src/main.rs` | Example binary (connects to a hardcoded IP for manual testing)                     |

---

## Part 1 — Cast protocol (src/lib.rs)

### Connection

`Chromecast::connect(device: Device)` establishes the Cast session:

1. Open a TLS TCP connection to the device on port 8009
   (`CastDevice::connect_without_host_verification`) — host verification is
   skipped because Chromecast devices use self-signed certificates.
2. Connect to the built-in `receiver-0` destination.
3. Ping the heartbeat channel.
4. Launch the **Default Media Receiver** (app ID `"88DCBD57"`).
5. Save the resulting `transport_id` and `session_id`.
6. Spawn a background Tokio task (`CastPlayerInternal`) that owns the
   `CastDevice` handle and processes commands.

The method returns a `Box<dyn Player>` usable through the shared `Player` trait.

### Command dispatch

All player operations (play, pause, next, previous, load_tracks, …) send a
`CastPlayerCommand` over an unbounded MPSC channel to the background task.
This avoids blocking the calling thread and keeps the Cast device handle
single-owner.

```
caller thread                background task (CastPlayerInternal)
     │                                   │
     │──CastPlayerCommand::Play─────────►│
     │                                   │── cast_device.media.play()
     │──CastPlayerCommand::Next─────────►│
     │                                   │── cast_device.media.next()
     │──CastPlayerCommand::LoadTracks──►│
     │                                   │── cast_device.media.queue_load([…])
```

### Status polling

Every 500 ms the background task calls `media.get_status()` and updates a
shared `Arc<Mutex<CurrentPlayback>>`. `get_current_playback()` reads this value
without talking to the device, so it is always fast and non-blocking.

### Track metadata

When loading a track or a queue, Rockbox builds a Cast `Media` object with:

- `content_id` — the stream URL (`http://<host>:7881/stream.wav`)
- `content_type` — `"audio/wav"`
- `stream_type` — `Buffered` when duration is known, `Live` otherwise
- `duration` — seconds from the Rockbox track entry
- `metadata` — `MusicTrackMediaMetadata` with title, artist, album, and an
  album art URL (`http://<host>:7881/now-playing/art?t=<seq>`)

The `?t=<seq>` cache-buster increments on every track change so the Chromecast
re-fetches the art rather than using its internal cache.

### Queue management

- `load_tracks(tracks, start_index)` → `media.queue_load([Media…])` — replaces
  the entire queue and starts playback from `start_index`.
- `play_next(track)` → `media.queue_insert(Media)` — inserts one track before
  the next queue item.

### Session lifecycle

| Player method  | Cast action                                             |
|----------------|---------------------------------------------------------|
| `play()`       | `media.play()`                                          |
| `pause()`      | `media.pause()`                                         |
| `resume()`     | `media.play()`                                          |
| `stop()`       | no-op — session stays alive so pause/resume is seamless |
| `next()`       | `media.next()`                                          |
| `previous()`   | `media.previous()`                                      |
| `disconnect()` | `receiver.stop_app(session_id)`                         |

`stop()` is intentionally a no-op at the Cast level. The Cast app is only
stopped when `disconnect()` is called explicitly (e.g. the user switches to
another output or closes the app). This prevents the Chromecast UI from
dismissing the "Now Playing" card between tracks.

---

## Part 2 — WAV/HTTP broadcast (src/pcm.rs)

### BroadcastBuffer

```
firmware (writer)
     │  pcm_chromecast_write(buf, len)
     ▼
┌────────────────────────────────────────────┐
│  BroadcastBuffer  (max 4 MB)               │
│  - sequence counter (u64)                  │
│  - Vec of (seq, chunk) pairs               │
│  - Condvar to wake sleeping readers        │
│  - evict oldest chunks when full           │
└────────────────────────────────────────────┘
     │  reader cursor per HTTP client
     ▼
HTTP client 1, HTTP client 2, …
```

Each HTTP client (i.e. the Chromecast device) gets its own cursor into the
buffer. A slow client skips forward to the current position rather than
blocking the writer — this prevents audio glitches when network conditions are
poor.

### WAV HTTP server

The server listens on `chromecast_http_port` (default **7881**) and handles two
routes:

#### `GET /stream.wav` (or any path)

1. Computes a WAV RIFF header from the current track's sample rate, channel
   count, and duration:
   - `data_size = (duration_ms × byte_rate) / 1000`
   - `byte_rate = sample_rate × 4` (stereo 16-bit)
   - `Content-Length = 44 (header) + data_size`
   - If duration is unknown, `data_size = 0xFFFFFFFF` (Chromecast shows ∞).
2. Streams the header, then sends chunks from the broadcast buffer as they
   arrive until `data_size` bytes have been sent or the client disconnects.

Using a finite `Content-Length` is important: Chromecast's Default Media
Receiver uses it to show a progress bar and to know when a track ends so it
can automatically advance to the next queue item.

#### `GET /now-playing/art`

Searches the directory of the currently playing track for common album art
filenames:

```
cover.jpg  cover.png  cover.webp  folder.jpg  folder.png
front.jpg  front.png  artwork.jpg  artwork.jpeg  artwork.png
```

Returns the first match with the correct MIME type, or 404 if none is found.

### Track change detection

A dedicated `cast_loop` thread polls `rockbox_sys::playback::current_track()`
every 500 ms and compares the path to the previously known path. On a change:

1. Increments the global `art_seq` counter.
2. Calls `media.load(Media{…})` on the active Cast session with fresh
   metadata and a cache-busted art URL.
3. The Chromecast fetches the new `/stream.wav` URL and the updated album art.

### FFI surface

The C firmware calls these symbols (defined with `#[no_mangle]` in `pcm.rs`):

```c
void  pcm_chromecast_set_http_port(uint16_t port);
void  pcm_chromecast_set_device_host(const char *host);
void  pcm_chromecast_set_device_port(uint16_t port);
void  pcm_chromecast_set_sample_rate(uint32_t rate);
int   pcm_chromecast_start(void);   /* idempotent; starts HTTP + cast threads */
int   pcm_chromecast_write(const uint8_t *buf, size_t len);
void  pcm_chromecast_stop(void);    /* no-op — session stays open */
void  pcm_chromecast_close(void);   /* stops threads, frees resources */
```

The C implementation lives in
`firmware/target/hosted/pcm-chromecast.c`, which is registered as
`PCM_SINK_CHROMECAST` in `firmware/export/pcm_sink.h`.

---

## Device discovery

Chromecast devices are discovered automatically via **mDNS** (`_googlecast._tcp.local.`).
The `rockbox-discovery` crate (using `mdns_sd`) browses the LAN and publishes
each found device as a `Device` struct:

| Field            | Value                                 |
|------------------|---------------------------------------|
| `app`            | `"chromecast"`                        |
| `port`           | `8009` (Cast protocol)                |
| `ip`             | IPv4 address of the device            |
| `name`           | Friendly name from mDNS `fn` property |
| `is_cast_device` | `true`                                |

Discovered devices appear in the GraphQL `devices` query and in the web/desktop
UI device picker in real time.

---

## Connecting a device

Once discovered, connect via:

```
POST /devices/:id/connect
```

This calls `Chromecast::connect(device)`, stores the resulting `Player` in the
shared HTTP server context, and marks the device as active. All subsequent
play/pause/next/… calls go through this player.

Disconnect with:

```
POST /devices/:id/disconnect
```

This stops the Cast app session and clears the player context.

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

`chromecast_host` must be the LAN IP of the Chromecast device you want to use
as the fixed PCM output sink. If you prefer to select the device dynamically
through the UI, use `audio_output = "builtin"` and connect via the device
picker instead — the WAV stream and Cast session are started on demand.

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
| Play / pause / stop / next / previous | ✅ Implemented                               |
| Queue management (load, insert)       | ✅ Implemented                               |
| Metadata + album art display          | ✅ Implemented                               |
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
