//! CMAF (fragmented MP4) PCM sink — AAC-LC encoded audio served as both
//! HLS and DASH from the same in-memory segment ring buffer.
//!
//! Pipeline:
//!
//!     pcm_cmaf_write(PCM)
//!         → PcmIntake (Mutex<VecDeque<u8>> + Condvar)
//!             → encoder thread: fdk-aac → AAC frames
//!                 → segmenter: accumulate N frames into one fMP4 segment
//!                     → SegmentStore (ring buffer, last K segments)
//!                         → HTTP server: /init.mp4, /seg/{n}.m4s,
//!                           /hls/master.m3u8, /hls/audio.m3u8,
//!                           /dash/manifest.mpd
//!
//! NOTE on licensing: libfdk-aac ships under the "Software License for The
//! Fraunhofer FDK AAC Codec Library for Android" — it is OSS but is *not*
//! GPL-compatible. Distributing a binary that combines this sink with the
//! GPLv2 Rockbox firmware is a license conflict for redistribution.
//! Personal / non-distributed use is the intended scope.

mod dash;
mod encoder;
mod hls;
mod http;
mod mp4;

use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

// Called from rockbox-cli to force this crate's symbols into librockbox_cli.a.
#[doc(hidden)]
pub fn _link_cmaf() {}

// ---------------------------------------------------------------------------
// Audio parameters — fixed for now (matches the upstream Rockbox PCM format
// when the sink is selected).
// ---------------------------------------------------------------------------

pub(crate) const SAMPLE_RATE: u32 = 44_100;
pub(crate) const CHANNELS: u16 = 2;
/// fdk-aac AAC-LC frame size, in samples per channel.
pub(crate) const AAC_FRAME_SAMPLES: usize = 1024;
/// AAC frames per fMP4 segment — 86 × 1024 / 44100 = 1.997 s.
pub(crate) const FRAMES_PER_SEGMENT: usize = 86;
/// Sliding-window size for both HLS playlist and DASH timeShiftBufferDepth.
pub(crate) const SEGMENT_WINDOW: usize = 6;
/// Maximum segments retained in memory (window + a little slack so a slow
/// client can still pull the oldest in-window segment).
pub(crate) const SEGMENT_CAPACITY: usize = 12;

// ---------------------------------------------------------------------------
// PCM intake — single-producer, single-consumer byte queue.
// ---------------------------------------------------------------------------

pub(crate) struct PcmIntake {
    inner: Mutex<PcmIntakeInner>,
    cv: Condvar,
}

struct PcmIntakeInner {
    buf: VecDeque<u8>,
    closed: bool,
}

impl PcmIntake {
    fn new() -> Self {
        PcmIntake {
            inner: Mutex::new(PcmIntakeInner {
                buf: VecDeque::with_capacity(64 * 1024),
                closed: false,
            }),
            cv: Condvar::new(),
        }
    }

    pub(crate) fn push(&self, data: &[u8]) {
        let mut g = self.inner.lock().unwrap();
        if g.closed {
            return;
        }
        g.buf.extend(data.iter().copied());
        self.cv.notify_all();
    }

    /// Pull at least `min` bytes (or until closed). Returns None when the
    /// intake has been closed and is empty.
    pub(crate) fn pull_at_least(&self, min: usize) -> Option<Vec<u8>> {
        let mut g = self.inner.lock().unwrap();
        loop {
            if g.buf.len() >= min {
                let out: Vec<u8> = g.buf.drain(..).collect();
                return Some(out);
            }
            if g.closed {
                if g.buf.is_empty() {
                    return None;
                }
                let out: Vec<u8> = g.buf.drain(..).collect();
                return Some(out);
            }
            g = self.cv.wait(g).unwrap();
        }
    }

    fn reset(&self) {
        let mut g = self.inner.lock().unwrap();
        g.buf.clear();
        g.closed = false;
    }

    fn close(&self) {
        let mut g = self.inner.lock().unwrap();
        g.closed = true;
        self.cv.notify_all();
    }
}

