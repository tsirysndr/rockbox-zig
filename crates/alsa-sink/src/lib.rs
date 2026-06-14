// Direct libasound PCM sink for ARM Linux (arm-unknown-linux-gnueabihf).
// All real code is gated on cfg(target_os = "linux") — libasound is not
// available on macOS or WASM. pcm-alsa.c is only compiled for ARMHFHOST.
//
// Uses snd_pcm_writei (RWInterleaved access) — exactly what `aplay` does.
// This avoids cpal's ALSA backend entirely and the problematic
// snd_pcm_status_get_htstamp / mmap code paths that crash on some ARM builds.
//
// Architecture mirrors crates/cpal-sink: a ring buffer + condvar carries PCM
// data from the Rockbox firmware DMA thread; a dedicated OS thread owns the
// ALSA PCM handle and drains the ring via snd_pcm_writei. ALSA stalls cannot
// wedge the firmware thread beyond the ring's backpressure timeout.
//
// C side: firmware/target/hosted/headless/pcm-alsa.c

// Force-linkage sentinel: always present so cli/src/lib.rs can reference it
// unconditionally on any host (the #[cfg(feature = "alsa-sink")] guard is
// sufficient; we don't need a separate #[cfg(target_os)] there).
pub fn _link_alsa_sink() {}

#[cfg(target_os = "linux")]
mod linux {

use alsa::pcm::{Access, Format, HwParams, PCM};
use alsa::{Direction, ValueOr};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Condvar, Mutex, OnceLock};
use std::thread::JoinHandle;
use std::time::Duration;

const RING_CAPACITY: usize = 512 * 1024; // 512 KB ≈ 3 s at 44.1 kHz stereo S16LE
const PERIOD_FRAMES: alsa::pcm::Frames = 1024; // ~23 ms @ 44.1 kHz
const BUFFER_FRAMES: alsa::pcm::Frames = 8192; // ~185 ms

// ── Ring buffer ───────────────────────────────────────────────────────────────

struct Ring {
    buf: VecDeque<u8>,
    running: bool,
}

static RING: OnceLock<(Mutex<Ring>, Condvar)> = OnceLock::new();

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

// ── Writer thread state ───────────────────────────────────────────────────────

static WRITER_RUNNING: AtomicBool = AtomicBool::new(false);
static CURRENT_RATE: OnceLock<Mutex<u32>> = OnceLock::new();
static WRITER_HANDLE: OnceLock<Mutex<Option<JoinHandle<()>>>> = OnceLock::new();

fn current_rate() -> &'static Mutex<u32> {
    CURRENT_RATE.get_or_init(|| Mutex::new(44100))
}

fn writer_handle() -> &'static Mutex<Option<JoinHandle<()>>> {
    WRITER_HANDLE.get_or_init(|| Mutex::new(None))
}

// ── ALSA helpers ──────────────────────────────────────────────────────────────

fn open_pcm(rate: u32) -> Option<PCM> {
    let pcm = match PCM::new("default", Direction::Playback, false) {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("pcm-alsa: open 'default' failed: {e}");
            return None;
        }
    };
    {
        let hwp = match HwParams::any(&pcm) {
            Ok(h) => h,
            Err(e) => {
                tracing::error!("pcm-alsa: HwParams::any failed: {e}");
                return None;
            }
        };
        if hwp.set_access(Access::RWInterleaved).is_err()
            || hwp.set_format(Format::s16()).is_err()
            || hwp.set_channels(2).is_err()
            || hwp.set_rate(rate, ValueOr::Nearest).is_err()
        {
            tracing::error!("pcm-alsa: HwParams setup failed for {rate} Hz");
            return None;
        }
        let _ = hwp.set_buffer_size_near(BUFFER_FRAMES);
        let _ = hwp.set_period_size_near(PERIOD_FRAMES, ValueOr::Nearest);
        if let Err(e) = pcm.hw_params(&hwp) {
            tracing::error!("pcm-alsa: hw_params apply failed: {e}");
            return None;
        }
    }
    if let Err(e) = pcm.prepare() {
        tracing::error!("pcm-alsa: prepare failed: {e}");
        return None;
    }
    tracing::info!("pcm-alsa: opened 'default' at {rate} Hz stereo S16LE");
    Some(pcm)
}

// ── Writer thread ─────────────────────────────────────────────────────────────

