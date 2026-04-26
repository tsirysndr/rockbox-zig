use crate::{BroadcastBuffer, RecvResult};
use std::io::Write;
use std::net::TcpListener;
use std::path::Path;
use std::sync::Arc;

// ICY metadata is inserted every ICY_METAINT audio bytes.
// 16 384 bytes ≈ 93 ms of S16LE stereo @ 44100 Hz.
const ICY_METAINT: usize = 16384;

// ─── WAV header ──────────────────────────────────────────────────────────────

fn wav_header(sample_rate: u32) -> [u8; 44] {
    let channels: u32 = 2;
    let bits: u32 = 16;
    let byte_rate = sample_rate * channels * bits / 8;
    let block_align = (channels * bits / 8) as u16;
    let mut h = [0u8; 44];
    h[0..4].copy_from_slice(b"RIFF");
    h[4..8].copy_from_slice(&0xFFFF_FFFFu32.to_le_bytes()); // streaming size
    h[8..12].copy_from_slice(b"WAVE");
    h[12..16].copy_from_slice(b"fmt ");
    h[16..20].copy_from_slice(&16u32.to_le_bytes());
    h[20..22].copy_from_slice(&1u16.to_le_bytes()); // PCM
    h[22..24].copy_from_slice(&(channels as u16).to_le_bytes());
    h[24..28].copy_from_slice(&sample_rate.to_le_bytes());
    h[28..32].copy_from_slice(&byte_rate.to_le_bytes());
    h[32..34].copy_from_slice(&block_align.to_le_bytes());
    h[34..36].copy_from_slice(&(bits as u16).to_le_bytes());
    h[36..40].copy_from_slice(b"data");
    h[40..44].copy_from_slice(&0xFFFF_FFFFu32.to_le_bytes()); // streaming size
    h
}

// ─── Track metadata ──────────────────────────────────────────────────────────

struct TrackMeta {
    title: String,
    artist: String,
    album: String,
    albumartist: String,
    year: i32,
    genre: String,
    tracknum: i32,
    path: String,
    elapsed_ms: u64,
    duration_ms: u64,
}

impl TrackMeta {
    fn from_current() -> Option<Self> {
        rockbox_sys::playback::current_track().map(|t| Self {
            title: t.title,
            artist: t.artist,
            album: t.album,
            albumartist: t.albumartist,
            year: t.year,
            genre: t.genre_string,
            tracknum: t.tracknum,
            path: t.path,
            elapsed_ms: t.elapsed,
            duration_ms: t.length,
        })
    }

    fn display_title(&self) -> String {
        match (!self.artist.is_empty(), !self.title.is_empty()) {
            (true, true) => format!("{} - {}", self.artist, self.title),
            (false, true) => self.title.clone(),
            (true, false) => self.artist.clone(),
            (false, false) => String::new(),
        }
    }

    /// Key used to detect track changes (ignores playback position).
    fn track_key(&self) -> String {
        format!("{}\x00{}\x00{}", self.artist, self.title, self.path)
    }
}

// ─── ICY metadata ────────────────────────────────────────────────────────────

