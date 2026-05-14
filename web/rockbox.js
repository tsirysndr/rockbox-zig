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
    balance: 0,
    channel_mode: 0,
    stereo_width: 100,
    crossfeed: {
      type: 0,
      direct_gain: -115,   // tenths of dB (Meier defaults)
      cross_gain: -320,
      hf_attenuation: -160,
      hf_cutoff: 700,
    },
    surround: {
      enabled: 0,
      balance: 50,
      fx1: 1200,
      fx2: 100,
      method2: 0,
      mix: 100,
    },
    bass: 0,
    treble: 0,
    dithering: false,
    afr: 0,
    pbe: 0,
    pbe_precut: 0,
    timestretch: 0,
    repeat: 0,
  };
}

export class RockboxPlayer {
  /**
   * @param {object} opts
   * @param {string} opts.wasmUrl    - URL to the emcc-generated rockboxd.js loader
   * @param {string} [opts.workletUrl] - Override AudioWorklet URL
   */
  constructor(opts = {}) {
    this._wasmUrl      = opts.wasmUrl    ?? 'rockboxd.js';
    this._workletUrl   = opts.workletUrl ?? WORKLET_URL;

    this._mod          = null; // Emscripten Module object
    this._audioCtx     = null;
    this._worklet      = null;
    this._sampleRate   = 44100;
    this._idb          = null;  // native IndexedDB handle for file persistence
    this._configDir    = '/config';

    // JS-side mirror of firmware settings.
    // Source of truth for localStorage persistence — updated synchronously on
    // every set*() call so saves never depend on an async WASM round-trip.
    this._settings = _defaultSettings();
  }

  // ── Init ────────────────────────────────────────────────────────────────

