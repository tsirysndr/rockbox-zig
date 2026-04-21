# Snapcast / FIFO PCM Sink

This document traces every hop an audio frame takes from the Rockbox C firmware
through the FIFO PCM sink to a Snapcast server (or any other pipe consumer).

---

## Table of contents

1. [Overview](#overview)
2. [Layer map](#layer-map)
3. [PCM sink vtable (`pcm-fifo.c`)](#pcm-sink-vtable-pcm-fifoc)
4. [The DMA thread](#the-dma-thread)
5. [FIFO pre-open strategy](#fifo-pre-open-strategy)
6. [stdout mode](#stdout-mode)
7. [Track transitions and EOF prevention](#track-transitions-and-eof-prevention)
8. [FFI boundary (`crates/sys`)](#ffi-boundary-cratessys)
9. [Settings and startup (`crates/settings`)](#settings-and-startup-cratessettings)
10. [Snapcast integration](#snapcast-integration)
11. [Startup order](#startup-order)
12. [Snapserver configuration (macOS)](#snapserver-configuration-macos)
13. [Other pipe consumers](#other-pipe-consumers)
14. [Gotchas and known limits](#gotchas-and-known-limits)

---

## Overview

The FIFO sink writes raw **S16LE stereo PCM at 44100 Hz** to either a named
FIFO (pipe) or stdout. Its primary use case is feeding
[Snapcast](https://github.com/badaix/snapcast) for synchronized multi-room
playback, but any consumer that reads a raw PCM stream works — `ffplay`,
`aplay`, `sox`, custom scripts, etc.

There is no Rust crate involved. This is a pure-C PCM sink with a thin Rust
FFI wrapper for configuration.

---

## Layer map

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

---

## PCM sink vtable (`pcm-fifo.c`)

`firmware/target/hosted/pcm-fifo.c` implements `struct pcm_sink` with the
following vtable:

| Op                | Implementation                                              |
|-------------------|-------------------------------------------------------------|
| `init`            | `pthread_mutex_init` (recursive)                            |
| `postinit`        | no-op                                                       |
| `set_freq`        | no-op (output is always 44100 Hz; snapserver must match)    |
| `lock` / `unlock` | `pthread_mutex_lock/unlock`                                 |
| `play`            | `sink_dma_start` — opens fd if needed, spawns `fifo_thread` |
| `stop`            | `sink_dma_stop` — signals thread, joins; keeps fd open      |

`fifo_pcm_sink` is registered at index `PCM_SINK_FIFO = 1` in the `sinks[]`
array in `firmware/pcm.c`.

---

## The DMA thread

`sink_dma_start(addr, size)` stores the initial PCM pointer/length under the
mutex, then spawns `fifo_thread`. The thread mimics a hardware DMA interrupt
loop:

```
while not stopped:
    1. lock → grab (data, size) → clear pcm_data/pcm_size → unlock
    2. if data:
           while size > 0 and not stopped:
               n = write(fifo_fd, data, size)
               handle EINTR/EAGAIN (retry)
               advance data pointer, decrement size
    3. lock → pcm_play_dma_complete_callback(OK, &pcm_data, &pcm_size) → unlock
    4. if no more data: break
    5. pcm_play_dma_status_callback(STARTED)   ← tells audio engine chunk consumed
```

The inner write loop handles partial writes and `EINTR`/`EAGAIN` correctly,
advancing the pointer on each successful `write()` call. Pacing comes
naturally from the blocking FIFO write — the kernel suspends the thread until
the reader drains data, keeping throughput locked to the consumer's read rate.

---

## FIFO pre-open strategy

`pcm_fifo_set_path(path)` is called once at startup from Rust settings code.
It does two things:

### 1. Create the FIFO

```c
mkfifo(path, 0666);   // EEXIST is ignored
```

This ensures the filesystem entry exists before snapserver starts, so
snapserver's `open()` can succeed immediately.

### 2. Open with a permanent writer reference

```c
fd = open(path, O_RDWR | O_NONBLOCK);
// then clear O_NONBLOCK:
flags = fcntl(fd, F_GETFL);
fcntl(fd, F_SETFL, flags & ~O_NONBLOCK);
```

**Why `O_RDWR`?** On Linux and macOS, opening a FIFO `O_WRONLY` blocks until
a reader is present. `O_RDWR` succeeds immediately even with no reader, and
critically it keeps the FIFO's open-writer-count at ≥1 for the entire lifetime
of the process. This means:

- A reader that comes and goes (snapserver restart, client reconnect) never
  causes the FIFO writer to receive `EPIPE` or the reader to see premature EOF.
- Between tracks, when `fifo_thread` exits and `fifo_fd` is not closed,
  snapserver's reader stays connected and continues to block-read cleanly.

**Why clear `O_NONBLOCK`?** After pre-opening, writes must block when the
kernel pipe buffer is full, providing natural back-pressure / pacing. If
`O_NONBLOCK` were left set, writes would return `EAGAIN` when the consumer is
slow, corrupting the stream.

---

## stdout mode

When `fifo_path = "-"`, the sink writes to stdout. This enables piping:

```sh
rockboxd | ffplay -f s16le -ar 44100 -ac 2 -
```

Because Rockbox C code internally uses `printf()`/`puts()` on fd 1, stdout
mode redirects fd 1 to stderr before any PCM is written:

```c
static void redirect_stdout_to_stderr(void)
{
    stdout_pcm_fd = dup(STDOUT_FILENO);   // save real stdout
    dup2(STDERR_FILENO, STDOUT_FILENO);   // fd 1 → stderr
}
```

All subsequent `printf()` output goes to stderr (visible in the terminal but
not in the pipe), while PCM writes go to `stdout_pcm_fd` — the saved copy of
the original stdout. The PCM stream is never polluted by log output.

This works only if `redirect_stdout_to_stderr()` is called before any C code
writes to fd 1. `pcm_fifo_set_path("-")` calls it at startup, which is before
any audio decoding begins.

---

## Track transitions and EOF prevention

`sink_dma_stop()` **does not close `fifo_fd`**:

```c
static void sink_dma_stop(void)
{
    fifo_stop = true;
    if (fifo_running) {
        pthread_join(fifo_tid, NULL);
        fifo_running = false;
    }
    // fifo_fd intentionally left open
}
```

On POSIX, a named FIFO's read end sees EOF only when all write-side file
descriptors are closed. By keeping `fifo_fd` open across track boundaries,
the consumer (snapserver) sees a continuous stream with no gaps. It never has
to reconnect or re-open the pipe.

The only time `fifo_fd` is closed is if `pcm_fifo_set_path()` is called again
with a new path — an operation that doesn't happen at runtime.

---

## FFI boundary (`crates/sys`)

`crates/sys/src/lib.rs` declares the C function:

```rust
extern "C" {
    pub fn pcm_fifo_set_path(path: *const c_char);
}
```

`crates/sys/src/sound/pcm.rs` wraps it safely:

```rust
pub fn fifo_set_path(path: &str) {
    let cpath = CString::new(path).expect("path must not contain null bytes");
    unsafe { crate::pcm_fifo_set_path(cpath.as_ptr()) }
    std::mem::forget(cpath);  // C code only reads during init — leaking is fine
}
```

`std::mem::forget` is used because `pcm_fifo_set_path` stores a raw pointer to
the string and reads it later (e.g. in `sink_dma_start`'s fallback path).
Dropping the `CString` would free the memory while C still holds a dangling
pointer. Since this is a one-time startup call, leaking is acceptable.

`pcm_switch_sink(PCM_SINK_FIFO)` switches the active sink. This is also an
`extern "C"` call wrapped in `pcm::switch_sink(sink: i32) -> bool`.

---

## Settings and startup (`crates/settings`)

`crates/settings/src/lib.rs:load_settings()` reads
`~/.config/rockbox.org/settings.toml` and handles the FIFO case:

```rust
Some("fifo") => {
    let path = settings.fifo_path.as_deref().unwrap_or("/tmp/rockbox.fifo");
    pcm::fifo_set_path(path);
    pcm::switch_sink(pcm::PCM_SINK_FIFO);
    tracing::info!("audio output: fifo ({})", path);
}
```

Relevant `settings.toml` keys:

| Key            | Type   | Default               | Description                           |
|----------------|--------|-----------------------|---------------------------------------|
| `audio_output` | string | `"builtin"`           | Set to `"fifo"` to activate this sink |
| `fifo_path`    | string | `"/tmp/rockbox.fifo"` | FIFO path, or `"-"` for stdout        |

---

## Snapcast integration

[Snapcast](https://github.com/badaix/snapcast) is a synchronised multi-room
audio system. snapserver reads a PCM source and distributes it to one or more
snapclient instances with sub-millisecond synchronisation.

The FIFO sink is designed to feed snapserver's `pipe://` source type. Once
configured, the stream looks like:

```
rockboxd ──write()──▶ /tmp/snapfifo ──read()──▶ snapserver
                                                     │
                                          ┌──────────┴──────────┐
                                          ▼                      ▼
                                    snapclient              snapclient
                                  (living room)             (kitchen)
```

### snapserver configuration

Add a stream source to `/etc/snapserver.conf` (or
`/usr/local/etc/snapserver.conf` on macOS):

```ini
[stream]
source = pipe:///tmp/snapfifo?name=default&sampleformat=44100:16:2
```

The `sampleformat=44100:16:2` parameter is required on snapserver v0.35+.
The `-s` CLI flag is **ignored** on macOS; it must be set in the config file.

Start snapserver after rockboxd is running (see startup order below):

```sh
snapserver
```

Connect clients:

```sh
snapclient --host localhost --player default
```

---

## Startup order

**rockboxd must start before snapserver.**

If snapserver opens the FIFO first (before rockboxd calls `pcm_fifo_set_path`
which does the `O_RDWR` open), it gets the sole writer reference. When
snapserver later closes its end, the FIFO appears to have no writers and
subsequent readers see immediate EOF.

Correct order:

```
1. rockboxd starts  → pcm_fifo_set_path() → FIFO created, O_RDWR fd held
2. snapserver starts → opens FIFO O_RDONLY → blocks until data flows
3. Playback begins  → fifo_thread writes → snapserver reads → clients play
```

---

## Snapserver configuration (macOS)

On macOS, snapserver's `-s` (stream source) command-line flag is silently
ignored. The only way to configure the source is via the config file:

```ini
# /usr/local/etc/snapserver.conf
[stream]
source = pipe:///tmp/snapfifo?name=default&sampleformat=44100:16:2
```

Verify snapserver is reading it:

```sh
snapserver --config /usr/local/etc/snapserver.conf
```

---

## Other pipe consumers

Since the FIFO carries raw S16LE stereo 44100 Hz PCM, it works with any tool
that accepts that format:

```sh
# Play directly with ffplay
rockboxd | ffplay -f s16le -ar 44100 -ac 2 -

# Encode on the fly with ffmpeg
rockboxd | ffmpeg -f s16le -ar 44100 -ac 2 -i - output.mp3

# Play with sox
rockboxd | play -t raw -r 44100 -e signed -b 16 -c 2 -

# Inspect levels with aplay (Linux)
rockboxd | aplay -f S16_LE -r 44100 -c 2
```

All of these require `fifo_path = "-"` in `settings.toml` so rockboxd writes
to stdout.

---

## Gotchas and known limits

### 1. Startup order is critical

As described above, rockboxd must open the FIFO before snapserver. If
snapserver opens it first and later closes its write end, snapclient may see
EOF and stop buffering. Restart snapserver after rockboxd in that case.

### 2. Fixed 44100 Hz, S16LE stereo

The FIFO sink does not resample. The `set_freq` op is a no-op. If Rockbox
decodes a 48 kHz or 96 kHz track, the firmware resamples it internally to
the codec's sample rate, but the PCM output is always delivered to the sink
at 44100 Hz. Configure snapserver and any other consumer to match.

### 3. No volume control through the sink

Volume is applied by the Rockbox DSP pipeline before PCM reaches the sink.
The FIFO sink itself has no volume knob. Adjust volume through the Rockbox
API or client applications.

### 4. Consumer back-pressure controls playback speed

Because `fifo_fd` is in blocking mode, a slow or stalled consumer will cause
`write()` to block, which stalls `fifo_thread`, which eventually stalls the
DMA callback loop, which pauses decoding. This is correct behavior for
synchronized output, but it means a crashed or frozen snapserver will freeze
playback. Restart snapserver to recover.

### 5. macOS `snapserver.conf` vs CLI flag

The `-s` flag to snapserver is silently ignored on macOS (at least v0.35.0).
Always use the config file. See [Snapserver configuration (macOS)](#snapserver-configuration-macos).

### 6. Logging uses `tracing`, never `println!`

All Rust-side diagnostic output must go through `tracing`. `println!` and
`eprintln!` bypass the log filter and — in stdout mode — corrupt the PCM
stream. Use `RUST_LOG=debug rockboxd` to see debug output on stderr.