/// Build a full ICY metadata block with all available track fields.
///
/// Wire format: [1-byte length_in_16s][length * 16 bytes: Key='Val'; pairs, zero-padded]
/// A single zero byte is an empty/no-update block.
///
/// Players that understand extended ICY (VLC, foobar2000, many DLNA renderers) will
/// display artist, album, year, genre and fetch album art from `art_url`.
fn icy_block(meta: &TrackMeta, art_url: &str) -> Vec<u8> {
    if meta.title.is_empty() && meta.artist.is_empty() {
        return vec![0];
    }

    let mut parts = Vec::<String>::new();

    // Standard field — always present when there is something to show.
    parts.push(format!(
        "StreamTitle='{}';",
        icy_escape(&meta.display_title())
    ));

    if !meta.artist.is_empty() {
        parts.push(format!("StreamArtist='{}';", icy_escape(&meta.artist)));
    }
    if !meta.album.is_empty() {
        parts.push(format!("StreamAlbum='{}';", icy_escape(&meta.album)));
    }
    // Only include albumartist when it differs from track artist to save space.
    if !meta.albumartist.is_empty() && meta.albumartist != meta.artist {
        parts.push(format!(
            "StreamAlbumArtist='{}';",
            icy_escape(&meta.albumartist)
        ));
    }
    if meta.year > 0 {
        parts.push(format!("StreamYear='{}';", meta.year));
    }
    if !meta.genre.is_empty() {
        parts.push(format!("StreamGenre='{}';", icy_escape(&meta.genre)));
    }
    if meta.tracknum > 0 {
        parts.push(format!("StreamTrackNum='{}';", meta.tracknum));
    }
    // StreamUrl is the standard ICY field many players use to fetch artwork.
    if !art_url.is_empty() {
        parts.push(format!("StreamUrl='{}';", art_url));
    }

    let s = parts.concat();
    let b = s.as_bytes();
    let chunks = (b.len() + 15) / 16;
    let mut out = vec![0u8; 1 + chunks * 16];
    out[0] = chunks as u8;
    out[1..1 + b.len()].copy_from_slice(b);
    out
}

/// Replace characters that break ICY `Key='Value';` parsing.
fn icy_escape(s: &str) -> String {
    s.replace('\'', "\u{2019}") // right single quotation mark
        .replace(';', ",")
}

// ─── Album art ───────────────────────────────────────────────────────────────

/// Search the track's parent directory for common cover art filenames.
/// Returns `(image_bytes, mime_type)` for the first match found.
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
            tracing::debug!("upnp/pcm: album art → {}", p.display());
            return Some((data, mime));
        }
    }
    None
}