  async init(configDir = '/', musicDir = '/music') {
    this._configDir = configDir;
    await this._loadWasm();
    await this._initAudio();
    // Persistence files must be in MEMFS BEFORE _bootDaemon() so the firmware's
    // settings_load() / nvram read at startup finds the previously saved state.
    await this._initPersistence(configDir);
    this._bootDaemon(configDir, musicDir);
    await this._waitForDaemon();
    this._startPolling();
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
      // Return firmware values merged on top of JS mirror defaults.
      // Do NOT mutate this._settings — it is the source of truth for
      // localStorage persistence and must not be overwritten with stale
      // firmware state (e.g. right after restoreState() posts commands
      // that haven't been processed yet).
      return {
        ...this._settings,
        ...fw,
        crossfade: fw.crossfade ?? this._settings.crossfade,
      };
    }
    return { ...this._settings };
  }

  /**
   * Returns the JS-side settings mirror — always up-to-date after any
   * set*() call, without doing a firmware round-trip.  Use this right
   * after restoreState() to populate the UI before the firmware has
   * processed the queued commands.
   */
  getLocalSettings() {
    return { ...this._settings };
  }

  // ── Settings (set) ────────────────────────────────────────────────────────

  /** @param {boolean} enabled */
  setEqEnabled(enabled) {
    this._settings.eq.enabled = !!enabled;
    this._saveSettings();
    console.log('[RB] setEqEnabled', !!enabled);
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
      // Auto-enable EQ when any band is set to a non-zero gain.
      if ((gain | 0) !== 0 && !this._settings.eq.enabled) {
        this._settings.eq.enabled = true;
        this._call('rb_set_eq_enabled', [1]);
      }
      this._saveSettings();
    }
    console.log('[RB] setEqBand', b, cutoff | 0, q | 0, gain | 0);
    // Firmware stores gain in tenths of dB (e.g. -80 = -8.0 dB).
    this._call('rb_set_eq_band', [b, cutoff | 0, q | 0, (gain | 0) * 10]);
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
    console.log('[RB] setCrossfade mode=%d fi_dur=%d fo_dur=%d fi_delay=%d fo_delay=%d mix=%d',
      mode | 0, fadeInDuration | 0, fadeOutDuration | 0,
      fadeInDelay | 0, fadeOutDelay | 0, mixmode | 0);
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

  /**
   * Set stereo balance.
   * @param {number} value  -100 (full left) to +100 (full right), 0 = centre.
   */
  setBalance(value) {
    this._settings.balance = value | 0;
    this._saveSettings();
    this._call('rb_set_balance', [value | 0]);
  }

  /**
   * Set channel mode.
   * @param {number} value  0=stereo, 1=mono, 2=custom, 3=mono-L, 4=mono-R,
   *                        5=karaoke, 6=swap.
   */
  setChannelMode(value) {
    this._settings.channel_mode = value | 0;
    this._saveSettings();
    this._call('rb_set_channel_mode', [value | 0]);
  }

  /**
   * Set stereo width.
   * @param {number} value  0–250, where 100 = normal stereo.
   */
  setStereoWidth(value) {
    this._settings.stereo_width = value | 0;
    this._saveSettings();
    this._call('rb_set_stereo_width', [value | 0]);
  }

  /**
   * Set crossfeed (headphone DSP) parameters.
   * @param {number} type_  0=off, 1=Meier, 2=custom.
   * @param {object} [opts]
   * @param {number} [opts.directGain=-115]    In tenths of dB (e.g. -115 = -11.5 dB)
   * @param {number} [opts.crossGain=-320]     In tenths of dB
   * @param {number} [opts.hfAttenuation=-160] In tenths of dB
   * @param {number} [opts.hfCutoff=700]       In Hz (500–2000)
   */
  setCrossfeed(type_, opts = {}) {
    const defaults = this._settings.crossfeed;
    const {
      directGain    = defaults.direct_gain,
      crossGain     = defaults.cross_gain,
      hfAttenuation = defaults.hf_attenuation,
      hfCutoff      = defaults.hf_cutoff,
    } = opts;
    this._settings.crossfeed = {
      type: type_ | 0,
      direct_gain:    directGain    | 0,
      cross_gain:     crossGain     | 0,
      hf_attenuation: hfAttenuation | 0,
      hf_cutoff:      hfCutoff      | 0,
    };
    this._saveSettings();
    this._call('rb_set_crossfeed', [
      type_ | 0,
      directGain    | 0,
      crossGain     | 0,
      hfAttenuation | 0,
      hfCutoff      | 0,
    ]);
  }

  /**
   * Set surround (Haas effect) parameters.
   * @param {object} [opts]
   * @param {number} [opts.enabled=0]    0=off, or delay in ms (5, 8, 10, 15, 30).
   * @param {number} [opts.balance=50]   0–99 %.
   * @param {number} [opts.fx1=1200]     Low-pass cutoff Hz (600–8000).
   * @param {number} [opts.fx2=100]      High-pass cutoff Hz (40–400).
   * @param {number} [opts.method2=0]    0=method1, 1=method2.
   * @param {number} [opts.mix=100]      0–100 %.
   */
  setSurround(opts = {}) {
    const defaults = this._settings.surround;
    const {
      enabled = defaults.enabled,
      balance = defaults.balance,
      fx1     = defaults.fx1,
      fx2     = defaults.fx2,
      method2 = defaults.method2,
      mix     = defaults.mix,
    } = opts;
    this._settings.surround = {
      enabled: enabled | 0,
      balance: balance | 0,
      fx1:     fx1     | 0,
      fx2:     fx2     | 0,
      method2: method2 | 0,
      mix:     mix     | 0,
    };
    this._saveSettings();
    this._call('rb_set_surround', [
      enabled | 0,
      balance | 0,
      fx1     | 0,
      fx2     | 0,
      method2 | 0,
      mix     | 0,
    ]);
  }

  /**
   * Set bass tone control.
   * @param {number} value  Whole dB, typically -24..+24.
   */
  setBass(value) {
    this._settings.bass = value | 0;
    this._saveSettings();
    this._call('rb_set_bass', [value | 0]);
  }

  /**
   * Set treble tone control.
   * @param {number} value  Whole dB, typically -24..+24.
   */
  setTreble(value) {
    this._settings.treble = value | 0;
    this._saveSettings();
    this._call('rb_set_treble', [value | 0]);
  }

  /** @param {boolean} enabled */
  setDithering(enabled) {
    this._settings.dithering = !!enabled;
    this._saveSettings();
    this._call('rb_set_dithering', [enabled ? 1 : 0]);
  }

  /**
   * Set Adaptive Frequency Response.
   * @param {number} value  0=off, 1–3=mode.
   */
  setAfr(value) {
    this._settings.afr = value | 0;
    this._saveSettings();
    this._call('rb_set_afr', [value | 0]);
  }

  /**
   * Set Perceptual Bass Enhancement.
   * @param {number} pbe     0=off, 1–3=strength.
   * @param {number} precut  Pre-cut in tenths of dB (0–240).
   */
  setPbe(pbe, precut = 0) {
    this._settings.pbe       = pbe    | 0;
    this._settings.pbe_precut = precut | 0;
    this._saveSettings();
    this._call('rb_set_pbe', [pbe | 0, precut | 0]);
  }

  /**
   * Enable time-stretch and set speed.
   * @param {number} stretchPct  0 = disable; 35–250 = stretch % (100 = normal).
   */
  setTimestretch(stretchPct) {
    this._settings.timestretch = stretchPct | 0;
    this._saveSettings();
    this._call('rb_set_timestretch', [stretchPct | 0]);
  }

  /**
   * Set repeat mode.
   * @param {number} mode  0=off, 1=all, 2=one, 3=shuffle.
   */
  setRepeat(mode) {
    this._settings.repeat = mode | 0;
    this._saveSettings();
    this._call('rb_set_repeat', [mode | 0]);
  }

  /** Returns the current repeat mode from the JS settings mirror. */
  getRepeat() { return this._settings.repeat ?? 0; }

  /** Flush current settings to the Rockbox config file (no-op before rebuild). */
  saveSettings() { this._call('rb_save_settings'); }

  // ── Event emitter ─────────────────────────────────────────────────────────

  /**
   * Subscribe to a player event.
   * @param {'progress'|'track'|'status'|'playlist'} event
   * @param {Function} callback
   *   progress → { elapsed_ms, duration_ms, status, track }
   *   track    → { title, artist, album, path, duration_ms }
   *   status   → { status: 0|1|2 }
   *   playlist → { index: number, amount: number }
   */
  on(event, callback) {
    if (!this._listeners) this._listeners = {};
    if (!this._listeners[event]) this._listeners[event] = new Set();
    this._listeners[event].add(callback);
    return this; // chainable
  }

  /** Unsubscribe a previously registered callback. */
  off(event, callback) {
    this._listeners?.[event]?.delete(callback);
    return this;
  }

  _emit(event, data) {
    this._listeners?.[event]?.forEach(cb => { try { cb(data); } catch(_) {} });
  }

  _startPolling(intervalMs = 200) {
    let lastPath           = null;
    let lastStatus         = -1;
    let lastPlaylistIndex  = -1;
    let lastPlaylistAmount = -1;
    const tick = () => {
      if (!this._mod) return;
      const { status } = this.status() ?? { status: 0 };
      const track      = this.currentTrack() ?? {};
      const pl         = this.playlist() ?? {};
      const progress = {
        elapsed_ms:  track.elapsed_ms  ?? 0,
        duration_ms: track.duration_ms ?? 0,
        status,
        track,
      };
      this._emit('progress', progress);
      if (status !== lastStatus) {
        lastStatus = status;
        this._emit('status', { status });
      }
      if (track.path !== lastPath) {
        lastPath = track.path ?? null;
        if (track.path) this._emit('track', track);
      }
      const plIdx = pl.index  ?? -1;
      const plAmt = pl.amount ?? 0;
      if (plIdx !== lastPlaylistIndex || plAmt !== lastPlaylistAmount) {
        lastPlaylistIndex  = plIdx;
        lastPlaylistAmount = plAmt;
        this._emit('playlist', { index: plIdx, amount: plAmt });
      }
    };
    this._pollHandle = setInterval(tick, intervalMs);
  }

  // ── Persistence helpers ───────────────────────────────────────────────────

  /**
   * Force-flush playback state + settings to MEMFS then copy key files to IndexedDB.
   * The firmware persists the playlist via its own .playlist_control mechanism,
   * so no URL re-serialisation is needed here.
   */
  persistState() {
    const fn = this._mod?._rb_flush_status;
    if (typeof fn === 'function') {
      try { fn(); } catch (e) { if (!_isEmscriptenUnwind(e)) throw e; }
    }
    this._persistFiles();
  }

  /**
   * Restore DSP/EQ settings from localStorage into the firmware and the JS
   * settings mirror.  Playlist and resume position are handled natively by the
   * firmware from the files restored by _initPersistence() — do NOT re-enqueue URLs here.
   */
  restoreState() {
    try {
      const settingsRaw = localStorage.getItem('rockbox:settings');
      if (!settingsRaw) return;
      const s = JSON.parse(settingsRaw);
      // Update JS mirror first so getLocalSettings() is consistent immediately.
      this._settings = { ..._defaultSettings(), ...s };
      // Apply to firmware.
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
      if (s.replaygain) this.setReplaygain(s.replaygain);
      if (s.balance != null) this.setBalance(s.balance);
      if (s.channel_mode != null) this.setChannelMode(s.channel_mode);
      if (s.stereo_width != null) this.setStereoWidth(s.stereo_width);
      if (s.crossfeed) {
        this.setCrossfeed(s.crossfeed.type ?? 0, {
          directGain:    s.crossfeed.direct_gain,
          crossGain:     s.crossfeed.cross_gain,
          hfAttenuation: s.crossfeed.hf_attenuation,
          hfCutoff:      s.crossfeed.hf_cutoff,
        });
      }
      if (s.surround) this.setSurround(s.surround);
      if (s.bass   != null) this.setBass(s.bass);
      if (s.treble != null) this.setTreble(s.treble);
      if (s.dithering != null) this.setDithering(s.dithering);
      if (s.afr != null) this.setAfr(s.afr);
      if (s.pbe != null) this.setPbe(s.pbe, s.pbe_precut ?? 0);
      if (s.timestretch != null) this.setTimestretch(s.timestretch);
      if (s.repeat != null) this.setRepeat(s.repeat);
    } catch (e) {
      console.warn('[Rockbox] restoreState failed:', e);
    }
  }

  /**
   * Read key Rockbox state files from WASM MEMFS and write them to IndexedDB.
   * Called by persistState() and safe to call at any time after daemon boot.
   * Fire-and-forget: safe to call from beforeunload without awaiting.
   */
  _persistFiles() {
    if (!this._idb) return;
    const FS = this._mod?.FS;
    if (!FS) return;
    for (const path of this._persistedFilePaths()) {
      let data;
      try { data = FS.readFile(path); } catch (_) { continue; }
      const tx = this._idb.transaction('files', 'readwrite');
      tx.objectStore('files').put(data, path);
    }
  }

  _persistedFilePaths() {
    const d = this._configDir;
    return [
      `${d}/.rockbox/config.cfg`,
      `${d}/.rockbox/nvram.bin`,
    ];
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

  /**
   * Open the IndexedDB persistence store and restore any previously saved
   * Rockbox state files into WASM MEMFS before the daemon boots.
   * Uses native indexedDB instead of Emscripten IDBFS to avoid the crash
   * that -lidbfs.js causes in Rockbox pthread worker initialization.
   */
  async _initPersistence(configDir) {
    try {
      this._idb = await new Promise((resolve, reject) => {
        const req = indexedDB.open('rockbox-persist', 1);
        req.onupgradeneeded = (e) => e.target.result.createObjectStore('files');
        req.onsuccess = (e) => resolve(e.target.result);
        req.onerror = () => reject(req.error);
      });
    } catch (e) {
      console.warn('[Rockbox] IndexedDB unavailable; persistence disabled:', e);
      return;
    }
    const FS = this._mod?.FS;
    if (!FS) return;
    // Ensure config directories exist in MEMFS.
    for (const dir of [configDir, `${configDir}/.rockbox`]) {
      try { FS.mkdir(dir); } catch (_) {}
    }
    // Restore saved files from IndexedDB into MEMFS.
    for (const path of this._persistedFilePaths()) {
      await new Promise((resolve) => {
        const tx = this._idb.transaction('files', 'readonly');
        const req = tx.objectStore('files').get(path);
        req.onsuccess = () => {
          if (req.result) {
            try {
              FS.writeFile(path, req.result);
              console.log('[Rockbox] Restored', path, 'from IndexedDB');
            } catch (e) {
              console.warn('[Rockbox] Failed to restore', path, ':', e);
            }
          }
          resolve();
        };
        req.onerror = () => resolve();
      });
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
