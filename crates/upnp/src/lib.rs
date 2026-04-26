pub mod db;
pub(crate) mod didl;
pub mod format;
pub(crate) mod pcm_server;
pub mod renderer;
pub mod server;
pub(crate) mod ssdp;

// Called from rockbox-cli to force this crate's symbols into librockbox_cli.a
#[doc(hidden)]
pub fn _link_upnp() {}

use std::collections::VecDeque;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
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
static SERVER_STARTED: Mutex<bool> = Mutex::new(false);
static DEVICE_UUID: OnceLock<String> = OnceLock::new();
static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

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
// ---------------------------------------------------------------------------

/// Set the HTTP port for the WAV PCM stream (default: 7879).
#[no_mangle]
pub extern "C" fn pcm_upnp_set_http_port(port: u16) {
    CONFIG.lock().unwrap().pcm_port = port;
}

/// Set the AVTransport control URL of a UPnP renderer to auto-command on start.
/// Pass NULL to clear.
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

/// Inform the PCM sink of the current sample rate (called by set_freq).
#[no_mangle]
pub extern "C" fn pcm_upnp_set_sample_rate(rate: u32) {
    CONFIG.lock().unwrap().sample_rate = rate;
}

/// Start the WAV PCM stream HTTP server.  Idempotent.
#[no_mangle]
pub extern "C" fn pcm_upnp_start() -> c_int {
    let mut started = PCM_STARTED.lock().unwrap();
    if *started {
        return 0;
    }
    let buf = get_buffer();
    buf.reset();
    let (port, sample_rate, renderer_url) = {
        let cfg = CONFIG.lock().unwrap();
        (cfg.pcm_port, cfg.sample_rate, cfg.renderer_url.clone())
    };
    let buf_http = buf.clone();
    std::thread::spawn(move || pcm_server::serve(port, sample_rate, buf_http));
    if let Some(url) = renderer_url {
        let local_ip = get_local_ip();
        let rt = get_runtime();
        rt.spawn(async move {
            let stream_url = format!("http://{}:{}/stream.wav", local_ip, port);
            if let Err(e) = avtransport_play(&url, &stream_url).await {
                tracing::warn!("UPnP AVTransport play failed: {}", e);
            }
        });
    }
    *started = true;
    tracing::info!("UPnP PCM sink: WAV stream on :{port}");
    0
}

/// Push raw S16LE stereo PCM into the WAV broadcast buffer.
#[no_mangle]
pub extern "C" fn pcm_upnp_write(data: *const u8, len: usize) -> c_int {
    if data.is_null() || len == 0 {
        return 0;
    }
    let slice = unsafe { std::slice::from_raw_parts(data, len) };
    get_buffer().push(slice);
    0
}

/// No-op between tracks — HTTP connections stay alive.
#[no_mangle]
pub extern "C" fn pcm_upnp_stop() {}

/// Shut down the PCM stream server (called on daemon exit).
#[no_mangle]
pub extern "C" fn pcm_upnp_close() {
    let mut started = PCM_STARTED.lock().unwrap();
    get_buffer().close();
    *started = false;
}

// ---------------------------------------------------------------------------
// AVTransport SOAP client — tells a UPnP renderer to play our WAV stream
// ---------------------------------------------------------------------------

async fn avtransport_play(control_url: &str, stream_url: &str) -> anyhow::Result<()> {
    use bytes::Bytes;
    use http_body_util::Full;
    use hyper::Request;
    use hyper_util::client::legacy::Client;
    use hyper_util::rt::TokioExecutor;

    let client: Client<_, Full<Bytes>> = Client::builder(TokioExecutor::new()).build_http();

    let set_uri_body = format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/"
            s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/">
  <s:Body>
    <u:SetAVTransportURI xmlns:u="urn:schemas-upnp-org:service:AVTransport:1">
      <InstanceID>0</InstanceID>
      <CurrentURI>{stream_url}</CurrentURI>
      <CurrentURIMetaData></CurrentURIMetaData>
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
