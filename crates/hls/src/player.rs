//! Player state machine + FFI entry points.
//!
//! One player instance per active stream. Most callers will only ever have
//! one playing at a time, so the FFI surface treats the player as a global
//! singleton: `rb_hls_play(url)` stops any current player and starts a new
//! one. Status is queryable via `rb_hls_status_json`.

use anyhow::{anyhow, Result};
use bytes::Bytes;
use once_cell::sync::Lazy;
use std::collections::VecDeque;
use std::ffi::CStr;
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU8, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;

use crate::decoder::decode_segment;
use crate::demux;
use crate::fetcher::{self, SegmentCache};
use crate::manifest::{self, ManifestKind};
use crate::output;

/// Shared tokio runtime for all HLS player work. Multi-threaded so segment
/// prefetch can overlap with decoding.
static RT: Lazy<Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .thread_name("rockbox-hls")
        .build()
        .expect("hls: build tokio runtime")
});

/// Singleton player. Replacing it via `rb_hls_play` cancels any prior task.
static PLAYER: Lazy<Mutex<Option<Arc<Player>>>> = Lazy::new(|| Mutex::new(None));

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerState {
    Stopped = 0,
    Buffering = 1,
    Playing = 2,
    Paused = 3,
    Errored = 4,
}

pub struct Player {
    url: String,
    /// Base URL of the broadcaster's gRPC endpoint, derived from the
    /// HLS/DASH URL host. Used by the gRPC layer to proxy
    /// seek / next / previous / current_track / status to the upstream
    /// Rockbox so the consumer behaves as a thin remote control. Default
    /// gRPC port is 6061 (same as `ROCKBOX_PORT` on the broadcaster).
    remote_api_base: String,
    state: AtomicU8,
    paused: AtomicBool,
    stop_flag: Arc<AtomicBool>,
    /// Decoded position in milliseconds.
    position_ms: AtomicI64,
    /// Total duration in ms (or -1 for live).
    duration_ms: AtomicI64,
    is_live: AtomicBool,
    task: Mutex<Option<JoinHandle<()>>>,
    last_error: Mutex<Option<String>>,
}

impl Player {
    fn new(url: String) -> Arc<Self> {
        let remote_api_base = derive_remote_api_base(&url);
        Arc::new(Player {
            url,
            remote_api_base,
            state: AtomicU8::new(PlayerState::Stopped as u8),
            paused: AtomicBool::new(false),
            stop_flag: Arc::new(AtomicBool::new(false)),
            position_ms: AtomicI64::new(0),
            duration_ms: AtomicI64::new(-1),
            is_live: AtomicBool::new(false),
            task: Mutex::new(None),
            last_error: Mutex::new(None),
        })
    }

    pub fn remote_api_base(&self) -> &str {
        &self.remote_api_base
    }

    fn set_state(&self, s: PlayerState) {
        self.state.store(s as u8, Ordering::SeqCst);
    }

    pub fn state(&self) -> PlayerState {
        match self.state.load(Ordering::SeqCst) {
            0 => PlayerState::Stopped,
            1 => PlayerState::Buffering,
            2 => PlayerState::Playing,
            3 => PlayerState::Paused,
            _ => PlayerState::Errored,
        }
    }

    fn record_error(&self, msg: String) {
        tracing::error!("hls: {msg}");
        *self.last_error.lock().unwrap() = Some(msg);
        self.set_state(PlayerState::Errored);
    }

    fn cancel(&self) {
        self.stop_flag.store(true, Ordering::SeqCst);
        if let Some(h) = self.task.lock().unwrap().take() {
            h.abort();
        }
    }
}

/// The actual playback loop. Runs on the shared tokio runtime.
async fn run_player(player: Arc<Player>) {
    if let Err(e) = run_player_inner(&player).await {
        player.record_error(format!("{e:#}"));
    } else {
        player.set_state(PlayerState::Stopped);
    }
}

