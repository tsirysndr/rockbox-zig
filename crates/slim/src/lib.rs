mod http;
mod slimproto;

// Called from rockbox-cli to force this crate's symbols into librockbox_cli.a
#[doc(hidden)]
pub fn _link_slim() {}

use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex, OnceLock};

// ---------------------------------------------------------------------------
// Broadcast buffer — one writer, N independent readers.
//
// Each chunk is stored with a monotonically-increasing sequence number.
// Every reader (one per squeezelite HTTP connection) keeps its own
// `next_seq` cursor and reads chunks independently.  Old chunks are evicted
// once the buffer exceeds MAX_BUFFERED bytes; a lagging reader skips forward
// to the oldest available chunk rather than blocking the writer.
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
    chunks: VecDeque<(u64, Vec<u8>)>, // (seq, payload)
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

    /// Subscribe from the current write position (live stream, no old data).
    pub(crate) fn subscribe(self: &Arc<Self>) -> BroadcastReceiver {
        let next_seq = self.inner.lock().unwrap().next_seq;
        BroadcastReceiver { buf: Arc::clone(self), next_seq }
    }

    fn reset(&self) {
        let mut g = self.inner.lock().unwrap();
        g.chunks.clear();
        g.total_bytes = 0;
        g.closed = false;
        // next_seq is NOT reset — existing receivers skip forward automatically.
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
                // Lagging reader: skip to oldest available chunk.
                if self.next_seq < front_seq {
                    tracing::debug!(
                        "slim/broadcast: receiver lagging, skipping {} → {}",
                        self.next_seq, front_seq
                    );
                    self.next_seq = front_seq;
                }
                // Data is available for this reader.
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
static STARTED: Mutex<bool> = Mutex::new(false);

struct SlimConfig {
    slim_port: u16,
    http_port: u16,
}

static CONFIG: Mutex<SlimConfig> = Mutex::new(SlimConfig {
    slim_port: 3483,
    http_port: 9999,
});

fn get_buffer() -> Arc<BroadcastBuffer> {
    BUFFER.get_or_init(|| Arc::new(BroadcastBuffer::new())).clone()
}

// ---------------------------------------------------------------------------
// FFI exports
// ---------------------------------------------------------------------------

#[no_mangle]
pub extern "C" fn pcm_squeezelite_set_slim_port(port: u16) {
    CONFIG.lock().unwrap().slim_port = port;
}

#[no_mangle]
pub extern "C" fn pcm_squeezelite_set_http_port(port: u16) {
    CONFIG.lock().unwrap().http_port = port;
}

/// Start Slim Protocol + HTTP servers. Idempotent.
#[no_mangle]
pub extern "C" fn pcm_squeezelite_start() -> std::os::raw::c_int {
    let mut started = STARTED.lock().unwrap();
    if *started {
        return 0;
    }

    let cfg = CONFIG.lock().unwrap();
    let slim_port = cfg.slim_port;
    let http_port = cfg.http_port;
    drop(cfg);

    let buf = get_buffer();
    buf.reset();

    let buf_http = buf.clone();
    std::thread::spawn(move || http::serve(http_port, buf_http));
    std::thread::spawn(move || slimproto::serve(slim_port, http_port));

    *started = true;
    tracing::info!(
        "squeezelite sink: Slim Protocol on :{slim_port}, HTTP audio on :{http_port}"
    );
    0
}

/// Push raw S16LE stereo PCM into the broadcast buffer.
#[no_mangle]
pub extern "C" fn pcm_squeezelite_write(data: *const u8, len: usize) -> std::os::raw::c_int {
    if data.is_null() || len == 0 {
        return 0;
    }
    let slice = unsafe { std::slice::from_raw_parts(data, len) };
    get_buffer().push(slice);
    0
}

/// No-op between tracks — all squeezelite clients keep their HTTP connections.
#[no_mangle]
pub extern "C" fn pcm_squeezelite_stop() {}

/// Shut down servers (called on daemon exit).
#[no_mangle]
pub extern "C" fn pcm_squeezelite_close() {
    let mut started = STARTED.lock().unwrap();
    get_buffer().close();
    *started = false;
}
