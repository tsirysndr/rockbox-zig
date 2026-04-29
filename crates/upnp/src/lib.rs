pub mod api {
    #[path = ""]
    pub mod rockbox {
        #[path = "rockbox.v1alpha1.rs"]
        pub mod v1alpha1;
    }
}

pub mod control_point;
pub mod db;
pub(crate) mod didl;
pub mod format;
pub(crate) mod pcm_server;
pub mod renderer;
pub mod scan;
pub mod server;
pub(crate) mod ssdp;

// Called from rockbox-cli to force this crate's symbols into librockbox_cli.a
#[doc(hidden)]
pub fn _link_upnp() {}

use std::collections::VecDeque;
#[cfg(feature = "ffi")]
use std::ffi::CStr;
#[cfg(feature = "ffi")]
use std::os::raw::{c_char, c_int};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, OnceLock};

// ---------------------------------------------------------------------------
// Broadcast buffer — one writer, N independent readers (WAV PCM stream).
// Follows the same pattern as the rockbox-slim crate.
// ---------------------------------------------------------------------------

pub(crate) enum RecvResult {
    Data(Vec<u8>),
    Closed,
}

pub(crate) struct BroadcastBuffer {
    inner: Mutex<BroadcastInner>,
    condvar: Condvar,
}

struct BroadcastInner {
    chunks: VecDeque<(u64, Vec<u8>)>,
    next_seq: u64,
    total_bytes: usize,
    closed: bool,
}

// 4 MB — about 23 s of S16LE stereo at 44100 Hz
const MAX_BUFFERED: usize = 4 * 1024 * 1024;

impl BroadcastBuffer {
    fn new() -> Self {
        BroadcastBuffer {
            inner: Mutex::new(BroadcastInner {
                chunks: VecDeque::new(),
                next_seq: 0,
                total_bytes: 0,
                closed: false,
            }),
            condvar: Condvar::new(),
        }
    }

    pub(crate) fn push(&self, data: &[u8]) {
        let mut g = self.inner.lock().unwrap();
        if g.closed {
            return;
        }
        let seq = g.next_seq;
        g.next_seq += 1;
        g.total_bytes += data.len();
        g.chunks.push_back((seq, data.to_vec()));
        while g.total_bytes > MAX_BUFFERED {
            if let Some((_, old)) = g.chunks.pop_front() {
                g.total_bytes -= old.len();
            } else {
                break;
            }
        }
        self.condvar.notify_all();
    }

    pub(crate) fn subscribe(self: &Arc<Self>) -> BroadcastReceiver {
        let next_seq = self.inner.lock().unwrap().next_seq;
        BroadcastReceiver {
            buf: Arc::clone(self),
            next_seq,
        }
    }

    /// Start a subscriber `n` chunks *behind* the live edge so the remote
    /// client can drain historical buffered data at network speed instead of
    /// waiting for new chunks to arrive in real time.  Clamped to the oldest
    /// chunk still in the buffer.
    pub(crate) fn subscribe_from_behind(self: &Arc<Self>, n: u64) -> BroadcastReceiver {
        let g = self.inner.lock().unwrap();
        let front_seq = g.chunks.front().map(|(s, _)| *s).unwrap_or(g.next_seq);
        let next_seq = g.next_seq.saturating_sub(n).max(front_seq);
        BroadcastReceiver {
            buf: Arc::clone(self),
            next_seq,
        }
    }

    fn reset(&self) {
        let mut g = self.inner.lock().unwrap();
        g.chunks.clear();
        g.total_bytes = 0;
        g.closed = false;
        // next_seq is intentionally NOT reset so existing receivers skip forward.
    }

    fn close(&self) {
        let mut g = self.inner.lock().unwrap();
        g.closed = true;
        self.condvar.notify_all();
    }
}

pub(crate) struct BroadcastReceiver {
    buf: Arc<BroadcastBuffer>,
    next_seq: u64,
}