fn run_writer(initial_rate: u32) {
    let mut pcm: Option<PCM> = None;
    let mut rate = initial_rate;
    tracing::info!("pcm-alsa: writer thread started");

    loop {
        // Wait for data or stop signal.
        let chunk: Vec<u8> = {
            let (lock, cvar) = ring();
            let mut r = lock.lock().unwrap();
            loop {
                if !r.running && r.buf.is_empty() {
                    if let Some(p) = pcm.take() {
                        let _ = p.drain();
                    }
                    tracing::info!("pcm-alsa: writer thread exiting");
                    WRITER_RUNNING.store(false, Ordering::Relaxed);
                    return;
                }
                if r.buf.len() >= 4 {
                    break;
                }
                r = cvar.wait(r).unwrap();
            }
            // Drain up to 4× period frames per iteration.
            let n = r.buf.len().min(PERIOD_FRAMES as usize * 4 * 4);
            r.buf.drain(..n).collect()
        };
        ring().1.notify_all(); // signal producer that ring space freed

        if chunk.is_empty() {
            continue;
        }

        // Lazily open or reopen ALSA when sample rate changed.
        let new_rate = *current_rate().lock().unwrap();
        if new_rate != rate || pcm.is_none() {
            if let Some(old) = pcm.take() {
                let _ = old.drain();
            }
            rate = new_rate;
            pcm = open_pcm(rate);
        }

        let p = match pcm.as_ref() {
            Some(p) => p,
            None => continue,
        };

        let io = match p.io_i16() {
            Ok(io) => io,
            Err(e) => {
                tracing::error!("pcm-alsa: io_i16: {e}");
                pcm = None;
                continue;
            }
        };

        // Reinterpret raw S16LE bytes as i16 for snd_pcm_writei.
        let samples_i16 =
            unsafe { std::slice::from_raw_parts(chunk.as_ptr() as *const i16, chunk.len() / 2) };
        let frames_total = samples_i16.len() / 2; // stereo: 2 samples per frame
        let mut offset = 0usize;
        while offset < frames_total {
            match io.writei(&samples_i16[offset * 2..]) {
                Ok(n) => offset += n,
                Err(e) => {
                    tracing::warn!("pcm-alsa: writei at frame {offset}: {e}, recovering");
                    if let Err(re) = p.try_recover(e, true) {
                        tracing::error!("pcm-alsa: recover failed: {re}");
                        pcm = None;
                        break;
                    }
                }
            }
        }
    }
}

fn start_writer() {
    let rate = *current_rate().lock().unwrap();
    let mut guard = writer_handle().lock().unwrap();
    if guard.is_some() {
        return;
    }
    WRITER_RUNNING.store(true, Ordering::Relaxed);
    *guard = Some(
        std::thread::Builder::new()
            .name("rockbox-alsa".into())
            .spawn(move || run_writer(rate))
            .expect("spawn alsa writer thread"),
    );
}

fn stop_writer_join() {
    {
        let (lock, cvar) = ring();
        let mut r = lock.lock().unwrap();
        r.running = false;
        cvar.notify_all();
    }
    if let Some(h) = writer_handle().lock().unwrap().take() {
        let _ = h.join();
    }
    WRITER_RUNNING.store(false, Ordering::Relaxed);
}

// ── C ABI — called from firmware/target/hosted/headless/pcm-alsa.c ───────────

#[no_mangle]
pub extern "C" fn pcm_alsa_init() {
    let _ = ring();
    let _ = current_rate();
    let _ = writer_handle();
}

#[no_mangle]
pub extern "C" fn pcm_alsa_postinit() {
    // No-op: we open ALSA lazily on first write to avoid blocking firmware boot.
}

#[no_mangle]
pub extern "C" fn pcm_alsa_set_sample_rate(rate_hz: u32) {
    *current_rate().lock().unwrap() = rate_hz;
}

/// Push `size` bytes of S16LE stereo PCM from the firmware DMA thread.
/// Blocks (condvar) when the ring is too full, providing back-pressure.
///
/// # Safety
/// `addr` must be valid for `size` bytes for the duration of this call.
#[no_mangle]
pub unsafe extern "C" fn pcm_alsa_push(addr: *const u8, size: usize) {
    let data = unsafe { std::slice::from_raw_parts(addr, size) };
    let (lock, cvar) = ring();
    let mut r = lock.lock().unwrap();
    let mut stall_ms: u32 = 0;
    while r.running && r.buf.len() + size > RING_CAPACITY {
        let (new_r, timed_out) = cvar.wait_timeout(r, Duration::from_millis(200)).unwrap();
        r = new_r;
        if timed_out.timed_out() {
            stall_ms += 200;
            if stall_ms >= 3000 {
                tracing::warn!("pcm-alsa: ring not draining for 3 s, aborting push");
                r.running = false;
                cvar.notify_all();
                return;
            }
        }
    }
    if !r.running {
        return;
    }
    r.buf.extend(data.iter().copied());
    cvar.notify_all();
}

#[no_mangle]
pub extern "C" fn pcm_alsa_start() {
    {
        let (lock, cvar) = ring();
        let mut r = lock.lock().unwrap();
        r.running = true;
        cvar.notify_all();
    }
    start_writer();
}

#[no_mangle]
pub extern "C" fn pcm_alsa_stop() {
    let (lock, cvar) = ring();
    let mut r = lock.lock().unwrap();
    r.running = false;
    cvar.notify_all();
}

#[no_mangle]
pub extern "C" fn pcm_alsa_flush() {
    let (lock, cvar) = ring();
    let mut r = lock.lock().unwrap();
    r.buf.clear();
    cvar.notify_all();
}

#[no_mangle]
pub extern "C" fn pcm_alsa_is_running() -> bool {
    ring().0.lock().unwrap().running
}

/// Force-linkage sentinel — crates/cli pulls this in so alsa-sink symbols
/// are included in librockbox_cli.a even with --gc-sections.
pub fn _link_alsa_sink() {}
