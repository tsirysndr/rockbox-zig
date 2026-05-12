# Rockbox WASM — Browser Build

Rockbox can be compiled to WebAssembly so it runs entirely in the browser.
The result is `rockboxd.js` + `rockboxd.wasm` — a self-contained audio engine that plays HTTP(S) audio URLs using the full Rockbox codec stack, with no server process and no local disk access required.

---

## Architecture overview

```
Browser (main thread / page)
├── rockbox.js            RockboxPlayer helper class
│   ├── loads rockboxd.js (Emscripten WASM loader)
│   └── wires Module.onPcmData → SharedArrayBuffer ring writer
├── AudioContext
│   └── rockbox-audio-worklet.js  RockboxProcessor (AudioWorkletProcessor)
│       └── reads ring buffer → float32 output → speakers
└── Web Worker (Emscripten pthread pool)
    └── rockboxd.wasm     Rockbox firmware + codecs
        ├── pcm-webapi.c  PCM writer thread → EM_JS → Module.onPcmData
        ├── netstream/    HTTP Range-request layer (handles http:// URLs)
        └── rb_* exports  JS calls these directly (no gRPC/HTTP server)
```

### Threading model

Emscripten compiles Rockbox's native POSIX threads to Web Workers via
`USE_PTHREADS=1`. The firmware's audio decode loop, PCM writer, and playlist
threads all run as separate Web Workers. A pool of 8 workers is pre-allocated
(`PTHREAD_POOL_SIZE=8`).

The page **must** be served with:

```
Cross-Origin-Opener-Policy:   same-origin
Cross-Origin-Embedder-Policy: require-corp
```

These headers enable `SharedArrayBuffer` which is required for both pthreads
and the ring-buffer audio handoff.

### Audio pipeline

```
WASM decoder thread
  → pcm-webapi.c writer pthread
    → EM_JS pcm_web_push(ptr, bytes, sampleRate)
      → Module.onPcmData(ptr, bytes, sampleRate)   [called by RockboxPlayer]
        → writes S16LE frames into SharedArrayBuffer ring
          → AudioWorkletProcessor (rockbox-audio-worklet.js)
            → converts S16LE → Float32
              → Web Audio destination (speakers)
```

Ring buffer: `RING_FRAMES = 65536` stereo S16LE frames (~1.5 s at 44100 Hz,
power-of-two for cheap modulo). Write/read cursors are `Int32Array` views into
WASM linear memory (a `SharedArrayBuffer`); `Atomics` guards all accesses.

---

## Prerequisites

| Tool                         | Version          | Install                                                            |
| ---------------------------- | ---------------- | ------------------------------------------------------------------ |
| Emscripten SDK               | 3.1.x (latest)   | `git clone https://github.com/emscripten-core/emsdk && ./emsdk install latest && ./emsdk activate latest` |
| Rust + wasm32 target         | stable           | `rustup target add wasm32-unknown-emscripten`                      |
| llvm-objcopy (macOS)         | LLVM 21          | `brew install llvm@21`                                             |
| Node.js (dev server)         | 18+              | `brew install node` / system package manager                       |

Activate the Emscripten environment before every build session:

```sh
source /path/to/emsdk/emsdk_env.sh
```

---

## Build

```sh
# One-shot full build
bash scripts/build-wasm.sh

# Debug build (unoptimised, DWARF info in .wasm)
bash scripts/build-wasm.sh --debug
```

The script runs four steps:

| Step | What it does                                                                |
| ---- | --------------------------------------------------------------------------- |
| 1    | Configure `build-wasm/` for target 207 (`wasmapp`)                          |
| 2    | Build firmware static libs with emcc (`make lib` in `build-wasm/`)          |
| 2.5  | Extract per-codec `.o` files from per-codec `.a` archives                   |
| 2.6  | Rename `ogg_*` symbols in `libopus.a` to avoid libtremor ABI conflict       |
| 3    | Compile `crates/wasm/` with `cargo` for `wasm32-unknown-emscripten`         |
| 4    | `emcc` link step → `web/rockboxd.js` + `web/rockboxd.wasm`                  |

Outputs:

```
web/rockboxd.js          Emscripten JS loader (MODULARIZE=1, EXPORT_NAME=RockboxModule)
web/rockboxd.wasm        WebAssembly binary
```

---

## Running locally

```sh
node scripts/wasm-dev-server.mjs     # serves web/ on http://localhost:8080
```

The dev server injects `Cross-Origin-Opener-Policy: same-origin` and
`Cross-Origin-Embedder-Policy: require-corp` on every response, which is the
only special requirement compared to a normal static file server.

