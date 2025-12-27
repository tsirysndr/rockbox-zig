use libc::off_t;
use reqwest::blocking::{Client, Response};
use reqwest::header::{ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_RANGE, LOCATION, RANGE};
use std::ffi::{c_char, CStr};
use std::io::Read;

const READAHEAD_SIZE: usize = 64 * 1024; // 64 KB readahead

#[repr(C)]
pub struct HttpStream {
    client: Client,
    url: String,

    // stream state
    pos: u64,
    length: Option<u64>,
    accepts_ranges: bool,

    // readahead buffer
    buffer: Vec<u8>,
    buf_pos: usize,
}

impl HttpStream {
    fn new(url: &str) -> Result<Self, ()> {
        // reqwest can follow redirects automatically; keep it sane.
        let client = Client::builder()
            .redirect(reqwest::redirect::Policy::limited(10))
            .build()
            .map_err(|_| ())?;

        Ok(Self {
            client,
            url: url.to_string(),
            pos: 0,
            length: None,
            accepts_ranges: false,
            buffer: Vec::with_capacity(READAHEAD_SIZE),
            buf_pos: 0,
        })
    }

    fn connect(&mut self) -> Result<(), ()> {
        // Prefer HEAD to discover length / accept-ranges; fall back to a tiny GET.
        let head = self.client.head(&self.url).send();
        if let Ok(resp) = head {
            self.ingest_metadata_from_response(&resp);
            return Ok(());
        }

        let mut resp = self.client.get(&self.url).send().map_err(|_| ())?;
        self.ingest_metadata_from_response(&resp);

        // Drain nothing (we only needed headers), but ensure body isn't left unread in some impls.
        let mut sink = [0u8; 1];
        let _ = resp.read(&mut sink);

        Ok(())
    }

    fn ingest_metadata_from_response(&mut self, resp: &Response) {
        // Accept-Ranges: bytes
        self.accepts_ranges = resp
            .headers()
            .get(ACCEPT_RANGES)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_ascii_lowercase().contains("bytes"))
            .unwrap_or(false);

        // Content-Length
        self.length = resp
            .headers()
            .get(CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok());

        // If server responds with Content-Range on some requests, total might be there:
        // e.g. "bytes 123-456/789"
        if let Some(total) = resp
            .headers()
            .get(CONTENT_RANGE)
            .and_then(|v| v.to_str().ok())
            .and_then(parse_total_from_content_range)
        {
            self.length = Some(total);
        }

        // NOTE: Redirect handling is done by reqwest policy; LOCATION is here if you ever
        // decide to switch to manual redirect handling.
        let _ = resp.headers().get(LOCATION);
    }

    fn refill_buffer(&mut self) -> Result<(), ()> {
        self.buffer.clear();
        self.buf_pos = 0;

        // Request a bounded range if possible (better for latency / memory)
        let start = self.pos;
        let end = start.saturating_add((READAHEAD_SIZE as u64).saturating_sub(1));

        let mut req = self.client.get(&self.url);

        if self.accepts_ranges || start > 0 {
            // Ask for a range; many servers accept this even if Accept-Ranges isn't advertised.
            req = req.header(RANGE, format!("bytes={}-{}", start, end));
        }

        let resp = req.send().map_err(|_| ())?;

        // Update metadata if this response reveals it (e.g., Content-Range total)
        self.ingest_metadata_from_response(&resp);

        // Read up to READAHEAD_SIZE into the buffer
        let mut limited = resp.take(READAHEAD_SIZE as u64);
        let mut tmp = vec![0u8; READAHEAD_SIZE];
        let n = limited.read(&mut tmp).map_err(|_| ())?;
        tmp.truncate(n);

        self.buffer = tmp;
        Ok(())
    }

    pub fn read(&mut self, dest: &mut [u8]) -> Result<usize, ()> {
        let mut written = 0;

        while written < dest.len() {
            if self.buf_pos >= self.buffer.len() {
                self.refill_buffer()?;
                if self.buffer.is_empty() {
                    return Ok(written); // EOF
                }
            }

            let avail = self.buffer.len() - self.buf_pos;
            let to_copy = (dest.len() - written).min(avail);

            dest[written..written + to_copy]
                .copy_from_slice(&self.buffer[self.buf_pos..self.buf_pos + to_copy]);

            self.buf_pos += to_copy;
            self.pos += to_copy as u64;
            written += to_copy;
        }

        Ok(written)
    }

    pub fn seek(&mut self, pos: u64) -> Result<u64, ()> {
        // If server truly doesn't support ranges, seeking is not reliable unless pos == current
        if !self.accepts_ranges && pos != self.pos {
            return Err(());
        }

        self.pos = pos;
        self.buffer.clear();
        self.buf_pos = 0;
        Ok(pos)
    }
}

fn parse_total_from_content_range(s: &str) -> Option<u64> {
    // "bytes 123-456/789" -> 789
    let (_, rest) = s.split_once('/')?;
    rest.trim().parse::<u64>().ok()
}

// --------------------
// FFI
// --------------------

#[no_mangle]
pub extern "C" fn http_stream_open(url: *const c_char) -> *mut HttpStream {
    if url.is_null() {
        return std::ptr::null_mut();
    }

    let c_str = unsafe { CStr::from_ptr(url) };
    let Ok(url_str) = c_str.to_str() else {
        return std::ptr::null_mut();
    };

    let Ok(mut stream) = HttpStream::new(url_str) else {
        return std::ptr::null_mut();
    };

    if stream.connect().is_err() {
        return std::ptr::null_mut();
    }

    Box::into_raw(Box::new(stream))
}

#[no_mangle]
pub extern "C" fn http_stream_read(handle: *mut HttpStream, buf: *mut u8, len: usize) -> isize {
    if handle.is_null() || buf.is_null() || len == 0 {
        return -1;
    }

    let stream = unsafe { &mut *handle };
    let out = unsafe { std::slice::from_raw_parts_mut(buf, len) };

    match stream.read(out) {
        Ok(n) => n as isize,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn http_stream_lseek(handle: *mut HttpStream, offset: off_t, whence: i32) -> off_t {
    if handle.is_null() {
        return -1;
    }
    let stream = unsafe { &mut *handle };

    // Use signed math so negative offsets behave correctly.
    let cur = stream.pos as i128;
    let off = offset as i128;

    let new_pos_i128: i128 = match whence {
        0 => off,       // SEEK_SET
        1 => cur + off, // SEEK_CUR
        2 => {
            let len = match stream.length {
                Some(l) => l as i128,
                None => return -1,
            };
            len + off // SEEK_END: offset is typically negative or 0
        }
        _ => return -1,
    };

    if new_pos_i128 < 0 {
        return -1;
    }

    let new_pos = new_pos_i128 as u64;
    match stream.seek(new_pos) {
        Ok(p) => p as off_t,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn http_stream_filesize(handle: *mut HttpStream) -> off_t {
    if handle.is_null() {
        return -1;
    }
    let stream = unsafe { &*handle };
    stream.length.map(|l| l as off_t).unwrap_or(-1)
}

#[no_mangle]
pub extern "C" fn http_stream_close(handle: *mut HttpStream) -> i32 {
    if handle.is_null() {
        return -1;
    }
    unsafe {
        drop(Box::from_raw(handle));
    }
    0
}
