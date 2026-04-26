pub mod api {
    #[path = ""]
    pub mod rockbox {
        #[path = "rockbox.v1alpha1.rs"]
        pub mod v1alpha1;
    }
}

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
                let album_art_url = if let Some(ref t) = track {
                    get_album_art_url(&t.path, local_ip).await
                } else {
                    None
                };
                if let Err(e) = avtransport_play(
                    &url,
                    &stream_url,
                    track.as_ref(),
                    sample_rate,
                    true,
                    album_art_url.as_deref(),
                )
                .await
                {
                    tracing::warn!("UPnP AVTransport play failed: {}", e);
                }
            });
        }
        // On subsequent sink_dma_start calls (track change or resume after pause),
        // the monitor handles everything: it detects the new current_track() path
        // and sends SetAVTransportURI+Play with fresh metadata. No action needed here.
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

            if !current_path.is_empty() && current_path != last_current_path {
                let is_first = last_current_path.is_empty();
                last_current_path = current_path.clone();

                // Send SetAVTransportURI+Play on every track detection — including the
                // first one — so the renderer always has up-to-date metadata and the
                // stream URL is always correct. For the very first track this also
                // corrects any stale metadata from the initial async play call.
                if !is_first {
                    tracing::info!(
                        "UPnP monitor: track changed → «{}»",
                        current_track
                            .as_ref()
                            .map(|t| t.title.as_str())
                            .unwrap_or("?")
                    );
                } else {
                    tracing::info!(
                        "UPnP monitor: first track «{}»",
                        current_track
                            .as_ref()
                            .map(|t| t.title.as_str())
                            .unwrap_or("?")
                    );
                }

                let stream_url = format!("http://{}:{}/stream.wav", local_ip, port);
                let art = get_album_art_url(&current_path, local_ip).await;
                if let Err(e) = avtransport_play(
                    &renderer_url,
                    &stream_url,
                    current_track.as_ref(),
                    sample_rate,
                    // Only send Play on a real track change (not the first detection)
                    // because the renderer is already playing from the initial start.
                    // For the first track, re-sending Play causes an audible blip.
                    !is_first,
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
