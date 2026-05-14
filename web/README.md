# Rockbox WASM — web player

This directory contains the browser embedding of the Rockbox audio engine.
The full Rockbox C firmware (audio engine, DSP pipeline, all codecs) is
compiled to WebAssembly by Emscripten; JavaScript glue wires it to the Web
Audio API for real-time output.

## Directory contents

| File                       | Purpose                                                                         |
| -------------------------- | ------------------------------------------------------------------------------- |
| `rockboxd.js`              | Emscripten-generated JS loader (MODULARIZE=1, exports `RockboxModule`)          |
| `rockboxd.wasm`            | WebAssembly binary — the compiled Rockbox firmware + codecs                     |
| `rockbox.js`               | High-level `RockboxPlayer` ES-module class (the integration layer)              |
| `rockbox-audio-worklet.js` | `AudioWorkletProcessor` that reads PCM frames from the WASM ring buffer         |
| `index.html`               | Example single-page player with full DSP settings UI                            |

`rockboxd.js` and `rockboxd.wasm` are build artefacts — do not edit them.
All integration code lives in `rockbox.js` and `rockbox-audio-worklet.js`.

---

## Architecture

```
HTTP URL
  │
  ▼
netstream (crates/netstream)     — HTTP range-request fd multiplexer
  │
  ▼
Rockbox codec (C, .wasm)         — decodes to S16LE stereo PCM frames
  │
  ▼
DSP pipeline (C, .wasm)          — EQ / replaygain / tone / crossfeed / surround / …
  │
  ▼
pcmbuf (apps/pcmbuf.c)           — 1 s pre-decode ring (WASM-specific MIN_BUFFER_SIZE)
  │
  ▼
wa_thread (pcm-webapi.c)         — C pthread; pushes frames into s_ring[]
  │  SharedArrayBuffer (Int16Array + atomic Int32 indices)
  ▼
AudioWorkletProcessor            — runs in the audio thread; pulls frames every 128-sample block
  │
  ▼
Web Audio API destination        — speaker output
```

### Key design choices

**SharedArrayBuffer ring buffer.** The WASM module is compiled with
`-pthread`, so `Module.HEAP8.buffer` is a `SharedArrayBuffer`. The C
ring buffer (`s_ring`, `s_write_idx`, `s_read_idx` in `pcm-webapi.c`)
lives inside WASM linear memory. The AudioWorklet creates `Int16Array` /
`Int32Array` views directly into that buffer — no extra copy, no
`postMessage` round-trip per audio block.

**Atomic indices.** `s_write_idx` is written by `wa_thread` using
`__atomic_store_n(..., __ATOMIC_SEQ_CST)`. `s_read_idx` is written by
the AudioWorklet using `Atomics.store`. Both sides use `Atomics.load` /
`__atomic_load_n` to read.

**wasm_cmd thread.** All mutating firmware calls (play, pause, seek, DSP
settings) are dispatched via a Rockbox kernel event queue to a dedicated
`wasm_cmd` Rockbox thread. This avoids calling Rockbox mutex code from
the Emscripten main JS thread, which would corrupt the Rockbox scheduler
(`__cores[0].running` is a global, not TLS).

**pcmbuf pre-decode window.** The WASM build uses `MIN_BUFFER_SIZE = 1 s`
(defined conditionally in `apps/pcmbuf.c` on `PLATFORM_WASM`). This means
DSP/EQ changes become audible within ~1 s rather than the default 3 s for
large-memory targets, while still leaving enough buffer for crossfade
(`data_level(2)` = 0.5 s minimum). `low_latency_mode` is never set so
crossfade works normally; the crossfade path adds fade-out delay + duration
on top of the base 1 s.

---

## Prerequisites

### Server headers

`SharedArrayBuffer` requires two HTTP response headers on **every** served
file (HTML, JS, WASM):

```
Cross-Origin-Opener-Policy:   same-origin
Cross-Origin-Embedder-Policy: require-corp
```

Without these headers the WASM memory is a plain `ArrayBuffer` and the
AudioWorklet cannot share it — you will see `[Rockbox] WASM memory is NOT
a SharedArrayBuffer` in the console and hear no audio.

### Browser requirements

- Chromium 68+ / Firefox 79+ / Safari 15.2+
- `SharedArrayBuffer`, `AudioWorklet`, `Atomics` all required
- The page must be loaded over HTTPS or `http://localhost`

---

## Quick start — dev server

```sh
# From the repo root (after building):
node scripts/wasm-dev-server.mjs
# → http://localhost:8090
```

