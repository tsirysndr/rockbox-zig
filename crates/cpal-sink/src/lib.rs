/* SPDX-License-Identifier: GPL-2.0-or-later
 *
 * cpal PCM sink for the headless macOS / Linux build of Rockbox.
 *
 * The C side (firmware/target/hosted/headless/pcm-cpal.c) drives a writer
 * thread that calls pcm_cpal_push() for each firmware DMA chunk, then calls
 * pcm_play_dma_complete_callback() to get the next one. This Rust side
 * opens a cpal output stream and exposes a lock-free ring buffer that the
 * cpal audio callback drains.
 *
 * # Data flow
 *
 *   firmware DMA thread
 *     → pcm_cpal_push(data, size)    (blocks on back-pressure)
 *       → ring buffer (512 KB, S16LE stereo)
 *         ← cpal audio callback drains at device rate
 *           – linear-interpolation resample if in_rate ≠ out_rate
 *           – converts i16 → f32 if the device requires f32
 *
 * # Thread safety
 *
 * ring_mutex protects the VecDeque; pcm_cpal_push() waits on ring_cvar when
 * the buffer is full. The cpal callback holds the mutex only for the duration
 * of the drain and notifies ring_cvar when it frees space.
 */

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::collections::VecDeque;
use std::sync::{Condvar, Mutex, OnceLock};

const RING_CAPACITY: usize = 512 * 1024;

struct Ring {
    buf: VecDeque<u8>,
    running: bool,
}

static RING: OnceLock<(Mutex<Ring>, Condvar)> = OnceLock::new();

// cpal::Stream is not Send on macOS (CoreAudio callbacks are OS-managed, not
// Rust-managed, so the !Send bound is over-conservative for our use case).
// We only ever write/replace the stream from pcm_cpal_postinit /
// pcm_cpal_set_sample_rate, which are called sequentially by the firmware
// init path before any audio thread is running.
#[allow(dead_code)]
struct StreamHolder(cpal::Stream);
unsafe impl Send for StreamHolder {}
unsafe impl Sync for StreamHolder {}

static STREAM: OnceLock<Mutex<Option<StreamHolder>>> = OnceLock::new();
static CURRENT_RATE: OnceLock<Mutex<u32>> = OnceLock::new();

// State for the linear-interpolation resampler, shared with the cpal callback.
struct ResamplerState {
    // Fractional phase in input-sample units. Advances by in_rate/out_rate per
    // output stereo frame. When >= 1.0 we consume an input sample from the ring.
    phase: f64,
    step: f64, // in_rate / out_rate
    // Hold the last consumed stereo input sample for interpolation with the next.
    prev_l: f32,
    prev_r: f32,
    // The "current" sample (one ahead of prev) already read from the ring.
    cur_l: f32,
    cur_r: f32,
    // Whether cur_{l,r} is valid (false until we have read at least one input frame).
    cur_valid: bool,
}

static RESAMPLER: OnceLock<Mutex<ResamplerState>> = OnceLock::new();

fn resampler() -> &'static Mutex<ResamplerState> {
    RESAMPLER.get_or_init(|| {
        Mutex::new(ResamplerState {
            phase: 0.0,
            step: 1.0,
            prev_l: 0.0,
            prev_r: 0.0,
            cur_l: 0.0,
            cur_r: 0.0,
            cur_valid: false,
        })
    })
}

fn ring() -> &'static (Mutex<Ring>, Condvar) {
    RING.get_or_init(|| {
        (
            Mutex::new(Ring {
                buf: VecDeque::with_capacity(RING_CAPACITY),
                running: false,
            }),
            Condvar::new(),
        )
    })
}

fn stream_cell() -> &'static Mutex<Option<StreamHolder>> {
    STREAM.get_or_init(|| Mutex::new(None))
}

fn current_rate() -> &'static Mutex<u32> {
    CURRENT_RATE.get_or_init(|| Mutex::new(44100))
}

// Read one S16LE stereo frame (4 bytes) from the ring as (f32_L, f32_R).
// Returns None if fewer than 4 bytes are available.
fn pop_frame(ring: &mut Ring) -> Option<(f32, f32)> {
    if ring.buf.len() < 4 {
        return None;
    }
    let b0 = ring.buf.pop_front().unwrap();
    let b1 = ring.buf.pop_front().unwrap();
    let b2 = ring.buf.pop_front().unwrap();
    let b3 = ring.buf.pop_front().unwrap();
    let l = i16::from_le_bytes([b0, b1]) as f32 / 32768.0;
    let r = i16::from_le_bytes([b2, b3]) as f32 / 32768.0;
    Some((l, r))
}

