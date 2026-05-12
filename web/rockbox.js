/**
 * RockboxPlayer — JS integration layer for the Rockbox WASM module.
 *
 * Usage:
 *   const player = new RockboxPlayer({ wasmUrl: 'rockboxd.js' });
 *   await player.init('/config', '/music');
 *   player.playUrl('https://example.com/track.mp3');
 *
 * The page must be served with:
 *   Cross-Origin-Opener-Policy: same-origin
 *   Cross-Origin-Embedder-Policy: require-corp
 * (required for SharedArrayBuffer + Emscripten pthreads)
 */

const WORKLET_URL   = new URL('./rockbox-audio-worklet.js', import.meta.url).href;
const POLL_INTERVAL = 100; // ms between daemon-ready polls

// Emscripten throws 'unwind' (or an ExitStatus) when the WASM runtime yields.
function _isEmscriptenUnwind(e) {
  if (e === 'unwind') return true;
  if (typeof e === 'object' && e !== null) {
    if (e.name === 'unwind' || e.message === 'unwind') return true;
    if (typeof e.status === 'number') return true; // ExitStatus
  }
  return false;
}

export class RockboxPlayer {
  /**
   * @param {object} opts
   * @param {string} opts.wasmUrl    - URL to the emcc-generated rockboxd.js loader
   * @param {string} [opts.workletUrl] - Override AudioWorklet URL
   */
  constructor(opts = {}) {
    this._wasmUrl    = opts.wasmUrl    ?? 'rockboxd.js';
    this._workletUrl = opts.workletUrl ?? WORKLET_URL;

    this._mod      = null; // Emscripten Module object
    this._audioCtx = null;
    this._worklet  = null;
    this._sampleRate = 44100;
  }

  // ── Init ────────────────────────────────────────────────────────────────

  async init(configDir = '/', musicDir = '/music') {
    await this._loadWasm();
    await this._initAudio();
    this._bootDaemon(configDir, musicDir);
    await this._waitForDaemon();
  }

  // ── Playback ─────────────────────────────────────────────────────────────

  playUrl(url)    { this._call('rb_play_url',    [this._str(url)]); }
  enqueueUrl(url) { this._call('rb_enqueue_url', [this._str(url)]); }

  play()      { this._call('rb_play');      }
  pause()     { this._call('rb_pause');     }
  playPause() { this._call('rb_play_pause'); }
  next()      { this._call('rb_next');      }
  prev()      { this._call('rb_prev');      }
  stop()      { this._call('rb_stop');      }

  /** @param {number} positionMs */
  seek(positionMs) { this._call('rb_seek', [positionMs]); }

  clearQueue()   { this._call('rb_clear_queue');   }
  shuffleQueue() { this._call('rb_shuffle_queue'); }

  /** @param {number} pos 0-based queue index */
  jumpTo(pos) { this._call('rb_jump_to_queue_position', [pos]); }

  // ── Sound ────────────────────────────────────────────────────────────────

  /** @param {number} steps positive=louder, negative=quieter */
  adjustVolume(steps) { this._call('rb_adjust_volume', [steps]); }

  /** @param {number} setting SOUND_* constant (0 = volume) */
  soundCurrent(setting = 0) { return this._mod._rb_sound_current(setting); }

  // ── Status ───────────────────────────────────────────────────────────────

  /** @returns {{ status: 0|1|2 }} */
  status()       { return this._jsonCall('rb_status_json');        }

  /** @returns {{ title, artist, album, path, duration_ms, elapsed_ms }} */
  currentTrack() { return this._jsonCall('rb_current_track_json'); }

  /** @returns {{ index: number, amount: number }} */
  playlist()     { return this._jsonCall('rb_playlist_json');      }

  // ── Internal ─────────────────────────────────────────────────────────────

  async _loadWasm() {
    const script = document.createElement('script');
    script.src   = this._wasmUrl;
    document.head.appendChild(script);
    await new Promise((res) => { script.onload = res; });

    this._mod = await RockboxModule({});
  }

