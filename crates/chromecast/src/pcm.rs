// Chromecast PCM sink — streams WAV over HTTP and drives playback via Cast protocol.
//
// Architecture:
//   1. An HTTP server serves /stream.wav (live WAV from a broadcast buffer) and
//      /now-playing/art (album art from the track's directory).
//   2. A Cast thread connects to the Chromecast device, launches the Default Media
//      Receiver app, and tells it to load http://{local_ip}:{port}/stream.wav.
//   3. A monitor loop in the Cast thread detects track changes every 500 ms and
//      reloads the media with fresh title/artist/album/art metadata.
//
// FFI surface (called from firmware/target/hosted/pcm-chromecast.c):
//   pcm_chromecast_set_http_port(u16)
//   pcm_chromecast_set_device_host(*const c_char)
//   pcm_chromecast_set_device_port(u16)
//   pcm_chromecast_set_sample_rate(u32)
//   pcm_chromecast_start() -> c_int          (0 = ok, <0 = error)
//   pcm_chromecast_write(*const u8, usize) -> c_int
//   pcm_chromecast_stop()
//   pcm_chromecast_close()

use std::collections::VecDeque;
use std::io::Write as _;
use std::net::TcpListener;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::thread;
use std::time::Duration;

#[cfg(feature = "ffi")]
use std::ffi::CStr;
#[cfg(feature = "ffi")]
use std::os::raw::{c_char, c_int};

use chromecast::{
    channels::{
        media::{Image, Media, Metadata, MusicTrackMediaMetadata, StreamType},
        receiver::CastDeviceApp,
    },
    CastDevice,
};

// ---------------------------------------------------------------------------
// Broadcast buffer — one writer, N independent readers.
// ---------------------------------------------------------------------------

enum RecvResult {
    Data(Vec<u8>),
    Closed,
}

struct BroadcastBuffer {
    inner: Mutex<BroadcastInner>,
    condvar: Condvar,
}

struct BroadcastInner {
    chunks: VecDeque<(u64, Vec<u8>)>,
    next_seq: u64,
    total_bytes: usize,
    closed: bool,
}

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

    fn push(&self, data: &[u8]) {
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

    fn subscribe(self: &Arc<Self>) -> BroadcastReceiver {
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
    }

    fn close(&self) {
        let mut g = self.inner.lock().unwrap();
        g.closed = true;
        self.condvar.notify_all();
    }
}

struct BroadcastReceiver {
    buf: Arc<BroadcastBuffer>,
    next_seq: u64,
}