// ─── JSON helpers ────────────────────────────────────────────────────────────

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = std::fmt::Write::write_fmt(&mut out, format_args!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn build_now_playing_json(meta: &TrackMeta, art_url: &str) -> String {
    format!(
        concat!(
            r#"{{"title":{title},"artist":{artist},"album":{album},"albumartist":{albumartist},"#,
            r#""year":{year},"genre":{genre},"tracknum":{tracknum},"#,
            r#""duration_ms":{duration},"elapsed_ms":{elapsed},"art_url":{art_url}}}"#,
        ),
        title = json_escape(&meta.title),
        artist = json_escape(&meta.artist),
        album = json_escape(&meta.album),
        albumartist = json_escape(&meta.albumartist),
        year = meta.year,
        genre = json_escape(&meta.genre),
        tracknum = meta.tracknum,
        duration = meta.duration_ms,
        elapsed = meta.elapsed_ms,
        art_url = json_escape(art_url),
    )
}

// ─── HTTP request parsing ─────────────────────────────────────────────────────

struct Request {
    /// URL path, e.g. "/stream.wav" or "/now-playing/art"
    path: String,
    wants_icy: bool,
}

fn parse_request(stream: &mut std::net::TcpStream) -> std::io::Result<Request> {
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

    // First line: "GET /path HTTP/1.x"
    let path = raw
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .unwrap_or("/")
        .to_string();

    let wants_icy = raw.lines().any(|l| {
        let l = l.to_ascii_lowercase();
        l.strip_prefix("icy-metadata:")
            .map(|v| v.trim() == "1")
            .unwrap_or(false)
    });

    Ok(Request { path, wants_icy })
}

// ─── Endpoint helpers ─────────────────────────────────────────────────────────

fn art_base_url(local_ip: std::net::Ipv4Addr, port: u16) -> String {
    format!("http://{}:{}/now-playing/art", local_ip, port)
}

fn send_404(stream: &mut std::net::TcpStream) {
    let _ = stream.write_all(b"HTTP/1.0 404 Not Found\r\nContent-Length: 0\r\n\r\n");
}

fn serve_now_playing_json(
    stream: &mut std::net::TcpStream,
    local_ip: std::net::Ipv4Addr,
    port: u16,
) {
    let art_url = art_base_url(local_ip, port);
    let body = match TrackMeta::from_current() {
        Some(meta) => build_now_playing_json(&meta, &art_url),
        None => r#"{"error":"no track playing"}"#.to_string(),
    };
    let hdr = format!(
        "HTTP/1.0 200 OK\r\n\
         Content-Type: application/json\r\n\
         Access-Control-Allow-Origin: *\r\n\
         Cache-Control: no-cache\r\n\
         Content-Length: {}\r\n\
         \r\n",
        body.len()
    );
    let _ = stream.write_all(hdr.as_bytes());
    let _ = stream.write_all(body.as_bytes());
}

fn serve_now_playing_art(stream: &mut std::net::TcpStream) {
    let art = TrackMeta::from_current()
        .filter(|m| !m.path.is_empty())
        .and_then(|m| find_album_art(&m.path));

    match art {
        Some((data, mime)) => {
            let hdr = format!(
                "HTTP/1.0 200 OK\r\n\
                 Content-Type: {mime}\r\n\
                 Content-Length: {}\r\n\
                 Cache-Control: no-cache\r\n\
                 Access-Control-Allow-Origin: *\r\n\
                 \r\n",
                data.len()
            );
            let _ = stream.write_all(hdr.as_bytes());
            let _ = stream.write_all(&data);
        }
        None => send_404(stream),
    }
}

// ─── WAV stream with ICY metadata ────────────────────────────────────────────

fn serve_wav_stream(
    stream: &mut std::net::TcpStream,
    req: &Request,
    sample_rate: u32,
    buf: Arc<BroadcastBuffer>,
    port: u16,
    peer: &str,
) {
    let wav_hdr = wav_header(sample_rate);
    let local_ip = crate::get_local_ip();

    let http_hdr = if req.wants_icy {
        format!(
            "HTTP/1.0 200 OK\r\n\
             Content-Type: audio/wav\r\n\
             Cache-Control: no-cache\r\n\
             icy-metaint: {ICY_METAINT}\r\n\
             icy-name: Rockbox\r\n\
             TransferMode.DLNA.ORG: Streaming\r\n\
             Content-Features.DLNA.ORG: DLNA.ORG_OP=00;DLNA.ORG_CI=0\r\n\
             \r\n"
        )
    } else {
        "HTTP/1.0 200 OK\r\n\
         Content-Type: audio/wav\r\n\
         Cache-Control: no-cache\r\n\
         TransferMode.DLNA.ORG: Streaming\r\n\
         Content-Features.DLNA.ORG: DLNA.ORG_OP=01;DLNA.ORG_CI=0\r\n\
         \r\n"
            .to_string()
    };

    if stream.write_all(http_hdr.as_bytes()).is_err() || stream.write_all(&wav_hdr).is_err() {
        tracing::warn!("upnp/pcm: header write error to {peer}");
        return;
    }

    tracing::info!(
        "upnp/pcm: streaming WAV{} to {peer}",
        if req.wants_icy { " (ICY metadata)" } else { "" }
    );

    let mut rx = buf.subscribe();

    if req.wants_icy {
        let art_url = art_base_url(local_ip, port);
        // The WAV header bytes already occupy the start of the body, so the first
        // ICY boundary arrives after (ICY_METAINT - 44) bytes of PCM data.
        let mut bytes_since_meta: usize = wav_hdr.len();
        let mut last_track_key = String::new();

        loop {
            match rx.recv_blocking() {
                RecvResult::Data(chunk) => {
                    if write_icy_chunk(
                        stream,
                        &chunk,
                        &mut bytes_since_meta,
                        &mut last_track_key,
                        &art_url,
                    )
                    .is_err()
                    {
                        tracing::debug!("upnp/pcm: {peer} disconnected");
                        break;
                    }
                }
                RecvResult::Closed => break,
            }
        }
    } else {
        loop {
            match rx.recv_blocking() {
                RecvResult::Data(chunk) => {
                    if stream.write_all(&chunk).is_err() {
                        tracing::debug!("upnp/pcm: {peer} disconnected");
                        break;
                    }
                }
                RecvResult::Closed => break,
            }
        }
    }
}

/// Write `data` into `stream`, inserting an ICY metadata block every `ICY_METAINT` bytes.
///
/// Sends a populated block only when the track changes; otherwise sends a single
/// zero byte (empty block) to satisfy the protocol framing without cluttering logs.
fn write_icy_chunk(
    stream: &mut std::net::TcpStream,
    data: &[u8],
    bytes_since_meta: &mut usize,
    last_track_key: &mut String,
    art_url: &str,
) -> std::io::Result<()> {
    let mut pos = 0;
    while pos < data.len() {
        let space = ICY_METAINT - *bytes_since_meta;
        let to_write = space.min(data.len() - pos);

        stream.write_all(&data[pos..pos + to_write])?;
        pos += to_write;
        *bytes_since_meta += to_write;

        if *bytes_since_meta >= ICY_METAINT {
            *bytes_since_meta = 0;

            match TrackMeta::from_current() {
                Some(meta) => {
                    let key = meta.track_key();
                    if key != *last_track_key {
                        *last_track_key = key;
                        let block = icy_block(&meta, art_url);
                        stream.write_all(&block)?;
                        tracing::debug!(
                            "upnp/pcm: ICY → {} / {} / {}",
                            meta.artist,
                            meta.title,
                            meta.album
                        );
                    } else {
                        stream.write_all(&[0u8])?; // same track — empty block
                    }
                }
                None => {
                    stream.write_all(&[0u8])?; // nothing playing
                }
            }
        }
    }
    Ok(())
}

// ─── Main server ─────────────────────────────────────────────────────────────

/// HTTP server that serves live PCM audio as a streaming WAV file plus
/// companion metadata endpoints.
///
/// Endpoints:
///   GET /stream.wav          — live WAV stream (supports ICY metadata)
///   GET /now-playing.json    — current track metadata as JSON
///   GET /now-playing/art     — current track album art (JPEG/PNG)
///   (any other path)         — also serves the WAV stream (for compatibility)
pub fn serve(port: u16, sample_rate: u32, buf: Arc<BroadcastBuffer>) {
    let listener = match TcpListener::bind(("0.0.0.0", port)) {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("upnp/pcm: bind :{port} failed: {e}");
            return;
        }
    };
    tracing::info!(
        "upnp/pcm: WAV stream on :{port}  metadata on /now-playing.json and /now-playing/art"
    );

    for stream in listener.incoming() {
        match stream {
            Ok(mut tcp) => {
                let buf = buf.clone();
                std::thread::spawn(move || {
                    let peer = tcp.peer_addr().map(|a| a.to_string()).unwrap_or_default();

                    let req = match parse_request(&mut tcp) {
                        Ok(r) => r,
                        Err(e) => {
                            tracing::warn!("upnp/pcm: request read error from {peer}: {e}");
                            return;
                        }
                    };

                    let local_ip = crate::get_local_ip();

                    match req.path.as_str() {
                        "/now-playing.json" => {
                            serve_now_playing_json(&mut tcp, local_ip, port);
                        }
                        "/now-playing/art" | "/now-playing/art.jpg" | "/now-playing/art.png" => {
                            serve_now_playing_art(&mut tcp);
                        }
                        _ => {
                            serve_wav_stream(&mut tcp, &req, sample_rate, buf, port, &peer);
                        }
                    }
                });
            }
            Err(e) => tracing::warn!("upnp/pcm: accept error: {e}"),
        }
    }
}
