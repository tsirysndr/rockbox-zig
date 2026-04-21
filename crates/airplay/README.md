# rockbox-airplay — AirPlay PCM Sink

This document traces every hop an audio frame takes from the Rockbox C firmware
through the `rockbox-airplay` Rust crate to an AirPlay (RAOP) receiver.

---

## Table of contents

1. [Overview](#overview)
2. [Layer map](#layer-map)
3. [PCM sink vtable (`pcm-airplay.c`)](#pcm-sink-vtable-pcm-airplayc)
4. [The DMA thread](#the-dma-thread)
5. [FFI boundary](#ffi-boundary)
6. [Session lifecycle (`lib.rs`)](#session-lifecycle-librs)
7. [RTSP handshake (`rtsp.rs`)](#rtsp-handshake-rtsprs)
8. [ALAC encoding (`alac.rs`)](#alac-encoding-alacrs)
9. [RTP audio stream (`rtp.rs`)](#rtp-audio-stream-rtprs)
10. [RTCP synchronisation](#rtcp-synchronisation)
11. [NTP timing responder](#ntp-timing-responder)
12. [Track transitions](#track-transitions)
13. [Configuration](#configuration)
14. [AirPlay 2 probe](#airplay-2-probe)
15. [Gotchas and known limits](#gotchas-and-known-limits)

---

## Overview

The AirPlay sink lets Rockbox stream audio to any RAOP-compatible receiver —
Apple TV, HomePod, Airport Express, or third-party software such as
[shairport-sync](https://github.com/mikebrady/shairport-sync). It implements
**AirPlay 1 (RAOP)** entirely in pure Rust with no external C libraries.

The protocol stack looks like:

```
RTSP/TCP  ──  session negotiation (ANNOUNCE, SETUP, RECORD, TEARDOWN)
RTP/UDP   ──  ALAC-encoded audio frames
RTCP/UDP  ──  synchronisation (NTP send-report) every ~350 ms
UDP       ──  NTP timing response service
```

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
│  firmware/target/hosted/pcm-airplay.c                  │
│    sink_dma_start()  → pcm_airplay_connect()           │
│    airplay_thread()  → pcm_airplay_write()             │
│    sink_dma_stop()   → pcm_airplay_stop()              │
└───────────────────┬────────────────────────────────────┘
                    │  extern "C" FFI
┌───────────────────▼────────────────────────────────────┐
│  crates/airplay/src/lib.rs                             │
│    AirPlaySession { sender, rtsp, buf, first_frame }   │
│    pcm_airplay_connect() — RTSP handshake              │
│    pcm_airplay_write()   — ALAC frame dispatch         │
│    pcm_airplay_stop()    — TEARDOWN + session clear    │
└───────┬───────────────────────┬────────────────────────┘
        │ RTSP/TCP              │ ALAC frames
┌───────▼────────────┐  ┌───────▼──────────────────────┐
│  rtsp.rs           │  │  alac.rs                     │
│  RtspClient        │  │  encode_frame()              │
│  ANNOUNCE / SETUP  │  │  BitWriter                   │
│  RECORD / TEARDOWN │  │  352 S16LE → 1411-byte frame │
└────────────────────┘  └───────┬──────────────────────┘
                                │ encoded frames
                        ┌───────▼──────────────────────┐
                        │  rtp.rs                      │
                        │  RtpSender                   │
                        │  send_audio() — RTP/UDP      │
                        │  send_sync()  — RTCP         │
                        │  timing_responder() — NTP    │
                        └──────────────────────────────┘
                                │ UDP packets
                        ┌───────▼──────────────────────┐
                        │  AirPlay receiver            │
                        │  (Apple TV, shairport-sync…) │
                        └──────────────────────────────┘
```

---

## PCM sink vtable (`pcm-airplay.c`)

`firmware/target/hosted/pcm-airplay.c` implements `struct pcm_sink` with the
following vtable:

| Op                | Implementation                                                      |
|-------------------|---------------------------------------------------------------------|
| `init`            | `pthread_mutex_init` (recursive)                                    |
| `postinit`        | no-op                                                               |
| `set_freq`        | records `current_sample_rate` from `hw_freq_sampr[freq]`            |
| `lock` / `unlock` | `pthread_mutex_lock/unlock`                                         |
| `play`            | `sink_dma_start` — connects, spawns `airplay_thread`                |
| `stop`            | `sink_dma_stop` — signals thread, joins, calls `pcm_airplay_stop()` |

`airplay_pcm_sink` is registered at index `PCM_SINK_AIRPLAY = 2` in the
`sinks[]` array in `firmware/pcm.c`.

---

## The DMA thread

`sink_dma_start(addr, size)` stores the initial PCM pointer/length under the
mutex, then spawns `airplay_thread`. The thread mimics a hardware DMA
interrupt loop:

```
while not stopped:
    1. lock → grab (data, size) → clear pcm_data/pcm_size → unlock
    2. if data:  pcm_airplay_write(data, size)
    3. lock → pcm_play_dma_complete_callback(OK, &pcm_data, &pcm_size) → unlock
    4. if no more data: break
    5. pcm_play_dma_status_callback(STARTED)   ← tells audio engine chunk consumed
```

Unlike the FIFO sink, there is **no explicit real-time pacing** in C. Pacing is
handled inside `rtp.rs` — the RTP sender sleeps to maintain the correct
wall-clock transmission rate based on the RTP timestamp increment.

---

## FFI boundary

`crates/airplay/src/lib.rs` exports three `#[no_mangle] extern "C"` functions:

| C symbol               | Rust function          | Purpose                              |
|------------------------|------------------------|--------------------------------------|
| `pcm_airplay_set_host` | `pcm_airplay_set_host` | Store `HOST` + `PORT` atomics/mutex  |
| `pcm_airplay_connect`  | `pcm_airplay_connect`  | Open RTSP + RTP session (idempotent) |
| `pcm_airplay_write`    | `pcm_airplay_write`    | Buffer PCM, encode ALAC, send RTP    |
| `pcm_airplay_stop`     | `pcm_airplay_stop`     | Send TEARDOWN, clear session         |

`HOST` is a `Mutex<Option<String>>` and `PORT` is an `AtomicU16` (default
5000). `SESSION` is a `Mutex<Option<AirPlaySession>>` — the session is
created once and reused across `write` calls for the lifetime of a track.

### Force-link shim

Because `rockbox-airplay` is an `rlib`, its symbols are only included in
`librockbox_cli.a` if something references them. `crates/cli/src/lib.rs`
contains:

```rust
use rockbox_airplay::_link_airplay as _;
```

where `_link_airplay` is a public no-op function in `lib.rs`. This is enough
to pull the entire crate into the link graph.

---

## Session lifecycle (`lib.rs`)

`pcm_airplay_connect()` is called from `sink_dma_start()` at the start of
every track. It is guarded by `SESSION`:

```
if SESSION is already Some → return OK immediately (idempotent)

1. Probe AirPlay 2 (non-fatal — logs and falls through on failure)
2. RtpSender::bind(host, ports)     ← binds three UDP sockets
3. RtspClient::new(host, port)      ← opens TCP connection to receiver
4. rtsp.announce(sdp)               ← sends SDP describing the ALAC stream
5. rtsp.setup(transport)            ← negotiates UDP port numbers
6. rtsp.record()                    ← starts the session
7. sender.send_initial_sync()       ← sends first RTCP sync packet
8. SESSION = Some(AirPlaySession { sender, rtsp, buf: [], first_frame: true })
```

`pcm_airplay_write(data, len)` appends the incoming PCM bytes to `buf`, then
drains complete 352-sample (1408-byte) frames in a loop:

```rust
while buf.len() >= FRAME_SIZE:
    frame_pcm = buf.drain(..FRAME_SIZE)
    alac_frame = alac::encode_frame(&frame_pcm)
    sender.send_audio(&alac_frame, first_frame)
    first_frame = false
```

`pcm_airplay_stop()` sends RTSP TEARDOWN and sets `SESSION = None`.

---

## RTSP handshake (`rtsp.rs`)

`RtspClient` speaks synchronous RTSP over a single TCP connection. The full
exchange for one session is:

### 1. ANNOUNCE

Sends an SDP body describing the ALAC codec:

```
v=0
o=iTunes <session_id> 0 IN IP4 <local_ip>
s=iTunes
c=IN IP4 <receiver_ip>
t=0 0
m=audio 0 RTP/AVP 96
a=rtpmap:96 AppleLossless
a=fmtp:96 352 0 16 40 10 14 2 255 0 0 44100
```

The `fmtp` parameters encode:
`<frames_per_packet> <version> <bit_depth> <rice_history_mult>
<rice_initial_history> <rice_limit> <channels> <max_run> <max_frame_bytes>
<avg_bit_rate> <sample_rate>`

### 2. SETUP

Sends a `Transport` header requesting UDP:

```
Transport: RTP/AVP/UDP;unicast;interleaved=0-1;
           client_port=<audio_port>-<ctrl_port>
```

`interleaved=0-1` is required by many receivers even though the transport is
UDP (not RTSP interleaved). The response carries the server's UDP port pair,
extracted by `parse_port()`.

### 3. RECORD

Starts the stream. Sends `RTP-Info` with sequence number and RTP timestamp.

### 4. SET_PARAMETER (volume)

Sets playback volume. Sent as a float string in a `text/parameters` body:
`volume: -20.0` (range −144 to 0; 0 is full volume).

### 5. TEARDOWN

Gracefully terminates the session. Called from `pcm_airplay_stop()`.

---

## ALAC encoding (`alac.rs`)

`encode_frame(samples: &[i16])` encodes exactly **352 stereo S16LE samples**
(1408 bytes of PCM) into an ALAC verbatim ("uncompressed escape") frame.

### Frame format

The Hammerton ALAC decoder expects this exact bit layout:

```
Bits  Width  Field
0–2     3    channels − 1  (= 1 for stereo)
3–6     4    discarded (0)
7–18   12    discarded (0)
19      1    hassize = 0
20–23   4    uncompressed_bytes = 0
24      1    isNotCompressed = 1   ← verbatim frame flag
25+    32    each sample as big-endian signed 16-bit, left then right
```

Output size = 4 bytes header + 352 × 2 channels × 2 bytes/sample
            = **1412 bytes** (rounded up to byte boundary).

### BitWriter

`BitWriter` accumulates bits MSB-first into a `Vec<u8>`:

```rust
fn write(&mut self, value: u64, nbits: u32)
fn align(&mut self)   // zero-pad to next byte boundary
```

The encoder calls `write` for the 25-bit header fields and then for each
sample (16 bits per channel, interleaved L/R), then `align()` to flush the
final byte.

---

## RTP audio stream (`rtp.rs`)

`RtpSender` opens **three UDP sockets** at construction time:

| Socket        | Direction               | Purpose             |
|---------------|-------------------------|---------------------|
| `audio_sock`  | → receiver audio port   | RTP audio frames    |
| `ctrl_sock`   | ↔ receiver control port | RTCP sync packets   |
| `timing_sock` | ↔ receiver timing port  | NTP timing exchange |

### `send_audio(frame, marker)`

Builds a 12-byte RTP header:

```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
├─┤─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┤
│V=2│P│X│  CC   │M│    PT=96    │       Sequence Number       │
├───────────────────────────────┼─────────────────────────────┤
│                  Timestamp (RTP clock units)                │
├─────────────────────────────────────────────────────────────┤
│                          SSRC                               │
└─────────────────────────────────────────────────────────────┘
```

- `M` (marker) = 1 on the first frame of a session, 0 thereafter.
- Timestamp increments by **352** per frame (one ALAC frame = 352 samples).
- SSRC is a random 32-bit value chosen at sender creation.

**Real-time pacing**: `send_audio` tracks the expected transmission instant
using `Instant` and `frame_count × Duration_per_frame` and calls
`thread::sleep` when the sender is running ahead.

---

## RTCP synchronisation

`send_sync(first)` sends a 20-byte RTCP NTP Send Report to the control socket
every **44 frames** (~350 ms at 44100 Hz):

```
Byte  Field
0     V=2, P=0, RC=0
1     PT=200 (SR) or 0xD4 (first sync)
2–3   length = 4 (words after fixed header)
4–7   SSRC
8–11  NTP timestamp seconds (since 1900-01-01)
12–15 NTP timestamp fraction (2^32 units)
16–19 RTP timestamp (matching the next audio frame's timestamp)
```

`NTP_EPOCH_DELTA = 0x83AA_7E80` converts UNIX time (seconds since 1970) to NTP
time (seconds since 1900).

The first sync packet (`first=true`) uses PT=`0xD4` (not standard SR) — some
receivers require this to accept the initial synchronisation.

---

## NTP timing responder

A background thread (`timing_responder`) listens on `timing_sock` and answers
NTP timing requests from the receiver:

```
Request  PT = 0xD2  (timing request)
Response PT = 0xD3  (timing response)

Response body (32 bytes):
  [0–3]   SSRC
  [4–7]   0 (reference seconds)
  [8–11]  0 (reference fraction)
  [12–15] received seconds  (echoed from request)
  [16–19] received fraction (echoed from request)
  [20–23] send seconds      (current NTP time)
  [24–27] send fraction     (current NTP time)
```

Many receivers stall playback if timing responses stop arriving. The thread
runs for the entire duration of the session.

---

## Track transitions

When Rockbox moves to the next track:

1. `sink_dma_stop()` is called → `pcm_airplay_stop()` → RTSP TEARDOWN →
   `SESSION = None`.
2. `sink_dma_start()` is called for the new track → `pcm_airplay_connect()` →
   new RTSP session with fresh RTP sequence/timestamp counters.

There is a brief gap (TEARDOWN round-trip + new ANNOUNCE/SETUP/RECORD) between
tracks. This is inherent to RAOP and is typically inaudible (<100 ms).

---

## Configuration

In `~/.config/rockbox.org/settings.toml`:

```toml
audio_output = "airplay"
airplay_host = "192.168.1.x"   # IP of the AirPlay receiver
airplay_port = 5000             # optional, default 5000
```

`crates/settings/src/lib.rs:load_settings()` reads these values and calls:

```rust
pcm::airplay_set_host(&host, port);
pcm::switch_sink(PCM_SINK_AIRPLAY);
```

`airplay_set_host` stores the host in `HOST: Mutex<Option<String>>` and the
port in `PORT: AtomicU16`. These are read by `pcm_airplay_connect()` at the
start of each track.

---

## AirPlay 2 probe

`pcm_airplay_connect()` first attempts an AirPlay 2 handshake (PTP-based). If
it fails (connection refused, or the receiver does not support AirPlay 2) the
error is logged at `tracing::debug!` level and the function falls through to the
AirPlay 1 / RAOP path. This makes the probe transparent to the user.

The AirPlay 2 path uses the cryptographic dependencies declared in
`Cargo.toml`:

```toml
x25519-dalek        # key exchange
ed25519-dalek       # signature
chacha20poly1305    # AEAD encryption
sha2, hkdf, hmac    # key derivation
num-bigint          # SRP big-integer arithmetic
```

None of these are needed for the AirPlay 1 code path.

---

## Gotchas and known limits

### 1. Only one simultaneous receiver

The `SESSION` mutex holds a single `AirPlaySession`. Sending to multiple
AirPlay devices simultaneously is not supported. For multi-room output use
the Squeezelite sink with multiple clients, or run multiple rockboxd instances.

### 2. Receiver must be on the local network

RAOP uses UDP with no NAT traversal. The receiver must be directly reachable
at the configured IP. Multicast discovery (mDNS/Bonjour) is not implemented —
you must supply the IP manually.

### 3. `interleaved=0-1` in Transport header

Even though the transport is plain UDP, most receivers require the
`interleaved=0-1` parameter in the SETUP `Transport` header. Omitting it causes
the receiver to ignore the `RECORD` command silently.

### 4. Verbatim ALAC only (no compression)

`alac.rs` only implements the verbatim escape frame (`isNotCompressed=1`).
Bitrate is fixed at `sample_rate × 4 bytes/s = 176,400 bytes/s` at 44.1 kHz.
This is fine for LAN streaming but wasteful compared to the compressed ALAC
path.

### 5. Fixed 44100 Hz sample rate

The RTSP SDP and ALAC frame size constants are hard-coded for 44100 Hz.
Playback of 48 kHz or 96 kHz tracks is not tested and may produce incorrect
pitch or receiver errors.

### 6. Logging uses `tracing`, never `println!`

All diagnostic output is routed through the `tracing` crate. To see the full
AirPlay negotiation:

```sh
RUST_LOG=rockbox_airplay=debug rockboxd
```

Never add `println!` or `eprintln!` — those bypass the log filter and pollute
stdout, breaking FIFO/pipe mode.
