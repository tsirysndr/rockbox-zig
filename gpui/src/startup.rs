use std::net::TcpStream;
use std::os::raw::{c_char, c_int};
use std::time::Duration;

const LOCALHOST_GRPC: &str = "127.0.0.1:6061";
const CONNECT_TIMEOUT: Duration = Duration::from_millis(500);

// C ABI from librockboxd.a (built by `cd zig && zig build lib`).
extern "C" {
    pub fn rb_daemon_start(music_dir_ptr: *const c_char, device_name_ptr: *const c_char)
        -> c_int;
    pub fn rb_daemon_stop() -> c_int;
    pub fn rb_daemon_port() -> c_int;
    pub fn rb_daemon_state() -> c_int;
}

/// Returns true if gRPC port 6061 is already accepting connections.
pub fn is_running() -> bool {
    TcpStream::connect_timeout(&LOCALHOST_GRPC.parse().unwrap(), CONNECT_TIMEOUT).is_ok()
}

/// Ensure the daemon is running.
///
/// If port 6061 already responds, returns the port immediately without
/// starting an embedded daemon (useful when a standalone `rockboxd` or a
/// remote instance is accessible on localhost).
///
/// Otherwise boots the embedded daemon (headless/cpal build) in-process.
/// Blocks up to 30 s waiting for gRPC to bind, then returns the port
/// (positive) or a negative error code:
///   -110  timeout — firmware did not bind within 30 s
///   -114  already starting / running
pub fn ensure_running() -> i32 {
    if is_running() {
        return 6061;
    }
    let device_name = b"Rockbox Desktop\0".as_ptr() as *const c_char;
    unsafe { rb_daemon_start(std::ptr::null(), device_name) }
}