async fn run_player_inner(player: &Arc<Player>) -> Result<()> {
    let client = Arc::new(
        reqwest::Client::builder()
            .user_agent("rockbox-hls/0.1")
            .timeout(Duration::from_secs(20))
            .build()?,
    );
    player.set_state(PlayerState::Buffering);

    // Resolve master → media playlist once if needed (single redirect hop).
    let url = player.url.clone();
    let kind = manifest::is_hls_or_dash_url(&url)
        .ok_or_else(|| anyhow!("URL does not look like HLS or DASH: {url}"))?;
    let mut snap = match manifest::fetch_and_parse(&client, &url, kind).await {
        Ok(s) => s,
        Err(e) if e.to_string().contains("re-fetch variant") => {
            // Extract variant URL from the error and re-fetch.
            let s = e.to_string();
            let variant = s
                .rsplit(' ')
                .next()
                .ok_or_else(|| anyhow!("variant url parse: {e}"))?
                .to_string();
            manifest::fetch_and_parse(&client, &variant, ManifestKind::Hls).await?
        }
        Err(e) => return Err(e),
    };
    player.is_live.store(snap.is_live, Ordering::SeqCst);
    player.duration_ms.store(
        snap.duration.map(|d| (d * 1000.0) as i64).unwrap_or(-1),
        Ordering::SeqCst,
    );

    // Init segment (fMP4 only — MPEG-TS has its own self-describing headers).
    let cache = Arc::new(SegmentCache::default());
    let mut init_bytes: Option<Bytes> = None;
    let mut current_sample_rate: u32 = 0;
    if let Some(init_url) = snap.init_url.clone() {
        let init = fetcher::fetch_bytes(&client, &init_url).await?;
        match demux::parse_init(&init) {
            Ok(h) => {
                if let Some(sr) = h.sample_rate {
                    current_sample_rate = sr;
                    output::set_sample_rate(sr);
                }
            }
            Err(e) => {
                tracing::warn!("hls: parse init failed ({e}); decoder will probe per-segment")
            }
        }
        init_bytes = Some(init);
    }

    // Index of the next segment we will play out.
    let mut next_play_seq = snap
        .segments
        .first()
        .map(|s| s.seq)
        .ok_or_else(|| anyhow!("manifest has no segments"))?;
    // For live, jump near the live edge.
    if snap.is_live {
        let n = snap.segments.len();
        if n > 3 {
            next_play_seq = snap.segments[n - 3].seq;
        }
    }

    // Initial prefetch of next few segments.
    let initial: Vec<_> = snap
        .segments
        .iter()
        .filter(|s| s.seq >= next_play_seq)
        .take(3)
        .cloned()
        .collect();
    fetcher::prefetch(client.clone(), cache.clone(), initial).await;

    player.set_state(PlayerState::Playing);

    // Track of the last "known" segment list so refresher can append.
    let known: Arc<Mutex<VecDeque<crate::manifest::SegmentRef>>> =
        Arc::new(Mutex::new(snap.segments.iter().cloned().collect()));

    // Spawn refresher for live streams.
    let refresher = if snap.is_live {
        let client = client.clone();
        let known = known.clone();
        let cache = cache.clone();
        let stop = player.stop_flag.clone();
        let interval = snap.refresh_interval;
        let url = url.clone();
        let kind = snap.kind;
        Some(tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;
                if stop.load(Ordering::SeqCst) {
                    break;
                }
                match manifest::fetch_and_parse(&client, &url, kind).await {
                    Ok(new_snap) => {
                        let snapshot: Vec<_> = {
                            let mut g = known.lock().unwrap();
                            let last_seen = g.back().map(|s| s.seq).unwrap_or(0);
                            for s in new_snap.segments {
                                if s.seq > last_seen {
                                    g.push_back(s);
                                }
                            }
                            // Keep the deque from growing unboundedly.
                            while g.len() > 64 {
                                g.pop_front();
                            }
                            g.iter().rev().take(3).cloned().collect()
                        };
                        fetcher::prefetch(client.clone(), cache.clone(), snapshot).await;
                    }
                    Err(e) => tracing::warn!("hls refresh: {e}"),
                }
            }
        }))
    } else {
        None
    };

    // Main play loop.
    loop {
        if player.stop_flag.load(Ordering::SeqCst) {
            break;
        }
        if player.paused.load(Ordering::SeqCst) {
            tokio::time::sleep(Duration::from_millis(100)).await;
            continue;
        }

        // Find the segment we need next.
        let seg = {
            let g = known.lock().unwrap();
            g.iter().find(|s| s.seq == next_play_seq).cloned()
        };
        let Some(seg) = seg else {
            // No segment with this seq exists yet.
            if snap.is_live {
                tokio::time::sleep(Duration::from_millis(200)).await;
                continue;
            }
            // VOD: end of stream.
            break;
        };

        // Pull segment bytes from cache or fetch.
        let bytes = match cache.get(seg.seq).await {
            Some(b) => b,
            None => match fetcher::fetch_bytes(&client, &seg.url).await {
                Ok(b) => {
                    cache.put(seg.seq, b.clone()).await;
                    b
                }
                Err(e) => {
                    tracing::warn!("hls: fetch seg {} failed: {e}; skipping", seg.seq);
                    next_play_seq += 1;
                    continue;
                }
            },
        };

        // Symphonia: demux + decode in one pass.
        let decoded = match decode_segment(init_bytes.as_deref(), &bytes) {
            Ok(d) => d,
            Err(e) => {
                tracing::warn!("hls: decode seg {} failed: {e}; skipping", seg.seq);
                next_play_seq += 1;
                continue;
            }
        };

        // Pause-honoring write: chunk the PCM so we can check paused/stop
        // between writes without holding up sink callbacks indefinitely.
        if decoded.sample_rate != current_sample_rate && decoded.sample_rate != 0 {
            current_sample_rate = decoded.sample_rate;
            output::set_sample_rate(decoded.sample_rate);
        }
        let ch = decoded.channels.max(1) as usize;
        let chunk_frames = decoded.sample_rate.max(1) as usize / 20; // ~50 ms
        let chunk_samples = chunk_frames * ch;
        for window in decoded.samples.chunks(chunk_samples.max(ch)) {
            if player.stop_flag.load(Ordering::SeqCst) {
                break;
            }
            while player.paused.load(Ordering::SeqCst) && !player.stop_flag.load(Ordering::SeqCst) {
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
            output::write_pcm(window);
            let frames_pushed = window.len() / ch;
            let ms = (frames_pushed as i64 * 1000) / decoded.sample_rate.max(1) as i64;
            player.position_ms.fetch_add(ms, Ordering::SeqCst);
        }

        // Prefetch the next-next segment so we stay ahead.
        let upcoming: Vec<_> = {
            let g = known.lock().unwrap();
            g.iter()
                .filter(|s| s.seq > next_play_seq && s.seq <= next_play_seq + 3)
                .cloned()
                .collect()
        };
        fetcher::prefetch(client.clone(), cache.clone(), upcoming).await;

        next_play_seq += 1;
    }

    if let Some(h) = refresher {
        h.abort();
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Public Rust API — used by crates/rpc to route HLS/DASH URLs from
// PlaybackService.PlayTrack into the standalone player without going
// through the FFI surface.
// ---------------------------------------------------------------------------

fn replace_player(new_player: Arc<Player>) {
    let mut g = PLAYER.lock().unwrap();
    if let Some(old) = g.take() {
        old.cancel();
    }
    *g = Some(new_player);
}

fn current() -> Option<Arc<Player>> {
    PLAYER.lock().unwrap().clone()
}

/// Returns true if the standalone HLS/DASH player has an active session
/// (Buffering, Playing, or Paused). Used by the gRPC layer to decide
/// whether to dispatch playback commands to the remote broadcaster vs.
/// the local Rockbox playback engine.
pub fn is_active() -> bool {
    match current().as_ref().map(|p| p.state()) {
        Some(PlayerState::Buffering | PlayerState::Playing | PlayerState::Paused) => true,
        _ => false,
    }
}

/// gRPC base URL of the broadcaster that owns the currently-playing HLS/DASH
/// stream (e.g. `http://192.168.1.42:6061`). None if no active session.
pub fn remote_api_base() -> Option<String> {
    current().map(|p| p.remote_api_base.clone())
}

/// Derive `http://<host>:6061` from any HLS / DASH URL. Falls back to
/// `http://127.0.0.1:6061` if parsing fails — the caller's request will
/// just hit the local daemon which is also harmless.
fn derive_remote_api_base(manifest_url: &str) -> String {
    // gRPC port — matches `ROCKBOX_PORT` on the broadcaster.
    let port = std::env::var("ROCKBOX_PORT").unwrap_or_else(|_| "6061".to_string());
    match url::Url::parse(manifest_url) {
        Ok(u) => match u.host_str() {
            Some(host) => format!("http://{host}:{port}"),
            None => format!("http://127.0.0.1:{port}"),
        },
        Err(_) => format!("http://127.0.0.1:{port}"),
    }
}

/// Start playing an HLS or DASH URL. Returns Err if the URL doesn't look
/// like a manifest (caller should fall back to regular Rockbox playback).
pub fn play(url: &str) -> Result<(), String> {
    if manifest::is_hls_or_dash_url(url).is_none() {
        return Err(format!("not an HLS or DASH URL: {url}"));
    }
    let player = Player::new(url.to_string());
    let runner = player.clone();
    let task = RT.spawn(async move { run_player(runner).await });
    *player.task.lock().unwrap() = Some(task);
    replace_player(player);
    Ok(())
}

pub fn pause() -> bool {
    if let Some(p) = current() {
        p.paused.store(true, Ordering::SeqCst);
        p.set_state(PlayerState::Paused);
        true
    } else {
        false
    }
}

pub fn resume() -> bool {
    if let Some(p) = current() {
        p.paused.store(false, Ordering::SeqCst);
        p.set_state(PlayerState::Playing);
        true
    } else {
        false
    }
}

pub fn stop() -> bool {
    let mut g = PLAYER.lock().unwrap();
    if let Some(p) = g.take() {
        p.cancel();
        p.set_state(PlayerState::Stopped);
        true
    } else {
        false
    }
}

pub fn status_json() -> String {
    let p = current();
    let mut obj = serde_json::Map::new();
    let state_str = match p.as_ref().map(|p| p.state()) {
        Some(PlayerState::Stopped) | None => "stopped",
        Some(PlayerState::Buffering) => "buffering",
        Some(PlayerState::Playing) => "playing",
        Some(PlayerState::Paused) => "paused",
        Some(PlayerState::Errored) => "errored",
    };
    obj.insert("state".into(), serde_json::Value::String(state_str.into()));
    if let Some(p) = p.as_ref() {
        obj.insert("url".into(), serde_json::Value::String(p.url.clone()));
        obj.insert(
            "position_ms".into(),
            serde_json::Value::Number(p.position_ms.load(Ordering::SeqCst).into()),
        );
        obj.insert(
            "duration_ms".into(),
            serde_json::Value::Number(p.duration_ms.load(Ordering::SeqCst).into()),
        );
        obj.insert(
            "is_live".into(),
            serde_json::Value::Bool(p.is_live.load(Ordering::SeqCst)),
        );
        if let Some(e) = p.last_error.lock().unwrap().clone() {
            obj.insert("error".into(), serde_json::Value::String(e));
        }
    }
    serde_json::to_string(&obj).unwrap_or_else(|_| "{}".to_string())
}

// ---------------------------------------------------------------------------
// FFI surface — thin wrappers over the Rust API above.
// ---------------------------------------------------------------------------

/// Returns 0 on success, negative on error. The URL must outlive the call.
#[cfg(feature = "ffi")]
#[no_mangle]
pub extern "C" fn rb_hls_play(url: *const std::os::raw::c_char) -> std::os::raw::c_int {
    if url.is_null() {
        return -1;
    }
    let url = unsafe { CStr::from_ptr(url) };
    let url = match url.to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };
    match play(url) {
        Ok(()) => 0,
        Err(_) => -3,
    }
}

#[cfg(feature = "ffi")]
#[no_mangle]
pub extern "C" fn rb_hls_pause() -> std::os::raw::c_int {
    if pause() {
        0
    } else {
        -1
    }
}

#[cfg(feature = "ffi")]
#[no_mangle]
pub extern "C" fn rb_hls_resume() -> std::os::raw::c_int {
    if resume() {
        0
    } else {
        -1
    }
}

#[cfg(feature = "ffi")]
#[no_mangle]
pub extern "C" fn rb_hls_stop() -> std::os::raw::c_int {
    if stop() {
        0
    } else {
        -1
    }
}

/// VOD seek by milliseconds. Live streams ignore the seek and return -2.
#[cfg(feature = "ffi")]
#[no_mangle]
pub extern "C" fn rb_hls_seek(_position_ms: i64) -> std::os::raw::c_int {
    let Some(p) = current() else { return -1 };
    if p.is_live.load(Ordering::SeqCst) {
        return -2;
    }
    // First-cut: not implemented. Returning -3 lets the caller surface a
    // "seek not yet supported" rather than silently failing. Once we wire
    // seek-by-segment-index this becomes the normal entry point.
    -3
}

/// Returns a JSON status blob: `{state, position_ms, duration_ms, is_live, url, error?}`.
/// Caller MUST `rb_hls_free_string` the result.
#[cfg(feature = "ffi")]
#[no_mangle]
pub extern "C" fn rb_hls_status_json() -> *mut std::os::raw::c_char {
    match std::ffi::CString::new(status_json()) {
        Ok(cs) => cs.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Frees a string returned by `rb_hls_status_json`.
#[cfg(feature = "ffi")]
#[no_mangle]
pub extern "C" fn rb_hls_free_string(s: *mut std::os::raw::c_char) {
    if !s.is_null() {
        unsafe {
            let _ = std::ffi::CString::from_raw(s);
        }
    }
}
