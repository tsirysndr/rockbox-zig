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
    // start_broker is the third entry point — apps/broker_thread.c::broker_init
    // spawns a kernel thread that calls into it. Same dead-code-strip risk
    // as start_server, so it gets the same keepalive treatment.
    fn start_broker();
}

/// `#[used]` keepalives: take the address of start_server / start_servers /
/// start_broker so the symbols themselves don't get GC'd at link time.
#[used]
static _KEEPALIVE_START_SERVER: unsafe extern "C" fn() = start_server;
#[used]
static _KEEPALIVE_START_SERVERS: unsafe extern "C" fn() = start_servers;
#[used]
static _KEEPALIVE_START_BROKER: unsafe extern "C" fn() = start_broker;

/// Force-pull rockbox-server's rlib into the link. `extern "C"` decls alone
/// don't do this — rustc treats them as external and waits for the linker
/// to satisfy them, which fails because rockbox-server's code was already
/// dead-code-stripped. Referencing any pub Rust item (a const here) makes
/// rustc include rockbox-server's compilation unit, which in turn provides
/// the start_server / start_servers symbols.
#[used]
static _KEEPALIVE_ROCKBOX_SERVER: &[&str] = &rockbox_server::AUDIO_EXTENSIONS;

/// Same trick for the PCM sink crates. Take the address of one actual
/// C-ABI export from each crate — `#[used]` on the empty `_link_<name>()`
/// helper isn't enough because rustc keeps just that one fn and GCs the
/// unrelated `pcm_<sink>_*` exports. Referencing a real C-ABI fn pulls
/// the entire crate's #[no_mangle] export set into the cdylib.
#[used]
static _KEEPALIVE_AIRPLAY: unsafe extern "C" fn(*const c_char, u16) =
    rockbox_airplay::pcm_airplay_set_host;
#[used]
static _KEEPALIVE_SLIM: extern "C" fn(u16) = rockbox_slim::pcm_squeezelite_set_slim_port;
#[used]
static _KEEPALIVE_CHROMECAST: unsafe extern "C" fn(*const c_char) =
    rockbox_chromecast::pcm::pcm_chromecast_set_device_host;
#[used]
static _KEEPALIVE_UPNP: extern "C" fn(u16) = rockbox_upnp::pcm_upnp_set_http_port;

