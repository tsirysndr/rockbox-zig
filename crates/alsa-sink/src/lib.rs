// Direct libasound PCM sink for ARM Linux (arm-unknown-linux-gnueabihf).
// All real code is gated on cfg(target_os = "linux") — libasound is not
// available on macOS or WASM. pcm-alsa.c is only compiled for ARMHFHOST.
//
// Uses snd_pcm_writei (RWInterleaved access) — exactly what `aplay` does.
// Avoids cpal's ALSA backend and the snd_pcm_status_get_htstamp / mmap paths
// that crash on some ARM libasound builds.
//
// Architecture mirrors crates/cpal-sink: a ring buffer + condvar carries PCM
// from the Rockbox firmware DMA thread to a dedicated OS thread that owns the
// ALSA PCM handle and drains it via snd_pcm_writei.
//
// C side: firmware/target/hosted/headless/pcm-alsa.c

/// Force-linkage sentinel. Always present so crates/cli can reference it
/// with #[cfg(feature = "alsa-sink")] without needing a cfg(target_os) guard.
pub fn _link_alsa_sink() {}

// ── Linux-only implementation ─────────────────────────────────────────────────

#[cfg(target_os = "linux")]
use alsa::pcm::{Access, Format, HwParams, PCM};
#[cfg(target_os = "linux")]
use alsa::{Direction, ValueOr};
#[cfg(target_os = "linux")]
use std::collections::VecDeque;
#[cfg(target_os = "linux")]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(target_os = "linux")]
use std::sync::{Condvar, Mutex, OnceLock};
#[cfg(target_os = "linux")]
use std::thread::JoinHandle;
#[cfg(target_os = "linux")]
use std::time::Duration;

#[cfg(target_os = "linux")]
const RING_CAPACITY: usize = 512 * 1024; // 512 KB ≈ 3 s at 44.1 kHz stereo S16LE
#[cfg(target_os = "linux")]
const PERIOD_FRAMES: alsa::pcm::Frames = 1024; // ~23 ms @ 44.1 kHz
#[cfg(target_os = "linux")]
const BUFFER_FRAMES: alsa::pcm::Frames = 8192; // ~185 ms

// ── Ring buffer ───────────────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
struct Ring {
    buf: VecDeque<u8>,
    running: bool,
}

#[cfg(target_os = "linux")]
static RING: OnceLock<(Mutex<Ring>, Condvar)> = OnceLock::new();

#[cfg(target_os = "linux")]
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

#[cfg(target_os = "linux")]
static WRITER_RUNNING: AtomicBool = AtomicBool::new(false);
#[cfg(target_os = "linux")]
static CURRENT_RATE: OnceLock<Mutex<u32>> = OnceLock::new();
#[cfg(target_os = "linux")]
static WRITER_HANDLE: OnceLock<Mutex<Option<JoinHandle<()>>>> = OnceLock::new();

#[cfg(target_os = "linux")]
fn current_rate() -> &'static Mutex<u32> {
    CURRENT_RATE.get_or_init(|| Mutex::new(44100))
}

#[cfg(target_os = "linux")]
fn writer_handle() -> &'static Mutex<Option<JoinHandle<()>>> {
    WRITER_HANDLE.get_or_init(|| Mutex::new(None))
}

// ── ALSA helpers ──────────────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
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

#[cfg(target_os = "linux")]
fn run_writer(initial_rate: u32) {
    let mut pcm: Option<PCM> = None;
    let mut rate = initial_rate;
    tracing::info!("pcm-alsa: writer thread started");

    loop {
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
            let n = r.buf.len().min(PERIOD_FRAMES as usize * 4 * 4);
            r.buf.drain(..n).collect()
        };
        ring().1.notify_all();

        if chunk.is_empty() {
            continue;
        }

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

        let samples_i16 =
            unsafe { std::slice::from_raw_parts(chunk.as_ptr() as *const i16, chunk.len() / 2) };
        let frames_total = samples_i16.len() / 2;
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

#[cfg(target_os = "linux")]
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

// ── C ABI — called from firmware/target/hosted/headless/pcm-alsa.c ───────────

#[cfg(target_os = "linux")]
#[no_mangle]
pub extern "C" fn pcm_alsa_init() {
    let _ = ring();
    let _ = current_rate();
    let _ = writer_handle();
}

#[cfg(target_os = "linux")]
#[no_mangle]
pub extern "C" fn pcm_alsa_postinit() {}

#[cfg(target_os = "linux")]
#[no_mangle]
pub extern "C" fn pcm_alsa_set_sample_rate(rate_hz: u32) {
    *current_rate().lock().unwrap() = rate_hz;
}

/// # Safety
/// `addr` must be valid for `size` bytes for the duration of this call.
#[cfg(target_os = "linux")]
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

#[cfg(target_os = "linux")]
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

#[cfg(target_os = "linux")]
#[no_mangle]
pub extern "C" fn pcm_alsa_stop() {
    let (lock, cvar) = ring();
    let mut r = lock.lock().unwrap();
    r.running = false;
    cvar.notify_all();
}

#[cfg(target_os = "linux")]
#[no_mangle]
pub extern "C" fn pcm_alsa_flush() {
    let (lock, cvar) = ring();
    let mut r = lock.lock().unwrap();
    r.buf.clear();
    cvar.notify_all();
}

#[cfg(target_os = "linux")]
#[no_mangle]
pub extern "C" fn pcm_alsa_is_running() -> bool {
    ring().0.lock().unwrap().running
}