The dev server (`scripts/wasm-dev-server.mjs`) sets the COOP/COEP headers
automatically and serves `web/` as the document root.

---

## Building

```sh
# From the repo root — full build:
bash scripts/build-wasm.sh

# Debug build (unoptimised, with DWARF info):
bash scripts/build-wasm.sh --debug
```

### Prerequisites

```sh
# Emscripten SDK (3.1.x)
source /path/to/emsdk/emsdk_env.sh

# Rust WASM target
rustup target add wasm32-unknown-emscripten
```

The build script (`scripts/build-wasm.sh`) runs four steps:

1. Configure `build-wasm/` for the `wasmapp` target (target 207, `memory=256 MB`).
2. Build firmware static libraries with `emcc` (`make lib`).
3. Compile `crates/wasm/` with Cargo (`wasm32-unknown-emscripten`).
4. Link everything with `emcc` into `web/rockboxd.js` + `web/rockboxd.wasm`.

Output files are written directly into `web/`. The wasm binary is ~1.6 MB
in release mode.

---

## Using `RockboxPlayer`

`rockbox.js` exports a single ES-module class. Import it from a `<script type="module">`.

### Initialisation

```js
import { RockboxPlayer } from './rockbox.js';

const player = new RockboxPlayer({ wasmUrl: 'rockboxd.js' });

// configDir: MEMFS path used as HOME (Rockbox writes config.cfg / nvram.bin here)
// musicDir:  base path for library scans (unused by WASM; kept for API symmetry)
await player.init('/config', '/music');
```

`init()` performs in order:

1. Dynamically loads `rockboxd.js` and instantiates `RockboxModule`.
2. Sets up the AudioContext + AudioWorklet + ring buffer.
3. Opens IndexedDB (`rockbox-persist`) and restores any previously saved
   `config.cfg` / `nvram.bin` into WASM MEMFS.
4. Boots the Rockbox firmware (`rb_daemon_start`).
5. Polls `rb_daemon_state()` until the firmware reports ready (state 2).

### Playback

```js
player.playUrl('https://example.com/track.mp3'); // clear queue + play
player.enqueueUrl('https://example.com/b.mp3');  // append to queue

player.play();        // resume or restart from current position
player.pause();
player.playPause();   // toggle
player.next();
player.prev();
player.stop();
player.seek(30000);   // ms from track start

player.clearQueue();
player.shuffleQueue();
player.jumpTo(2);     // 0-based queue index

player.adjustVolume(+1);   // one step louder (Rockbox volume steps)
player.adjustVolume(-3);   // three steps quieter
player.soundCurrent(0);    // read current value of SOUND_VOLUME (0)
```

### Status

All status reads are synchronous — they read from a cache refreshed by the
`wasm_cmd` thread every ~100 ms and after each command.

```js
player.status();       // → { status: 0|1|2 }  (0=stopped, 1=playing, 2=paused)
player.currentTrack(); // → { title, artist, album, path, duration_ms, elapsed_ms }
player.playlist();     // → { index, amount }
player.settings();     // → full settings object (see DSP reference below)
```

### AudioContext

The `AudioContext` is created at `_initAudio()` time but may be in a
`suspended` state until a user gesture. All `_call()` invocations
auto-resume it. You can also call `await player.resumeAudio()` directly
inside a click handler.

---

## DSP settings reference

All settings are applied immediately through the `wasm_cmd` Rockbox thread.
Changes take effect as the pre-decoded audio in pcmbuf (~1 s) drains.

### Equalizer

```js
player.setEqEnabled(true);          // true / false
player.setEqPrecut(60);             // 0–240 in tenths of dB (60 = 6.0 dB)
player.setEqBand(band, cutoff, q, gain);
//   band:   0–9 (10-band EQ)
//   cutoff: Hz  (e.g. 1000)
//   q:      Q × 10  (e.g. 70 = Q 7.0)
//   gain:   dB × 10  (e.g. 60 = +6.0 dB, -80 = -8.0 dB)
```

Default band centres: 60, 200, 500, 1000, 2000, 4000, 7000, 10000, 14000, 20000 Hz.

**Important:** all EQ numeric fields (`gain`, `q`, `precut`) are in
**tenths** of their natural unit. The JS layer in `rockbox.js` multiplies
slider values by 10 before calling the C export:

```js
// rockbox.js internally:
this._call('rb_set_eq_band', [b, cutoff | 0, q | 0, (gain | 0) * 10]);
```

### Crossfade