impl BroadcastReceiver {
    pub(crate) fn recv_blocking(&mut self) -> RecvResult {
        let mut g = self.buf.inner.lock().unwrap();
        loop {
            if g.closed {
                return RecvResult::Closed;
            }
            if let Some(&(front_seq, _)) = g.chunks.front() {
                if self.next_seq < front_seq {
                    tracing::debug!(
                        "upnp/pcm: receiver lagging, skipping {} → {}",
                        self.next_seq,
                        front_seq
                    );
                    self.next_seq = front_seq;
                }
                if self.next_seq < g.next_seq {
                    let idx = (self.next_seq - front_seq) as usize;
                    let chunk = g.chunks[idx].1.clone();
                    self.next_seq += 1;
                    return RecvResult::Data(chunk);
                }
            }
            g = self.buf.condvar.wait(g).unwrap();
        }
    }
}

// ---------------------------------------------------------------------------
// Global state
// ---------------------------------------------------------------------------

static BUFFER: OnceLock<Arc<BroadcastBuffer>> = OnceLock::new();
static PCM_STARTED: Mutex<bool> = Mutex::new(false);
static RENDERER_PLAYING: Mutex<bool> = Mutex::new(false);
static SERVER_STARTED: Mutex<bool> = Mutex::new(false);
static DEVICE_UUID: OnceLock<String> = OnceLock::new();
static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
static MONITOR_GEN: AtomicU64 = AtomicU64::new(0);

/// The track path that `pcm_upnp_start()` most recently sent a SetAVTransportURI
/// for (without Play). The monitor uses this to avoid double-sending Play on
/// normal track changes, while still sending Play as a fallback for any renderer
/// that disconnects after a metadata-only update.
static LAST_PCM_TRACK_PATH: Mutex<String> = Mutex::new(String::new());

pub(crate) struct UpnpConfig {
    pub server_port: u16,
    pub pcm_port: u16,
    pub friendly_name: String,
    pub renderer_url: Option<String>,
    pub sample_rate: u32,
}

static CONFIG: Mutex<UpnpConfig> = Mutex::new(UpnpConfig {
    server_port: 7878,
    pcm_port: 7879,
    friendly_name: String::new(),
    renderer_url: None,
    sample_rate: 44100,
});

pub(crate) fn get_buffer() -> Arc<BroadcastBuffer> {
    BUFFER
        .get_or_init(|| Arc::new(BroadcastBuffer::new()))
        .clone()
}

pub(crate) fn get_runtime() -> &'static tokio::runtime::Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Runtime::new().expect("failed to create UPnP tokio runtime")
    })
}

pub(crate) fn device_uuid() -> &'static str {
    DEVICE_UUID.get_or_init(|| uuid::Uuid::new_v4().to_string())
}

pub(crate) fn get_local_ip() -> std::net::Ipv4Addr {
    if let Ok(socket) = std::net::UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(addr) = socket.local_addr() {
                if let std::net::IpAddr::V4(ip) = addr.ip() {
                    return ip;
                }
            }
        }
    }
    std::net::Ipv4Addr::LOCALHOST
}

// ---------------------------------------------------------------------------
// Public API — UPnP Media Server (ContentDirectory + SSDP)
// ---------------------------------------------------------------------------

/// Pre-initialize the UPnP tokio runtime.  Call this once at server startup,
/// before any HTTP handler runs, to ensure `Runtime::new()` is never called
/// from inside a `block_on` context (tokio 1.27+ panics in that case).
pub fn init() {
    let _ = get_runtime();
}

/// Start the UPnP/DLNA Media Server so control points can browse and stream
/// the music library.  Idempotent.
pub fn start_media_server(port: u16, friendly_name: &str) {
    let mut started = SERVER_STARTED.lock().unwrap();
    if *started {
        return;
    }
    let name = if friendly_name.is_empty() {
        "Rockbox".to_string()
    } else {
        friendly_name.to_string()
    };
    {
        let mut cfg = CONFIG.lock().unwrap();
        cfg.server_port = port;
        cfg.friendly_name = name;
    }
    let _ = device_uuid(); // initialise UUID before spawning threads
    let rt = get_runtime();
    rt.spawn(async move {
        if let Err(e) = server::run(port).await {
            tracing::error!("UPnP HTTP server error: {}", e);
        }
    });
    rt.spawn(async move {
        ssdp::run(port).await;
    });
    *started = true;
    tracing::info!("UPnP media server started on :{port}");
}

/// Start the UPnP/DLNA MediaRenderer:1 so control points can push media to
/// this device.  Idempotent (tracked by the renderer module itself).
pub fn start_renderer(port: u16, friendly_name: &str) {
    renderer::start(port, friendly_name);
}