---

## JS integration

### Minimal usage

```html
<!-- web/index.html -->
<script type="module">
  import { RockboxPlayer } from './rockbox.js';

  const player = new RockboxPlayer({ wasmUrl: 'rockboxd.js' });
  await player.init('/config', '/music');

  player.playUrl('https://example.com/track.mp3');
</script>
```

### `RockboxPlayer` API (`web/rockbox.js`)

```js
// Construction
const player = new RockboxPlayer({
  wasmUrl:    'rockboxd.js',    // path to Emscripten loader
  workletUrl: './rockbox-audio-worklet.js', // optional override
});

// Initialisation (async — resolves when firmware is running)
await player.init(configDir, musicDir);

// Playback
player.playUrl(url);        // clear queue, play URL immediately
player.enqueueUrl(url);     // append URL to current queue
player.play();
player.pause();
player.playPause();         // toggle
player.next();
player.prev();
player.stop();
player.seek(positionMs);    // seek to ms from start of current track

// Queue
player.clearQueue();
player.shuffleQueue();
player.jumpTo(index);       // 0-based queue index

// Volume
player.adjustVolume(steps); // positive = louder, negative = quieter
player.soundCurrent(0);     // read SOUND_VOLUME (returns integer)

// Status — synchronous reads (returns parsed object)
player.status();            // → { status: 0|1|2 }  0=stopped 1=playing 2=paused
player.currentTrack();      // → { title, artist, album, path, duration_ms, elapsed_ms }
player.playlist();          // → { index, amount }

// Settings — synchronous reads
player.getSettings();       // → { eq, crossfade?, replaygain }
player.getPlaylistState();  // → { urls, index, elapsed, amount }

// Settings — writes (routed through the wasm_cmd thread; apply immediately + persist)
player.setEqEnabled(true);
player.setEqPrecut(0);                        // 0–240 (tenths of dB)
player.setEqBand(band, cutoff, q, gain);      // band 0–9
player.setCrossfade(mode, { fadeInDelay, fadeOutDelay, fadeInDuration,
                             fadeOutDuration, mixmode });
player.setReplaygain({ noclip, type, preamp });
player.saveSettings();                        // explicit flush (normally auto-called)

// Persistence — localStorage helpers
player.persistState();      // save settings + full playlist to localStorage
player.restoreState();      // → { resumeIndex, resumeElapsed } | null
                            //   re-enqueues saved URLs; call play() afterwards
```

---

## C-ABI exports (raw WASM surface)

These are exported directly from the `.wasm` module. `RockboxPlayer` wraps
them; use these only if you're integrating without the JS helper.

| Export                            | Signature                           | Notes                                       |
| --------------------------------- | ----------------------------------- | ------------------------------------------- |
| `rb_daemon_start`                 | `(configDir, musicDir) → i32`       | Boot firmware; returns 0 ok, -1 if running  |
| `rb_daemon_state`                 | `() → i32`                          | 0=stopped 1=starting 2=running              |
| `rb_free_string`                  | `(ptr) → void`                      | Free any `*_json` return value              |
| `rb_play_url`                     | `(url) → i32`                       | Clear queue + play URL                      |
| `rb_enqueue_url`                  | `(url) → i32`                       | Append URL to queue                         |
| `rb_play`                         | `() → i32`                          |                                             |
| `rb_pause`                        | `() → i32`                          |                                             |
| `rb_play_pause`                   | `() → i32`                          | Toggle play/pause                           |
| `rb_next`                         | `() → i32`                          |                                             |
| `rb_prev`                         | `() → i32`                          |                                             |
| `rb_seek`                         | `(positionMs) → i32`                |                                             |
| `rb_stop`                         | `() → i32`                          |                                             |
| `rb_clear_queue`                  | `() → i32`                          |                                             |
| `rb_shuffle_queue`                | `() → i32`                          |                                             |
| `rb_jump_to_queue_position`       | `(index) → i32`                     | 0-based                                     |
| `rb_adjust_volume`                | `(steps) → i32`                     | ±1 per hardware step                        |
| `rb_sound_current`                | `(setting) → i32`                   | 0 = SOUND_VOLUME                            |
| `rb_status_json`                  | `() → *char`                        | `{"status":0\|1\|2}` — free with `rb_free_string`                   |
| `rb_current_track_json`           | `() → *char`                        | Track metadata JSON — free with `rb_free_string`                     |
| `rb_playlist_json`                | `() → *char`                        | `{"index":n,"amount":n}` — free with `rb_free_string`                |
| `rb_settings_json`                | `() → *char`                        | EQ + crossfade + replaygain JSON — free with `rb_free_string`        |
| `rb_playlist_state_json`          | `() → *char`                        | `{"urls":[…],"index":n,"elapsed":n,"amount":n}` — free with `rb_free_string` |
| `rb_set_eq_enabled`               | `(enabled) → i32`                   | 0=off, 1=on; persists                                                |
| `rb_set_eq_precut`                | `(precut) → i32`                    | 0–240 (tenths of dB); persists                                       |
| `rb_set_eq_band`                  | `(band, cutoff, q, gain) → i32`     | band 0–9; persists                                                   |
| `rb_set_crossfade`                | `(mode,fi_delay,fo_delay,fi_dur,fo_dur,mixmode) → i32` | persists         |
| `rb_set_replaygain`               | `(noclip, type, preamp) → i32`      | persists                                                             |
| `rb_save_settings`                | `() → i32`                          | Explicit flush (called automatically by every set_* command)         |