// Fill `output` (f32 interleaved stereo) by resampling from the ring buffer.
// Uses linear interpolation between consecutive input frames.
fn fill_output_f32(output: &mut [f32], ring: &mut Ring, rs: &mut ResamplerState, cvar: &Condvar) {
    let frames = output.len() / 2; // output.len() is always even (stereo)
    let mut wrote = 0usize;

    // Ensure we have a "current" frame loaded.
    if !rs.cur_valid {
        if let Some((l, r)) = pop_frame(ring) {
            rs.cur_l = l;
            rs.cur_r = r;
            rs.cur_valid = true;
        }
    }

    for i in 0..frames {
        if !rs.cur_valid {
            // No more input — silence the rest.
            output[i * 2] = 0.0;
            output[i * 2 + 1] = 0.0;
            continue;
        }

        // Advance phase. When it crosses 1.0, consume input frames.
        rs.phase += rs.step;
        while rs.phase >= 1.0 {
            rs.prev_l = rs.cur_l;
            rs.prev_r = rs.cur_r;
            rs.phase -= 1.0;
            if let Some((l, r)) = pop_frame(ring) {
                rs.cur_l = l;
                rs.cur_r = r;
            } else {
                // Ring empty — hold last sample and break.
                rs.cur_valid = false;
                break;
            }
        }

        // Linear interpolate between prev and cur using fractional phase.
        let t = rs.phase as f32;
        output[i * 2] = rs.prev_l + t * (rs.cur_l - rs.prev_l);
        output[i * 2 + 1] = rs.prev_r + t * (rs.cur_r - rs.prev_r);
        wrote += 1;
    }

    if wrote < frames {
        let start = wrote * 2;
        output[start..].fill(0.0);
    }

    if wrote > 0 {
        cvar.notify_all();
    }
}

fn open_stream(rate: u32) {
    let host = cpal::default_host();
    let device = match host.default_output_device() {
        Some(d) => d,
        None => {
            tracing::error!("pcm-cpal: no default output device");
            return;
        }
    };

    // Find the best supported config: prefer stereo + our rate, f32 format.
    let supported_configs = match device.supported_output_configs() {
        Ok(c) => c.collect::<Vec<_>>(),
        Err(e) => {
            tracing::error!("pcm-cpal: failed to query supported configs: {e}");
            return;
        }
    };

    tracing::debug!(
        "pcm-cpal: {} supported config ranges",
        supported_configs.len()
    );

    // Try stereo at the requested rate first.
    let exact = supported_configs
        .iter()
        .filter(|r| {
            r.channels() == 2 && r.min_sample_rate().0 <= rate && r.max_sample_rate().0 >= rate
        })
        .min_by_key(|r| match r.sample_format() {
            cpal::SampleFormat::F32 => 0u8,
            cpal::SampleFormat::I16 => 1,
            _ => 2,
        })
        .map(|r| r.clone().with_sample_rate(cpal::SampleRate(rate)));

    // If not available, use the device default (may differ in rate or channels).
    let chosen = match exact {
        Some(c) => c,
        None => match device.default_output_config() {
            Ok(c) => {
                tracing::warn!(
                    "pcm-cpal: device has no config covering {rate} Hz; \
                     falling back to default ({} Hz, {} ch, {:?})",
                    c.sample_rate().0,
                    c.channels(),
                    c.sample_format()
                );
                c
            }
            Err(e) => {
                tracing::error!("pcm-cpal: default_output_config failed: {e}");
                return;
            }
        },
    };

    let out_rate = chosen.sample_rate().0;
    let fmt = chosen.sample_format();
    let channels = chosen.channels();

    // Update resampler step: how many input samples per output sample.
    {
        let mut rs = resampler().lock().unwrap();
        rs.step = rate as f64 / out_rate as f64;
        rs.phase = 0.0;
        rs.cur_valid = false;
        rs.prev_l = 0.0;
        rs.prev_r = 0.0;
        rs.cur_l = 0.0;
        rs.cur_r = 0.0;
    }

    if out_rate != rate {
        tracing::info!(
            "pcm-cpal: resampling {rate} Hz → {out_rate} Hz (step={:.6})",
            rate as f64 / out_rate as f64
        );
    }

    let config: cpal::StreamConfig = chosen.into();
    let ring_ref = ring();
    let rs_ref = resampler();

    // Build an f32 stream (by far the most common on macOS/CoreAudio).
    // If the device truly wants i16 and exact rate matches, use i16 directly.
    let stream_result = if fmt == cpal::SampleFormat::I16 && out_rate == rate && channels == 2 {
        device.build_output_stream(
            &config,
            move |output: &mut [i16], _| {
                let (lock, cvar) = ring_ref;
                let mut r = lock.lock().unwrap();
                let need_bytes = output.len() * 2;
                let have = r.buf.len().min(need_bytes);
                for (i, chunk) in r
                    .buf
                    .drain(..have)
                    .collect::<Vec<_>>()
                    .chunks(2)
                    .enumerate()
                {
                    if chunk.len() == 2 {
                        output[i] = i16::from_le_bytes([chunk[0], chunk[1]]);
                    }
                }
                let filled = have / 2;
                if filled < output.len() {
                    output[filled..].fill(0);
                }
                if have > 0 {
                    cvar.notify_all();
                }
            },
            |err| tracing::error!("pcm-cpal stream error: {err}"),
            None,
        )
    } else {
        // f32 path — also handles resampling and mono fallback.
        device.build_output_stream(
            &config,
            move |output: &mut [f32], _| {
                let (lock, cvar) = ring_ref;
                let mut r = lock.lock().unwrap();
                let mut rs = rs_ref.lock().unwrap();

                if channels == 2 {
                    fill_output_f32(output, &mut r, &mut rs, cvar);
                } else {
                    // Mono output: mix L+R from stereo input.
                    let frames = output.len();
                    let mut tmp = vec![0f32; frames * 2];
                    fill_output_f32(&mut tmp, &mut r, &mut rs, cvar);
                    for i in 0..frames {
                        output[i] = (tmp[i * 2] + tmp[i * 2 + 1]) * 0.5;
                    }
                }
            },
            |err| tracing::error!("pcm-cpal stream error: {err}"),
            None,
        )
    };

    let stream = match stream_result {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("pcm-cpal: failed to open stream at {out_rate} Hz ({fmt:?}): {e}");
            return;
        }
    };

    if let Err(e) = stream.play() {
        tracing::error!("pcm-cpal: stream.play() failed: {e}");
        return;
    }

    *stream_cell().lock().unwrap() = Some(StreamHolder(stream));
    *current_rate().lock().unwrap() = out_rate;
    tracing::info!("pcm-cpal: opened cpal stream at {out_rate} Hz ({fmt:?}, {channels} ch)");
}

