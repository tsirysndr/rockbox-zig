use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::ffi::CStr;
use std::fs::File;
use std::io::{self, BufReader, Read, Seek};
use std::os::raw::c_char;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Mutex};
use tracing::{debug, warn};

/// Sentinel handle ID returned on error.
const INVALID_HANDLE: i32 = -1;

// ─── Stream source ───────────────────────────────────────────────────────────

/// Where bytes for a stream come from.
enum StreamSource {
    /// Live HTTP response (standard path).
    Http {
        response: Option<reqwest::blocking::Response>,
    },
    /// Local file served from the on-disk cache (cache-hit path).
    CachedFile { reader: BufReader<File> },
}

// ─── Per-stream state ────────────────────────────────────────────────────────

struct StreamState {
    url: String,
    pos: u64,
    content_length: Option<u64>,
    content_type: Option<String>,
    source: StreamSource,
}

impl StreamState {
    fn response_content_type(resp: &reqwest::blocking::Response) -> Option<String> {
        let value = resp
            .headers()
            .get(reqwest::header::CONTENT_TYPE)?
            .to_str()
            .ok()?;
        let value = value.split(';').next()?.trim();
        if value.is_empty() {
            None
        } else {
            Some(value.to_ascii_lowercase())
        }
    }

    /// Open a new HTTP stream.
    fn new_http(url: String) -> Option<Self> {
        let response = CLIENT.get(&url).send().ok()?;
        if !response.status().is_success() {
            return None;
        }
        let content_length = response.content_length();
        let content_type = Self::response_content_type(&response);
        Some(StreamState {
            url,
            pos: 0,
            content_length,
            content_type,
            source: StreamSource::Http {
                response: Some(response),
            },
        })
    }

    /// Open a cached file as a stream.
    fn new_from_cache(url: &str, path: &std::path::Path) -> Option<Self> {
        let file = File::open(path).ok()?;
        let size = file.metadata().ok()?.len();
        let reader = BufReader::new(file);
        // Best-effort content-type from the URL's file extension (strip query string first).
        let url_path = url.split('?').next().unwrap_or(url);
        let ext = url_path
            .rsplit('.')
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();
        let content_type = match ext.as_str() {
            "mp3" => Some("audio/mpeg".to_string()),
            "flac" => Some("audio/flac".to_string()),
            "ogg" | "oga" => Some("audio/ogg".to_string()),
            "m4a" | "aac" => Some("audio/aac".to_string()),
            "wav" => Some("audio/wav".to_string()),
            "opus" => Some("audio/opus".to_string()),
            "wv" => Some("audio/x-wavpack".to_string()),
            "ape" => Some("audio/x-ape".to_string()),
            "mpc" => Some("audio/x-musepack".to_string()),
            _ => None,
        };
        Some(StreamState {
            url: url.to_string(),
            pos: 0,
            content_length: Some(size),
            content_type,
            source: StreamSource::CachedFile { reader },
        })
    }

    fn skip_bytes(resp: &mut reqwest::blocking::Response, mut to_skip: u64) -> bool {
        let mut buf = [0u8; 8192];
        while to_skip > 0 {
            let chunk = usize::min(to_skip as usize, buf.len());
            match resp.read(&mut buf[..chunk]) {
                Ok(0) => return false,
                Ok(n) => to_skip -= n as u64,
                Err(_) => return false,
            }
        }
        true
    }

    fn update_content_length_from_content_range(&mut self, resp: &reqwest::blocking::Response) {
        if self.content_length.is_some() {
            return;
        }
        if let Some(cr) = resp.headers().get("content-range") {
            if let Ok(cr_str) = cr.to_str() {
                if let Some(total_str) = cr_str.split('/').last() {
                    if let Ok(total) = total_str.trim().parse::<u64>() {
                        self.content_length = Some(total);
                    }
                }
            }
        }
    }

    fn parse_content_range_start(resp: &reqwest::blocking::Response) -> Option<u64> {
        let value = resp.headers().get("content-range")?.to_str().ok()?;
        let value = value.strip_prefix("bytes ")?;
        let (range, _) = value.split_once('/')?;
        let (start, _) = range.split_once('-')?;
        start.trim().parse::<u64>().ok()
    }