/// `save_remote_track_metadata` — port of crates/cli's identical function
/// (we don't depend on rockbox-cli because of its host-only assumptions:
/// SIGTERM dance, getenv("HOME") panic, prctl). crates/server calls this
/// for HTTP-streamed tracks to index their metadata into the library DB.
#[no_mangle]
pub extern "C" fn save_remote_track_metadata(url: *const c_char) -> i32 {
    if url.is_null() {
        tracing::warn!("save_remote_track_metadata: null url");
        return -1;
    }
    let url = unsafe { std::ffi::CStr::from_ptr(url) };
    let url = match url.to_str() {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("save_remote_track_metadata: invalid utf-8: {}", e);
            return -1;
        }
    };

    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            tracing::error!(
                "save_remote_track_metadata: failed to create runtime: {}",
                e
            );
            return -1;
        }
    };

    match rt.block_on(async {
        let pool = rockbox_library::create_connection_pool().await?;
        rockbox_library::audio_scan::save_audio_metadata(pool, url, None).await
    }) {
        Ok(()) => 0,
        Err(e) => {
            tracing::error!("save_remote_track_metadata: {}", e);
            -1
        }
    }
}

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
    // Canonical music-dir env var read by crates/{settings,server,graphql,sys}.
    // ROCKBOX_MUSIC_DIR was a misnomer — nothing reads it. The browse
    // resolvers fall back to $HOME/Music when this is unset, which on Android
    // resolves to /data/.../files/Music (doesn't exist) → ENOENT.
    std::env::set_var("ROCKBOX_LIBRARY", music_dir);

    // Redirect anyone calling `std::env::temp_dir()` (e.g. the HTTP-stream
    // metadata probe in crates/library that writes
    // `rockbox-remote-probe-<md5>.<ext>` files) into the app sandbox.
    // Stdlib's temp_dir() honours $TMPDIR before falling back to /tmp,
    // and /tmp doesn't exist (or isn't writable) for non-root Android
    // app processes. The dir lives under HOME so it's persistent app
    // storage — fine for our short-lived probes that are removed via
    // RemoteProbeFile's Drop impl.
    let tmp = format!("{}/tmp", config_dir);
    let _ = std::fs::create_dir_all(&tmp);
    std::env::set_var("TMPDIR", &tmp);

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
    // Also install a panic hook that routes Rust panics to logcat — by
    // default a panic in a spawned thread produces no output anywhere
    // we can see.
    std::panic::set_hook(Box::new(|info| {
        tracing::error!(target: "rockbox", "Rust panic: {}", info);
    }));
    // Default to debug for our crates + info for the noisy 3rd-party ones.
    // Override at any time with `setprop log.tag.rockbox D` from adb shell,
    // or by setting RUST_LOG before the app launches (e.g. via the build).
    let default_filter = "info,rockbox_expo=debug,rockbox_server=debug,rockbox_rpc=debug,\
         rockbox_graphql=debug,rockbox_library=debug,rockbox_sys=debug,\
         rockbox_airplay=debug,rockbox_chromecast=debug,rockbox_slim=debug,\
         rockbox_upnp=debug";
    let _ = tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(default_filter)),
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
    tracing::info!("daemon: install_logcat_subscriber done");
    configure_environment(config_dir, music_dir, device_name);
    tracing::info!(
        "daemon: env configured (HOME={} MUSIC={} NAME={})",
        config_dir,
        music_dir,
        device_name
    );

    let port: u16 = std::env::var("ROCKBOX_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(6061);

    // Catch panics in the engine thread — they're otherwise silent and
    // we'd see "did not bind within 5s" with no idea why. Plus log a
    // checkpoint right before main_c() so we know if the C side was
    // even reached.
    tracing::info!("daemon: spawning rockbox-engine thread");
    thread::Builder::new()
        .name("rockbox-engine".into())
        .stack_size(2 * 1024 * 1024)
        .spawn(move || {
            tracing::info!("rockbox-engine: thread started, calling main_c()");
            let rc = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe { main_c() }));
            match rc {
                Ok(code) => tracing::warn!("rockbox-engine: main_c returned rc={}", code),
                Err(p) => tracing::error!(
                    "rockbox-engine: PANICKED — {:?}",
                    p.downcast_ref::<&str>()
                        .map(|s| *s)
                        .or_else(|| p.downcast_ref::<String>().map(|s| s.as_str()))
                        .unwrap_or("<non-string panic>")
                ),
            }
            STATE.store(STATE_STOPPED, Ordering::Release);
            LOCAL_PORT.store(0, Ordering::Release);
        })
        .expect("spawn rockbox-engine thread");

    tracing::info!("daemon: waiting up to 30s for gRPC :{} to bind", port);
    if !wait_for_grpc(port, Instant::now() + Duration::from_secs(30)) {
        tracing::error!(
            "daemon: gRPC server did not bind within 30s — engine stuck or crashed silently"
        );
        STATE.store(STATE_STOPPED, Ordering::Release);
        return -110;
    }

    LOCAL_PORT.store(port as i32, Ordering::Release);
    STATE.store(STATE_RUNNING, Ordering::Release);
    tracing::info!("daemon: started, gRPC bound :{port}");

    // Tell the in-process gRPC client to target our own daemon — but ONLY if
    // the JS layer hasn't already picked a server. The daemon takes ~30s to
    // bind; in that window `hydrateSelectedServer` typically runs and applies
    // the persisted remote URL (e.g. a LAN macOS daemon). Clobbering it here
    // silently flips the app back to localhost, which is the right default
    // only on a fresh install.
    let default_url = "http://127.0.0.1:6061";
    let already_overridden = crate::SERVER_URL
        .read()
        .map(|g| g.as_str() != default_url)
        .unwrap_or(false);
    if already_overridden {
        tracing::info!(
            "daemon: SERVER_URL already set by JS — leaving it (in-process daemon still bound :{port})"
        );
    } else {
        let url = format!("http://127.0.0.1:{port}\0");
        let url_c = url.as_ptr() as *const c_char;
        let _ = crate::rb_set_server_url(url_c);
    }

    let default_http = "http://127.0.0.1:6063";
    let http_overridden = crate::HTTP_URL
        .read()
        .map(|g| g.as_str() != default_http)
        .unwrap_or(false);
    if !http_overridden {
        let http_port = std::env::var("ROCKBOX_TCP_PORT").unwrap_or_else(|_| "6063".into());
        let http_url = format!("http://127.0.0.1:{http_port}\0");
        let http_url_c = http_url.as_ptr() as *const c_char;
        let _ = crate::rb_set_http_url(http_url_c);
    }

    spawn_library_scan(/* force */ false);

    port as i32
}