```js
player.setCrossfade(mode, {
  fadeInDelay:     0,   // seconds, 0–7
  fadeOutDelay:    0,   // seconds, 0–7
  fadeInDuration:  8,   // seconds, 0–15
  fadeOutDuration: 8,   // seconds, 0–15
  mixmode:         0,   // 0=crossfade, 1=mix
});
// mode: 0=off, 1=auto-skip, 2=manual-skip, 3=shuffle, 4=shuffle+manual, 5=always
```

Crossfade requires a REMAKE of the audio buffer (brief codec re-seek).
The pcmbuf is sized to `1 s + fade_out_delay + fade_out_duration` when enabled.

### Replaygain

```js
player.setReplaygain({
  type:   3,     // 0=track, 1=album, 2=shuffle-aware, 3=off
  preamp: 0,     // tenths of dB, -120..+120  (0 = 0.0 dB)
  noclip: false, // true = scale to prevent clipping
});
```

### Tone controls

```js
player.setBass(6);     // whole dB, -24..+24
player.setTreble(-3);  // whole dB, -24..+24
```

The SW tone control chain: `sound_set_bass(dB)` → `tone_set_bass(dB×10)` +
`tone_set_prescale(max_boost)` → `filter_bishelf_coefs()` → DSP proc
enabled. The prescaler reduces overall gain by `max(bass, treble)` dB to
prevent digital clipping.

### Balance

```js
player.setBalance(0);   // -100 (full left) … 0 (centre) … +100 (full right)
```

Balance is applied at the ring-buffer write level (`ring_push()` in
`pcm-webapi.c`) rather than through the Rockbox sound prescaler. Changes
take effect on the very next PCM chunk (~46 ms), with no gap or cut.

### Channel mode & stereo width

```js
player.setChannelMode(0);    // 0=stereo, 1=mono, 2=custom width, 3=mono-L,
                              // 4=mono-R, 5=karaoke, 6=swap L/R
player.setStereoWidth(100);  // 0–250, active only in mode 2 (custom)
```

### Crossfeed (headphone DSP)

```js
player.setCrossfeed(type, {
  directGain:    -115,  // tenths of dB, 0..−600     (default: -11.5 dB)
  crossGain:     -320,  // tenths of dB, -300..−1200 (default: -32.0 dB)
  hfAttenuation: -160,  // tenths of dB, -600..−2400 (default: -16.0 dB)
  hfCutoff:       700,  // Hz, 500–2000              (default: 700 Hz)
});
// type: 0=off, 1=Meier (fixed), 2=custom (uses opts above)
```

Meier mode ignores the gain/cutoff parameters and uses hard-coded
binaural optimised coefficients.

### Surround (Haas effect)

```js
player.setSurround({
  enabled: 10,    // 0=off, or delay ms: 5, 8, 10, 15, 30
  balance: 50,    // 0–99 %
  fx1:    1200,   // low-pass cutoff Hz, 600–8000
  fx2:     100,   // high-pass cutoff Hz, 40–400
  method2:   0,   // 0=method1, 1=method2 (side-channel only)
  mix:     100,   // wet/dry mix 0–100 %
});
```

### DSP extras

```js
player.setDithering(true);  // add dither noise before requantisation

player.setAfr(1);           // Adaptive Frequency Response: 0=off, 1–3=mode

player.setPbe(2, 30);       // Perceptual Bass Enhancement
                             //   pbe:    0=off, 1=low, 2=medium, 3=high
                             //   precut: tenths of dB, 0–240 (prevents bass clipping)

player.setTimestretch(80);  // 0=disabled; 35–250 = playback speed % (100=normal)
```

### Saving settings

DSP settings are auto-saved to `localStorage` (key `rockbox:settings`) on
every `set*()` call. Rockbox's own config file (`config.cfg`) is written by
the firmware on `rb_save_settings` and persisted to IndexedDB on
`persistState()`.

```js
player.saveSettings();   // flush firmware config.cfg to MEMFS
player.persistState();   // saveSettings() + copy config.cfg + nvram.bin to IndexedDB
```

### Restoring state

```js
// Called automatically by index.html after init():
player.restoreState();
// Reads localStorage 'rockbox:settings' and re-applies all DSP settings.
// Playlist / resume position are restored natively by the firmware from
// config.cfg / nvram.bin pre-loaded into MEMFS by _initPersistence().
```

---

## Persistence model

Two independent persistence mechanisms run in parallel:

