# Snapcast — FIFO & TCP PCM Sinks

This document traces every hop an audio frame takes from the Rockbox C firmware
through the Snapcast PCM sinks to a Snapcast server.

Two complementary sinks are available:

| Sink | Setting value | Transport | Snapserver source type |
|------|--------------|-----------|------------------------|
| FIFO / pipe | `audio_output = "fifo"` | Named FIFO or stdout | `pipe://` |
| TCP (direct) | `audio_output = "snapcast_tcp"` | TCP socket | `tcp://` |

The **FIFO sink** is the traditional approach: rockboxd writes to a named pipe
that snapserver reads. The **TCP sink** connects directly to snapserver's TCP
source port — no FIFO, no filesystem dependency, auto-discoverable via mDNS.

---

## Table of contents

1. [Overview](#overview)
2. [Choosing FIFO vs TCP](#choosing-fifo-vs-tcp)
3. [FIFO sink](#fifo-sink)
   - [Layer map](#fifo-layer-map)
   - [PCM sink vtable](#pcm-sink-vtable-pcm-fifoc)
   - [The DMA thread](#the-dma-thread)
   - [FIFO pre-open strategy](#fifo-pre-open-strategy)
   - [stdout mode](#stdout-mode)
   - [Track transitions and EOF prevention](#track-transitions-and-eof-prevention)
   - [Startup order](#startup-order-fifo)
   - [Snapserver configuration](#snapserver-configuration-fifo)
4. [TCP sink](#tcp-sink)
   - [Layer map](#tcp-layer-map)
   - [PCM sink vtable](#pcm-sink-vtable-pcm-tcpc)
   - [Connection lifecycle](#connection-lifecycle)
   - [Reconnect on error](#reconnect-on-error)
   - [Startup order](#startup-order-tcp)
   - [Snapserver configuration](#snapserver-configuration-tcp)
5. [Auto-discovery via mDNS](#auto-discovery-via-mdns)
6. [FFI boundary (`crates/sys`)](#ffi-boundary-cratessys)
7. [Settings and startup (`crates/settings`)](#settings-and-startup-cratessettings)
8. [Other pipe consumers](#other-pipe-consumers)
9. [Gotchas and known limits](#gotchas-and-known-limits)

---

## Overview

Both sinks write raw **S16LE stereo PCM at 44100 Hz** — the same byte stream
snapserver expects regardless of source type. There is no Rust crate involved;
both are pure-C PCM sinks with a thin Rust FFI wrapper for configuration.

---

## Choosing FIFO vs TCP

| | FIFO sink | TCP sink |
|---|---|---|
| Filesystem entry required | Yes (`/tmp/snapfifo`) | No |
| Snapserver source type | `pipe://` | `tcp://` |
| Startup order sensitive | Yes — rockboxd first | Yes — snapserver first |
| Reconnect on snapserver restart | No (FIFO stays open) | Yes (auto on next play) |
| Auto-discovered in UI | No (static virtual device) | Yes (mDNS `_snapcast._tcp.local.`) |
| stdout pipe support | Yes (`fifo_path = "-"`) | No |
| Config | `fifo_path` | `snapcast_tcp_host` + `snapcast_tcp_port` |

**Use FIFO** when you want stdout piping or prefer the traditional pipe model.

**Use TCP** when you want UI-based auto-discovery, multiple snapservers, or
don't want a filesystem dependency.

---

## FIFO sink

### FIFO layer map

```
┌────────────────────────────────────────────────────────┐
│  Rockbox C firmware  (pcm.c, audio thread)             │
│    pcm_play_data() → sink.ops.play()                   │
│    pcm_play_dma_complete_callback() per chunk          │
└───────────────────┬────────────────────────────────────┘
                    │ raw S16LE stereo PCM chunks
┌───────────────────▼────────────────────────────────────┐
│  firmware/target/hosted/pcm-fifo.c                     │
│    pcm_fifo_set_path()  — pre-creates FIFO, opens fd   │
│    sink_dma_start()     — spawns fifo_thread           │
│    fifo_thread()        — blocking write() loop        │
│    sink_dma_stop()      — signals thread, keeps fd     │
└───────────────────┬────────────────────────────────────┘
                    │ blocking write() to FIFO or stdout
┌───────────────────▼────────────────────────────────────┐
│  Named FIFO  (/tmp/snapfifo)  or  stdout               │
└───────────────────┬────────────────────────────────────┘
                    │ read()
┌───────────────────▼────────────────────────────────────┐
│  snapserver  (pipe:// source)                          │
│  — or —                                                │
│  ffplay / aplay / custom consumer                      │
└────────────────────────────────────────────────────────┘
```

### PCM sink vtable (`pcm-fifo.c`)

`firmware/target/hosted/pcm-fifo.c` implements `struct pcm_sink`:

| Op                | Implementation                                              |
|-------------------|-------------------------------------------------------------|
| `init`            | `pthread_mutex_init` (recursive)                            |
| `postinit`        | no-op                                                       |
| `set_freq`        | no-op (output is always 44100 Hz; snapserver must match)    |
| `lock` / `unlock` | `pthread_mutex_lock/unlock`                                 |
| `play`            | `sink_dma_start` — opens fd if needed, spawns `fifo_thread` |
| `stop`            | `sink_dma_stop` — signals thread, joins; keeps fd open      |

`fifo_pcm_sink` is registered at index `PCM_SINK_FIFO = 1` in `firmware/pcm.c`.

### The DMA thread

`sink_dma_start(addr, size)` stores the initial PCM pointer/length under the
mutex, then spawns `fifo_thread`. The thread mimics a hardware DMA interrupt
loop:

```
while not stopped:
    1. lock → grab (data, size) → clear pcm_data/pcm_size → unlock
    2. while size > 0 and not stopped:
           n = write(fifo_fd, data, size)
           handle EINTR/EAGAIN (retry)
           advance data pointer, decrement size
    3. lock → pcm_play_dma_complete_callback(OK, &pcm_data, &pcm_size) → unlock
    4. if no more data: break
    5. pcm_play_dma_status_callback(STARTED)
```

Pacing comes naturally from the blocking FIFO write — the kernel suspends the
thread until the reader drains data, locking throughput to the consumer's rate.

### FIFO pre-open strategy

`pcm_fifo_set_path(path)` is called once at startup:

#### 1. Create the FIFO

```c
mkfifo(path, 0666);   // EEXIST is ignored
```

#### 2. Open with a permanent writer reference

```c
fd = open(path, O_RDWR | O_NONBLOCK);
// then clear O_NONBLOCK:
fcntl(fd, F_SETFL, flags & ~O_NONBLOCK);
```

**Why `O_RDWR`?** Opening `O_WRONLY` blocks until a reader is present. `O_RDWR`
succeeds immediately and keeps the open-writer-count at ≥1 for the process
lifetime — snapserver never sees premature EOF between tracks.

**Why clear `O_NONBLOCK`?** Writes must block when the kernel buffer is full to
provide natural back-pressure. Leaving `O_NONBLOCK` set would produce `EAGAIN`
and corrupt the stream.

### stdout mode

When `fifo_path = "-"`, the sink writes to stdout:

```sh
rockboxd | ffplay -f s16le -ar 44100 -ac 2 -
```

`pcm_fifo_set_path("-")` redirects fd 1 to stderr before any PCM is written so
internal `printf()` output never pollutes the PCM stream.

### Track transitions and EOF prevention

`sink_dma_stop()` does **not** close `fifo_fd`. On POSIX, a named FIFO's read
end sees EOF only when all write-side fds are closed. By keeping `fifo_fd` open
across track boundaries, snapserver sees a continuous stream with no gaps.

### Startup order (FIFO)

**rockboxd must start before snapserver.**

```
1. rockboxd starts  → pcm_fifo_set_path() → FIFO created, O_RDWR fd held
2. snapserver starts → opens FIFO O_RDONLY → blocks until data flows
3. Playback begins  → fifo_thread writes → snapserver distributes to clients
```

### Snapserver configuration (FIFO)

```ini
# /etc/snapserver.conf  (or /usr/local/etc/snapserver.conf on macOS)
[stream]
source = pipe:///tmp/snapfifo?name=default&sampleformat=44100:16:2
```

> On macOS, snapserver ≥ v0.35.0 ignores the `-s` CLI flag. Use the config
> file.

---

## TCP sink

### TCP layer map

```
┌────────────────────────────────────────────────────────┐
│  Rockbox C firmware  (pcm.c, audio thread)             │
│    pcm_play_data() → sink.ops.play()                   │
│    pcm_play_dma_complete_callback() per chunk          │
└───────────────────┬────────────────────────────────────┘
                    │ raw S16LE stereo PCM chunks
┌───────────────────▼──────────────────────────────────────┐
│  firmware/target/hosted/pcm-tcp.c                        │
│    pcm_tcp_set_host() / pcm_tcp_set_port()               │
│    sink_dma_start()  — connects if needed, spawns thread │
│    tcp_thread()      — blocking write() loop             │
│    sink_dma_stop()   — signals thread, keeps socket      │
└───────────────────┬──────────────────────────────────────┘
                    │ blocking write() over TCP
┌───────────────────▼────────────────────────────────────┐
│  TCP socket  (snapserver host:port)                    │
└───────────────────┬────────────────────────────────────┘
                    │ recv()
┌───────────────────▼────────────────────────────────────┐
│  snapserver  (tcp:// source, server mode)              │
│      │                                                 │
│  ┌───┴──────┬──────────┐                               │
│  ▼          ▼          ▼                               │
│ snapclient snapclient snapclient                       │
└────────────────────────────────────────────────────────┘
```

### PCM sink vtable (`pcm-tcp.c`)

`firmware/target/hosted/pcm-tcp.c` implements `struct pcm_sink`:

| Op                | Implementation                                              |
|-------------------|-------------------------------------------------------------|
| `init`            | `pthread_mutex_init` (recursive)                            |
| `postinit`        | no-op                                                       |
| `set_freq`        | no-op (output is always 44100 Hz; snapserver must match)    |
| `lock` / `unlock` | `pthread_mutex_lock/unlock`                                 |
| `play`            | `sink_dma_start` — connects if needed, spawns `tcp_thread`  |
| `stop`            | `sink_dma_stop` — signals thread, joins; keeps socket open  |

`tcp_pcm_sink` is registered at index `PCM_SINK_SNAPCAST_TCP = 6` in
`firmware/pcm.c`.

### Connection lifecycle

`sink_dma_start()` calls `tcp_connect_once()` if `tcp_fd < 0`:

```c
static int tcp_connect_once(void)
{
    // getaddrinfo(tcp_host, port) → socket() → connect()
    // returns fd on success, -1 on failure (logs error, drops audio)
}
```

The socket is kept open across `stop()` → `play()` transitions, just as the
FIFO fd is. snapserver's reader sees a continuous stream between tracks.

### Reconnect on error

If `write()` returns a hard error (`EPIPE`, `ECONNRESET`, etc.), `tcp_thread`
closes the socket (`tcp_fd = -1`) and sets `tcp_stop = true`. The next call to
`sink_dma_start()` finds `tcp_fd < 0` and attempts a fresh `connect()`. This
handles snapserver restarts gracefully — the connection is re-established
automatically on the next track or resume.

### Startup order (TCP)

**snapserver must be running and listening before playback starts.**

```
1. snapserver starts  → listens on tcp://0.0.0.0:4953
2. rockboxd starts    → pcm_tcp_set_host/port() stores config
3. Playback begins    → sink_dma_start() → tcp_connect_once() → connects
4. tcp_thread writes  → snapserver receives → distributes to clients
```

Unlike the FIFO sink there is no permanent pre-connection at startup. The
socket is opened on the first `play()` call.

### Snapserver configuration (TCP)

```ini
# /etc/snapserver.conf  (or /usr/local/etc/snapserver.conf on macOS)
[stream]
source = tcp://0.0.0.0:4953?name=default&sampleformat=44100:16:2
```

`settings.toml` (manual config, not needed when selecting from the UI):

```toml
audio_output      = "snapcast_tcp"
snapcast_tcp_host = "192.168.1.x"   # IP of the machine running snapserver
snapcast_tcp_port = 4953            # default snapserver TCP source port
```

---

## Auto-discovery via mDNS

snapserver advertises itself via mDNS as `_snapcast._tcp.local.`. rockboxd
scans for this service at startup via `scan_snapcast_servers()` in
`crates/server/src/scan.rs`, which browses `_snapcast._tcp.local.` using the
`mdns-sd` crate and adds discovered servers to the shared devices list.

Discovered servers appear immediately in:

- **Web UI** — the device picker in the control bar (lime-green radio icon)
- **Desktop app (GPUI)** — the device picker popup

Clicking a discovered server calls `PUT /devices/:id/connect`, which:

1. Calls `pcm_tcp_set_host(device.ip)` and `pcm_tcp_set_port(device.port)`.
2. Calls `pcm_switch_sink(PCM_SINK_SNAPCAST_TCP)`.
3. Persists `audio_output = "snapcast_tcp"`, `snapcast_tcp_host`, and
   `snapcast_tcp_port` to `settings.toml` so the selection survives restart.

No manual `settings.toml` editing is needed when using the UI.

---

## FFI boundary (`crates/sys`)

### FIFO

```rust
// crates/sys/src/lib.rs
extern "C" { fn pcm_fifo_set_path(path: *const c_char); }

// crates/sys/src/sound/pcm.rs
pub fn fifo_set_path(path: &str) {
    let cpath = CString::new(path).expect("path must not contain null bytes");
    unsafe { crate::pcm_fifo_set_path(cpath.as_ptr()) }
    std::mem::forget(cpath);  // C code stores and re-reads pointer at runtime
}
```

### TCP

```rust
// crates/sys/src/lib.rs
extern "C" {
    fn pcm_tcp_set_host(host: *const c_char);
    fn pcm_tcp_set_port(port: c_ushort);
}

// crates/sys/src/sound/pcm.rs
pub fn tcp_set_host(host: &str) {
    let chost = CString::new(host).expect("host must not contain null bytes");
    unsafe { crate::pcm_tcp_set_host(chost.as_ptr()) }
    std::mem::forget(chost);
}

pub fn tcp_set_port(port: u16) {
    unsafe { crate::pcm_tcp_set_port(port) }
}
```

`std::mem::forget` is used in both cases because the C code stores the raw
pointer and reads it later (in `sink_dma_start`'s connect / fallback path).
Dropping the `CString` would free the memory while C holds a dangling pointer.
Since these are startup-time config calls, leaking is acceptable.

---

## Settings and startup (`crates/settings`)

`crates/settings/src/lib.rs:load_settings()` handles both sinks:

```rust
Some("fifo") => {
    let path = settings.fifo_path.as_deref().unwrap_or("/tmp/rockbox.fifo");
    pcm::fifo_set_path(path);
    pcm::switch_sink(pcm::PCM_SINK_FIFO);
}
Some("snapcast_tcp") => {
    if let Some(ref host) = settings.snapcast_tcp_host {
        let port = settings.snapcast_tcp_port.unwrap_or(4953);
        pcm::tcp_set_host(host);
        pcm::tcp_set_port(port);
        pcm::switch_sink(pcm::PCM_SINK_SNAPCAST_TCP);
    }
}
```

### All Snapcast settings keys

| Key                  | Type   | Default               | Sink  | Description                              |
|----------------------|--------|-----------------------|-------|------------------------------------------|
| `audio_output`       | string | `"builtin"`           | both  | `"fifo"` or `"snapcast_tcp"`             |
| `fifo_path`          | string | `"/tmp/rockbox.fifo"` | FIFO  | FIFO path, or `"-"` for stdout           |
| `snapcast_tcp_host`  | string | —                     | TCP   | IP / hostname of the snapserver machine  |
| `snapcast_tcp_port`  | u16    | `4953`                | TCP   | snapserver TCP source port               |

---

## Other pipe consumers

Since both sinks carry raw S16LE stereo 44100 Hz PCM, the FIFO sink works with
any tool that accepts that format:

```sh
# Play directly with ffplay (stdout mode)
rockboxd | ffplay -f s16le -ar 44100 -ac 2 -

# Encode on the fly
rockboxd | ffmpeg -f s16le -ar 44100 -ac 2 -i - output.mp3

# Play with sox
rockboxd | play -t raw -r 44100 -e signed -b 16 -c 2 -

# Inspect levels with aplay (Linux)
rockboxd | aplay -f S16_LE -r 44100 -c 2
```

All of these require `fifo_path = "-"` and are only available with the FIFO
sink. The TCP sink does not support stdout mode.

---

## Gotchas and known limits

### 1. Startup order is critical for both sinks

- **FIFO**: rockboxd must open the FIFO before snapserver. Reverse order causes
  snapserver to hold the only writer reference; when it closes, readers see EOF.
  Restart snapserver if this happens.
- **TCP**: snapserver must be listening before playback starts. If snapserver
  is not yet running, `sink_dma_start` logs a warning and drops audio for that
  buffer. It reconnects automatically on the next play call.

### 2. Fixed 44100 Hz, S16LE stereo

Neither sink resamples. `set_freq` is a no-op. The firmware resamples tracks
internally before they reach the sink, but the output is always 44100 Hz.
Configure `sampleformat=44100:16:2` on the snapserver side.

### 3. No volume control through the sink

Volume is applied by the Rockbox DSP pipeline before PCM reaches the sink.
Adjust volume through the Rockbox API or client applications.

### 4. Consumer back-pressure controls playback speed

Both sinks use blocking `write()`. A slow or stalled consumer stalls
`write()`, which stalls the DMA callback loop, which pauses decoding. This
is correct for synchronized output but means a crashed consumer freezes
playback. Restart snapserver to recover.

### 5. macOS `snapserver.conf` vs CLI flag

The `-s` flag to snapserver is silently ignored on macOS (≥ v0.35.0).
Always use the config file for both `pipe://` and `tcp://` sources.

### 6. TCP reconnect drops in-flight buffer

When the write loop detects `EPIPE` it closes the socket immediately. The
current audio buffer is discarded. Reconnection happens on the next
`sink_dma_start()` call, so there will be a brief audio gap when snapserver
restarts.

### 7. Logging uses `tracing`, never `println!`

All Rust-side diagnostic output must go through `tracing`. `println!` and
`eprintln!` bypass the log filter and — in stdout/FIFO mode — can corrupt the
PCM stream. Use `RUST_LOG=debug rockboxd` to see debug output on stderr.