/// Re-scan trigger callable from JS. Forces a full rescan of `$ROCKBOX_LIBRARY`
/// regardless of how many tracks are already indexed. Returns 0 immediately
/// (the scan runs in the background — watch logcat for "scan: ..." lines).
/// Returns -1 if the daemon isn't running.
#[no_mangle]
pub extern "C" fn rb_rescan_library() -> c_int {
    if STATE.load(Ordering::Acquire) != STATE_RUNNING {
        return -1;
    }
    spawn_library_scan(/* force */ true);
    0
}

/// Mirror what the desktop `crates/cli` does at boot: open the library DB
/// and run `scan_audio_files($ROCKBOX_LIBRARY)`. With `force=false` we skip
/// the scan when the DB already has tracks (startup path); `force=true`
/// always scans (manual re-scan / `ROCKBOX_UPDATE_LIBRARY=1`).
///
/// Spawned on its own OS thread + tokio current-thread runtime so we don't
/// block the daemon boot path or the JNI caller.
fn spawn_library_scan(force_arg: bool) {
    thread::Builder::new()
        .name("rockbox-library-scan".into())
        .stack_size(2 * 1024 * 1024)
        .spawn(move || {
            let path = std::env::var("ROCKBOX_LIBRARY").unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
                format!("{}/Music", home)
            });
            let force = force_arg
                || matches!(
                    std::env::var("ROCKBOX_UPDATE_LIBRARY").as_deref(),
                    Ok("1") | Ok("true")
                );
            tracing::info!("scan: target={} force={}", path, force);

            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(e) => {
                    tracing::error!("scan: tokio runtime build failed: {e}");
                    return;
                }
            };

            rt.block_on(async move {
                let pool = match rockbox_library::create_connection_pool().await {
                    Ok(p) => p,
                    Err(e) => {
                        tracing::error!("scan: open library DB failed: {e}");
                        return;
                    }
                };
                let count = rockbox_library::repo::track::all(pool.clone())
                    .await
                    .map(|t| t.len())
                    .unwrap_or(0);
                if count > 0 && !force {
                    tracing::info!(
                        "scan: library has {} tracks, skipping file scan (force=false)",
                        count
                    );
                } else {
                    tracing::info!("scan: scanning {} ...", path);
                    match rockbox_library::audio_scan::scan_audio_files(pool.clone(), path.into())
                        .await
                    {
                        Ok(files) => tracing::info!("scan: done, {} files", files.len()),
                        Err(e) => tracing::error!("scan: failed: {e}"),
                    }
                }

                // Rocksky enrichment runs on every boot (not just after a file
                // scan) so that artists/genres get populated even when the
                // library DB already has tracks. update_metadata() filters to
                // artists with no image, so it's cheap when everything is
                // already enriched. Uses webpki-roots so TLS works on Android
                // without relying on the system cert store.
                if let Err(e) = rockbox_library::artists::update_metadata(pool.clone()).await {
                    tracing::warn!("scan: rocksky enrichment skipped: {e}");
                } else {
                    tracing::info!("scan: rocksky enrichment done");
                }

                // Signal gRPC StreamLibrary subscribers so the UI can
                // invalidate and refetch all library queries (tracks, artists,
                // albums, playlists, genres).
                rockbox_graphql::simplebroker::SimpleBroker::publish(
                    rockbox_graphql::types::ScanCompleted,
                );
                tracing::info!("scan: ScanCompleted published");
            });
        })
        .expect("spawn rockbox-library-scan thread");
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
