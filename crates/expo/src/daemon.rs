//! Embedded rockbox daemon — Android cdylib only.
//!
//! Boots the C firmware (apps/main.c::main_c) on a dedicated pthread.
//! The firmware spawns its own Rockbox kernel threads, one of which runs
//! `crates/server::start_server` and binds gRPC on 127.0.0.1:6061.
//! From there the existing tonic client (rb_play, rb_pause, …) targets
//! that local port and audio plays out via AAudio.
//!
//! The mere existence of `extern fn main_c()` here is what keeps the C
//! firmware code from being --gc-sections'd out of the final cdylib.

use std::ffi::CStr;
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::os::raw::{c_char, c_int};
use std::sync::atomic::{AtomicI32, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use tracing_subscriber::prelude::*;

// State machine. Distinct atomic values rather than a Mutex<enum> so JNI
// callers can cheap-read from any thread.
const STATE_STOPPED: i32 = 0;
const STATE_STARTING: i32 = 1;
const STATE_RUNNING: i32 = 2;

static STATE: AtomicI32 = AtomicI32::new(STATE_STOPPED);
static LOCAL_PORT: AtomicI32 = AtomicI32::new(0);

// The firmware boot is `main_c(void)` in apps/main.c, gated by ZIG_APP
// (which the androidcdylibcc helper sets in extradefines). Same entry
// the desktop Zig wrapper at zig/src/main.zig uses.
extern "C" {
    fn main_c() -> c_int;
    // start_server / start_servers are extern "C" fns in crates/server.
    // We don't call them from Rust (the C firmware does, from
    // apps/server_thread.c::server_init), but the references below force
    // rustc to keep the rockbox-server rlib in the cdylib link — without
    // them rustc dead-code-strips the entire crate, taking with it the
    // _netstream keepalive trick that exports rb_net_open / rb_net_read /
    // etc. from rbnetstream. The C side then can't find those symbols
    // and dlopen fails at runtime.
    fn start_server();
    fn start_servers();
}

/// `#[used]` keepalives: take the address of start_server / start_servers
/// so the symbols themselves don't get GC'd at link time.
#[used]
static _KEEPALIVE_START_SERVER: unsafe extern "C" fn() = start_server;
#[used]
static _KEEPALIVE_START_SERVERS: unsafe extern "C" fn() = start_servers;

/// Force-pull rockbox-server's rlib into the link. `extern "C"` decls alone
/// don't do this — rustc treats them as external and waits for the linker
/// to satisfy them, which fails because rockbox-server's code was already
/// dead-code-stripped. Referencing any pub Rust item (a const here) makes
/// rustc include rockbox-server's compilation unit, which in turn provides
/// the start_server / start_servers symbols.
#[used]
static _KEEPALIVE_ROCKBOX_SERVER: &[&str] = &rockbox_server::AUDIO_EXTENSIONS;

/// Same keepalive trick for the netstream Rust crate's C-ABI exports —
/// the C firmware's streamfd.c calls rb_net_open / rb_net_read / etc., but
/// rustc dead-code-strips them from the cdylib link unless we reference
/// them from our own (the cdylib's) Rust code. rockbox-server already
/// has its own keepalive mod, but it's in an rlib and `#[used]` doesn't
/// propagate across rlib boundaries.
#[used]
static _KEEPALIVE_RB_NET_OPEN: unsafe extern "C" fn(*const c_char) -> i32 =
    rbnetstream::rb_net_open;
#[used]
static _KEEPALIVE_RB_NET_READ: unsafe extern "C" fn(i32, *mut std::ffi::c_void, usize) -> i64 =
    rbnetstream::rb_net_read;
#[used]
static _KEEPALIVE_RB_NET_LEN: extern "C" fn(i32) -> i64 = rbnetstream::rb_net_len;
#[used]
static _KEEPALIVE_RB_NET_LSEEK: extern "C" fn(i32, i64, i32) -> i64 = rbnetstream::rb_net_lseek;
#[used]
static _KEEPALIVE_RB_NET_CLOSE: extern "C" fn(i32) = rbnetstream::rb_net_close;

unsafe fn cstr_to_str<'a>(p: *const c_char) -> Option<&'a str> {
    if p.is_null() {
        return None;
    }
    CStr::from_ptr(p).to_str().ok()
}

fn configure_environment(config_dir: &str, music_dir: &str, device_name: &str) {
    // Safety: env::set_var is only safe before any other thread that reads
    // env exists. We're called from JNI before the engine pthread spawns.
    std::env::set_var("HOME", config_dir);
    std::env::set_var("ROCKBOX_DEVICE_NAME", device_name);
    std::env::set_var("ROCKBOX_MUSIC_DIR", music_dir);

    // mDNS-advertised LAN ports (match crates/discovery defaults).
    if std::env::var_os("ROCKBOX_PORT").is_none() {
        std::env::set_var("ROCKBOX_PORT", "6061");
    }
    if std::env::var_os("ROCKBOX_GRAPHQL_PORT").is_none() {
        std::env::set_var("ROCKBOX_GRAPHQL_PORT", "6062");
    }
    if std::env::var_os("ROCKBOX_TCP_PORT").is_none() {
        std::env::set_var("ROCKBOX_TCP_PORT", "6063");
    }
    if std::env::var_os("ROCKBOX_MPD_PORT").is_none() {
        std::env::set_var("ROCKBOX_MPD_PORT", "6600");
    }
}

fn wait_for_grpc(port: u16, deadline: Instant) -> bool {
    let addr: SocketAddr = (Ipv4Addr::LOCALHOST, port).into();
    while Instant::now() < deadline {
        if TcpStream::connect_timeout(&addr, Duration::from_millis(50)).is_ok() {
            return true;
        }
        thread::sleep(Duration::from_millis(50));
    }
    false
}

fn install_logcat_subscriber() {
    // Idempotent: try_init returns Err on second call, which we ignore.
    let _ = tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with(tracing_android::layer("rockbox").expect("init logcat layer"))
        .try_init();
}

/// Boot the embedded rockbox daemon. Returns the gRPC port (positive)
/// on success, or a negative error code:
///   -22  invalid input (null/non-UTF8 string)
///   -110 timeout (firmware did not bind gRPC port within 5s)
///   -114 already starting / running
///
/// # Safety
/// Pointers must be NUL-terminated UTF-8 strings.
#[no_mangle]
pub unsafe extern "C" fn rb_daemon_start(
    config_dir_ptr: *const c_char,
    music_dir_ptr: *const c_char,
    device_name_ptr: *const c_char,
) -> c_int {
    let Some(config_dir) = cstr_to_str(config_dir_ptr) else {
        return -22;
    };
    let Some(music_dir) = cstr_to_str(music_dir_ptr) else {
        return -22;
    };
    let Some(device_name) = cstr_to_str(device_name_ptr) else {
        return -22;
    };

    if STATE
        .compare_exchange(
            STATE_STOPPED,
            STATE_STARTING,
            Ordering::AcqRel,
            Ordering::Acquire,
        )
        .is_err()
    {
        return -114;
    }

    install_logcat_subscriber();
    configure_environment(config_dir, music_dir, device_name);

    let port: u16 = std::env::var("ROCKBOX_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(6061);

    // Boot the firmware on a dedicated pthread. apps/main.c::main_c never
    // returns under normal operation; if it does (panic, exit), we transition
    // back to STOPPED so the next start works.
    thread::Builder::new()
        .name("rockbox-engine".into())
        .stack_size(2 * 1024 * 1024)
        .spawn(move || {
            let rc = unsafe { main_c() };
            tracing::warn!("rockbox engine exited rc={}", rc);
            STATE.store(STATE_STOPPED, Ordering::Release);
            LOCAL_PORT.store(0, Ordering::Release);
        })
        .expect("spawn rockbox-engine thread");

    if !wait_for_grpc(port, Instant::now() + Duration::from_secs(5)) {
        tracing::error!("daemon: gRPC server did not bind within 5s");
        STATE.store(STATE_STOPPED, Ordering::Release);
        return -110;
    }

    LOCAL_PORT.store(port as i32, Ordering::Release);
    STATE.store(STATE_RUNNING, Ordering::Release);
    tracing::info!("daemon: started, gRPC bound :{port}");

    // Tell the in-process gRPC client to target our own daemon.
    let url = format!("http://127.0.0.1:{port}\0");
    let url_c = url.as_ptr() as *const c_char;
    let _ = crate::rb_set_server_url(url_c);

    let http_port = std::env::var("ROCKBOX_TCP_PORT").unwrap_or_else(|_| "6063".into());
    let http_url = format!("http://127.0.0.1:{http_port}\0");
    let http_url_c = http_url.as_ptr() as *const c_char;
    let _ = crate::rb_set_http_url(http_url_c);

    port as i32
}

/// Returns the gRPC port of the running daemon, or 0 if not running.
#[no_mangle]
pub extern "C" fn rb_daemon_port() -> c_int {
    LOCAL_PORT.load(Ordering::Acquire)
}

/// Returns the daemon state: 0=stopped, 1=starting, 2=running.
#[no_mangle]
pub extern "C" fn rb_daemon_state() -> c_int {
    STATE.load(Ordering::Acquire)
}