// ---------------------------------------------------------------------------
// DiskMirror — optional side-car that writes the same artefacts to a
// directory. All I/O is best-effort: failures are logged at warn level and
// never abort the encoder thread, since the in-memory ring is authoritative.
//
// Layout written under `dir`:
//
//     init.mp4
//     seg/{N}.m4s
//     hls/master.m3u8
//     hls/audio.m3u8        (re-written every segment)
//     dash/manifest.mpd     (re-written every segment)
//
// External HTTP servers (nginx etc.) can be pointed at `dir` and the paths
// inside the manifests resolve correctly because they are root-relative
// (`/init.mp4`, `/seg/{n}.m4s`).
// ---------------------------------------------------------------------------

pub(crate) struct DiskMirror {
    dir: PathBuf,
}

impl DiskMirror {
    fn new(dir: PathBuf) -> std::io::Result<Self> {
        fs::create_dir_all(&dir)?;
        fs::create_dir_all(dir.join("seg"))?;
        fs::create_dir_all(dir.join("hls"))?;
        fs::create_dir_all(dir.join("dash"))?;
        tracing::info!("cmaf: mirroring segments to {}", dir.display());
        Ok(DiskMirror { dir })
    }

    fn write_init(&self, init: &[u8]) {
        let p = self.dir.join("init.mp4");
        if let Err(e) = fs::write(&p, init) {
            tracing::warn!("cmaf mirror: write {} failed: {e}", p.display());
        }
        let master = self.dir.join("hls").join("master.m3u8");
        if let Err(e) = fs::write(&master, hls::master_m3u8()) {
            tracing::warn!("cmaf mirror: write {} failed: {e}", master.display());
        }
    }

    fn write_segment(&self, seq: u64, bytes: &[u8]) {
        let p = self.dir.join("seg").join(format!("{seq}.m4s"));
        if let Err(e) = fs::write(&p, bytes) {
            tracing::warn!("cmaf mirror: write {} failed: {e}", p.display());
        }
    }

    fn remove_segment(&self, seq: u64) {
        let p = self.dir.join("seg").join(format!("{seq}.m4s"));
        if let Err(e) = fs::remove_file(&p) {
            if e.kind() != std::io::ErrorKind::NotFound {
                tracing::warn!("cmaf mirror: rm {} failed: {e}", p.display());
            }
        }
    }

    fn write_manifests(&self, media_seq: u64, segs: &[Segment], start_ms: u64) {
        let audio = self.dir.join("hls").join("audio.m3u8");
        if let Err(e) = fs::write(&audio, hls::audio_m3u8(media_seq, segs)) {
            tracing::warn!("cmaf mirror: write {} failed: {e}", audio.display());
        }
        let mpd = self.dir.join("dash").join("manifest.mpd");
        if let Err(e) = fs::write(&mpd, dash::manifest_mpd(start_ms, media_seq)) {
            tracing::warn!("cmaf mirror: write {} failed: {e}", mpd.display());
        }
    }
}

// ---------------------------------------------------------------------------
// SegmentStore — sliding ring buffer of completed fMP4 segments.
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub(crate) struct Segment {
    /// Sequence number; first segment after a stream start is `start_seq`.
    pub seq: u64,
    /// Sample count (in mdhd timescale = SAMPLE_RATE) for this segment.
    pub duration: u32,
    /// fMP4 payload (styp + moof + mdat).
    pub bytes: Arc<Vec<u8>>,
}

pub(crate) struct SegmentStore {
    inner: Mutex<SegmentStoreInner>,
    cv: Condvar,
    /// Optional on-disk mirror — when Some, every segment + manifest is also
    /// written to this directory so external HTTP servers (nginx, Caddy, a
    /// CDN origin) can serve them directly. The in-memory ring is always the
    /// source of truth; disk is purely a side-car.
    mirror: Mutex<Option<Arc<DiskMirror>>>,
}

struct SegmentStoreInner {
    init: Option<Arc<Vec<u8>>>,
    segments: VecDeque<Segment>,
    /// Sequence number of the first segment ever produced for this stream.
    start_seq: u64,
    /// availabilityStartTime equivalent — wall-clock when the producer started.
    start_unix_ms: u64,
}