// ---------------------------------------------------------------------------
// FFI exports — UPnP PCM sink (WAV streaming to UPnP/DLNA renderers)
// Only compiled when the `ffi` feature is enabled (rockbox-cli staticlib).
// ---------------------------------------------------------------------------

#[cfg(feature = "ffi")]
#[no_mangle]
pub extern "C" fn pcm_upnp_set_http_port(port: u16) {
    CONFIG.lock().unwrap().pcm_port = port;
}

#[cfg(feature = "ffi")]
#[no_mangle]
pub extern "C" fn pcm_upnp_set_renderer_url(url: *const c_char) {
    let mut cfg = CONFIG.lock().unwrap();
    if url.is_null() {
        cfg.renderer_url = None;
        return;
    }
    let s = unsafe { CStr::from_ptr(url) }
        .to_str()
        .unwrap_or("")
        .to_string();
    cfg.renderer_url = if s.is_empty() { None } else { Some(s) };
}

#[cfg(feature = "ffi")]
#[no_mangle]
pub extern "C" fn pcm_upnp_set_sample_rate(rate: u32) {
    CONFIG.lock().unwrap().sample_rate = rate;
}

#[cfg(feature = "ffi")]
#[no_mangle]
pub extern "C" fn pcm_upnp_start() -> c_int {
    // --- Start the HTTP broadcast server once ---
    {
        let mut started = PCM_STARTED.lock().unwrap();
        if !*started {
            let buf = get_buffer();
            buf.reset();
            let (port, sample_rate) = {
                let cfg = CONFIG.lock().unwrap();
                (cfg.pcm_port, cfg.sample_rate)
            };
            let buf_http = buf.clone();
            std::thread::spawn(move || pcm_server::serve(port, sample_rate, buf_http));
            *started = true;
            tracing::info!("UPnP PCM sink: WAV stream on :{port}");
        }
    }

    // --- Notify the renderer on initial play only ---
    let (port, sample_rate, renderer_url) = {
        let cfg = CONFIG.lock().unwrap();
        (cfg.pcm_port, cfg.sample_rate, cfg.renderer_url.clone())
    };
    if let Some(url) = renderer_url {
        let need_play = {
            let mut rp = RENDERER_PLAYING.lock().unwrap();
            let was = *rp;
            *rp = true;
            !was
        };

        if need_play {
            ensure_track_monitor(url.clone(), port);
            let local_ip = get_local_ip();
            let stream_url = format!("http://{}:{}/stream.wav", local_ip, port);
            let rt = get_runtime();
            rt.spawn(async move {
                let track = rockbox_sys::playback::current_track();
                // Record path so the monitor knows pcm_upnp_start handled this track.
                if let Some(ref t) = track {
                    *LAST_PCM_TRACK_PATH.lock().unwrap() = t.path.clone();
                }
                // Send SetAVTransportURI + Play immediately without waiting for the
                // album art DB lookup — this cuts 100–500 ms of startup latency.
                if let Err(e) =
                    avtransport_play(&url, &stream_url, track.as_ref(), sample_rate, true, None)
                        .await
                {
                    tracing::warn!("UPnP AVTransport play failed: {}", e);
                    return;
                }
                // Follow up with album art once the DB lookup completes (metadata-
                // only update, no second Play so the renderer stays connected).
                if let Some(ref t) = track {
                    if let Some(art) = get_album_art_url(&t.path, local_ip).await {
                        let _ = avtransport_play(
                            &url,
                            &stream_url,
                            track.as_ref(),
                            sample_rate,
                            false,
                            Some(&art),
                        )
                        .await;
                    }
                }
            });
        } else {
            // Subsequent sink_dma_start: could be a track change OR a resume after pause.
            // Distinguish by comparing the current track path against what we last played.
            let current_path = rockbox_sys::playback::current_track()
                .map(|t| t.path)
                .unwrap_or_default();

            let is_track_change = if current_path.is_empty() {
                false // can't tell yet — treat as resume
            } else {
                let mut last = LAST_PCM_TRACK_PATH.lock().unwrap();
                if current_path != *last {
                    *last = current_path.clone();
                    true
                } else {
                    false // same path → resume from pause
                }
            };

            // If the "new track" is our own WAV stream the renderer is re-ingesting
            // (http://<ip>:<port>/stream.wav), skip SetAVTransportURI entirely.
            // Sending it would overwrite the correct DIDL metadata (duration, album
            // art) stored for the real file with stream.wav's codec values (duration
            // 0, no art), and would create an infinite feedback loop.
            let is_our_stream =
                current_path.starts_with("http://") && current_path.ends_with("/stream.wav");

            if is_track_change && !is_our_stream {
                // Fire SetAVTransportURI *without* Play at the exact PCM boundary.
                // The renderer updates its metadata display and — if it supports
                // metadata-only updates — stays connected to the live /stream.wav
                // with zero gap. The monitor will confirm the update ~500 ms later
                // and send Play only if it detects the renderer has stopped.
                let rt = get_runtime();
                let url_c = url.clone();
                let local_ip = get_local_ip();
                rt.spawn(async move {
                    let track = rockbox_sys::playback::current_track();
                    let stream_url = format!("http://{}:{}/stream.wav", local_ip, port);
                    let art = if let Some(ref t) = track {
                        get_album_art_url(&t.path, local_ip).await
                    } else {
                        None
                    };
                    tracing::info!(
                        "UPnP: track change at PCM boundary → «{}»",
                        track.as_ref().map(|t| t.title.as_str()).unwrap_or("?")
                    );
                    if let Err(e) = avtransport_play(
                        &url_c,
                        &stream_url,
                        track.as_ref(),
                        sample_rate,
                        false, // no Play — keep renderer connected
                        art.as_deref(),
                    )
                    .await
                    {
                        tracing::warn!("UPnP: SetAVTransportURI at PCM boundary failed: {e}");
                    }
                });
            }
            // Resume (same path): do nothing — buffer streams naturally, no gap.
        }
    }
    0
}