// ── C ABI — called from firmware/target/hosted/headless/pcm-cpal.c ────────

#[no_mangle]
pub extern "C" fn pcm_cpal_init() {
    let _ = ring();
    let _ = stream_cell();
    let _ = current_rate();
    let _ = resampler();
}

#[no_mangle]
pub extern "C" fn pcm_cpal_postinit() {
    open_stream(44100);
    ring().0.lock().unwrap().running = true;
}

#[no_mangle]
pub extern "C" fn pcm_cpal_set_sample_rate(rate_hz: u32) {
    let current = *current_rate().lock().unwrap();
    if current != rate_hz {
        tracing::debug!("pcm-cpal: sample rate change {current} → {rate_hz} Hz");
        open_stream(rate_hz);
    }
}

/// Push `size` bytes of S16LE stereo PCM from the firmware DMA thread.
/// Blocks (condvar wait) when the ring buffer is too full to accept the
/// chunk, providing back-pressure so the firmware thread paces itself.
///
/// # Safety
/// `addr` must be valid for `size` bytes for the duration of this call.
#[no_mangle]
pub unsafe extern "C" fn pcm_cpal_push(addr: *const u8, size: usize) {
    let data = std::slice::from_raw_parts(addr, size);
    let (lock, cvar) = ring();
    let mut r = lock.lock().unwrap();
    while r.running && r.buf.len() + size > RING_CAPACITY {
        r = cvar.wait(r).unwrap();
    }
    if !r.running {
        return;
    }
    r.buf.extend(data.iter().copied());
}

#[no_mangle]
pub extern "C" fn pcm_cpal_start() {
    let (lock, _cvar) = ring();
    let mut r = lock.lock().unwrap();
    r.running = true;
}

#[no_mangle]
pub extern "C" fn pcm_cpal_stop() {
    let (lock, cvar) = ring();
    let mut r = lock.lock().unwrap();
    r.running = false;
    r.buf.clear();
    cvar.notify_all();
}

/// Force-linkage sentinel. crates/cli pulls this in so that the cpal-sink
/// symbols are included in librockbox_cli.a even with --gc-sections.
pub fn _link_cpal_sink() {}