impl BroadcastReceiver {
    fn recv_blocking(&mut self) -> RecvResult {
        let mut g = self.buf.inner.lock().unwrap();
        loop {
            if g.closed {
                return RecvResult::Closed;
            }
            if let Some(&(front_seq, _)) = g.chunks.front() {
                if self.next_seq < front_seq {
                    tracing::debug!(
                        "chromecast/pcm: receiver lagging, skipping {} → {}",
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
static CAST_PLAYING: Mutex<bool> = Mutex::new(false);
static CAST_STOP: AtomicBool = AtomicBool::new(false);

struct ChromecastPcmConfig {
    device_host: String,
    device_port: u16,
    http_port: u16,
    sample_rate: u32,
}

static CONFIG: Mutex<ChromecastPcmConfig> = Mutex::new(ChromecastPcmConfig {
    device_host: String::new(),
    device_port: 8009,
    http_port: 7881,
    sample_rate: 44100,
});

fn get_buffer() -> Arc<BroadcastBuffer> {
    BUFFER
        .get_or_init(|| Arc::new(BroadcastBuffer::new()))
        .clone()
}

fn get_local_ip() -> std::net::Ipv4Addr {
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
// Minimal HTTP server — WAV stream + album art endpoint
// ---------------------------------------------------------------------------

fn wav_header(sample_rate: u32) -> [u8; 44] {
    let channels: u32 = 2;
    let bits: u32 = 16;
    let byte_rate = sample_rate * channels * bits / 8;
    let block_align = (channels * bits / 8) as u16;
    let mut h = [0u8; 44];
    h[0..4].copy_from_slice(b"RIFF");
    h[4..8].copy_from_slice(&0xFFFF_FFFFu32.to_le_bytes());
    h[8..12].copy_from_slice(b"WAVE");
    h[12..16].copy_from_slice(b"fmt ");
    h[16..20].copy_from_slice(&16u32.to_le_bytes());
    h[20..22].copy_from_slice(&1u16.to_le_bytes());
    h[22..24].copy_from_slice(&(channels as u16).to_le_bytes());
    h[24..28].copy_from_slice(&sample_rate.to_le_bytes());
    h[28..32].copy_from_slice(&byte_rate.to_le_bytes());
    h[32..34].copy_from_slice(&block_align.to_le_bytes());
    h[34..36].copy_from_slice(&(bits as u16).to_le_bytes());
    h[36..40].copy_from_slice(b"data");
    h[40..44].copy_from_slice(&0xFFFF_FFFFu32.to_le_bytes());
    h
}

fn find_album_art(track_path: &str) -> Option<(Vec<u8>, &'static str)> {
    let dir = Path::new(track_path).parent()?;
    const CANDIDATES: &[(&str, &'static str)] = &[
        ("cover.jpg", "image/jpeg"),
        ("cover.jpeg", "image/jpeg"),
        ("cover.png", "image/png"),
        ("cover.webp", "image/webp"),
        ("folder.jpg", "image/jpeg"),
        ("folder.jpeg", "image/jpeg"),
        ("folder.png", "image/png"),
        ("album.jpg", "image/jpeg"),
        ("album.png", "image/png"),
        ("front.jpg", "image/jpeg"),
        ("front.jpeg", "image/jpeg"),
        ("front.png", "image/png"),
        ("artwork.jpg", "image/jpeg"),
        ("artwork.png", "image/png"),
        ("AlbumArt.jpg", "image/jpeg"),
        ("AlbumArt.jpeg", "image/jpeg"),
        ("AlbumArt.png", "image/png"),
    ];
    for (name, mime) in CANDIDATES {
        let p = dir.join(name);
        if let Ok(data) = std::fs::read(&p) {
            return Some((data, mime));
        }
    }
    None
}

fn parse_request_path(stream: &mut std::net::TcpStream) -> std::io::Result<String> {
    use std::io::Read;
    let mut buf: Vec<u8> = Vec::with_capacity(1024);
    let mut byte = [0u8; 1];
    loop {
        stream.read_exact(&mut byte)?;
        buf.push(byte[0]);
        if buf.ends_with(b"\r\n\r\n") || buf.ends_with(b"\n\n") {
            break;
        }
        if buf.len() > 8192 {
            break;
        }
    }
    let raw = String::from_utf8_lossy(&buf);
    let path = raw
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .unwrap_or("/")
        .to_string();
    Ok(path)
}

pub(crate) fn serve_http(port: u16, sample_rate: u32, buf: Arc<BroadcastBuffer>) {
    let listener = match TcpListener::bind(("0.0.0.0", port)) {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("chromecast/pcm: bind :{port} failed: {e}");
            return;
        }
    };
    tracing::info!("chromecast/pcm: WAV stream on :{port}");

    for stream in listener.incoming() {
        match stream {
            Ok(mut tcp) => {
                let buf = buf.clone();
                thread::spawn(move || {
                    let peer = tcp.peer_addr().map(|a| a.to_string()).unwrap_or_default();
                    let path = match parse_request_path(&mut tcp) {
                        Ok(p) => p,
                        Err(e) => {
                            tracing::warn!("chromecast/pcm: request read error from {peer}: {e}");
                            return;
                        }
                    };

                    match path.as_str() {
                        "/now-playing/art" | "/now-playing/art.jpg" | "/now-playing/art.png" => {
                            serve_art(&mut tcp);
                        }
                        _ => {
                            serve_wav(&mut tcp, sample_rate, buf, &peer);
                        }
                    }
                });
            }
            Err(e) => tracing::warn!("chromecast/pcm: accept error: {e}"),
        }
    }
}

fn serve_art(stream: &mut std::net::TcpStream) {
    let art = rockbox_sys::playback::current_track()
        .filter(|t| !t.path.is_empty())
        .and_then(|t| find_album_art(&t.path));

    match art {
        Some((data, mime)) => {
            let hdr = format!(
                "HTTP/1.0 200 OK\r\nContent-Type: {mime}\r\nContent-Length: {}\r\nCache-Control: no-cache\r\n\r\n",
                data.len()
            );
            let _ = stream.write_all(hdr.as_bytes());
            let _ = stream.write_all(&data);
        }
        None => {
            let _ = stream.write_all(b"HTTP/1.0 404 Not Found\r\nContent-Length: 0\r\n\r\n");
        }
    }
}

fn serve_wav(
    stream: &mut std::net::TcpStream,
    sample_rate: u32,
    buf: Arc<BroadcastBuffer>,
    peer: &str,
) {
    let hdr = "HTTP/1.0 200 OK\r\nContent-Type: audio/wav\r\nCache-Control: no-cache\r\n\r\n";
    let wav_hdr = wav_header(sample_rate);
    if stream.write_all(hdr.as_bytes()).is_err() || stream.write_all(&wav_hdr).is_err() {
        return;
    }
    tracing::info!("chromecast/pcm: streaming WAV to {peer}");

    let mut rx = buf.subscribe();
    loop {
        match rx.recv_blocking() {
            RecvResult::Data(chunk) => {
                if stream.write_all(&chunk).is_err() {
                    tracing::debug!("chromecast/pcm: {peer} disconnected");
                    break;
                }
            }
            RecvResult::Closed => break,
        }
    }
}

// ---------------------------------------------------------------------------
// Cast protocol thread — connects, loads stream, monitors track changes
// ---------------------------------------------------------------------------

fn build_media(stream_url: &str, art_url: &str) -> Media {
    let track = rockbox_sys::playback::current_track();
    let (title, artist, album) = match &track {
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
            (title, t.artist.clone(), t.album.clone())
        }
        None => ("Live Stream".to_string(), String::new(), String::new()),
    };

    let images = if art_url.is_empty() {
        vec![]
    } else {
        vec![Image {
            url: art_url.to_string(),
            dimensions: None,
        }]
    };

    Media {
        content_id: stream_url.to_string(),
        content_type: "audio/wav".to_string(),
        stream_type: StreamType::Live,
        duration: None,
        metadata: Some(Metadata::MusicTrack(MusicTrackMediaMetadata {
            title: Some(title),
            artist: Some(artist),
            album_name: Some(album),
            album_artist: None,
            track_number: None,
            disc_number: None,
            images,
            release_date: None,
            composer: None,
        })),
    }
}

fn cast_session(host: &str, device_port: u16, stream_url: &str, art_url: &str) -> bool {
    let cast_device = match CastDevice::connect_without_host_verification(host, device_port) {
        Ok(d) => d,
        Err(e) => {
            tracing::error!(
                "chromecast/pcm: connect to {}:{} failed: {}",
                host,
                device_port,
                e
            );
            return false;
        }
    };

    if cast_device
        .connection
        .connect("receiver-0".to_string())
        .is_err()
    {
        tracing::error!("chromecast/pcm: connection.connect failed");
        return false;
    }
    if cast_device.heartbeat.ping().is_err() {
        tracing::error!("chromecast/pcm: initial ping failed");
        return false;
    }

    let app = match cast_device
        .receiver
        .launch_app(&CastDeviceApp::DefaultMediaReceiver)
    {
        Ok(app) => app,
        Err(e) => {
            tracing::error!("chromecast/pcm: launch_app failed: {}", e);
            return false;
        }
    };

    if cast_device
        .connection
        .connect(app.transport_id.as_str())
        .is_err()
    {
        tracing::error!("chromecast/pcm: connect to app transport failed");
        return false;
    }

    let media = build_media(stream_url, art_url);
    let track_title = media
        .metadata
        .as_ref()
        .and_then(|m| match m {
            Metadata::MusicTrack(m) => m.title.clone(),
            _ => None,
        })
        .unwrap_or_default();

    if let Err(e) = cast_device
        .media
        .load(app.transport_id.as_str(), "", &media)
    {
        tracing::warn!("chromecast/pcm: media.load failed: {}", e);
        return false;
    }
    tracing::info!(
        "chromecast/pcm: playing «{}» on {}:{}",
        track_title,
        host,
        device_port
    );

    let mut last_track_path = rockbox_sys::playback::current_track()
        .map(|t| t.path)
        .unwrap_or_default();

    // Monitor loop: heartbeat + track-change metadata updates
    loop {
        if CAST_STOP.load(Ordering::SeqCst) {
            let _ = cast_device.receiver.stop_app(app.session_id.as_str());
            return true;
        }

        thread::sleep(Duration::from_millis(500));

        if cast_device.heartbeat.ping().is_err() {
            tracing::warn!("chromecast/pcm: heartbeat lost, reconnecting");
            return false; // caller will retry
        }

        let current = rockbox_sys::playback::current_track();
        let current_path = current.as_ref().map(|t| t.path.clone()).unwrap_or_default();

        if !current_path.is_empty() && current_path != last_track_path {
            last_track_path = current_path;
            let updated = build_media(stream_url, art_url);
            let new_title = updated
                .metadata
                .as_ref()
                .and_then(|m| match m {
                    Metadata::MusicTrack(m) => m.title.clone(),
                    _ => None,
                })
                .unwrap_or_default();

            // Reload with new metadata; brief HTTP reconnect is acceptable
            if let Err(e) = cast_device
                .media
                .load(app.transport_id.as_str(), "", &updated)
            {
                tracing::warn!("chromecast/pcm: track reload failed: {}, reconnecting", e);
                return false;
            }
            tracing::info!("chromecast/pcm: track change → «{}»", new_title);
        }
    }
}

fn cast_loop(host: String, device_port: u16, http_port: u16) {
    let local_ip = get_local_ip();
    let stream_url = format!("http://{}:{}/stream.wav", local_ip, http_port);
    let art_url = format!("http://{}:{}/now-playing/art", local_ip, http_port);

    while !CAST_STOP.load(Ordering::SeqCst) {
        let ok = cast_session(&host, device_port, &stream_url, &art_url);
        if ok || CAST_STOP.load(Ordering::SeqCst) {
            break;
        }
        // Brief pause before reconnect attempt
        thread::sleep(Duration::from_secs(3));
    }

    *CAST_PLAYING.lock().unwrap() = false;
}

// ---------------------------------------------------------------------------
// FFI exports
// ---------------------------------------------------------------------------

#[cfg(feature = "ffi")]
#[no_mangle]
pub extern "C" fn pcm_chromecast_set_http_port(port: u16) {
    CONFIG.lock().unwrap().http_port = port;
}

#[cfg(feature = "ffi")]
#[no_mangle]
pub extern "C" fn pcm_chromecast_set_device_host(host: *const c_char) {
    if host.is_null() {
        CONFIG.lock().unwrap().device_host = String::new();
        return;
    }
    let s = unsafe { CStr::from_ptr(host) }
        .to_str()
        .unwrap_or("")
        .to_string();
    CONFIG.lock().unwrap().device_host = s;
}

#[cfg(feature = "ffi")]
#[no_mangle]
pub extern "C" fn pcm_chromecast_set_device_port(port: u16) {
    CONFIG.lock().unwrap().device_port = port;
}

#[cfg(feature = "ffi")]
#[no_mangle]
pub extern "C" fn pcm_chromecast_set_sample_rate(rate: u32) {
    CONFIG.lock().unwrap().sample_rate = rate;
}

#[cfg(feature = "ffi")]
#[no_mangle]
pub extern "C" fn pcm_chromecast_start() -> c_int {
    // Start the HTTP broadcast server once
    {
        let mut started = PCM_STARTED.lock().unwrap();
        if !*started {
            let buf = get_buffer();
            buf.reset();
            let (http_port, sample_rate) = {
                let cfg = CONFIG.lock().unwrap();
                (cfg.http_port, cfg.sample_rate)
            };
            let buf_http = buf.clone();
            thread::spawn(move || serve_http(http_port, sample_rate, buf_http));
            *started = true;
            tracing::info!("chromecast/pcm: WAV stream started on :{http_port}");
        }
    }

    // Spawn the Cast protocol thread if not already running
    let already_playing = {
        let mut p = CAST_PLAYING.lock().unwrap();
        let was = *p;
        if !was {
            *p = true;
        }
        was
    };

    if !already_playing {
        CAST_STOP.store(false, Ordering::SeqCst);
        let (host, device_port, http_port) = {
            let cfg = CONFIG.lock().unwrap();
            (cfg.device_host.clone(), cfg.device_port, cfg.http_port)
        };
        if host.is_empty() {
            tracing::warn!(
                "chromecast/pcm: no device host configured — WAV stream active but Cast disabled"
            );
            *CAST_PLAYING.lock().unwrap() = false;
        } else {
            thread::spawn(move || cast_loop(host, device_port, http_port));
        }
    }

    0
}

#[cfg(feature = "ffi")]
#[no_mangle]
pub extern "C" fn pcm_chromecast_write(data: *const u8, len: usize) -> c_int {
    if data.is_null() || len == 0 {
        return 0;
    }
    let slice = unsafe { std::slice::from_raw_parts(data, len) };
    get_buffer().push(slice);
    0
}

#[cfg(feature = "ffi")]
#[no_mangle]
pub extern "C" fn pcm_chromecast_stop() {
    CAST_STOP.store(true, Ordering::SeqCst);
}

#[cfg(feature = "ffi")]
#[no_mangle]
pub extern "C" fn pcm_chromecast_close() {
    CAST_STOP.store(true, Ordering::SeqCst);
    let mut started = PCM_STARTED.lock().unwrap();
    get_buffer().close();
    *started = false;
    *CAST_PLAYING.lock().unwrap() = false;
}