#[cfg(feature = "ffi")]
#[no_mangle]
pub extern "C" fn pcm_upnp_write(data: *const u8, len: usize) -> c_int {
    if data.is_null() || len == 0 {
        return 0;
    }
    let slice = unsafe { std::slice::from_raw_parts(data, len) };
    get_buffer().push(slice);
    0
}

#[cfg(feature = "ffi")]
#[no_mangle]
pub extern "C" fn pcm_upnp_stop() {}

#[cfg(feature = "ffi")]
#[no_mangle]
pub extern "C" fn pcm_upnp_close() {
    let mut started = PCM_STARTED.lock().unwrap();
    let mut rp = RENDERER_PLAYING.lock().unwrap();
    get_buffer().close();
    *started = false;
    *rp = false;
}

/// Reset the renderer-side state so the next `pcm_upnp_start()` always sends
/// a fresh SetAVTransportURI + Play to the renderer.  Call this whenever the
/// UPnP sink is selected (e.g. the user connects to a UPnP device from the UI)
/// to make output switch-in work without restarting the daemon.
#[cfg(feature = "ffi")]
#[no_mangle]
pub extern "C" fn pcm_upnp_reset_renderer() {
    // Kill any running track-change monitor — a new one will start with pcm_upnp_start().
    MONITOR_GEN.fetch_add(1, Ordering::SeqCst);
    // Clear the last-track hint so the next start is always treated as "first play".
    *LAST_PCM_TRACK_PATH.lock().unwrap() = String::new();
    // Reset the "renderer already playing" flag so pcm_upnp_start() sends Play.
    *RENDERER_PLAYING.lock().unwrap() = false;
    tracing::debug!("UPnP PCM sink: renderer state reset");
}

// ---------------------------------------------------------------------------
// Album art DB lookup
// ---------------------------------------------------------------------------

async fn get_album_art_url(path: &str, local_ip: std::net::Ipv4Addr) -> Option<String> {
    let pool = db::open_pool().await.ok()?;
    let db_track = db::track_by_path(&pool, path).await.ok()??;
    let filename = db_track.album_art?;
    if filename.is_empty() {
        return None;
    }
    let graphql_port = std::env::var("ROCKBOX_GRAPHQL_PORT").unwrap_or_else(|_| "6062".to_string());
    Some(format!(
        "http://{}:{}/covers/{}",
        local_ip, graphql_port, filename
    ))
}

// ---------------------------------------------------------------------------
// Track-change monitor — sends updated SetAVTransportURI when track changes
// ---------------------------------------------------------------------------

