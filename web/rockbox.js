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

/** Default band centre frequencies (Rockbox 10-band EQ). */
const EQ_BAND_CUTOFFS = [60, 200, 500, 1000, 2000, 4000, 7000, 10000, 14000, 20000];

/** Factory for the JS-side settings mirror. */
function _defaultSettings() {
  return {
    eq: {
      enabled: false,
      precut: 0,
      bands: EQ_BAND_CUTOFFS.map(cutoff => ({ cutoff, q: 70, gain: 0 })),
    },
    crossfade: {
      mode: 0,
      fade_in_delay: 0,
      fade_out_delay: 0,
      fade_in_duration: 8,
      fade_out_duration: 8,
      mixmode: 0,
    },
    replaygain: { noclip: false, type: 3, preamp: 0 },
  };
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

    this._mod        = null; // Emscripten Module object
    this._audioCtx   = null;
    this._worklet    = null;
    this._sampleRate = 44100;

    // JS-side mirror of firmware settings.
    // Source of truth for localStorage persistence — updated synchronously on
    // every set*() call so saves never depend on an async WASM round-trip.
    this._settings = _defaultSettings();
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

  play()      { this._call('rb_play');       }
  pause()     { this._call('rb_pause');      }
  playPause() { this._call('rb_play_pause'); }
  next()      { this._call('rb_next');       }
  prev()      { this._call('rb_prev');       }
  stop()      { this._call('rb_stop');       }

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
   * Returns the current settings object.
   * When the new WASM build is active, values come from the firmware and the
   * JS mirror is synced.  Otherwise the JS mirror (updated by every set* call)
   * is returned — it always reflects what was last applied.
   *
   * @returns {{ eq, crossfade, replaygain }}
   */
  getSettings() {
    const fw = this._jsonCall('rb_settings_json');
    if (fw) {
      // Merge firmware values into our JS mirror so the cache stays accurate.
      // Keep crossfade from JS mirror when firmware doesn't include it
      // (HAVE_CROSSFADE not defined for this build).
      this._settings = { ...this._settings, ...fw };
      if (!fw.crossfade) this._settings.crossfade = this._settings.crossfade ?? _defaultSettings().crossfade;
    }
    return this._settings;
  }

  /**
   * Returns the complete playlist state: all queued URLs plus resume position.
   * @returns {{ urls: string[], index: number, elapsed: number, amount: number }}
   */
  getPlaylistState() { return this._jsonCall('rb_playlist_state_json'); }

  // ── Settings (set) ────────────────────────────────────────────────────────

  /** @param {boolean} enabled */
  setEqEnabled(enabled) {
    this._settings.eq.enabled = !!enabled;
    this._saveSettings();
    this._call('rb_set_eq_enabled', [enabled ? 1 : 0]);
  }

  /**
   * @param {number} precut  0–240 (tenths of a dB pre-cut before the EQ).
   *                         Rockbox default is 0.
   */
  setEqPrecut(precut) {
    this._settings.eq.precut = precut | 0;
    this._saveSettings();
    this._call('rb_set_eq_precut', [precut | 0]);
  }

  /**
   * @param {number} band    0–9
   * @param {number} cutoff  Centre/cutoff frequency in Hz
   * @param {number} q       Q-factor (Rockbox stores Q×10, e.g. 70 = Q 7.0)
   * @param {number} gain    Gain in dB (integer)
   */
  setEqBand(band, cutoff, q, gain) {
    const b = band | 0;
    if (b >= 0 && b < 10) {
      this._settings.eq.bands[b] = { cutoff: cutoff | 0, q: q | 0, gain: gain | 0 };
      this._saveSettings();
    }
    this._call('rb_set_eq_band', [b, cutoff | 0, q | 0, gain | 0]);
  }

  /**
   * @param {number} mode     0=off, 1=auto-skip, 2=manual-skip, 3=shuffle,
   *                          4=shuffle+manual-skip, 5=always
   *                          (matches CROSSFADE_ENABLE_* enum in apps/settings.h)
   * @param {object} [opts]
   * @param {number} [opts.fadeInDelay=0]      0–15 s
   * @param {number} [opts.fadeOutDelay=0]     0–15 s
   * @param {number} [opts.fadeInDuration=8]   0–15 s
   * @param {number} [opts.fadeOutDuration=8]  0–15 s
   * @param {number} [opts.mixmode=0]          0=crossfade, 1=mix
   */
  setCrossfade(mode, opts = {}) {
    const {
      fadeInDelay     = 0,
      fadeOutDelay    = 0,
      fadeInDuration  = 8,
      fadeOutDuration = 8,
      mixmode         = 0,
    } = opts;
    this._settings.crossfade = {
      mode:             mode | 0,
      fade_in_delay:    fadeInDelay | 0,
      fade_out_delay:   fadeOutDelay | 0,
      fade_in_duration: fadeInDuration | 0,
      fade_out_duration: fadeOutDuration | 0,
      mixmode:          mixmode | 0,
    };
    this._saveSettings();
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
    this._settings.replaygain = { noclip: !!noclip, type: type | 0, preamp: preamp | 0 };
    this._saveSettings();
    this._call('rb_set_replaygain', [noclip ? 1 : 0, type | 0, preamp | 0]);
  }

  /** Flush current settings to the Rockbox config file (no-op before rebuild). */
  saveSettings() { this._call('rb_save_settings'); }

  // ── Persistence helpers ───────────────────────────────────────────────────

  /**
   * Save the current playlist state to localStorage.
   * Settings are already saved incrementally by every set*() call;
   * this only needs to capture the playlist.
   */
  persistState() {
    // Settings are always up-to-date via _saveSettings() — no need to re-read.
    const playlist = this.getPlaylistState();
    try {
      if (playlist && playlist.amount > 0) {
        localStorage.setItem('rockbox:playlist', JSON.stringify(playlist));
      }
    } catch (e) {
      console.warn('[Rockbox] persistState failed:', e);
    }
  }

  /**
   * Restore settings + playlist from localStorage.
   *
   * Settings are applied immediately to the firmware via set*() calls.
   * Playlist URLs are re-enqueued but playback is NOT started automatically.
   *
   * @returns {{ resumeIndex: number, resumeElapsed: number } | null}
   */
  restoreState() {
    let resumeInfo = null;
    try {
      const settingsRaw = localStorage.getItem('rockbox:settings');
      if (settingsRaw) {
        const s = JSON.parse(settingsRaw);
        // Update JS mirror first so getSettings() is consistent immediately.
        this._settings = { ..._defaultSettings(), ...s };
        // Apply to firmware (set* methods will re-save to localStorage, which
        // is a no-op write of the same data — acceptable).
        if (s.eq) {
          this.setEqEnabled(s.eq.enabled ?? false);
          if (s.eq.precut != null) this.setEqPrecut(s.eq.precut);
          (s.eq.bands ?? []).forEach((b, i) => {
            if (b) this.setEqBand(i, b.cutoff ?? EQ_BAND_CUTOFFS[i], b.q ?? 70, b.gain ?? 0);
          });
        }
        if (s.crossfade) {
          this.setCrossfade(s.crossfade.mode ?? 0, {
            fadeInDelay:     s.crossfade.fade_in_delay,
            fadeOutDelay:    s.crossfade.fade_out_delay,
            fadeInDuration:  s.crossfade.fade_in_duration,
            fadeOutDuration: s.crossfade.fade_out_duration,
            mixmode:         s.crossfade.mixmode,
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
          resumeInfo = { resumeIndex: p.index ?? 0, resumeElapsed: p.elapsed ?? 0 };
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

    this._wiPtr      = wiPtr;
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

  /** Persist _settings to localStorage immediately. */
  _saveSettings() {
    try {
      localStorage.setItem('rockbox:settings', JSON.stringify(this._settings));
    } catch { /* quota exceeded or private browsing — silently ignore */ }
  }

  /** Call a void rb_* function with optional integer arguments. */
  _call(name, args = []) {
    if (!this._mod) throw new Error('RockboxPlayer not initialised');
    const fn = this._mod[`_${name}`];
    if (typeof fn !== 'function') return; // export absent in this build
    // Resume AudioContext on any user-gesture call (play/pause/etc.)
    if (this._audioCtx && this._audioCtx.state !== 'running') {
      this._audioCtx.resume().then(() =>
        console.log('[Rockbox] AudioContext auto-resumed, state:', this._audioCtx.state)
      );
    }
    try {
      fn(...args);
    } catch (e) {
      if (!_isEmscriptenUnwind(e)) throw e;
    }
  }

  /** Call a rb_*_json function and return the parsed object. */
  _jsonCall(name) {
    if (!this._mod) throw new Error('RockboxPlayer not initialised');
    const fn = this._mod[`_${name}`];
    if (typeof fn !== 'function') return null; // export absent in this build
    let ptr = 0;
    try {
      ptr = fn();
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
