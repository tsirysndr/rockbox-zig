use once_cell::sync::Lazy;
use std::collections::{HashMap, VecDeque};
use std::ffi::CStr;
use std::io::Read;
use std::os::raw::c_char;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;
use tracing::{debug, warn};

/// Sentinel handle ID returned on error.
const INVALID_HANDLE: i32 = -1;

/// Background prefetch buffer capacity per stream.
/// Android: 16 MB — at 3 Mbps FLAC (375 KB/s) that's ~42 s of buffer.
///   A 15-second read_timeout fires and reconnects well before the buffer drains.
/// Desktop: 4 MB — LAN speeds make stalls rare; 4 MB is plenty.
#[cfg(target_os = "android")]
const PREFETCH_CAP: usize = 16 * 1024 * 1024;
#[cfg(not(target_os = "android"))]
const PREFETCH_CAP: usize = 4 * 1024 * 1024;

/// Max reconnect attempts on TCP error before giving up.
const MAX_RETRIES: u32 = 8;

struct Prefetch {
    data: VecDeque<u8>,
    /// Total bytes the reader thread has written into `data` (wraps on u64 overflow after ~18 EB).
    bytes_written: u64,
    eof: bool,
    error: bool,
    stop: bool,
}

impl Prefetch {
    fn new() -> Self {
        Prefetch {
            data: VecDeque::with_capacity(PREFETCH_CAP + 64 * 1024),
            bytes_written: 0,
            eof: false,
            error: false,
            stop: false,
        }
    }
}

/// Per-stream state.
struct StreamState {
    url: String,
    pos: u64,
    content_length: Option<u64>,
    content_type: Option<String>,
    pair: Arc<(Mutex<Prefetch>, Condvar)>,
    _thread: Option<thread::JoinHandle<()>>,
}

impl Drop for StreamState {
    fn drop(&mut self) {
        let (lock, cv) = &*self.pair;
        lock.lock().unwrap().stop = true;
        cv.notify_all();
    }
}