fn ensure_track_monitor(renderer_url: String, port: u16) {
    let monitor_gen = MONITOR_GEN.fetch_add(1, Ordering::SeqCst) + 1;
    let rt = get_runtime();
    rt.spawn(async move {
        let mut last_current_path = String::new();

        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

            if MONITOR_GEN.load(Ordering::SeqCst) != monitor_gen {
                return;
            }
            if !*RENDERER_PLAYING.lock().unwrap() {
                return;
            }

            let sample_rate = CONFIG.lock().unwrap().sample_rate;
            let local_ip = get_local_ip();

            let current_track = rockbox_sys::playback::current_track();
            let current_path = current_track
                .as_ref()
                .map(|t| t.path.clone())
                .unwrap_or_default();

            let is_our_stream =
                current_path.starts_with("http://") && current_path.ends_with("/stream.wav");

            if !current_path.is_empty() && current_path != last_current_path && !is_our_stream {
                let is_first = last_current_path.is_empty();
                last_current_path = current_path.clone();

                // Did pcm_upnp_start() already fire SetAVTransportURI (no Play) at the
                // PCM boundary for this exact track? If so, the renderer is already
                // streaming and we should NOT send Play — that would force a reconnect
                // and reintroduce the gap we're trying to eliminate.
                // If pcm_upnp_start() didn't fire (e.g. natural end-of-track on some
                // Rockbox builds, or initial play), send Play as a fallback so the
                // renderer reconnects if it stopped.
                let pcm_boundary_fired = *LAST_PCM_TRACK_PATH.lock().unwrap() == current_path;

                let send_play = !is_first && !pcm_boundary_fired;

                tracing::info!(
                    "UPnP monitor: {} «{}» (pcm_boundary={pcm_boundary_fired}, play={send_play})",
                    if is_first {
                        "first track"
                    } else {
                        "track changed →"
                    },
                    current_track
                        .as_ref()
                        .map(|t| t.title.as_str())
                        .unwrap_or("?")
                );

                let stream_url = format!("http://{}:{}/stream.wav", local_ip, port);
                let art = get_album_art_url(&current_path, local_ip).await;
                if let Err(e) = avtransport_play(
                    &renderer_url,
                    &stream_url,
                    current_track.as_ref(),
                    sample_rate,
                    send_play,
                    art.as_deref(),
                )
                .await
                {
                    tracing::warn!("UPnP: SetAVTransportURI failed: {e}");
                }
            }
        }
    });
}

// ---------------------------------------------------------------------------
// AVTransport SOAP client — tells a UPnP renderer to play our WAV stream
// ---------------------------------------------------------------------------

async fn avtransport_play(
    control_url: &str,
    stream_url: &str,
    track: Option<&rockbox_sys::types::mp3_entry::Mp3Entry>,
    sample_rate: u32,
    send_play: bool,
    album_art_url: Option<&str>,
) -> anyhow::Result<()> {
    use bytes::Bytes;
    use http_body_util::Full;
    use hyper::Request;
    use hyper_util::client::legacy::Client;
    use hyper_util::rt::TokioExecutor;

    let client: Client<_, Full<Bytes>> = Client::builder(TokioExecutor::new()).build_http();

    let metadata = build_didl_metadata(track, stream_url, sample_rate, album_art_url);
    let metadata_escaped = xml_escape(&metadata);
    tracing::info!("UPnP AVTransport: setting URI to {stream_url}");

    let set_uri_body = format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/"
            s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/">
  <s:Body>
    <u:SetAVTransportURI xmlns:u="urn:schemas-upnp-org:service:AVTransport:1">
      <InstanceID>0</InstanceID>
      <CurrentURI>{stream_url}</CurrentURI>
      <CurrentURIMetaData>{metadata_escaped}</CurrentURIMetaData>
    </u:SetAVTransportURI>
  </s:Body>
</s:Envelope>"#
    );

    let req = Request::builder()
        .method("POST")
        .uri(control_url)
        .header("Content-Type", "text/xml; charset=\"utf-8\"")
        .header(
            "SOAPAction",
            "\"urn:schemas-upnp-org:service:AVTransport:1#SetAVTransportURI\"",
        )
        .header("Content-Length", set_uri_body.len().to_string())
        .body(Full::from(Bytes::from(set_uri_body)))?;

    let resp: hyper::Response<hyper::body::Incoming> = client.request(req).await?;
    if !resp.status().is_success() {
        anyhow::bail!("SetAVTransportURI returned HTTP {}", resp.status().as_u16());
    }
    tracing::debug!("UPnP AVTransport: SetAVTransportURI OK");

    if !send_play {
        return Ok(());
    }

    let play_body = r#"<?xml version="1.0" encoding="utf-8"?>
