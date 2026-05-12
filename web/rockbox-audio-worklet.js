/**
 * Rockbox AudioWorkletProcessor — reads S16LE stereo PCM frames from the
 * WASM linear-memory ring buffer written by the C wa_thread pthread.
 */

class RockboxProcessor extends AudioWorkletProcessor {
  constructor(options) {
    super();

    try {
      const p   = options.processorOptions;
      const mem = p.wasmMemory;

      this.port.postMessage({
        type:  'init',
        isSAB: mem instanceof SharedArrayBuffer,
        ringPtr: p.ringPtr, ringFrames: p.ringFrames,
        wiPtr: p.wiPtr, riPtr: p.riPtr,
      });

      this._ring       = new Int16Array(mem, p.ringPtr,  p.ringFrames * 2);
      this._writeIdx   = new Int32Array(mem, p.wiPtr,    1);
      this._readIdx    = new Int32Array(mem, p.riPtr,    1);
      this._ringFrames = p.ringFrames;
      this._ok         = true;
      this._diagCount  = 0;
    } catch (e) {
      this.port.postMessage({ type: 'error', message: String(e) });
      this._ok = false;
    }
  }

  process(_inputs, outputs) {
    const left        = outputs[0][0];
    const right       = outputs[0][1];
    const blockFrames = left.length; // 128 per Web Audio block

    if (!this._ok) {
      left.fill(0); right.fill(0);
      return true;
    }

    // Log ring state every ~5 s (5 s / 2.9 ms per block ≈ 1700 blocks)
    if (++this._diagCount % 1700 === 1) {
      const wi = Atomics.load(this._writeIdx, 0);
      const ri = Atomics.load(this._readIdx,  0);
      this.port.postMessage({ type: 'diag', wi, ri });
    }

    for (let i = 0; i < blockFrames; i++) {
      const ri = Atomics.load(this._readIdx,  0);
      const wi = Atomics.load(this._writeIdx, 0);

      if (ri === wi) {
        left[i]  = 0;
        right[i] = 0;
      } else {
        const pos = ri * 2;
        left[i]   = this._ring[pos]     / 32768.0;
        right[i]  = this._ring[pos + 1] / 32768.0;
        Atomics.store(this._readIdx, 0, (ri + 1) % this._ringFrames);
      }
    }

    return true;
  }
}

registerProcessor('rockbox-processor', RockboxProcessor);