    /// Seek the stream to `new_pos`.
    ///
    /// For cached files: instant O(1) file seek.
    /// For HTTP streams: inline byte-skip for small forward seeks, Range
    /// request for anything larger or backward.
    fn seek_to(&mut self, new_pos: u64) -> bool {
        // CachedFile: direct seek — no network round-trip ever needed.
        if let StreamSource::CachedFile { reader } = &mut self.source {
            use std::io::{Seek, SeekFrom};
            match reader.seek(SeekFrom::Start(new_pos)) {
                Ok(_) => {
                    self.pos = new_pos;
                    return true;
                }
                Err(_) => return false,
            }
        }

        // HTTP path ──────────────────────────────────────────────────────────

        // Small forward seek: skip bytes in the existing response body rather
        // than issuing a new Range request.
        const INLINE_SKIP_MAX: u64 = 128 * 1024; // 128 KB
        if new_pos > self.pos && new_pos - self.pos <= INLINE_SKIP_MAX {
            if let StreamSource::Http {
                response: Some(resp),
            } = &mut self.source
            {
                let to_skip = new_pos - self.pos;
                if Self::skip_bytes(resp, to_skip) {
                    self.pos = new_pos;
                    return true;
                }
                // Inline skip failed; fall through to a Range request.
            }
        }

        let range_header = match self.content_length {
            Some(cl) if cl > 0 => format!("bytes={}-{}", new_pos, cl - 1),
            _ => format!("bytes={}-", new_pos),
        };
        let result = CLIENT.get(&self.url).header("Range", range_header).send();

        match result {
            Ok(resp) if resp.status().as_u16() == 206 => {
                self.update_content_length_from_content_range(&resp);
                if self.content_type.is_none() {
                    self.content_type = Self::response_content_type(&resp);
                }
                if Self::parse_content_range_start(&resp) != Some(new_pos) {
                    return false;
                }
                if let StreamSource::Http { response } = &mut self.source {
                    *response = Some(resp);
                }
                self.pos = new_pos;
                true
            }
            Ok(mut resp) if resp.status().is_success() => {
                if self.content_length.is_none() {
                    self.content_length = resp.content_length();
                }
                if self.content_type.is_none() {
                    self.content_type = Self::response_content_type(&resp);
                }
                if new_pos > 0 && !Self::skip_bytes(&mut resp, new_pos) {
                    return false;
                }
                if let StreamSource::Http { response } = &mut self.source {
                    *response = Some(resp);
                }
                self.pos = new_pos;
                true
            }
            _ => false,
        }
    }
}

fn read_as_file<R: Read>(reader: &mut R, buf: &mut [u8]) -> io::Result<usize> {
    let mut total = 0;
    while total < buf.len() {
        match reader.read(&mut buf[total..]) {
            Ok(0) => break,
            Ok(n) => total += n,
            Err(err) if err.kind() == io::ErrorKind::Interrupted => continue,
            Err(err) => {
                if total > 0 {
                    break;
                }
                return Err(err);
            }
        }
    }
    Ok(total)
}

static STREAMS: Lazy<Mutex<HashMap<i32, Arc<Mutex<StreamState>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

static CLIENT: Lazy<reqwest::blocking::Client> = Lazy::new(|| {
    reqwest::blocking::Client::builder()
        .use_rustls_tls()
        .build()
        .expect("failed to build global HTTP client")
});

static NEXT_HANDLE: AtomicI32 = AtomicI32::new(0);

// ─── Public C ABI ────────────────────────────────────────────────────────────