impl SegmentStore {
    fn new() -> Self {
        SegmentStore {
            inner: Mutex::new(SegmentStoreInner {
                init: None,
                segments: VecDeque::with_capacity(SEGMENT_CAPACITY),
                start_seq: 1,
                start_unix_ms: 0,
            }),
            cv: Condvar::new(),
            mirror: Mutex::new(None),
        }
    }

    /// Install or remove the on-disk mirror. Pass `None` to disable.
    fn set_mirror(&self, dir: Option<PathBuf>) {
        let new = match dir {
            None => None,
            Some(d) => match DiskMirror::new(d) {
                Ok(m) => Some(Arc::new(m)),
                Err(e) => {
                    tracing::warn!("cmaf: disk mirror setup failed: {e}");
                    None
                }
            },
        };
        *self.mirror.lock().unwrap() = new;
    }

    fn mirror_snapshot(&self) -> Option<Arc<DiskMirror>> {
        self.mirror.lock().unwrap().clone()
    }

    fn set_init(&self, init: Vec<u8>) {
        let init_arc = Arc::new(init);
        {
            let mut g = self.inner.lock().unwrap();
            g.init = Some(init_arc.clone());
            g.start_unix_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            self.cv.notify_all();
        }
        if let Some(m) = self.mirror_snapshot() {
            m.write_init(&init_arc);
        }
    }

    fn push(&self, seg: Segment) {
        let (evicted, mirror_snapshot) = {
            let mut g = self.inner.lock().unwrap();
            g.segments.push_back(seg.clone());
            let evicted = if g.segments.len() > SEGMENT_CAPACITY {
                g.segments.pop_front().map(|s| s.seq)
            } else {
                None
            };
            let n = g.segments.len();
            let take = n.min(SEGMENT_WINDOW);
            let start = n - take;
            let segs: Vec<Segment> = g.segments.iter().skip(start).cloned().collect();
            let media_sequence = segs.first().map(|s| s.seq).unwrap_or(g.start_seq);
            let start_unix_ms = g.start_unix_ms;
            self.cv.notify_all();
            (evicted, (segs, media_sequence, start_unix_ms))
        };
        if let Some(m) = self.mirror_snapshot() {
            m.write_segment(seg.seq, &seg.bytes);
            if let Some(e) = evicted {
                m.remove_segment(e);
            }
            let (segs, media_seq, start_ms) = mirror_snapshot;
            m.write_manifests(media_seq, &segs, start_ms);
        }
    }

    pub(crate) fn init(&self) -> Option<Arc<Vec<u8>>> {
        self.inner.lock().unwrap().init.clone()
    }

    pub(crate) fn get(&self, seq: u64) -> Option<Arc<Vec<u8>>> {
        let g = self.inner.lock().unwrap();
        g.segments
            .iter()
            .find(|s| s.seq == seq)
            .map(|s| s.bytes.clone())
    }

    /// Snapshot of the current playlist window. Returns
    /// `(media_sequence, segments_in_order, start_unix_ms)` where
    /// `segments_in_order` is at most SEGMENT_WINDOW most-recent segments.
    pub(crate) fn snapshot(&self) -> (u64, Vec<Segment>, u64) {
        let g = self.inner.lock().unwrap();
        let n = g.segments.len();
        let take = n.min(SEGMENT_WINDOW);
        let start = n - take;
        let segs: Vec<Segment> = g.segments.iter().skip(start).cloned().collect();
        let media_sequence = segs.first().map(|s| s.seq).unwrap_or(g.start_seq);
        (media_sequence, segs, g.start_unix_ms)
    }

    fn reset(&self) {
        let mut g = self.inner.lock().unwrap();
        g.init = None;
        g.segments.clear();
        g.start_seq = 1;
        g.start_unix_ms = 0;
    }
}

// ---------------------------------------------------------------------------
// Global state
// ---------------------------------------------------------------------------

static INTAKE: OnceLock<Arc<PcmIntake>> = OnceLock::new();
static STORE: OnceLock<Arc<SegmentStore>> = OnceLock::new();
static STARTED: Mutex<bool> = Mutex::new(false);

struct CmafConfig {
    http_port: u16,
    bitrate_bps: u32,
    /// When Some, segments + manifests are mirrored to this directory.
    /// None = in-memory only (default).
    segment_dir: Option<PathBuf>,
}