<s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/"
            s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/">
  <s:Body>
    <u:Play xmlns:u="urn:schemas-upnp-org:service:AVTransport:1">
      <InstanceID>0</InstanceID>
      <Speed>1</Speed>
    </u:Play>
  </s:Body>
</s:Envelope>"#;

    let req = Request::builder()
        .method("POST")
        .uri(control_url)
        .header("Content-Type", "text/xml; charset=\"utf-8\"")
        .header(
            "SOAPAction",
            "\"urn:schemas-upnp-org:service:AVTransport:1#Play\"",
        )
        .header("Content-Length", play_body.len().to_string())
        .body(Full::from(Bytes::from(play_body)))?;

    let resp: hyper::Response<hyper::body::Incoming> = client.request(req).await?;
    if !resp.status().is_success() {
        anyhow::bail!("Play returned HTTP {}", resp.status().as_u16());
    }
    tracing::info!("UPnP AVTransport: renderer playing WAV stream at {stream_url}");
    Ok(())
}

// ---------------------------------------------------------------------------
// DIDL-Lite metadata helpers
// ---------------------------------------------------------------------------

fn ms_to_upnp_duration(ms: u64) -> String {
    let total_secs = ms / 1000;
    let frac_ms = ms % 1000;
    let h = total_secs / 3600;
    let m = (total_secs % 3600) / 60;
    let s = total_secs % 60;
    format!("{}:{:02}:{:02}.{:03}", h, m, s, frac_ms)
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn build_didl_metadata(
    track: Option<&rockbox_sys::types::mp3_entry::Mp3Entry>,
    stream_url: &str,
    sample_rate: u32,
    album_art_url: Option<&str>,
) -> String {
    let (title, artist, album, duration_ms) = match track {
        Some(t) => {
            let title = if t.title.trim().is_empty() {
                t.path
                    .rsplit('/')
                    .next()
                    .and_then(|f| {
                        f.rsplit('.')
                            .nth(1)
                            .map(|_| f.rsplit('.').skip(1).collect::<Vec<_>>().join("."))
                    })
                    .unwrap_or_else(|| t.path.clone())
            } else {
                t.title.clone()
            };
            (title, t.artist.clone(), t.album.clone(), t.length)
        }
        None => ("stream.wav".to_string(), String::new(), String::new(), 0u64),
    };

    let duration_str = if duration_ms > 0 {
        ms_to_upnp_duration(duration_ms)
    } else {
        "0:00:00.000".to_string()
    };

    let t = xml_escape(&title);
    let ar = xml_escape(&artist);
    let al = xml_escape(&album);
    let url_esc = xml_escape(stream_url);

    let artist_elem = if ar.is_empty() {
        String::new()
    } else {
        format!("\n    <upnp:artist>{ar}</upnp:artist>")
    };
    let album_elem = if al.is_empty() {
        String::new()
    } else {
        format!("\n    <upnp:album>{al}</upnp:album>")
    };
    let art_elem = match album_art_url.filter(|s| !s.is_empty()) {
        Some(url) => {
            let art_url = xml_escape(url);
            format!("\n    <upnp:albumArtURI>{art_url}</upnp:albumArtURI>")
        }
        None => String::new(),
    };

    format!(
        r#"<DIDL-Lite xmlns="urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:upnp="urn:schemas-upnp-org:metadata-1-0/upnp/">
  <item id="1" parentID="0" restricted="1">
    <dc:title>{t}</dc:title>{artist_elem}{album_elem}{art_elem}
    <upnp:class>object.item.audioItem.musicTrack</upnp:class>
    <res protocolInfo="http-get:*:audio/wav:DLNA.ORG_PN=LPCM;DLNA.ORG_FLAGS=01700000000000000000000000000000" duration="{duration_str}" sampleFrequency="{sample_rate}" nrAudioChannels="2" bitsPerSample="16">{url_esc}</res>
  </item>
</DIDL-Lite>"#
    )
}