/// Open a URL and return an integer handle, or -1 on failure.
///
/// For remote HTTP(S) URLs:
///   - If the URL is already in the local cache the stream is served from the
///     cached file (instant open, zero network traffic, full seek support).
///   - On a cache miss, the URL is opened via HTTP as normal **and** a
///     background thread is spawned to download and cache the full file in
///     parallel so subsequent opens are instant.
///
/// # Safety
/// `url` must be a valid, NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rb_net_open(url: *const c_char) -> i32 {
    if url.is_null() {
        return INVALID_HANDLE;
    }
    let url_str = match CStr::from_ptr(url).to_str() {
        Ok(s) => {
            let s = s.trim();
            let s = s.split('#').next().unwrap_or(s); // drop URL fragment
            s.to_owned()
        }
        Err(_) => return INVALID_HANDLE,
    };

    // ── Cache integration ──────────────────────────────────────────────────
    let is_http = url_str.starts_with("http://") || url_str.starts_with("https://");
    if is_http {
        // Cache hit: serve directly from disk, no network needed.
        if let Some(cached_path) = rockbox_cache::lookup(&url_str) {
            if let Some(state) = StreamState::new_from_cache(&url_str, &cached_path) {
                let handle = NEXT_HANDLE.fetch_add(1, Ordering::SeqCst);
                STREAMS
                    .lock()
                    .unwrap()
                    .insert(handle, Arc::new(Mutex::new(state)));
                debug!(
                    "[netstream] rb_net_open: cache HIT url={} -> handle={}",
                    url_str, handle
                );
                return handle;
            }
            // File was deleted between lookup and open; fall through to HTTP.
        }
        // Cache miss: kick off a parallel background download while this
        // stream continues reading from the live HTTP connection.
        rockbox_cache::start_background_fetch(&url_str);
    }
    // ──────────────────────────────────────────────────────────────────────

    let state = match StreamState::new_http(url_str.clone()) {
        Some(s) => {
            debug!(
                "[netstream] rb_net_open: url={} content_length={:?} content_type={:?}",
                url_str, s.content_length, s.content_type
            );
            s
        }
        None => {
            warn!("[netstream] rb_net_open: FAILED url={}", url_str);
            return INVALID_HANDLE;
        }
    };

    let handle = NEXT_HANDLE.fetch_add(1, Ordering::SeqCst);
    STREAMS
        .lock()
        .unwrap()
        .insert(handle, Arc::new(Mutex::new(state)));
    debug!(
        "[netstream] rb_net_open: url={} -> handle={}",
        url_str, handle
    );
    handle
}

/// Read up to `n` bytes from stream `h` into `dst`.
/// Returns the number of bytes read, 0 on EOF, or -1 on error.
///
/// # Safety
/// `dst` must point to a buffer of at least `n` bytes.
#[no_mangle]
pub unsafe extern "C" fn rb_net_read(h: i32, dst: *mut libc::c_void, n: libc::size_t) -> i64 {
    if dst.is_null() || n == 0 {
        return 0;
    }
    let handle_arc = {
        let streams = STREAMS.lock().unwrap();
        match streams.get(&h) {
            Some(arc) => arc.clone(),
            None => return -1,
        }
    };
    let mut state = handle_arc.lock().unwrap();
    let pos_before = state.pos;
    let buf = std::slice::from_raw_parts_mut(dst as *mut u8, n);

    // ── Cached file: read directly from local disk ─────────────────────────
    if let StreamSource::CachedFile { reader } = &mut state.source {
        match read_as_file(reader, buf) {
            Ok(bytes_read) => {
                state.pos += bytes_read as u64;
                tracing::trace!(
                    "[netstream] rb_net_read (cached): h={} n={} pos_before={} -> read={} pos_after={}",
                    h, n, pos_before, bytes_read, state.pos
                );
                return bytes_read as i64;
            }
            Err(e) => {
                warn!(
                    "[netstream] rb_net_read (cached): h={} n={} pos={} -> ERROR {:?}",
                    h, n, pos_before, e
                );
                return -1;
            }
        }
    }

    // ── HTTP stream ────────────────────────────────────────────────────────
    let resp = match &mut state.source {
        StreamSource::Http {
            response: Some(r), ..
        } => r,
        StreamSource::Http { response: None, .. } => {
            if state.content_length.map_or(false, |cl| state.pos >= cl) {
                return 0; // EOF
            }
            return -1;
        }
        StreamSource::CachedFile { .. } => unreachable!(),
    };
    match read_as_file(resp, buf) {
        Ok(bytes_read) => {
            state.pos += bytes_read as u64;
            tracing::trace!(
                "[netstream] rb_net_read: h={} n={} pos_before={} -> read={} pos_after={}",
                h,
                n,
                pos_before,
                bytes_read,
                state.pos
            );
            bytes_read as i64
        }
        Err(e) => {
            warn!(
                "[netstream] rb_net_read: h={} n={} pos={} -> ERROR {:?}",
                h, n, pos_before, e
            );
            -1
        }
    }
}