String arguments (`url`, `configDir`, `musicDir`) must be WASM heap pointers
to NUL-terminated UTF-8. Use `Module.stringToUTF8` / `Module._malloc` /
`Module._free` to allocate them, or use the `RockboxPlayer._str()` helper.

### `Module.onPcmData` callback

Set this **before** calling `rb_daemon_start`:

```js
const mod = await RockboxModule({ ... });
mod.onPcmData = (ptr, bytes, sampleRate) => {
  // ptr        — byte offset into Module.HEAP8.buffer (Int16Array view)
  // bytes      — byte count (stereo S16LE: frames × 4)
  // sampleRate — current output Hz (usually 44100)
};
```

The callback is invoked from a Web Worker (the PCM writer pthread). It must
not block; copy the data out immediately if needed.

---

## Media support

All media is accessed as HTTP(S) URLs. The firmware's `netstream/` layer
handles `Range` requests and maps them to virtual file descriptors (`≤ -1000`
encoding convention). There is no local filesystem access in the WASM build.

Supported codecs (same as desktop): MP3, AAC, AAC-BSF, FLAC, Ogg Vorbis, Opus,
WMA, WMA Pro, ALAC, APE, WAV, AIFF, WavPack, Speex, COOK, ATRAC3, TTA, SHN,
ADX, MOD, MPC, RA-AAC, MP2, VOX, WAV64, SMAF.

---

## File inventory

### New files

| File                                                    | Purpose                                           |
| ------------------------------------------------------- | ------------------------------------------------- |
| `firmware/export/config/wasmapp.h`                      | Per-target config header (PLATFORM_WASM, no SDL, BINFMT_STATIC) |
| `firmware/target/hosted/wasm/system-wasm.c`             | System init — boot marker, `EM_ASM` log redirect  |
| `firmware/target/hosted/wasm/pcm-webapi.c`              | PCM sink — writer pthread → `EM_JS` → `Module.onPcmData` |
| `firmware/target/hosted/wasm/wasm-bridge.c`             | Command dispatcher (`wasm_cmd` thread + `queue_post` API); track/playlist/settings/playlist-state JSON cache; HTTP netstream layer |
| `firmware/target/hosted/wasm/debug-wasm.c`              | `debugf()` → `emscripten_log(EM_LOG_CONSOLE)`     |
| `firmware/target/hosted/wasm/kernel-wasm.c`             | Kernel tick + timers (nanosleep-based, no timer_create) |
| `firmware/target/hosted/wasm/lc-wasm.c`                 | Static codec table loader (mirrors `lc-android.c`) |
| `firmware/target/hosted/wasm/rb_zig_compat.c`           | C wrappers for 18 `rb_*` symbols from `crates/sys` |
| `firmware/target/hosted/wasm/lcd-noop.c`                | LCD stub (no display in headless/WASM)            |
| `firmware/target/hosted/wasm/button-noop.c`             | Button stub                                       |
| `firmware/target/hosted/wasm/cpuinfo-noop.c`            | CPU info stub                                     |
| `firmware/target/hosted/wasm/audiohw-noop.c`            | Audio HW stub                                     |
| `firmware/target/hosted/wasm/SOURCES`                   | Lists all WASM-target `.c` files for Make         |
| `firmware/target/hosted/wasm/wasm.make`                 | Make glue — included by `tools/root.make` for `wasm_app` |
| `crates/wasm/Cargo.toml`                                | `staticlib` crate; no tokio, no gRPC              |
| `crates/wasm/src/lib.rs`                                | All `rb_*` Rust exports; routes all firmware calls through `wasm_cmd` thread; settings + playlist-state exports |
| `web/rockbox.js`                                        | `RockboxPlayer` JS class — playback, settings get/set, `persistState` / `restoreState` localStorage helpers |
| `web/rockbox-audio-worklet.js`                          | `RockboxProcessor` — ring buffer consumer         |
| `scripts/build-wasm.sh`                                 | Full WASM build pipeline                          |
| `scripts/wasm-dev-server.mjs`                           | Dev HTTP server with COOP/COEP headers            |

