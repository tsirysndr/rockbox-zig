# rockbox-slim — Squeezelite PCM sink internals

This document traces the full path of a PCM audio frame from the Rockbox C
firmware through the Slim Protocol and HTTP broadcast layer to one or more
squeezelite instances.

---

## Table of contents

1. [Overview](#overview)
2. [Architecture diagram](#architecture-diagram)
3. [Layer 1 — PCM sink abstraction (C)](#layer-1--pcm-sink-abstraction-c)
4. [Layer 2 — DMA thread and real-time pacing (C)](#layer-2--dma-thread-and-real-time-pacing-c)
5. [Layer 3 — FFI boundary](#layer-3--ffi-boundary)
6. [Layer 4 — Broadcast buffer (Rust)](#layer-4--broadcast-buffer-rust)
7. [Layer 5 — HTTP stream server (Rust)](#layer-5--http-stream-server-rust)
8. [Layer 6 — Slim Protocol server (Rust)](#layer-6--slim-protocol-server-rust)
9. [Layer 7 — squeezelite client](#layer-7--squeezelite-client)
10. [Startup sequence](#startup-sequence)
11. [Track transition](#track-transition)
12. [Multi-room](#multi-room)
13. [Configuration](#configuration)
14. [Gotchas and non-obvious invariants](#gotchas-and-non-obvious-invariants)

---

## Overview

The squeezelite sink turns rockboxd into a minimal Logitech Media Server (LMS)
that speaks just enough of the Slim Protocol to command squeezelite clients to
fetch a continuous raw PCM stream over HTTP.

```
rockboxd                           squeezelite
─────────                          ───────────
Codec → PCM engine                 audio device (CoreAudio / ALSA / …)
          │                               ▲
  pcm-squeezelite.c (C)                   │
          │                               │
  pcm_squeezelite_write()          [HTTP audio stream]
          │                               │
  BroadcastBuffer (Rust)  ←── pcm_squeezelite_write() pushes chunks
          │
          ├── HTTP server :9999 ─── one thread per squeezelite client
          │
          └── Slim server :3483 ─── sends STRM, keeps connection alive
```

The PCM data is **never transcoded** — rockboxd pushes raw signed 16-bit
little-endian stereo (`S16LE`) at 44 100 Hz, squeezelite's built-in `pcm`
codec copies it straight to the audio device.

---

## Architecture diagram

```
┌─────────────────────────────────────────────────────────────────┐
│  Rockbox firmware (C)                                           │
│                                                                 │
│  codec thread ──► PCM buffer ──► pcm_play_dma_complete_callback │
│                                           │                     │
│                              squeezelite_thread (pthread)       │
│                                           │                     │
│                              pcm_squeezelite_write()  ◄── FFI   │
└─────────────────────────────────────────────────────────────────┘
                                            │
                              ══════════════╪══════════ FFI boundary
                                            │
┌─────────────────────────────────────────────────────────────────┐
│  rockbox-slim crate (Rust)                                      │
│                                                                 │
│  BroadcastBuffer                                                │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  VecDeque<(seq: u64, chunk: Vec<u8>)>   max 4 MB         │   │
│  │  next_seq (writer cursor)                                │   │
│  └──────────────────────────────────────────────────────────┘   │
│          │                          │                           │
│  HTTP server :9999          Slim Protocol server :3483          │
│  (one thread per client)    (one thread per client)             │
│          │                          │                           │
│  BroadcastReceiver          HELO → STRM → audg keepalive        │
│  (per-client seq cursor)                                        │
└────────────────────────────┬────────────────────────────────────┘
                             │ TCP :9999
              ┌──────────────┼──────────────┐
              ▼              ▼              ▼
         squeezelite    squeezelite    squeezelite
          "Room 1"       "Room 2"       "Room 3"
```

---

## Layer 1 — PCM sink abstraction (C)

**Files:** `firmware/export/pcm_sink.h`, `firmware/pcm.c`

Rockbox's hosted audio output is built around `struct pcm_sink`, a vtable of
seven operations:

```c
struct pcm_sink_ops {
    void (*init)(void);
    void (*postinit)(void);
    void (*set_freq)(uint16_t freq);   // called when sample rate changes
    void (*lock)(void);
    void (*unlock)(void);
    void (*play)(const void *addr, size_t size);  // sink_dma_start
    void (*stop)(void);                            // sink_dma_stop
};
```

The squeezelite sink is registered as `PCM_SINK_SQUEEZELITE = 3`:

```c
// firmware/export/pcm_sink.h
enum pcm_sink_ids {
    PCM_SINK_BUILTIN     = 0,  // SDL audio
    PCM_SINK_FIFO        = 1,  // named FIFO → Snapcast
    PCM_SINK_AIRPLAY     = 2,  // RAOP/RTP → AirPlay receiver
    PCM_SINK_SQUEEZELITE = 3,  // Slim Protocol + HTTP → squeezelite
    PCM_SINK_NUM
};
```

`pcm_switch_sink(PCM_SINK_SQUEEZELITE)` (called from `crates/settings`) swaps
the active vtable.  All subsequent `pcm_play_data()` calls drive the
squeezelite sink.

---

## Layer 2 — DMA thread and real-time pacing (C)

**File:** `firmware/target/hosted/pcm-squeezelite.c`

### sink_dma_start / sink_dma_stop

`sink_dma_start(addr, size)` is called by the Rockbox PCM engine whenever a
new track (or track segment) begins.  It:

1. Calls `pcm_squeezelite_start()` — idempotent; starts the Slim and HTTP
   servers on first call only.
2. Captures `play_start = CLOCK_MONOTONIC` and resets `play_bytes = 0` to
   anchor real-time pacing.
3. Stores the initial `(addr, size)` buffer pointer.
4. Spawns `squeezelite_thread`.

`sink_dma_stop()` sets `squeezelite_stop = true`, joins the thread, and calls
the no-op `pcm_squeezelite_stop()`.

### squeezelite_thread — the DMA loop

The thread mimics a hardware DMA interrupt loop in software:

```
loop:
  1. Grab current (data, size) under mutex, clear pcm_data/pcm_size
  2. pcm_squeezelite_write(data, size)  →  push to BroadcastBuffer
  3. Real-time pace (sleep if ahead of wall clock)
  4. pcm_play_dma_complete_callback()  →  get next (pcm_data, pcm_size)
  5. pcm_play_dma_status_callback(STARTED)
  6. goto loop
```

`pcm_play_dma_complete_callback()` is the hook Rockbox uses to advance the
codec's read pointer — calling it at the wrong rate directly affects the
playback-position counter shown in the UI.

### Real-time pacing

Without pacing, the loop drains an entire track in milliseconds (the ring
buffer write is a memory copy).  Pacing is implemented with a running byte
counter:

```c
play_bytes += size;
uint64_t bps         = (uint64_t)current_sample_rate * 4; // S16LE stereo
uint64_t expected_us = play_bytes * 1000000ULL / bps;

struct timespec now;
clock_gettime(CLOCK_MONOTONIC, &now);

// IMPORTANT: use int64_t, not uint64_t.
// tv_nsec subtraction wraps to ~10^19 when tv_nsec rolls over,
// making elapsed_us astronomically large and disabling all sleeps.
int64_t elapsed_us =
    (int64_t)(now.tv_sec  - play_start.tv_sec)  * 1000000LL +
    ((int64_t)now.tv_nsec - (int64_t)play_start.tv_nsec) / 1000LL;

if (elapsed_us >= 0 && expected_us > (uint64_t)elapsed_us)
    usleep((useconds_t)(expected_us - (uint64_t)elapsed_us));
```

`play_start` and `play_bytes` are **reset on every `sink_dma_start()`** (i.e.
every track), so drift does not accumulate across tracks.

---

## Layer 3 — FFI boundary

**Files:** `crates/sys/src/lib.rs`, `crates/sys/src/sound/pcm.rs`

The C symbols exported by `pcm-squeezelite.c` are declared as `extern "C"` in
`crates/sys/src/lib.rs` and wrapped with safe Rust helpers in
`crates/sys/src/sound/pcm.rs`:

| C symbol                             | Rust wrapper                             |
|--------------------------------------|------------------------------------------|
| `pcm_squeezelite_set_slim_port(u16)` | `pcm::squeezelite_set_slim_port(u16)`    |
| `pcm_squeezelite_set_http_port(u16)` | `pcm::squeezelite_set_http_port(u16)`    |
| `pcm_switch_sink(i32)`               | `pcm::switch_sink(PCM_SINK_SQUEEZELITE)` |

These wrappers are called from `crates/settings/src/lib.rs:load_settings()`
when `audio_output = "squeezelite"` is found in `settings.toml`.

### Force-link shim

`rockbox-slim` is an `rlib`, not a `staticlib`.  The Rust linker would dead-
strip it unless something directly references its symbols.  A dummy function
in `crates/cli/src/lib.rs` forces all FFI exports into `librockbox_cli.a`:

```rust
// crates/cli/src/lib.rs
#[allow(unused_imports)]
use rockbox_slim::_link_slim as _;
```

```rust
// crates/slim/src/lib.rs
#[doc(hidden)]
pub fn _link_slim() {}
```

---

## Layer 4 — Broadcast buffer (Rust)

**File:** `crates/slim/src/lib.rs`

### Design

The `BroadcastBuffer` stores PCM chunks as `(seq: u64, data: Vec<u8>)` tuples
in a `VecDeque`.  The writer holds a single monotonically-increasing
`next_seq` cursor.  Each reader (`BroadcastReceiver`) holds its own
independent `next_seq` cursor.

```
write cursor ──────────────────────────────────────────► next_seq
                                                              │
chunks: [(100,…), (101,…), (102,…), (103,…), (104,…)]         │
          ▲                  ▲                    ▲           │
          │                  │                    │           │
       reader C           reader A             reader B       │
       (lagging)          (live)               (live)         │
```

### Push (writer)

```rust
pub(crate) fn push(&self, data: &[u8]) {
    // assign sequence number, append chunk
    // evict oldest chunks while total_bytes > MAX_BUFFERED (4 MB)
    // notify all waiting receivers
}
```

Eviction happens from the front; the `next_seq` counter is never reset, so
existing receivers automatically detect that they have fallen behind.

### Subscribe (new reader)

```rust
pub(crate) fn subscribe(self: &Arc<Self>) -> BroadcastReceiver {
    // start at the current write cursor — live stream, no stale data
}
```

### recv_blocking (reader)

```rust
pub(crate) fn recv_blocking(&mut self) -> RecvResult {
    loop {
        // 1. If receiver is behind front_seq → skip forward (log warning)
        // 2. If chunk is available → return it, advance next_seq
        // 3. If closed → return Closed
        // 4. Otherwise → condvar.wait (release lock, sleep)
    }
}
```

The index into the `VecDeque` is computed as
`(self.next_seq - front_seq) as usize`, which is O(1).

### Capacity and eviction

The buffer is capped at **4 MB ≈ 23 seconds** of S16LE stereo at 44 100 Hz.
Chunks are evicted oldest-first when the cap is exceeded.  A lagging reader
whose `next_seq` points to an evicted chunk silently skips forward to the
oldest available chunk — it experiences a brief audio discontinuity but does
not block the writer or other readers.

---

## Layer 5 — HTTP stream server (Rust)

**File:** `crates/slim/src/http.rs`

The HTTP server accepts connections on `0.0.0.0:9999` (configurable).  Each
accepted connection is handled in a **dedicated thread**:

```
listener.incoming() loop
    │
    └── thread::spawn per connection
            │
            ├── drain_request()  — read & discard HTTP request headers
            ├── write HTTP 200 response headers
            ├── buf.subscribe()  — get a fresh BroadcastReceiver
            └── loop { recv_blocking() → write_all() }
```

### Response headers

```
HTTP/1.0 200 OK
Content-Type: audio/L16;rate=44100;channels=2
Cache-Control: no-cache
```

`audio/L16` is the MIME type for raw signed 16-bit PCM.  squeezelite ignores
the Content-Type and uses the codec declared in the `STRM` command (`p` =
raw PCM), but the header documents the wire format.

### Back-pressure

squeezelite reads from the TCP socket at exactly the rate its audio device
consumes PCM (176 400 bytes/s at 44 100 Hz stereo 16-bit).  The OS TCP send
buffer provides natural back-pressure: if squeezelite is slow, `write_all()`
blocks, which causes `recv_blocking()` to hold the lock longer, which in turn
causes the C DMA thread to block in `pcm_squeezelite_write()` — naturally
slowing the DMA loop.

---

## Layer 6 — Slim Protocol server (Rust)

**File:** `crates/slim/src/slimproto.rs`

The Slim Protocol server listens on `0.0.0.0:3483` (configurable).  Each
squeezelite client gets its own thread.

### Packet framing

The Slim Protocol uses **asymmetric framing**:

| Direction       | Wire format                                         |
|-----------------|-----------------------------------------------------|
| Client → Server | `opcode[4]` + `u32 length BE` + `payload[length]`   |
| Server → Client | `u16 length BE` + `opcode[4]` + `payload[length-4]` |

The `u16 length` field in server→client packets counts the opcode (4 bytes)
plus the payload, but **not the 2-byte length field itself**.

### Session flow

```
squeezelite                         Slim server
───────────                         ───────────
HELO ──────────────────────────────►
                                     (no kick — all clients share the stream)
       ◄────────────────────────── STRM 's'
HTTP GET :9999/stream.pcm ─────────────────────────────► HTTP server
       ◄────────────────────── HTTP 200 + raw PCM stream
STAT STMc ─────────────────────────►
STAT STMs ─────────────────────────►
STAT STMt ─────────────────────────►  (every ~1 s)
       ◄─────────────────────────── audg (full volume)  ← keepalive
STAT STMt ─────────────────────────►
       ◄─────────────────────────── audg
       …
```

### STRM 's' payload layout

The `STRM` packet instructs squeezelite where and how to fetch audio.  All
index fields are **ASCII-encoded**: squeezelite subtracts `'0'` before use.

```
Offset  Size  Value   Meaning
──────  ────  ──────  ───────────────────────────────────────────────
 0       1    's'     command: start
 1       1    '1'     autostart: 1 = buffer then play immediately
 2       1    'p'     format: raw PCM
 3       1    '1'     pcm_sample_size: '1'-'0'+1 = 2 bytes = 16-bit
 4       1    '3'     pcm_sample_rate: sample_rates[3] = 44 100 Hz
 5       1    '2'     pcm_channels: '2'-'0' = 2 (stereo)
 6       1    '1'     pcm_endianness: '1'≠'0' → little-endian
 7       1    255     threshold: 255 KB before auto-start
 8       1    0       spdif_enable
 9       1    0       transition_period
10       1    '0'     transition_type: none
11       1    0       flags
12       1    0       output_threshold
13       1    0       slaves
14       4    BE u32  replay_gain = 0x00010000 (1.0 in 16.16 fixed-point)
18       2    BE u16  server_port (HTTP audio port)
20       4    BE u32  server_ip = 0 → squeezelite uses the Slim server IP
24       *    bytes   "GET /stream.pcm HTTP/1.0\r\n\r\n"
```

`server_ip = 0` is a squeezelite convention: when the IP is zero, squeezelite
uses the IP address it connected to for Slim Protocol.  This means the HTTP
server must be on the same host as the Slim server.

### audg keepalive

squeezelite counts consecutive 1-second `select()` timeouts on the Slim
socket.  After **36 consecutive timeouts** with no data from the server it
logs `"No messages from server — connection dead"` and reconnects.

The fix: reply to every `STMt` (timer tick, sent by squeezelite once per
second) with an `audg` (audio gain) packet:

```
Offset  Size  Value       Meaning
──────  ────  ──────────  ───────────────────────────────────────
 0       4    0x00010000  left  gain = 1.0 (16.16 BE fixed-point)
 4       4    0x00010000  right gain = 1.0
 8       1    0           do not adjust by replay gain
```

This resets squeezelite's timeout counter to zero on every tick.

---

## Layer 7 — squeezelite client

squeezelite handles the audio pipeline in three internal threads:

| Thread                               | Role                                                          |
|--------------------------------------|---------------------------------------------------------------|
| `stream_thread`                      | Reads HTTP stream into `streambuf` (2 MB ring buffer)         |
| `decode_thread`                      | PCM "decoder": memcpy from `streambuf` → `outputbuf` (3.5 MB) |
| `output_thread` / PortAudio callback | Reads `outputbuf`, sends to audio device                      |

For raw PCM (`format = 'p'`), the decode step is a straight memory copy with
no transcoding, so latency between push and playback is determined solely by
squeezelite's buffer thresholds (default ~255 KB ≈ 1.4 seconds at 44 100 Hz).

---

## Startup sequence

```
rockboxd starts
    │
    └── load_settings() reads audio_output = "squeezelite"
            │
            ├── pcm::squeezelite_set_slim_port(3483)
            ├── pcm::squeezelite_set_http_port(9999)
            └── pcm::switch_sink(PCM_SINK_SQUEEZELITE)
                    │
                    └── swaps active pcm_sink vtable

user plays a track
    │
    └── sink_dma_start(addr, size)           [pcm-squeezelite.c]
            │
            ├── pcm_squeezelite_start()      [lib.rs — idempotent]
            │       ├── spawn HTTP server thread on :9999
            │       └── spawn Slim server thread on :3483
            │
            ├── reset play_start, play_bytes
            └── spawn squeezelite_thread

squeezelite -s localhost
    │
    ├── TCP connect :3483
    ├── send HELO
    ├── receive STRM 's'  →  TCP connect :9999
    ├── receive HTTP 200
    └── start buffering PCM
```

---

## Track transition

Between tracks, rockboxd calls `sink_dma_stop()` then `sink_dma_start()`:

1. `sink_dma_stop()`: sets `squeezelite_stop = true`, joins the DMA thread,
   clears `pcm_data`/`pcm_size`.  The broadcast buffer is **not flushed** —
   squeezelite's in-flight HTTP connection continues reading.

2. `sink_dma_start()`: `pcm_squeezelite_start()` is a no-op (already started).
   Resets `play_start`/`play_bytes`, spawns a new DMA thread.

The gap between stop and start is a few milliseconds.  squeezelite's
`outputbuf` (≈ 20 s capacity) absorbs the gap without a dropout.

---

## Multi-room

Any number of squeezelite instances can connect simultaneously:

- Each opens a Slim connection (`:3483`) and receives an identical `STRM 's'`
  command pointing at `:9999`.
- Each opens an HTTP connection (`:9999`) and gets an independent
  `BroadcastReceiver` starting at the current write cursor.
- Chunks flow from the single writer to all readers independently.
- A slow reader skips forward to the oldest available chunk; other readers
  are unaffected.

Squeezelite clients are not time-synchronised (no NTP/PTP layer).  Clock drift
between rooms is typically 100–500 ms.  For tighter sync, run squeezelite with
a Snapcast-compatible back-end or use the FIFO sink instead.

---

## Configuration

`~/.config/rockbox.org/settings.toml`:

```toml
audio_output = "squeezelite"
squeezelite_port      = 3483   # Slim Protocol TCP port (default)
squeezelite_http_port = 9999   # HTTP PCM stream port   (default)
```

To select a specific audio device in squeezelite:

```sh
squeezelite -s localhost -l          # list available devices
squeezelite -s localhost -o ""       # system default
squeezelite -s localhost -o "Built-in Output"
```

Debug logging:

```sh
RUST_LOG=debug ./zig/zig-out/bin/rockboxd       # rockboxd side
squeezelite -s localhost -d all=debug           # squeezelite side
```

---

## Gotchas and non-obvious invariants

### 1. `int64_t` for the pacing diff — never `uint64_t`

`struct timespec::tv_nsec` is `long`.  `(uint64_t)(now.tv_nsec - prev.tv_nsec)`
wraps to ≈ 1.8 × 10¹⁹ when `tv_nsec` rolls over (once per second).  This
makes `elapsed_us` enormous, disables all `usleep()` calls for the rest of the
track, and causes the DMA loop to drain PCM at CPU speed — advancing the
playback position many times faster than real time.  Always use `int64_t` for
the subtraction.

### 2. squeezelite's 36-second watchdog

squeezelite counts consecutive 1-second `select()` timeouts.  Without an
`audg` keepalive, it declares the connection dead after 36 seconds and
reconnects.  The reconnect is seamless but causes a brief gap in the HTTP
stream.

### 3. ASCII-encoded STRM fields

`pcm_sample_size`, `pcm_sample_rate`, `pcm_channels`, and `pcm_endianness` in
the `STRM` payload are ASCII characters, not binary integers.  squeezelite
subtracts `'0'` (0x30) before use.  Sending `0x03` instead of `'3'` (0x33)
for the sample rate would silently select the wrong rate.

### 4. `server_ip = 0` in STRM

squeezelite interprets `server_ip = 0` as "use the IP I connected to for Slim
Protocol".  The HTTP server must therefore be reachable on the same host as
the Slim server.  A non-zero `server_ip` would make squeezelite connect to
that explicit IP instead.

### 5. Force-link shim

`rockbox-slim` is an `rlib`.  Without `use rockbox_slim::_link_slim as _` in
`crates/cli/src/lib.rs`, the linker discards all its symbols and the
`extern "C"` declarations in `pcm-squeezelite.c` produce undefined-reference
errors at link time.

### 6. `next_seq` is never reset

`BroadcastBuffer::reset()` clears the chunk queue but does **not** reset
`next_seq`.  This ensures that receivers created before the reset
automatically skip forward to newly pushed chunks rather than getting stuck
waiting for sequence numbers that will never appear.