static CONFIG: Mutex<CmafConfig> = Mutex::new(CmafConfig {
    http_port: 7882,
    bitrate_bps: 128_000,
    segment_dir: None,
});

pub(crate) fn intake() -> Arc<PcmIntake> {
    INTAKE.get_or_init(|| Arc::new(PcmIntake::new())).clone()
}

pub(crate) fn store() -> Arc<SegmentStore> {
    STORE.get_or_init(|| Arc::new(SegmentStore::new())).clone()
}

pub(crate) fn bitrate_bps() -> u32 {
    CONFIG.lock().unwrap().bitrate_bps
}

// ---------------------------------------------------------------------------
// FFI exports — gated behind `ffi` feature to match the slim/upnp/chromecast
// pattern.
// ---------------------------------------------------------------------------

#[cfg(feature = "ffi")]
#[no_mangle]
pub extern "C" fn pcm_cmaf_set_http_port(port: u16) {
    CONFIG.lock().unwrap().http_port = port;
}

#[cfg(feature = "ffi")]
#[no_mangle]
pub extern "C" fn pcm_cmaf_set_bitrate(bps: u32) {
    let clamped = bps.clamp(32_000, 320_000);
    CONFIG.lock().unwrap().bitrate_bps = clamped;
}

/// Set (or clear, via NULL / empty) the directory the sink should mirror
/// segments + manifests into. Applies to the live store immediately so live
/// device-picker changes pick it up without restarting the daemon.
#[cfg(feature = "ffi")]
#[no_mangle]
pub extern "C" fn pcm_cmaf_set_segment_dir(path: *const std::os::raw::c_char) {
    let dir: Option<PathBuf> = if path.is_null() {
        None
    } else {
        let s = unsafe { std::ffi::CStr::from_ptr(path) };
        match s.to_str() {
            Ok(p) if !p.is_empty() => Some(PathBuf::from(p)),
            _ => None,
        }
    };
    CONFIG.lock().unwrap().segment_dir = dir.clone();
    store().set_mirror(dir);
}

/// Start the encoder + HTTP server. Idempotent.
#[cfg(feature = "ffi")]
#[no_mangle]
pub extern "C" fn pcm_cmaf_start() -> std::os::raw::c_int {
    let mut started = STARTED.lock().unwrap();
    if *started {
        return 0;
    }

    let cfg = CONFIG.lock().unwrap();
    let http_port = cfg.http_port;
    let bitrate = cfg.bitrate_bps;
    let segment_dir = cfg.segment_dir.clone();
    drop(cfg);

    let intake = intake();
    let store = store();
    intake.reset();
    store.reset();
    store.set_mirror(segment_dir);

    let intake_enc = intake.clone();
    let store_enc = store.clone();
    std::thread::spawn(move || {
        if let Err(e) = encoder::run(intake_enc, store_enc, bitrate) {
            tracing::error!("cmaf encoder thread exited: {e}");
        }
    });

    let store_http = store.clone();
    std::thread::spawn(move || http::serve(http_port, store_http));

    *started = true;
    tracing::info!(
        "cmaf sink: HLS at http://localhost:{http_port}/hls/master.m3u8, \
         DASH at http://localhost:{http_port}/dash/manifest.mpd \
         (AAC-LC {} kbps)",
        bitrate / 1000
    );
    0
}

/// Push raw S16LE stereo PCM into the intake queue.
#[cfg(feature = "ffi")]
#[no_mangle]
pub extern "C" fn pcm_cmaf_write(data: *const u8, len: usize) -> std::os::raw::c_int {
    if data.is_null() || len == 0 {
        return 0;
    }
    let slice = unsafe { std::slice::from_raw_parts(data, len) };
    intake().push(slice);
    0
}

/// No-op between tracks — HTTP listeners keep their connections.
#[cfg(feature = "ffi")]
#[no_mangle]
pub extern "C" fn pcm_cmaf_stop() {}

/// Shut down (called on daemon exit).
#[cfg(feature = "ffi")]
#[no_mangle]
pub extern "C" fn pcm_cmaf_close() {
    let mut started = STARTED.lock().unwrap();
    intake().close();
    *started = false;
}