  async _initAudio() {
    const mod = this._mod;

    // WASM linear memory is a SharedArrayBuffer (compiled with -pthread).
    // The C ring buffer and its atomic indices live inside it.
    const wasmMemory = mod.HEAP8.buffer;
    const ringPtr    = mod._rb_pcm_ring_ptr();
    const ringFrames = mod._rb_pcm_ring_frames();
    const wiPtr      = mod._rb_pcm_write_idx_ptr();
    const riPtr      = mod._rb_pcm_read_idx_ptr();

    const isSAB = wasmMemory instanceof SharedArrayBuffer;
    console.log('[Rockbox] WASM memory isSAB:', isSAB,
                'ringPtr:', ringPtr, 'ringFrames:', ringFrames,
                'wiPtr:', wiPtr, 'riPtr:', riPtr);
    if (!isSAB) console.error('[Rockbox] WASM memory is NOT a SharedArrayBuffer — ring buffer sharing will not work');

    this._wiPtr      = wiPtr;   // kept for the debug poller in index.html
    this._wasmMemory = wasmMemory;

    this._audioCtx = new AudioContext({ sampleRate: this._sampleRate });
    console.log('[Rockbox] AudioContext state after creation:', this._audioCtx.state);
    await this._audioCtx.audioWorklet.addModule(this._workletUrl);

    this._worklet = new AudioWorkletNode(
      this._audioCtx,
      'rockbox-processor',
      {
        processorOptions: { wasmMemory, ringPtr, ringFrames, wiPtr, riPtr },
        outputChannelCount: [2],
      }
    );
    this._worklet.port.onmessage = (e) => console.log('[Worklet]', e.data);
    this._worklet.connect(this._audioCtx.destination);
  }

  /** Resume the AudioContext. Must be called from a user-gesture handler. */
  async resumeAudio() {
    if (this._audioCtx && this._audioCtx.state !== 'running') {
      await this._audioCtx.resume();
      console.log('[Rockbox] AudioContext resumed, state:', this._audioCtx.state);
    }
  }

  _bootDaemon(configDir, musicDir) {
    const mod  = this._mod;
    const cDir = this._str(configDir);
    const mDir = this._str(musicDir);
    mod._rb_daemon_start(cDir, mDir);
    mod._free(cDir);
    mod._free(mDir);
  }

  _waitForDaemon() {
    return new Promise((resolve) => {
      const check = () => {
        if (this._mod._rb_daemon_state() === 2) return resolve();
        setTimeout(check, POLL_INTERVAL);
      };
      check();
    });
  }

  /** Allocate a C string in WASM heap; caller must _free it. */
  _str(s) {
    const len = this._mod.lengthBytesUTF8(s) + 1;
    const ptr = this._mod._malloc(len);
    this._mod.stringToUTF8(s, ptr, len);
    return ptr;
  }

  /** Call a void rb_* function with optional integer arguments. */
  _call(name, args = []) {
    if (!this._mod) throw new Error('RockboxPlayer not initialised');
    // Resume AudioContext on any user-gesture call (play/pause/etc.)
    if (this._audioCtx && this._audioCtx.state !== 'running') {
      this._audioCtx.resume().then(() =>
        console.log('[Rockbox] AudioContext auto-resumed, state:', this._audioCtx.state)
      );
    }
    try {
      this._mod[`_${name}`](...args);
    } catch (e) {
      if (!_isEmscriptenUnwind(e)) throw e;
    }
  }

  /** Call a rb_*_json function and return the parsed object. */
  _jsonCall(name) {
    if (!this._mod) throw new Error('RockboxPlayer not initialised');
    let ptr = 0;
    try {
      ptr = this._mod[`_${name}`]();
    } catch (e) {
      if (!_isEmscriptenUnwind(e)) throw e;
      return null;
    }
    if (!ptr) return null;
    try {
      return JSON.parse(this._mod.UTF8ToString(ptr));
    } finally {
      this._mod._rb_free_string(ptr);
    }
  }
}
