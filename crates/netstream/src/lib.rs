use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::ffi::CStr;
use std::io::Read;
use std::os::raw::c_char;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Mutex;

/// Sentinel handle ID returned on error.
const INVALID_HANDLE: i32 = -1;

/// Per-stream state.
struct StreamState {
    url: String,
    pos: u64,
    content_length: Option<u64>,
    response: Option<reqwest::blocking::Response>,
    client: reqwest::blocking::Client,
}

impl StreamState {
    fn new(url: String, client: reqwest::blocking::Client) -> Option<Self> {
        let response = client.get(&url).send().ok()?;
        if !response.status().is_success() {
            return None;
        }
        let content_length = response.content_length();
        Some(StreamState {
            url,
            pos: 0,
            content_length,
            response: Some(response),
            client,
        })
    }

    /// Re-issue the request starting at `new_pos` using an HTTP Range header.
    /// Returns `true` on success, `false` if the server doesn't support Range
    /// or if the request fails.
    fn seek_to(&mut self, new_pos: u64) -> bool {
        self.response = None;
        let result = self
            .client
            .get(&self.url)
            .header("Range", format!("bytes={}-", new_pos))
            .send();

        match result {
            Ok(resp)
                if resp.status().is_success() || resp.status().as_u16() == 206 =>
            {
                // Try to extract total length from Content-Range if not yet known.
                if self.content_length.is_none() {
                    if let Some(cr) = resp.headers().get("content-range") {
                        if let Ok(cr_str) = cr.to_str() {
                            // Format: "bytes START-END/TOTAL"
                            if let Some(total_str) = cr_str.split('/').last() {
                                if let Ok(total) = total_str.trim().parse::<u64>() {
                                    self.content_length = Some(total);
                                }
                            }
                        }
                    }
                }
                self.response = Some(resp);
                self.pos = new_pos;
                true
            }
            _ => false,
        }
    }
}

static STREAMS: Lazy<Mutex<HashMap<i32, StreamState>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

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
        Ok(s) => s.to_owned(),
        Err(_) => return INVALID_HANDLE,
    };

    let client = match reqwest::blocking::Client::builder()
        .use_rustls_tls()
        .build()
    {
        Ok(c) => c,
        Err(_) => return INVALID_HANDLE,
    };

    let state = match StreamState::new(url_str, client) {
        Some(s) => s,
        None => return INVALID_HANDLE,
    };

    let handle = NEXT_HANDLE.fetch_add(1, Ordering::SeqCst);
    STREAMS.lock().unwrap().insert(handle, state);
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
    let mut streams = STREAMS.lock().unwrap();
    let state = match streams.get_mut(&h) {
        Some(s) => s,
        None => return -1,
    };
    let resp = match &mut state.response {
        Some(r) => r,
        None => return -1,
    };
    let buf = std::slice::from_raw_parts_mut(dst as *mut u8, n);
    match resp.read(buf) {
        Ok(bytes_read) => {
            state.pos += bytes_read as u64;
            bytes_read as i64
        }
        Err(_) => -1,
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

    let mut streams = STREAMS.lock().unwrap();
    let state = match streams.get_mut(&h) {
        Some(s) => s,
        None => return -1,
    };

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

    // Fast-path: already there (no need to restart the request).
    if new_pos == state.pos {
        return state.pos as i64;
    }

    if state.seek_to(new_pos) {
        state.pos as i64
    } else {
        -1
    }
}

/// Return the total content length of stream `h`, or -1 if unknown.
#[no_mangle]
pub extern "C" fn rb_net_len(h: i32) -> i64 {
    let streams = STREAMS.lock().unwrap();
    match streams.get(&h) {
        Some(state) => state.content_length.map(|l| l as i64).unwrap_or(-1),
        None => -1,
    }
}

/// Close stream `h` and release its resources.
#[no_mangle]
pub extern "C" fn rb_net_close(h: i32) {
    STREAMS.lock().unwrap().remove(&h);
}