### Modified files

| File                               | Change                                                                  |
| ---------------------------------- | ----------------------------------------------------------------------- |
| `firmware/export/config.h`         | Added `#define PLATFORM_WASM (1<<4)`                                    |
| `firmware/export/pcm_sink.h`       | Added `PCM_SINK_WEBAPI`; guarded `PCM_SINK_CPAL` on `!PLATFORM_WASM`   |
| `firmware/pcm.c`                   | Wired `webapi_pcm_sink` for `PLATFORM_WASM`; guarded cpal slot          |
| `firmware/SOURCES`                 | Gated network sinks on `#ifndef PLATFORM_WASM`                          |
| `tools/configure`                  | Added `wasmappcc()` function + target `207|wasmapp`                     |
| `tools/root.make`                  | Included `wasm.make` for `wasm_app` app_type                            |

---

## Design decisions

### No gRPC / HTTP server

The server stack (actix-web, tonic, sqlx, typesense) is entirely absent. JS
calls the `rb_*` WASM exports directly. This keeps the `.wasm` binary small
and avoids spinning up network listeners inside the browser sandbox.

### No local library / database

Browsers have no filesystem access to a music library. All media is HTTP URLs;
the firmware's `netstream/` layer handles `Range`-based streaming. SQLite and
the tag cache are compiled out (no `ROCKBOX_SERVER` / `CONFIG_SERVER`).

### Static codecs (BINFMT_STATIC)

`dlopen` is not available in the browser sandbox. Codecs are statically linked
into the `.wasm` binary using the same `BINFMT_STATIC` mechanism as the Android
cdylib build. Per-codec `__header_*` symbols are renamed with `llvm-objcopy
--redefine-sym` to keep them distinct at link time.

### Command-thread serialisation

Rockbox's cooperative scheduler tracks the "current thread" via a global
`__cores[0].running` pointer (not TLS).  Any firmware function that acquires a
blocking mutex (`id3_mutex`, `playlist->mutex`) internally calls
`switch_thread()`, which reads that global.  The Emscripten main JS thread is
not a registered Rockbox kernel thread, so calling such functions from it
corrupts the scheduler and produces "memory access out of bounds" traps.

Fix: a dedicated Rockbox kernel thread (`wasm_cmd`) receives all
mutex-acquiring commands via `queue_post()` — a spinlock + condvar operation
that is safe to call from any OS thread — and executes them from a valid
Rockbox context.  Covered operations include audio transport, playlist mutation,
DSP setting changes (`dsp_eq_enable`, `dsp_set_eq_coefs`,
`dsp_replaygain_set_settings`), and `settings_save()`.

The main JS thread only calls `queue_post` and reads from a
`pthread_mutex_trylock`-protected cache.  `pthread_mutex_trylock` never blocks;
reads return immediately (possibly stale by ≤100 ms).

### `sink_set_freq` receives an INDEX, not Hz

`pcm-webapi.c::sink_set_freq` receives a frequency **index** into
`hw_freq_sampr[]`, not a Hz value. The rate is looked up via
`hw_freq_sampr[freq_index]` before use. Passing the raw index as Hz would open
the AudioContext at 4 Hz, causing audio to play ~9 % too fast (chipmunk
effect).

---

## Known limitations

- **Autoplay policy**: Browsers suspend the `AudioContext` until a user gesture.
  `RockboxPlayer._initAudio()` registers a one-shot `click` listener to resume
  it; users must click once before audio plays.
- **No seek-back in HTTP streams**: The firmware uses HTTP `Range` requests for
  seeking. Servers that do not honour `Range` (e.g., some live streams) cannot
  be seeked.
- **COOP/COEP required**: The hosting server must send these two headers. CDNs
  and static hosts (Netlify, Vercel, GitHub Pages) support them via config;
  check your deployment guide.
- **Binary size**: Statically-linked codecs make the `.wasm` file large
  (~40–60 MB uncompressed). Serve it with gzip/brotli compression; browsers
  decompress WASM before execution.