/// Seek within stream `h`.  `whence` follows POSIX semantics:
///   0 = SEEK_SET, 1 = SEEK_CUR, 2 = SEEK_END.
/// Returns the new position on success, or -1 on failure.
#[no_mangle]
pub extern "C" fn rb_net_lseek(h: i32, off: i64, whence: libc::c_int) -> i64 {
    const SEEK_SET: libc::c_int = 0;
    const SEEK_CUR: libc::c_int = 1;
    const SEEK_END: libc::c_int = 2;

    let handle_arc = {
        let streams = STREAMS.lock().unwrap();
        match streams.get(&h) {
            Some(arc) => arc.clone(),
            None => return -1,
        }
    };
    let mut state = handle_arc.lock().unwrap();

    let new_pos: u64 = match whence {
        x if x == SEEK_SET => {
            if off < 0 {
                return -1;
            }
            off as u64
        }
        x if x == SEEK_CUR => {
            if off < 0 {
                let abs_off = (-off) as u64;
                if abs_off > state.pos {
                    return -1;
                }
                state.pos - abs_off
            } else {
                state.pos + off as u64
            }
        }
        x if x == SEEK_END => {
            let len = match state.content_length {
                Some(l) => l,
                None => return -1,
            };
            if off > 0 {
                return -1;
            }
            let abs_off = (-off) as u64;
            if abs_off > len {
                return -1;
            }
            len - abs_off
        }
        _ => return -1,
    };

    // Guard 1: never seek gigabytes forward regardless of content_length.
    // (Only applies to HTTP streams; cached files handle any seek trivially.)
    const MAX_SKIP: u64 = 256 * 1024 * 1024;
    if matches!(state.source, StreamSource::Http { .. })
        && new_pos > state.pos
        && new_pos - state.pos > MAX_SKIP
    {
        warn!(
            "[netstream] rb_net_lseek: h={} off={} whence={} huge skip ({} bytes) clamped",
            h,
            off,
            whence,
            new_pos - state.pos
        );
        return -1;
    }

    // Guard 2: never seek past EOF — clamp to content_length.
    if let Some(cl) = state.content_length {
        if new_pos >= cl {
            warn!(
                "[netstream] rb_net_lseek: h={} off={} whence={} new_pos={} >= content_length={}, clamping to EOF",
                h, off, whence, new_pos, cl
            );
            state.pos = cl;
            match &mut state.source {
                StreamSource::Http { response } => *response = None,
                StreamSource::CachedFile { reader } => {
                    let _ = reader.seek(std::io::SeekFrom::Start(cl));
                }
            }
            return cl as i64;
        }
    }

    // Fast-path: already at the target position.
    if new_pos == state.pos {
        debug!(
            "[netstream] rb_net_lseek: h={} off={} whence={} -> already at pos={} (no-op)",
            h, off, whence, state.pos
        );
        return state.pos as i64;
    }

    let old_pos = state.pos;
    if state.seek_to(new_pos) {
        debug!(
            "[netstream] rb_net_lseek: h={} off={} whence={} old_pos={} -> new_pos={}",
            h, off, whence, old_pos, state.pos
        );
        state.pos as i64
    } else {
        warn!(
            "[netstream] rb_net_lseek: h={} off={} whence={} old_pos={} -> FAILED",
            h, off, whence, old_pos
        );
        -1
    }
}