| What                          | Where         | Managed by                                            |
| ----------------------------- | ------------- | ----------------------------------------------------- |
| DSP / EQ settings             | `localStorage` | `rockbox.js` — written on every `set*()` call        |
| Rockbox config (`config.cfg`) | IndexedDB     | `rockbox.js` — written on `persistState()`           |
| Playlist + resume position    | IndexedDB     | Rockbox firmware — written by the firmware itself    |

On page load, `_initPersistence()` copies `config.cfg` and `nvram.bin`
from IndexedDB back into WASM MEMFS **before** the daemon boots, so
`settings_load()` and the playlist resume code find the previously saved
state on startup.

`window.beforeunload` in `index.html` calls `player.persistState()`
automatically when the user closes the tab.

---

## Adding a new export

1. Implement `rb_<name>` in `crates/wasm/src/lib.rs` with `#[no_mangle] pub extern "C"`.
2. Add `"_rb_<name>"` to `EXPORTED_FUNCTIONS` in `scripts/build-wasm.sh`.
3. Add the JS-side wrapper in `rockbox.js`.
4. Rebuild with `bash scripts/build-wasm.sh` — a Rust-only recompile is not
   enough; the `emcc` link step must re-run to pick up the new export.

`rockbox.js` guards every call with:

```js
const fn = this._mod[`_${name}`];
if (typeof fn !== 'function') return; // export absent in this build
```

A missing entry in `EXPORTED_FUNCTIONS` fails silently at runtime — the
call returns immediately with no effect and no error.

---

## Ring buffer internals

The ring buffer is declared in `firmware/target/hosted/wasm/pcm-webapi.c`:

```c
#define RING_FRAMES 65536          // ~1.5 s at 44100 Hz

static int16_t  s_ring[RING_FRAMES * 2]; // interleaved S16LE stereo
static int32_t  s_write_idx = 0;         // written by wa_thread
static int32_t  s_read_idx  = 0;         // written by AudioWorklet
static int32_t  s_sample_rate_hz = 44100;
static int32_t  s_balance = 0;           // -100..+100, applied in ring_push()
```

The AudioWorklet obtains byte offsets of these arrays at startup via four
exported functions: `rb_pcm_ring_ptr()`, `rb_pcm_ring_frames()`,
`rb_pcm_write_idx_ptr()`, `rb_pcm_read_idx_ptr()`. It then builds typed-array
views directly into `Module.HEAP8.buffer` (the SharedArrayBuffer).

`wa_thread` calls `nanosleep(1 ms)` when the ring is full, giving the
AudioWorklet time to drain it. At 44100 Hz stereo the ring holds ~1.5 s of
audio; at normal playback rates the ring is never close to full.

`rb_pcm_flush()` atomically resets `s_write_idx` to the current `s_read_idx`,
making the ring appear empty. It is available for use (e.g., after a DSP
configuration change) but not called automatically — the natural pcmbuf drain
provides cut-free transitions.

---

## Troubleshooting

### No audio / AudioContext suspended

The Web Audio API requires a user gesture to start. Click any button before
expecting audio. `_call()` auto-calls `audioCtx.resume()` on every
interaction; you can also call `await player.resumeAudio()` inside a
button handler.

### "WASM memory is NOT a SharedArrayBuffer"

The page is missing `Cross-Origin-Opener-Policy: same-origin` and/or
`Cross-Origin-Embedder-Policy: require-corp` response headers. Use
`node scripts/wasm-dev-server.mjs` for local development or configure your
reverse proxy to add these headers.

### Audio plays but EQ / bass / treble has no effect

DSP changes are applied to the Rockbox codec output. They take effect as the
pre-decoded audio drains from pcmbuf (~1 s). If a DSP setting appears to have
no effect at all, ensure the firmware was rebuilt after any C changes
(`bash scripts/build-wasm.sh`).

### Crossfade not working

Crossfade requires at least `data_level(2)` = 0.5 s of pre-decoded audio in
pcmbuf when a track transition occurs. With `MIN_BUFFER_SIZE = 1 s` this
condition is met during normal playback. Crossfade also does not trigger on
`TRACK_CHANGE_END_OF_DATA` (natural track end with no next track buffered).

### Settings not restored after page reload

DSP settings are saved to `localStorage`; playlist and firmware settings to
IndexedDB. Check the browser console for `[Rockbox] Restored …` messages on
load. Private browsing mode may disable IndexedDB or localStorage. Call
`player.persistState()` explicitly to force a flush before closing the page,
or use the "Save state" button in `index.html`.

### WASM build fails on macOS (spc/asap errors)

Expected — `llvm-objcopy` on macOS crashes on these two codec archives.
Both codecs are unused in the WASM build. The `make -k` flag skips them and
the build continues normally.
