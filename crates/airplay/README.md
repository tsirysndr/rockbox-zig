# rockbox-airplay — AirPlay PCM Sink

This document traces every hop an audio frame takes from the Rockbox C firmware
through the `rockbox-airplay` Rust crate to one or more AirPlay (RAOP)
receivers.

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
12. [Multi-room fan-out](#multi-room-fan-out)
13. [Track transitions](#track-transitions)
14. [Configuration](#configuration)
15. [AirPlay 2 probe](#airplay-2-probe)
16. [Gotchas and known limits](#gotchas-and-known-limits)

---

## Overview

The AirPlay sink lets Rockbox stream audio to any RAOP-compatible receiver —
Apple TV, HomePod, Airport Express, or third-party software such as
[shairport-sync](https://github.com/mikebrady/shairport-sync). Multiple
receivers can be configured simultaneously for multi-room playback.

The implementation is **AirPlay 1 (RAOP)** in pure Rust with no external C
libraries. AirPlay 2 pairing (HAP SRP6a + x25519 ECDH) is attempted as a
non-fatal probe before falling through to the AirPlay 1 path.

The protocol stack looks like:

```
RTSP/TCP  ──  session negotiation per receiver (ANNOUNCE, SETUP, RECORD, TEARDOWN)
RTP/UDP   ──  ALAC-encoded audio frames (same frame broadcast to all receivers)
RTCP/UDP  ──  synchronisation (NTP send-report) every ~350 ms per receiver
UDP       ──  shared NTP timing response service (one port, all receivers)
```

---

## Layer map

```
┌─────────────────────────────────────────────────────────────┐
│  Rockbox C firmware  (pcm.c, audio thread)                  │
│    pcm_play_data() → sink.ops.play()                        │
│    pcm_play_dma_complete_callback() per chunk               │
└───────────────────┬─────────────────────────────────────────┘
                    │ raw S16LE stereo PCM chunks
┌───────────────────▼─────────────────────────────────────────┐
│  firmware/target/hosted/pcm-airplay.c                       │
│    sink_dma_start()  → pcm_airplay_connect()                │
│    airplay_thread()  → pcm_airplay_write()                  │
│    sink_dma_stop()   → pcm_airplay_stop()                   │
└───────────────────┬─────────────────────────────────────────┘
                    │  extern "C" FFI
┌───────────────────▼─────────────────────────────────────────┐
│  crates/airplay/src/lib.rs                                  │
│    AirPlaySession {                                         │
│      receivers:    Vec<ReceiverHandle>,                     │
│      rtsp_clients: Vec<RtspClient>,                         │
│      timing:       TimingSocket,   ← shared, one port       │
│      pacing:       PacingClock,    ← shared clock           │
│      buf, first_frame,                                      │
│    }                                                        │
│    pcm_airplay_connect()  — handshake per receiver          │
│    pcm_airplay_write()    — encode once, fan out            │
│    pcm_airplay_stop()     — TEARDOWN all + session clear    │
└───┬───────────────────────┬─────────────────────────────────┘
    │ RTSP/TCP (per rx)     │ ALAC frames
┌───▼────────────┐  ┌───────▼─────────────────────────────────┐
│  rtsp.rs       │  │  alac.rs                                │
│  RtspClient    │  │  encode_frame()  — called once/frame    │
│  ANNOUNCE      │  │  BitWriter                              │
│  SETUP         │  │  352 S16LE → 1411-byte verbatim frame   │
│  RECORD        │  └───────┬─────────────────────────────────┘
│  SET_PARAMETER │          │ encoded frame (shared reference)
│  TEARDOWN      │  ┌───────▼─────────────────────────────────┐
└────────────────┘  │  rtp.rs                                 │
                    │  ReceiverHandle  (per receiver)         │
                    │    send_audio_packet() — RTP/UDP        │
                    │    send_sync()         — RTCP           │
                    │  TimingSocket (shared, one port)        │
                    │    timing_responder()  — NTP thread     │
                    │  PacingClock (shared)                   │
                    │    pace()  — one sleep for all rooms    │
                    └───────┬─────────────────────────────────┘
                            │ UDP packets (fan-out)
              ┌─────────────┼─────────────┐
       ┌──────▼──────┐ ┌────▼──────┐ ┌───▼──────┐
       │  Receiver 1 │ │ Receiver 2│ │    …     │
       └─────────────┘ └───────────┘ └──────────┘
```

---

## PCM sink vtable (`pcm-airplay.c`)

`firmware/target/hosted/pcm-airplay.c` implements `struct pcm_sink` with the
following vtable:

| Op                | Implementation                                                      |
|-------------------|---------------------------------------------------------------------|
| `init`            | `pthread_mutex_init` (recursive)                                    |
| `postinit`        | no-op                                                               |
| `set_freq`        | no-op (sample rate is fixed at 44100 Hz)                            |
| `lock` / `unlock` | `pthread_mutex_lock/unlock`                                         |
| `play`            | `sink_dma_start` — connects all receivers, spawns `airplay_thread` |
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

Real-time pacing is handled inside `PacingClock` in `rtp.rs` — the shared
clock sleeps once per frame after fanning out to all receivers.

---

## FFI boundary

`crates/airplay/src/lib.rs` exports these `#[no_mangle] extern "C"` functions:

| C symbol                    | Purpose                                                  |
|-----------------------------|----------------------------------------------------------|
| `pcm_airplay_set_host`      | Set a single receiver (clears any previous list)         |
| `pcm_airplay_add_receiver`  | Append one receiver to the multi-room list               |
| `pcm_airplay_clear_receivers` | Clear the receiver list before re-configuring          |
| `pcm_airplay_connect`       | Open RTSP + RTP sessions for all configured receivers    |
| `pcm_airplay_write`         | Buffer PCM, encode ALAC once, fan out to every receiver  |
| `pcm_airplay_stop`          | Send TEARDOWN to all, clear session                      |
| `pcm_airplay_close`         | Same as stop (called on sink switch)                     |

`SESSION` is a `Mutex<Option<AirPlaySession>>`. `CONFIG` is a
`Mutex<AirPlayConfig>` holding `receivers: Vec<(String, u16)>`.

### Force-link shim

Because `rockbox-airplay` is an `rlib`, its symbols are only included in
`librockbox_cli.a` if something references them. `crates/cli/src/lib.rs`
contains:

```rust
use rockbox_airplay::_link_airplay as _;
```

where `_link_airplay` is a public no-op function in `lib.rs`.

---

## Session lifecycle (`lib.rs`)

`pcm_airplay_connect()` is called from `sink_dma_start()` at the start of
every track. It is guarded by `SESSION`:

```
if SESSION is already Some → return OK immediately (idempotent)

1. Read receiver list from CONFIG
2. TimingSocket::bind()          ← one shared NTP timing port + responder thread
3. Choose shared initial_rtptime ← same value for ALL receivers (sync anchor)
4. For each configured receiver:
     a. connect_one(host, port, initial_rtptime, timing_port)
        ├── Probe AirPlay 2 (non-fatal)
        ├── ReceiverHandle::bind()      ← audio_sock + ctrl_sock
        ├── RtspClient::connect()       ← TCP to receiver
        ├── rtsp.announce(sdp)          ← SDP with ALAC params
        ├── rtsp.setup(ctrl, timing)    ← get server UDP ports
        ├── rx.connect(audio, ctrl)     ← connect audio_sock
        ├── rtsp.record(seq=0, ts)      ← start stream
        └── rtsp.set_parameter_volume(0.0)
     b. On failure: log warning, continue (partial success OK)
5. Abort only if ZERO receivers connected
6. session.send_initial_sync()    ← RTCP sync to all receivers
7. SESSION = Some(AirPlaySession { receivers, rtsp_clients, timing, pacing, … })
```

`pcm_airplay_write(data, len)` accumulates PCM in `buf`, then for each
complete 352-sample frame:

```rust
alac = encode_frame(&frame_bytes)          // encode ONCE

for rx in &mut receivers:
    rx.send_audio_packet(&alac, rtptime, …) // send to EACH receiver

pacing.advance()                            // increment rtptime + frames_sent
if frames_sent % 44 == 0:
    for rx: rx.send_sync(current_ts, next_ts, false)

pacing.pace()                               // sleep ONCE for all rooms
```

`pcm_airplay_stop()` sends RTSP TEARDOWN to every receiver, then sets
`SESSION = None`.

---

## RTSP handshake (`rtsp.rs`)

`RtspClient` speaks synchronous RTSP over a single TCP connection **per
receiver**. The TCP connection is kept alive in `AirPlaySession.rtsp_clients`
for the duration of the track — dropping it would cause the receiver to detect
EOF and tear down its audio socket.

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
a=min-latency:3528
```

The `fmtp` parameters: `<frames/pkt> <version> <bit_depth> <rice_history_mult>
<rice_initial_history> <rice_limit> <channels> <max_run> <max_frame_bytes>
<avg_bit_rate> <sample_rate>`.

No `a=rsaaeskey` line — encryption is disabled. The receiver sets
`stream.encrypted = 0` and passes frames straight to the ALAC decoder.

### 2. SETUP

Requests UDP transport, advertising our local ctrl and timing ports:

```
Transport: RTP/AVP/UDP;unicast;interleaved=0-1;mode=record;
           control_port=<local_ctrl>;timing_port=<shared_timing>
```

All receivers are advertised the **same** `timing_port` (the shared
`TimingSocket`). The response carries the server's audio, ctrl, and timing
ports, extracted by `parse_port()`.

### 3. RECORD

Starts the stream. Sends `RTP-Info` with sequence number 0 and the shared
`initial_rtptime`.

### 4. SET_PARAMETER (volume)

Sets playback volume to maximum (0.0 in RAOP's −144…0 range).

### 5. TEARDOWN

Gracefully terminates the session. Called per-receiver from
`pcm_airplay_stop()`.

---

## ALAC encoding (`alac.rs`)

`encode_frame(pcm: &[u8])` encodes exactly **352 stereo S16LE samples**
(1408 bytes of PCM) into an ALAC verbatim ("uncompressed escape") frame.

### Frame format

The Hammerton ALAC decoder (used by shairport-sync) expects this exact bit
layout — note there is **no** 4-bit element-instance tag after the channel
field:

```
Bits   Width  Field
0–2      3    channels − 1  (= 1 for stereo)
3–6      4    output_waiting — read and discarded
7–18    12    unknown        — read and discarded
19       1    hassize = 0
20–21    2    uncompressed_bytes = 0
22       1    isNotCompressed = 1   ← verbatim frame flag
23+     32    each sample as big-endian signed 16-bit, left then right
             (352 × L + R pairs = 22,528 bits)
```

Total: 23 header bits + 352 × 32 sample bits = 11,287 bits → **1411 bytes**
(padded to byte boundary with no END tag).

### BitWriter

`BitWriter` accumulates bits MSB-first into a `[u8; 1411]` buffer:

```rust
fn write(&mut self, value: u32, nbits: usize)
fn align(&mut self)   // zero-pad to next byte boundary
```

---

## RTP audio stream (`rtp.rs`)

Three types in `rtp.rs` handle the per-receiver and shared concerns:

### `ReceiverHandle` — per receiver

Owns the two UDP sockets for one AirPlay endpoint:

| Socket       | Direction               | Purpose             |
|--------------|-------------------------|---------------------|
| `audio_sock` | → receiver audio port   | RTP audio frames    |
| `ctrl_sock`  | ↔ receiver control port | RTCP sync packets   |

Also holds `ssrc` (random per receiver) and `seqnum` (wrapping u16).

`send_audio_packet(alac_frame, rtptime, frame_index, first)` builds and sends
one 12-byte RTP packet:

```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
├─┤─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┤
│V=2│P│X│  CC   │M│    PT=96    │       Sequence Number         │
├───────────────────────────────┼───────────────────────────────┤
│            Timestamp (shared rtptime — same for all receivers) │
├────────────────────────────────────────────────────────────────┤
│                  SSRC (per-receiver random u32)                │
└────────────────────────────────────────────────────────────────┘
```

- `M` (marker) = 1 on the first frame of a session only.
- Timestamp is **shared** across all receivers — all rooms decode the same
  logical frame position.

### `TimingSocket` — shared

One UDP socket bound to a random port. All receivers are told this single
port in SETUP. `timing_responder` (a background thread) answers any PT=0xD2
timing request from any source with a PT=0xD3 response containing the current
NTP time.

### `PacingClock` — shared

Tracks `stream_start` (an `Instant`), `frames_sent`, and the current `rtptime`.
After all receivers have been sent a frame, `pace()` sleeps until the frame's
wall-clock deadline:

```rust
let expected = stream_start + frames_sent × FRAME_DURATION_US;
if expected > Instant::now() { thread::sleep(expected - now); }
```

`FRAME_DURATION_US = 352 × 1_000_000 / 44100 ≈ 7982 µs`.

---

## RTCP synchronisation

`ReceiverHandle::send_sync(current_ts, next_ts, first)` sends a 20-byte RTCP
packet on the ctrl socket every **44 frames** (~350 ms at 44100 Hz):

```
Byte   Field
0      0x80 (normal) or 0x90 (first sync, extension bit set)
1      0xD4  (PT=212, Apple proprietary sync)
2–3    0x0007  (length field)
4–7    current RTP timestamp (frame just sent)
8–11   NTP seconds (since 1900-01-01, = UNIX_time + 0x83AA7E80)
12–15  NTP fraction (2^32 units per second)
16–19  next RTP timestamp (next frame to be sent)
```

`current_ts` and `next_ts` are derived from the shared `PacingClock.rtptime`,
so all receivers receive consistent timestamps.

---

## NTP timing responder

A single background thread (`timing_responder`) listens on the shared
`TimingSocket` and answers NTP timing requests from **all** receivers:

```
Request  PT = 0xD2  (timing request, from any receiver)
Response PT = 0xD3  (timing response)

Response layout (32 bytes):
  [0]    0x80
  [1]    0xD3
  [2–3]  sequence number (copied from request)
  [4–7]  padding (zero)
  [8–15] reference NTP (zero)
  [16–23] originate NTP (copied from request bytes [16–23])
  [24–31] receive/transmit NTP (current system time)
```

Using one socket for all receivers works because the responder uses
`send_to(src)` to reply to the exact source address of each request.

---

## Multi-room fan-out

The complete per-frame processing path in `AirPlaySession::send_frame()`:

```
1. encode_frame(&pcm)               → alac: [u8; 1411]   (once, ~5 µs)
2. for rx in receivers:
     rx.send_audio_packet(&alac, …) → UDP send            (per receiver, ~1 µs each)
3. pacing.advance()                 → increment rtptime, frames_sent
4. if frames_sent % 44 == 0:
     for rx in receivers:
       rx.send_sync(…)              → RTCP UDP send       (per receiver)
5. pacing.pace()                    → thread::sleep       (once, ~7982 µs avg)
```

With N receivers, steps 2 and 4 take O(N) sequential UDP sends (~1–2 µs each).
Even with 10 receivers the added latency (~20 µs) is negligible compared to
the 7982 µs frame budget.

### Sync accuracy

All receivers share the same `initial_rtptime` and receive each frame within
the same loop iteration (a few microseconds apart). Their playout timestamps
are identical. Actual synchronisation accuracy is bounded by:
- Receiver buffer depth (typically 1–3 s for shairport-sync)
- NTP timing exchange accuracy (usually < 5 ms on LAN)

This gives **AirPlay 1-level sync** — adequate for multi-room on a LAN.
Sample-accurate sync across rooms requires AirPlay 2's clock-anchoring, which
is a different protocol.

### Partial failure

If one receiver fails to connect during `pcm_airplay_connect()`, the error is
logged at `warn` level and the session continues with the remaining receivers.
The session is only aborted when **zero** receivers connect successfully.

---

## Track transitions

When Rockbox moves to the next track:

1. `sink_dma_stop()` → `pcm_airplay_stop()` → RTSP TEARDOWN on every receiver
   → `SESSION = None`.
2. `sink_dma_start()` → `pcm_airplay_connect()` → new RTSP sessions with
   fresh RTP sequence/timestamp counters and a new random `initial_rtptime`.

There is a brief gap (TEARDOWN round-trip + new ANNOUNCE/SETUP/RECORD) between
tracks, inherent to RAOP and typically inaudible (< 100 ms).

---

## Configuration

### Single receiver (backward-compatible)

```toml
audio_output = "airplay"
airplay_host = "192.168.1.50"   # IP of the AirPlay receiver
airplay_port = 5000             # optional, default 5000
```

### Multi-room

```toml
audio_output = "airplay"

[[airplay_receivers]]
host = "192.168.1.50"
port = 5000              # optional, default 5000

[[airplay_receivers]]
host = "192.168.1.51"

[[airplay_receivers]]
host = "192.168.1.52"
port = 5001
```

`airplay_receivers` takes precedence over `airplay_host`/`airplay_port` when
both are present. `crates/settings/src/lib.rs` calls
`pcm_airplay_clear_receivers()` then `pcm_airplay_add_receiver()` for each
entry.

### Runtime control

The Rust FFI also exposes:

```rust
pcm::airplay_set_host("192.168.1.50", 5000);     // replace list with one receiver
pcm::airplay_add_receiver("192.168.1.51", 5000); // append to list
pcm::airplay_clear_receivers();                   // clear before re-configuring
```

---

## AirPlay 2 probe

`connect_one()` first attempts an AirPlay 2 handshake (HAP-based). If it
fails the error is logged at `tracing::debug!` and the function falls through
to the AirPlay 1 / RAOP path. This makes the probe transparent to the user.

The AirPlay 2 path uses:

```toml
x25519-dalek        # ephemeral key exchange (PAIR-VERIFY)
ed25519-dalek       # long-term identity signature
chacha20poly1305    # AEAD encryption of the identity payload
sha2, hkdf, hmac    # key derivation
num-bigint          # SRP 3072-bit big-integer arithmetic (PAIR-SETUP)
```

None of these are needed for the AirPlay 1 code path.

---

## Gotchas and known limits

### 1. Receiver must be reachable via UDP

RAOP uses UDP with no NAT traversal. Every configured receiver must be
directly reachable at its IP from the machine running rockboxd. Multicast
discovery (mDNS/Bonjour) is not implemented — supply the IP manually.

### 2. `interleaved=0-1` in Transport header

Even though the transport is plain UDP, most receivers require the
`interleaved=0-1` parameter in the SETUP `Transport` header. Omitting it
causes the receiver to silently ignore the `RECORD` command.

### 3. Verbatim ALAC only (no compression)

`alac.rs` only implements the verbatim escape frame (`isNotCompressed=1`).
Bitrate is fixed at `sample_rate × 4 bytes/s = 176,400 bytes/s` at 44.1 kHz.
Fine for LAN streaming but higher than compressed ALAC.

### 4. Fixed 44100 Hz sample rate

The SDP and ALAC frame size constants are hard-coded for 44100 Hz. Playback
of 48 kHz or 96 kHz tracks is not tested.

### 5. Multi-room sync is LAN-quality, not sample-accurate

See [Sync accuracy](#sync-accuracy). AirPlay 2-level clock anchoring is not
implemented.

### 6. Logging uses `tracing`, never `println!`

All diagnostic output is routed through the `tracing` crate:

```sh
RUST_LOG=rockbox_airplay=debug rockboxd   # full protocol trace
RUST_LOG=info rockboxd                    # lifecycle events only
```

Never add `println!` or `eprintln!` — those bypass the log filter and can
corrupt the stdout PCM stream in FIFO mode.