/// Return the total content length of stream `h`, or -1 if unknown.
#[no_mangle]
pub extern "C" fn rb_net_len(h: i32) -> i64 {
    let handle_arc = {
        let streams = STREAMS.lock().unwrap();
        match streams.get(&h) {
            Some(arc) => arc.clone(),
            None => return -1,
        }
    };
    let len = handle_arc
        .lock()
        .unwrap()
        .content_length
        .map(|l| l as i64)
        .unwrap_or(-1);
    debug!("[netstream] rb_net_len: h={} -> {}", h, len);
    len
}

/// Copy the normalized Content-Type for stream `h` into `dst`.
/// Returns the full string length on success, or -1 if unavailable.
///
/// # Safety
/// `dst` must point to a writable buffer of at least `n` bytes when `n > 0`.
#[no_mangle]
pub unsafe extern "C" fn rb_net_content_type(h: i32, dst: *mut c_char, n: libc::size_t) -> i64 {
    let handle_arc = {
        let streams = STREAMS.lock().unwrap();
        match streams.get(&h) {
            Some(arc) => arc.clone(),
            None => return -1,
        }
    };
    let state = handle_arc.lock().unwrap();
    let content_type = match state.content_type.as_deref() {
        Some(value) => value,
        None => return -1,
    };

    if !dst.is_null() && n > 0 {
        let bytes = content_type.as_bytes();
        let copy_len = usize::min(bytes.len(), n.saturating_sub(1));
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), dst as *mut u8, copy_len);
        *dst.add(copy_len) = 0;
    }

    content_type.len() as i64
}

/// Close stream `h` and release its resources.
#[no_mangle]
pub extern "C" fn rb_net_close(h: i32) {
    debug!("[netstream] rb_net_close: h={}", h);
    STREAMS.lock().unwrap().remove(&h);
}

