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

  // ── Settings (get) ────────────────────────────────────────────────────────

  /**
   * Returns the current firmware settings as a plain object.
   * @returns {{
   *   eq:         { enabled: boolean, precut: number, bands: Array<{cutoff,q,gain}> },
   *   crossfade?: { mode, fade_in_delay, fade_out_delay, fade_in_duration, fade_out_duration, mixmode },
   *   replaygain: { noclip: boolean, type: number, preamp: number }
   * }}
   */
  getSettings() { return this._jsonCall('rb_settings_json'); }

  /**
   * Returns the complete playlist state: all queued URLs plus resume position.
   * Use this to persist the queue across page reloads.
   * @returns {{ urls: string[], index: number, elapsed: number, amount: number }}
   */
  getPlaylistState() { return this._jsonCall('rb_playlist_state_json'); }

  // ── Settings (set) ────────────────────────────────────────────────────────

  /** @param {boolean} enabled */
  setEqEnabled(enabled) {
    this._call('rb_set_eq_enabled', [enabled ? 1 : 0]);
  }

  /**
   * @param {number} precut  0–240 (tenths of a dB pre-cut before the EQ).
   *                         Rockbox default is 0.
   */
  setEqPrecut(precut) { this._call('rb_set_eq_precut', [precut | 0]); }

  /**
   * @param {number} band    0–9
   * @param {number} cutoff  Centre/cutoff frequency in Hz
   * @param {number} q       Q-factor (Rockbox uses integer Q×10, e.g. 70 = Q 7.0)
   * @param {number} gain    Gain in dB (integer)
   */
  setEqBand(band, cutoff, q, gain) {
    this._call('rb_set_eq_band', [band | 0, cutoff | 0, q | 0, gain | 0]);
  }

  /**
   * @param {number} mode     0=off, 1=shuffle, 2=trackskip, 3=both, 4=always
   * @param {object} [opts]
   * @param {number} [opts.fadeInDelay=0]     0–15 s
   * @param {number} [opts.fadeOutDelay=0]    0–15 s
   * @param {number} [opts.fadeInDuration=8]  0–15 s
   * @param {number} [opts.fadeOutDuration=8] 0–15 s
   * @param {number} [opts.mixmode=0]         0=crossfade, 1=mix
   */
  setCrossfade(mode, opts = {}) {
    const {
      fadeInDelay    = 0,
      fadeOutDelay   = 0,
      fadeInDuration = 8,
      fadeOutDuration = 8,
      mixmode        = 0,
    } = opts;
    this._call('rb_set_crossfade', [
      mode | 0,
      fadeInDelay | 0, fadeOutDelay | 0,
      fadeInDuration | 0, fadeOutDuration | 0,
      mixmode | 0,
    ]);
  }

  /**
   * @param {object} [opts]
   * @param {boolean} [opts.noclip=false]  Scale down to prevent clipping
   * @param {number}  [opts.type=3]        0=track, 1=album, 2=shuffle, 3=off
   * @param {number}  [opts.preamp=0]      Extra gain in tenths of dB (−120 to 120)
   */
  setReplaygain({ noclip = false, type = 3, preamp = 0 } = {}) {
    this._call('rb_set_replaygain', [noclip ? 1 : 0, type | 0, preamp | 0]);
  }

  /** Flush current settings to the Rockbox config file. */
  saveSettings() { this._call('rb_save_settings'); }

  // ── Persistence helpers ───────────────────────────────────────────────────

  /**
   * Save the current settings + playlist state to localStorage.
   * Call this whenever you want a snapshot (e.g. on pause / page unload).
   */
  persistState() {
    const settings = this.getSettings();
    const playlist = this.getPlaylistState();
    try {
      if (settings) localStorage.setItem('rockbox:settings', JSON.stringify(settings));
      if (playlist) localStorage.setItem('rockbox:playlist', JSON.stringify(playlist));
    } catch (e) {
      console.warn('[Rockbox] persistState failed:', e);
    }
  }

  /**
   * Restore settings + playlist from localStorage.
   *
   * Settings are applied immediately.  The saved playlist URLs are re-enqueued
   * but playback is NOT started automatically — call `player.play()` (or
   * `player.jumpTo(n)` + `player.seek(ms)` + `player.play()`) after this
   * returns so the user controls when audio resumes.
   *
   * @returns {{ resumeIndex: number, resumeElapsed: number } | null}
   *   The saved queue position, or null if there was nothing to restore.
   */
  restoreState() {
    let resumeInfo = null;
    try {
      const settingsRaw = localStorage.getItem('rockbox:settings');
      if (settingsRaw) {
        const s = JSON.parse(settingsRaw);
        if (s.eq) {
          this.setEqEnabled(s.eq.enabled ?? false);
          if (s.eq.precut != null) this.setEqPrecut(s.eq.precut);
          (s.eq.bands ?? []).forEach((b, i) => {
            if (b != null) this.setEqBand(i, b.cutoff, b.q, b.gain);
          });
        }
        if (s.crossfade) {
          this.setCrossfade(s.crossfade.mode ?? 0, {
            fadeInDelay:    s.crossfade.fade_in_delay,
            fadeOutDelay:   s.crossfade.fade_out_delay,
            fadeInDuration: s.crossfade.fade_in_duration,
            fadeOutDuration: s.crossfade.fade_out_duration,
            mixmode:        s.crossfade.mixmode,
          });
        }
        if (s.replaygain) {
          this.setReplaygain(s.replaygain);
        }
      }

      const playlistRaw = localStorage.getItem('rockbox:playlist');
      if (playlistRaw) {
        const p = JSON.parse(playlistRaw);
        const urls = p.urls ?? [];
        if (urls.length > 0) {
          urls.forEach(url => this.enqueueUrl(url));
          resumeInfo = {
            resumeIndex:   p.index   ?? 0,
            resumeElapsed: p.elapsed ?? 0,
          };
        }
      }
    } catch (e) {
      console.warn('[Rockbox] restoreState failed:', e);
    }
    return resumeInfo;
  }

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