/// Spawn a background reader that fills `pair` from `response`, retrying from
/// `start_offset + bytes_written` on TCP error (up to MAX_RETRIES times).
///
/// `expected_bytes`: how many bytes this response should deliver
/// (`content_length - start_offset`). Pass `None` if unknown (live/infinite
/// streams). Used to detect premature EOF: Android mobile radio or power-save
/// can close the TCP connection mid-stream, which makes `response.read()`
/// return `Ok(0)` before all bytes have arrived. Without this check the stream
/// would be marked as EOF, buffering.c would truncate the track, and playback
/// would skip to the next file.
fn spawn_reader(
    pair: Arc<(Mutex<Prefetch>, Condvar)>,
    url: String,
    start_offset: u64,
    expected_bytes: Option<u64>,
    mut response: reqwest::blocking::Response,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut buf = vec![0u8; 64 * 1024];
        let mut retries: u32 = 0;

        loop {
            // Wait until there is room in the prefetch buffer.
            {
                let (lock, cv) = &*pair;
                let mut pf = lock.lock().unwrap();
                while pf.data.len() >= PREFETCH_CAP && !pf.stop {
                    pf = cv.wait(pf).unwrap();
                }
                if pf.stop {
                    return;
                }
            }
            // Network I/O with no lock held.
            match response.read(&mut buf) {
                Ok(0) => {
                    // Distinguish genuine EOF from a mid-stream TCP close.
                    // Android power-save or LTE handover can terminate the TCP
                    // connection cleanly (FIN), making read() return Ok(0) even
                    // though not all bytes have arrived. Check bytes_written
                    // against expected_bytes to detect this.
                    let bytes_written = {
                        let (lock, _) = &*pair;
                        lock.lock().unwrap().bytes_written
                    };
                    let premature = expected_bytes
                        .map(|exp| bytes_written < exp)
                        .unwrap_or(false);

                    if premature && retries < MAX_RETRIES {
                        retries += 1;
                        let resume_offset = start_offset + bytes_written;
                        warn!(
                            "[netstream] premature EOF at {}/{} bytes (attempt {}/{}), \
                             reconnecting at offset {}",
                            bytes_written,
                            expected_bytes.unwrap_or(0),
                            retries,
                            MAX_RETRIES,
                            resume_offset
                        );
                        thread::sleep(Duration::from_millis(200 * retries as u64));
                        {
                            let (lock, _) = &*pair;
                            if lock.lock().unwrap().stop {
                                return;
                            }
                        }
                        let range_header = format!("bytes={}-", resume_offset);
                        match CLIENT.get(&url).header("Range", range_header).send() {
                            Ok(resp) if resp.status().as_u16() == 206 => {
                                response = resp;
                            }
                            Ok(_) | Err(_) => {
                                debug!("[netstream] Range reconnect after premature EOF failed");
                            }
                        }
                        continue;
                    }

                    let (lock, cv) = &*pair;
                    lock.lock().unwrap().eof = true;
                    cv.notify_all();
                    return;
                }
                Ok(n) => {
                    let (lock, cv) = &*pair;
                    let mut pf = lock.lock().unwrap();
                    if pf.stop {
                        return;
                    }
                    pf.data.extend(&buf[..n]);
                    pf.bytes_written += n as u64;
                    retries = 0;
                    cv.notify_all();
                }
                Err(e) => {
                    if retries >= MAX_RETRIES {
                        warn!(
                            "[netstream] prefetch read error after {} retries: {:?}",
                            retries, e
                        );
                        let (lock, cv) = &*pair;
                        lock.lock().unwrap().error = true;
                        cv.notify_all();
                        return;
                    }
                    retries += 1;
                    // Compute the byte offset the next Range request must start from:
                    // everything the reader has written is either still in the prefetch
                    // buffer or has been consumed by read_into; either way, the server
                    // must resume from start_offset + bytes_written.
                    let resume_offset = {
                        let (lock, _) = &*pair;
                        let pf = lock.lock().unwrap();
                        if pf.stop {
                            return;
                        }
                        start_offset + pf.bytes_written
                    };
                    warn!(
                        "[netstream] TCP error (attempt {}/{}), reconnecting at offset {}: {:?}",
                        retries, MAX_RETRIES, resume_offset, e
                    );
                    // Exponential back-off: 200ms × retry_count
                    thread::sleep(Duration::from_millis(200 * retries as u64));
                    {
                        let (lock, _) = &*pair;
                        if lock.lock().unwrap().stop {
                            return;
                        }
                    }
                    let range_header = format!("bytes={}-", resume_offset);
                    match CLIENT.get(&url).header("Range", range_header).send() {
                        Ok(resp) if resp.status().as_u16() == 206 => {
                            response = resp;
                        }
                        Ok(_) | Err(_) => {
                            // Server rejected Range or network down; retry loop continues.
                            debug!("[netstream] Range reconnect failed, will retry");
                        }
                    }
                }
            }
        }
    })
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

    fn new(url: String) -> Option<Self> {
        let response = CLIENT.get(&url).send().ok()?;
        if !response.status().is_success() {
            return None;
        }
        let content_length = response.content_length();
        let content_type = Self::response_content_type(&response);
        let pair = Arc::new((Mutex::new(Prefetch::new()), Condvar::new()));
        let t = spawn_reader(Arc::clone(&pair), url.clone(), 0, content_length, response);
        Some(StreamState {
            url,
            pos: 0,
            content_length,
            content_type,
            pair,
            _thread: Some(t),
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

    /// Read up to `n` bytes from the prefetch buffer into `dst`.
    /// Returns bytes copied (may be less than n), 0 on EOF, or -1 on error/timeout.
    /// Only blocks if the buffer is completely empty; returns whatever is available otherwise.
    unsafe fn read_into(&mut self, dst: *mut u8, n: usize) -> i64 {
        if n == 0 {
            return 0;
        }
        let (lock, cv) = &*self.pair;
        let mut pf = lock.lock().unwrap();
        loop {
            if !pf.data.is_empty() {
                break;
            }
            if pf.eof {
                return 0;
            }
            if pf.error {
                return -1;
            }
            if pf.stop {
                return 0;
            }
            // Wait up to 5 s per iteration so we re-check flags promptly when
            // spawn_reader detects a TCP drop and sets pf.error after MAX_RETRIES.
            // No hard failure on timeout — TCP stalls on mobile can last 30-90 s
            // before the OS surfaces an error; giving up at 30 s was the cause of
            // the "30 seconds then silence" symptom.
            let (new_pf, _) = cv.wait_timeout(pf, Duration::from_secs(5)).unwrap();
            pf = new_pf;
        }
        let to_copy = usize::min(n, pf.data.len());
        let dst_slice = std::slice::from_raw_parts_mut(dst, to_copy);
        let (front, back) = pf.data.as_slices();
        if front.len() >= to_copy {
            dst_slice.copy_from_slice(&front[..to_copy]);
        } else {
            let fl = front.len();
            dst_slice[..fl].copy_from_slice(front);
            dst_slice[fl..to_copy].copy_from_slice(&back[..to_copy - fl]);
        }
        pf.data.drain(..to_copy);
        self.pos += to_copy as u64;
        cv.notify_one();
        to_copy as i64
    }

    /// Drain `count` bytes from the prefetch buffer, waiting until they arrive.
    fn drain_prefetch(&self, count: usize) -> bool {
        if count == 0 {
            return true;
        }
        let (lock, cv) = &*self.pair;
        let mut remaining = count;
        let mut pf = lock.lock().unwrap();
        loop {
            let available = pf.data.len();
            if available >= remaining {
                pf.data.drain(..remaining);
                cv.notify_one();
                return true;
            }
            if pf.eof || pf.error || pf.stop {
                return false;
            }
            if available > 0 {
                pf.data.drain(..available);
                remaining -= available;
                cv.notify_one();
            }
            let (new_pf, timed_out) = cv.wait_timeout(pf, Duration::from_secs(30)).unwrap();
            pf = new_pf;
            if timed_out.timed_out() {
                return false;
            }
        }
    }

    /// Stop the current bg reader and start a new one from `response` at `offset`.
    fn replace_reader(&mut self, response: reqwest::blocking::Response, offset: u64) {
        {
            let (lock, cv) = &*self.pair;
            lock.lock().unwrap().stop = true;
            cv.notify_all();
        }
        let expected = self.content_length.map(|cl| cl.saturating_sub(offset));
        let new_pair = Arc::new((Mutex::new(Prefetch::new()), Condvar::new()));
        let new_thread =
            spawn_reader(Arc::clone(&new_pair), self.url.clone(), offset, expected, response);
        self.pair = new_pair;
        self._thread = Some(new_thread);
    }

    /// Flush the prefetch buffer and mark the stream at EOF.
    fn mark_eof(&self) {
        let (lock, cv) = &*self.pair;
        let mut pf = lock.lock().unwrap();
        pf.stop = true;
        pf.eof = true;
        pf.data.clear();
        cv.notify_all();
    }

    fn seek_to(&mut self, new_pos: u64) -> bool {
        if new_pos == self.pos {
            return true;
        }

        // Forward seek within prefetch capacity: drain buffered bytes.
        if new_pos > self.pos {
            let to_skip = (new_pos - self.pos) as usize;
            if to_skip <= PREFETCH_CAP {
                if self.drain_prefetch(to_skip) {
                    self.pos = new_pos;
                    return true;
                }
                // drain_prefetch failed (EOF/error/timeout); fall through to Range request.
            }
        }

        // Large forward seek or backward seek: issue a Range request.
        // M4A/MP4 codecs do this twice during init when moov is at the end of the file
        // (seek to end to read moov, then seek back to first audio frame). On LTE a
        // single failed request is enough to abort the track, so we retry up to 3 times.
        let range_header = match self.content_length {
            Some(cl) if cl > 0 => format!("bytes={}-{}", new_pos, cl - 1),
            _ => format!("bytes={}-", new_pos),
        };

        const SEEK_RETRIES: u32 = 3;
        for attempt in 0..SEEK_RETRIES {
            if attempt > 0 {
                thread::sleep(Duration::from_millis(300 * attempt as u64));
            }
            match CLIENT
                .get(&self.url)
                .header("Range", &range_header)
                .send()
            {
                Ok(resp) if resp.status().as_u16() == 206 => {
                    self.update_content_length_from_content_range(&resp);
                    if self.content_type.is_none() {
                        self.content_type = Self::response_content_type(&resp);
                    }
                    if Self::parse_content_range_start(&resp) != Some(new_pos) {
                        return false;
                    }
                    self.replace_reader(resp, new_pos);
                    self.pos = new_pos;
                    return true;
                }
                Ok(mut resp) if resp.status().is_success() => {
                    // Server returned 200 — it doesn't support Range. Only continue if
                    // new_pos is small enough to skip by reading and discarding; for a
                    // moov-at-end seek (tens of MB) this would download the whole file,
                    // so fail fast and let the codec give up rather than hanging.
                    if new_pos > 512 * 1024 {
                        warn!(
                            "[netstream] server returned 200 for Range request; \
                             seek to {} is too far to skip without Range support",
                            new_pos
                        );
                        return false;
                    }
                    if self.content_length.is_none() {
                        self.content_length = resp.content_length();
                    }
                    if self.content_type.is_none() {
                        self.content_type = Self::response_content_type(&resp);
                    }
                    if new_pos > 0 && !Self::skip_bytes(&mut resp, new_pos) {
                        return false;
                    }
                    self.replace_reader(resp, new_pos);
                    self.pos = new_pos;
                    return true;
                }
                Ok(resp) => {
                    warn!(
                        "[netstream] seek_to {}: server returned {} (attempt {}/{})",
                        new_pos,
                        resp.status(),
                        attempt + 1,
                        SEEK_RETRIES
                    );
                    return false;
                }
                Err(e) => {
                    warn!(
                        "[netstream] seek_to {}: network error (attempt {}/{}): {:?}",
                        new_pos,
                        attempt + 1,
                        SEEK_RETRIES,
                        e
                    );
                }
            }
        }
        warn!(
            "[netstream] seek_to {} failed after {} attempts",
            new_pos, SEEK_RETRIES
        );
        false
    }
}

static STREAMS: Lazy<Mutex<HashMap<i32, Arc<Mutex<StreamState>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

static CLIENT: Lazy<reqwest::blocking::Client> = Lazy::new(|| {
    reqwest::blocking::Client::builder()
        .use_rustls_tls()
        .connect_timeout(Duration::from_secs(15))
        .build()
        .expect("failed to build global HTTP client")
});

static NEXT_HANDLE: AtomicI32 = AtomicI32::new(0);

// ------------------------------------------------------------------
// Public C ABI
// ------------------------------------------------------------------

/// Open a URL and return an integer handle, or -1 on failure.
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
            let s = s.split('#').next().unwrap_or(s);
            s.to_owned()
        }
        Err(_) => return INVALID_HANDLE,
    };

    let state = match StreamState::new(url_str.clone()) {
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
    let result = state.read_into(dst as *mut u8, n);
    tracing::trace!(
        "[netstream] rb_net_read: h={} n={} pos_before={} -> read={} pos_after={}",
        h,
        n,
        pos_before,
        result,
        state.pos
    );
    result
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
    const MAX_SKIP: u64 = 256 * 1024 * 1024;
    if new_pos > state.pos && new_pos - state.pos > MAX_SKIP {
        warn!(
            "[netstream] rb_net_lseek: h={} off={} whence={} huge skip ({} bytes) clamped",
            h,
            off,
            whence,
            new_pos - state.pos
        );
        return -1;
    }

    // Guard 2: never seek past EOF.
    if let Some(cl) = state.content_length {
        if new_pos >= cl {
            warn!(
                "[netstream] rb_net_lseek: h={} off={} whence={} new_pos={} >= content_length={}, clamping to EOF",
                h, off, whence, new_pos, cl
            );
            state.pos = cl;
            state.mark_eof();
            return cl as i64;
        }
    }

    // Fast-path: already at target position.
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

// ------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Matcher;
    use std::ffi::CString;

    fn c_url(server: &mockito::Server, path: &str) -> CString {
        CString::new(format!("{}{}", server.url(), path)).unwrap()
    }

    // ------------------------------------------------------------------
    // Open / close
    // ------------------------------------------------------------------

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
        assert_eq!(
            rb_net_len(handle),
            -1,
            "closed handle should return -1 from rb_net_len"
        );
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
        assert_eq!(
            handle, INVALID_HANDLE,
            "unreachable server should return INVALID_HANDLE"
        );
    }

    #[test]
    fn test_open_404() {
        let mut server = mockito::Server::new();
        let _mock = server.mock("GET", "/missing.mp3").with_status(404).create();

        let url = c_url(&server, "/missing.mp3");
        let handle = unsafe { rb_net_open(url.as_ptr()) };
        assert_eq!(
            handle, INVALID_HANDLE,
            "404 response should return INVALID_HANDLE"
        );
    }

    // ------------------------------------------------------------------
    // Content-Length / rb_net_len
    // ------------------------------------------------------------------

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

        let content_type = unsafe { CStr::from_ptr(buf.as_ptr()) }.to_str().unwrap();
        assert_eq!(content_type, "audio/m4a");

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

        let len = rb_net_len(handle);
        assert_eq!(
            len,
            body.len() as i64,
            "rb_net_len should reflect the server's content-length"
        );

        rb_net_close(handle);
    }

    // ------------------------------------------------------------------
    // Reading
    // ------------------------------------------------------------------

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
        assert_eq!(result, -1, "read on unknown handle should return -1");
    }

    // ------------------------------------------------------------------
    // Seeking
    // ------------------------------------------------------------------

    #[test]
    fn test_seek_set_range_request() {
        let full_body: &[u8] = b"0123456789ABCDEF"; // 16 bytes
        let mut server = mockito::Server::new();

        let _initial = server
            .mock("GET", "/seekable.mp3")
            .match_header("range", Matcher::Missing)
            .with_status(200)
            .with_header("content-length", "16")
            .with_body(full_body)
            .create();

        let url = c_url(&server, "/seekable.mp3");
        let handle = unsafe { rb_net_open(url.as_ptr()) };
        assert!(handle >= 0);

        let new_pos = rb_net_lseek(handle, 8, libc::SEEK_SET);
        assert_eq!(new_pos, 8, "SEEK_SET(8) should return position 8");

        let mut buf = vec![0u8; 16];
        let n = unsafe { rb_net_read(handle, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
        assert_eq!(n, 8);
        assert_eq!(&buf[..8], &full_body[8..]);

        rb_net_close(handle);
    }

    #[test]
    fn test_seek_cur_no_op() {
        let body: &[u8] = b"ABCDEFGHIJ"; // 10 bytes
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
        assert_eq!(pos, 5, "SEEK_CUR(0) should return current position");

        rb_net_close(handle);
    }

    #[test]
    fn test_seek_end() {
        let full_body: &[u8] = b"XXXXXXXXXX"; // 10 bytes
        let mut server = mockito::Server::new();

        let _initial = server
            .mock("GET", "/end.mp3")
            .match_header("range", Matcher::Missing)
            .with_status(200)
            .with_header("content-length", "10")
            .with_body(full_body)
            .create();

        let url = c_url(&server, "/end.mp3");
        let handle = unsafe { rb_net_open(url.as_ptr()) };
        assert!(handle >= 0);

        let pos = rb_net_lseek(handle, -2, libc::SEEK_END);
        assert_eq!(
            pos, 8,
            "SEEK_END(-2) on 10-byte file should give position 8"
        );

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
        assert_eq!(
            pos, -1,
            "SEEK_END with offset beyond file start should return -1"
        );

        rb_net_close(handle);
    }

    /// When the server returns 416, a backward seek (which requires a Range
    /// request) fails gracefully.
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

        // Read 60 bytes to advance position past 50.
        let mut buf = vec![0u8; 60];
        let n = unsafe { rb_net_read(handle, buf.as_mut_ptr() as *mut libc::c_void, 60) };
        assert_eq!(n, 60);

        // Backward seek requires a Range request; server returns 416 → must fail.
        let result = rb_net_lseek(handle, 10, libc::SEEK_SET);
        assert_eq!(
            result, -1,
            "backward seek should fail when Range returns 416"
        );

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

        let url = c_url(&server, "/ignore-range.mp3");
        let handle = unsafe { rb_net_open(url.as_ptr()) };
        assert!(handle >= 0);

        let new_pos = rb_net_lseek(handle, 8, libc::SEEK_SET);
        assert_eq!(new_pos, 8, "seek should land at byte 8");

        let mut buf = vec![0u8; 16];
        let n = unsafe { rb_net_read(handle, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
        assert_eq!(n, 8);
        assert_eq!(&buf[..8], &full_body[8..]);

        rb_net_close(handle);
    }

    /// A malformed 206 response that does not start at the requested offset
    /// must fail instead of silently desynchronizing the stream position.
    /// This test requires a backward seek to trigger a Range request.
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
            .match_header("range", Matcher::Any)
            .with_status(206)
            .with_header("content-range", "bytes 0-7/16")
            .with_body(&full_body[..8])
            .create();

        let url = c_url(&server, "/bad-range.mp3");
        let handle = unsafe { rb_net_open(url.as_ptr()) };
        assert!(handle >= 0);

        // Read 10 bytes so we can seek backward (requires Range request).
        let mut buf = vec![0u8; 10];
        let n = unsafe { rb_net_read(handle, buf.as_mut_ptr() as *mut libc::c_void, 10) };
        assert_eq!(n, 10);

        // Backward seek triggers a Range request; server returns wrong Content-Range start.
        let result = rb_net_lseek(handle, 4, libc::SEEK_SET);
        assert_eq!(result, -1, "mismatched Content-Range should fail");

        rb_net_close(handle);
    }

    #[test]
    fn test_content_length_from_content_range() {
        let full_body: &[u8] = b"0123456789"; // 10 bytes
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

        assert_eq!(
            rb_net_len(handle),
            10,
            "length should be known from initial response"
        );

        let pos = rb_net_lseek(handle, 5, libc::SEEK_SET);
        assert_eq!(pos, 5, "seek should succeed");

        assert_eq!(
            rb_net_len(handle),
            10,
            "length should remain correct after seek"
        );

        rb_net_close(handle);
    }

    /// Seeking past content_length is clamped to content_length.
    /// Reads after clamping return 0 (EOF).
    #[test]
    fn test_seek_past_eof_is_clamped() {
        let full_body: &[u8] = b"0123456789"; // 10 bytes, content-length=10
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

        // Seek to exactly content_length (clamps to EOF).
        let result = rb_net_lseek(handle, 10, libc::SEEK_SET);
        assert_eq!(result, 10, "seek to content_length should clamp to EOF");

        let mut buf = vec![0u8; 16];
        let n = unsafe { rb_net_read(handle, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
        assert_eq!(n, 0, "read after clamped seek should return 0 (EOF)");

        rb_net_close(handle);
    }
}