// ─── Tests ───────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Matcher;
    use std::ffi::CString;
    use std::io::Cursor;

    fn c_url(server: &mockito::Server, path: &str) -> CString {
        CString::new(format!("{}{}", server.url(), path)).unwrap()
    }

    struct PartialReader {
        inner: Cursor<Vec<u8>>,
        chunk_size: usize,
    }

    impl PartialReader {
        fn new(data: &[u8], chunk_size: usize) -> Self {
            Self {
                inner: Cursor::new(data.to_vec()),
                chunk_size,
            }
        }
    }

    impl Read for PartialReader {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            let chunk = usize::min(self.chunk_size, buf.len());
            self.inner.read(&mut buf[..chunk])
        }
    }

    // ── Open / close ──────────────────────────────────────────────────────

    #[test]
    fn test_open_and_close() {
        let mut server = mockito::Server::new();
        let _mock = server
            .mock("GET", "/audio.mp3")
            .with_status(200)
            .with_header("content-length", "4")
            .with_body(b"data")
            .create();

        let url = c_url(&server, "/audio.mp3");
        let handle = unsafe { rb_net_open(url.as_ptr()) };
        assert!(handle >= 0, "open should return a valid handle");

        rb_net_close(handle);
        assert_eq!(rb_net_len(handle), -1, "closed handle should return -1");
    }

    #[test]
    fn test_open_null_url() {
        let handle = unsafe { rb_net_open(std::ptr::null()) };
        assert_eq!(handle, INVALID_HANDLE);
    }

    #[test]
    fn test_open_unreachable_host() {
        let url = CString::new("http://127.0.0.1:19998/file.mp3").unwrap();
        let handle = unsafe { rb_net_open(url.as_ptr()) };
        assert_eq!(handle, INVALID_HANDLE);
    }

    #[test]
    fn test_open_404() {
        let mut server = mockito::Server::new();
        let _mock = server.mock("GET", "/missing.mp3").with_status(404).create();

        let url = c_url(&server, "/missing.mp3");
        let handle = unsafe { rb_net_open(url.as_ptr()) };
        assert_eq!(handle, INVALID_HANDLE);
    }

    // ── Content-Length / rb_net_len ───────────────────────────────────────

    #[test]
    fn test_known_content_length() {
        let mut server = mockito::Server::new();
        let _mock = server
            .mock("GET", "/known.mp3")
            .with_status(200)
            .with_header("content-length", "1234")
            .with_body(vec![0u8; 1234])
            .create();

        let url = c_url(&server, "/known.mp3");
        let handle = unsafe { rb_net_open(url.as_ptr()) };
        assert!(handle >= 0);
        assert_eq!(rb_net_len(handle), 1234);
        rb_net_close(handle);
    }

    #[test]
    fn test_content_type_is_available() {
        let mut server = mockito::Server::new();
        let _mock = server
            .mock("GET", "/typed")
            .with_status(200)
            .with_header("content-type", "audio/m4a; charset=binary")
            .with_body("data")
            .create();

        let url = c_url(&server, "/typed");
        let handle = unsafe { rb_net_open(url.as_ptr()) };
        assert!(handle >= 0);

        let mut buf = vec![0i8; 32];
        let n = unsafe { rb_net_content_type(handle, buf.as_mut_ptr(), buf.len()) };
        assert_eq!(n, "audio/m4a".len() as i64);

        let ct = unsafe { CStr::from_ptr(buf.as_ptr()) }.to_str().unwrap();
        assert_eq!(ct, "audio/m4a");
        rb_net_close(handle);
    }

    #[test]
    fn test_unknown_content_length() {
        let body: &[u8] = b"some data";
        let mut server = mockito::Server::new();
        let _mock = server
            .mock("GET", "/stream.mp3")
            .with_status(200)
            .with_body(body)
            .create();

        let url = c_url(&server, "/stream.mp3");
        let handle = unsafe { rb_net_open(url.as_ptr()) };
        assert!(handle >= 0);
        assert_eq!(rb_net_len(handle), body.len() as i64);
        rb_net_close(handle);
    }

    // ── Reading ───────────────────────────────────────────────────────────

    #[test]
    fn test_read_bytes() {
        let body: &[u8] = b"Hello, Rockbox!";
        let mut server = mockito::Server::new();
        let _mock = server
            .mock("GET", "/song.mp3")
            .with_status(200)
            .with_body(body)
            .create();

        let url = c_url(&server, "/song.mp3");
        let handle = unsafe { rb_net_open(url.as_ptr()) };
        assert!(handle >= 0);

        let mut buf = vec![0u8; 64];
        let n = unsafe { rb_net_read(handle, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
        assert_eq!(n, body.len() as i64);
        assert_eq!(&buf[..n as usize], body);
        rb_net_close(handle);
    }

    #[test]
    fn test_read_as_file_retries_partial_reads() {
        let mut reader = PartialReader::new(b"Hello, Rockbox!", 3);
        let mut buf = vec![0u8; 15];
        let n = read_as_file(&mut reader, &mut buf).unwrap();
        assert_eq!(n, 15);
        assert_eq!(&buf, b"Hello, Rockbox!");
    }

    #[test]
    fn test_read_eof() {
        let body: &[u8] = b"tiny";
        let mut server = mockito::Server::new();
        let _mock = server
            .mock("GET", "/eof.mp3")
            .with_status(200)
            .with_body(body)
            .create();

        let url = c_url(&server, "/eof.mp3");
        let handle = unsafe { rb_net_open(url.as_ptr()) };
        assert!(handle >= 0);

        let mut buf = vec![0u8; 1024];
        let n1 = unsafe { rb_net_read(handle, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
        assert_eq!(n1, body.len() as i64);

        let n2 = unsafe { rb_net_read(handle, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
        assert_eq!(n2, 0, "second read should return 0 at EOF");
        rb_net_close(handle);
    }

    #[test]
    fn test_read_invalid_handle() {
        let mut buf = vec![0u8; 16];
        let result =
            unsafe { rb_net_read(i32::MAX, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
        assert_eq!(result, -1);
    }

    // ── Seeking ───────────────────────────────────────────────────────────

    #[test]
    fn test_seek_set_range_request() {
        let full_body: &[u8] = b"0123456789ABCDEF";
        let mut server = mockito::Server::new();

        let _initial = server
            .mock("GET", "/seekable.mp3")
            .match_header("range", Matcher::Missing)
            .with_status(200)
            .with_header("content-length", "16")
            .with_body(full_body)
            .create();

        let _range = server
            .mock("GET", "/seekable.mp3")
            .match_header("range", "bytes=8-15")
            .with_status(206)
            .with_header("content-range", "bytes 8-15/16")
            .with_header("content-length", "8")
            .with_body(&full_body[8..])
            .create();

        let url = c_url(&server, "/seekable.mp3");
        let handle = unsafe { rb_net_open(url.as_ptr()) };
        assert!(handle >= 0);

        let new_pos = rb_net_lseek(handle, 8, libc::SEEK_SET);
        assert_eq!(new_pos, 8);

        let mut buf = vec![0u8; 16];
        let n = unsafe { rb_net_read(handle, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
        assert_eq!(n, 8);
        assert_eq!(&buf[..8], &full_body[8..]);
        rb_net_close(handle);
    }

    #[test]
    fn test_seek_cur_no_op() {
        let body: &[u8] = b"ABCDEFGHIJ";
        let mut server = mockito::Server::new();
        let _mock = server
            .mock("GET", "/cur.mp3")
            .with_status(200)
            .with_header("content-length", "10")
            .with_body(body)
            .create();

        let url = c_url(&server, "/cur.mp3");
        let handle = unsafe { rb_net_open(url.as_ptr()) };
        assert!(handle >= 0);

        let mut buf = vec![0u8; 5];
        let n = unsafe { rb_net_read(handle, buf.as_mut_ptr() as *mut libc::c_void, 5) };
        assert_eq!(n, 5);

        let pos = rb_net_lseek(handle, 0, libc::SEEK_CUR);
        assert_eq!(pos, 5);
        rb_net_close(handle);
    }

    #[test]
    fn test_seek_end() {
        let full_body: &[u8] = b"XXXXXXXXXX";
        let mut server = mockito::Server::new();

        let _initial = server
            .mock("GET", "/end.mp3")
            .match_header("range", Matcher::Missing)
            .with_status(200)
            .with_header("content-length", "10")
            .with_body(full_body)
            .create();

        let _range = server
            .mock("GET", "/end.mp3")
            .match_header("range", "bytes=8-9")
            .with_status(206)
            .with_header("content-range", "bytes 8-9/10")
            .with_body(&full_body[8..])
            .create();

        let url = c_url(&server, "/end.mp3");
        let handle = unsafe { rb_net_open(url.as_ptr()) };
        assert!(handle >= 0);

        let pos = rb_net_lseek(handle, -2, libc::SEEK_END);
        assert_eq!(pos, 8);
        rb_net_close(handle);
    }

    #[test]
    fn test_seek_end_unknown_length() {
        let full_body: &[u8] = b"data";
        let mut server = mockito::Server::new();
        let _mock = server
            .mock("GET", "/nosize.mp3")
            .with_status(200)
            .with_header("content-length", "4")
            .with_body(full_body)
            .create();

        let url = c_url(&server, "/nosize.mp3");
        let handle = unsafe { rb_net_open(url.as_ptr()) };
        assert!(handle >= 0);

        let pos = rb_net_lseek(handle, -100, libc::SEEK_END);
        assert_eq!(pos, -1);
        rb_net_close(handle);
    }

    #[test]
    fn test_seek_range_not_supported() {
        let mut server = mockito::Server::new();

        let _initial = server
            .mock("GET", "/noseek.mp3")
            .match_header("range", Matcher::Missing)
            .with_status(200)
            .with_header("content-length", "100")
            .with_body(vec![0u8; 100])
            .create();

        let _no_range = server
            .mock("GET", "/noseek.mp3")
            .match_header("range", Matcher::Any)
            .with_status(416)
            .create();

        let url = c_url(&server, "/noseek.mp3");
        let handle = unsafe { rb_net_open(url.as_ptr()) };
        assert!(handle >= 0);

        let result = rb_net_lseek(handle, 50, libc::SEEK_SET);
        assert_eq!(result, -1);
        rb_net_close(handle);
    }

    #[test]
    fn test_seek_falls_back_when_range_is_ignored() {
        let full_body: &[u8] = b"0123456789ABCDEF";
        let mut server = mockito::Server::new();

        let _initial = server
            .mock("GET", "/ignore-range.mp3")
            .match_header("range", Matcher::Missing)
            .with_status(200)
            .with_header("content-length", "16")
            .with_body(full_body)
            .create();

        let _ignored_range = server
            .mock("GET", "/ignore-range.mp3")
            .match_header("range", "bytes=8-15")
            .with_status(200)
            .with_header("content-length", "16")
            .with_body(full_body)
            .create();

        let url = c_url(&server, "/ignore-range.mp3");
        let handle = unsafe { rb_net_open(url.as_ptr()) };
        assert!(handle >= 0);

        let new_pos = rb_net_lseek(handle, 8, libc::SEEK_SET);
        assert_eq!(new_pos, 8);

        let mut buf = vec![0u8; 16];
        let n = unsafe { rb_net_read(handle, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
        assert_eq!(n, 8);
        assert_eq!(&buf[..8], &full_body[8..]);
        rb_net_close(handle);
    }

    #[test]
    fn test_seek_rejects_wrong_content_range_start() {
        let full_body: &[u8] = b"0123456789ABCDEF";
        let mut server = mockito::Server::new();

        let _initial = server
            .mock("GET", "/bad-range.mp3")
            .match_header("range", Matcher::Missing)
            .with_status(200)
            .with_header("content-length", "16")
            .with_body(full_body)
            .create();

        let _bad_range = server
            .mock("GET", "/bad-range.mp3")
            .match_header("range", "bytes=8-15")
            .with_status(206)
            .with_header("content-range", "bytes 0-7/16")
            .with_body(&full_body[..8])
            .create();

        let url = c_url(&server, "/bad-range.mp3");
        let handle = unsafe { rb_net_open(url.as_ptr()) };
        assert!(handle >= 0);

        let result = rb_net_lseek(handle, 8, libc::SEEK_SET);
        assert_eq!(result, -1);
        rb_net_close(handle);
    }

    #[test]
    fn test_content_length_from_content_range() {
        let full_body: &[u8] = b"0123456789";
        let mut server = mockito::Server::new();

        let _initial = server
            .mock("GET", "/range-len.mp3")
            .match_header("range", Matcher::Missing)
            .with_status(200)
            .with_header("content-length", "10")
            .with_body(full_body)
            .create();

        let _range = server
            .mock("GET", "/range-len.mp3")
            .match_header("range", "bytes=5-9")
            .with_status(206)
            .with_header("content-range", "bytes 5-9/10")
            .with_body(&full_body[5..])
            .create();

        let url = c_url(&server, "/range-len.mp3");
        let handle = unsafe { rb_net_open(url.as_ptr()) };
        assert!(handle >= 0);
        assert_eq!(rb_net_len(handle), 10);

        let pos = rb_net_lseek(handle, 5, libc::SEEK_SET);
        assert_eq!(pos, 5);
        assert_eq!(rb_net_len(handle), 10);
        rb_net_close(handle);
    }

    #[test]
    fn test_seek_past_eof_is_clamped() {
        let full_body: &[u8] = b"0123456789";
        let mut server = mockito::Server::new();

        let _initial = server
            .mock("GET", "/clamped.mp3")
            .match_header("range", Matcher::Missing)
            .with_status(200)
            .with_header("content-length", "10")
            .with_body(full_body)
            .create();

        let url = c_url(&server, "/clamped.mp3");
        let handle = unsafe { rb_net_open(url.as_ptr()) };
        assert!(handle >= 0);

        let result = rb_net_lseek(handle, 4_294_967_295, libc::SEEK_CUR);
        assert_eq!(result, 10, "seek past EOF should clamp to content_length");

        let mut buf = vec![0u8; 16];
        let n = unsafe { rb_net_read(handle, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
        assert_eq!(n, 0, "read after clamped seek should return 0 (EOF)");
        rb_net_close(handle);
    }
}
