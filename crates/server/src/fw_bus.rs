//! Firmware-command bus.
//!
//! On the Android cdylib (and any hosted-pthread build), Rockbox kernel
//! state is identified via the global `__cores[0].running` slot — there is
//! no thread-local-storage involved. Calling firmware kernel functions
//! (`audio_play`, `audio_set_crossfade`, `pcmbuf_*`, anything that
//! eventually reaches `queue_send` / `wakeup_thread`) from a non-Rockbox
//! pthread (e.g. an actix worker handling a gRPC request) reads/writes
//! the wrong thread_entry and corrupts the kernel scheduler. Symptoms:
//! SIGSEGV at PC=0 in `wakeup_thread_` on track switches or settings
//! changes, intermittent kernel coroutine corruption.
//!
//! Solution: serialise every kernel-affecting call through a single
//! mpsc channel that the **broker** thread drains. The broker is a real
//! Rockbox kernel thread (created via `create_thread` from
//! `apps/broker_thread.c`), so its calls into firmware run with a sane
//! `__running_self_entry()`.
//!
//! Synchronous handlers wait on a oneshot reply so they keep their
//! original signatures from the caller's POV.

use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::Mutex;
use std::sync::OnceLock;

/// Discriminated union of every firmware-mutating call we need to route
/// through the broker. Add a variant when a new handler proves to need it.
pub enum FwCmd {
    Play {
        elapsed: i64,
        offset: i64,
        reply: Option<mpsc::SyncSender<()>>,
    },
    Pause {
        reply: Option<mpsc::SyncSender<()>>,
    },
    Resume {
        reply: Option<mpsc::SyncSender<()>>,
    },
    Next {
        reply: Option<mpsc::SyncSender<()>>,
    },
    Prev {
        reply: Option<mpsc::SyncSender<()>>,
    },
    Stop {
        reply: Option<mpsc::SyncSender<()>>,
    },
    FfRewind {
        newtime: i32,
        reply: Option<mpsc::SyncSender<()>>,
    },
    FlushAndReloadTracks {
        reply: Option<mpsc::SyncSender<()>>,
    },
    SetCrossfade {
        value: i32,
        reply: Option<mpsc::SyncSender<()>>,
    },
    /// Escape hatch for anything not covered yet. Closure runs on the
    /// broker thread; sender is responsible for not panicking inside.
    Custom(Box<dyn FnOnce() + Send + 'static>),
}

static SENDER: OnceLock<Sender<FwCmd>> = OnceLock::new();
// Receiver is wrapped in a Mutex<Option> so the broker can `take()` it on
// startup; only one broker should ever exist.
static RECEIVER: OnceLock<Mutex<Option<Receiver<FwCmd>>>> = OnceLock::new();

/// Initialise the channel. Call once at startup, BEFORE the broker thread
/// is spawned and BEFORE any handler tries to `send()`. Idempotent.
pub fn init() {
    SENDER.get_or_init(|| {
        let (tx, rx) = mpsc::channel();
        RECEIVER
            .set(Mutex::new(Some(rx)))
            .unwrap_or_else(|_| panic!("fw_bus::init called twice"));
        tx
    });
}

/// The broker thread takes ownership of the receiver on its first iteration.
/// Returns None if init() wasn't called or the receiver was already taken.
pub fn take_receiver() -> Option<Receiver<FwCmd>> {
    RECEIVER.get()?.lock().ok()?.take()
}

/// Enqueue a command for the broker to execute on its next tick. Drops
/// silently if the bus isn't initialised (e.g. desktop / non-cdylib build);
/// callers should treat it as a fire-and-forget side effect.
pub fn send(cmd: FwCmd) {
    if let Some(tx) = SENDER.get() {
        let _ = tx.send(cmd);
    }
}

/// How long to wait for the broker to drain a queued command. Generous
/// because building a large playlist (1000+ tracks via `playlist_create` +
/// `build_playlist`) can take several seconds when the kernel is also
/// busy decoding audio. Under rapid user-driven play actions, multiple
/// commands queue up and each subsequent caller waits for the broker to
/// finish the ones ahead of it.
const BROKER_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

/// Send a command and block until the broker confirms execution. Use
/// from actix `web::block(...)` callbacks — the wait is bounded by the
/// broker tick (~10 ms idle, immediate when busy).
pub fn send_and_wait(make: impl FnOnce(mpsc::SyncSender<()>) -> FwCmd) {
    let (reply_tx, reply_rx) = mpsc::sync_channel::<()>(1);
    send(make(reply_tx));
    if reply_rx.recv_timeout(BROKER_TIMEOUT).is_err() {
        tracing::warn!("fw_bus::send_and_wait: broker timed out (no reply within 30 s)");
    }
}

/// Run an arbitrary closure on the broker thread and block until it
/// finishes. Returns the closure's value. Typical use from a handler:
///
///     let ret: i32 = fw_bus::run_on_broker(|| rb::playlist::shuffle(seed, idx));
///
/// The closure runs on a real Rockbox kernel thread, so any firmware
/// FFI it makes resolves `__cores[0].running` correctly. Blocks the
/// caller for up to 30 s; intended to be called from `web::block(...)`.
///
/// On timeout (e.g. the broker is wedged or the queue is backed up under
/// rapid-fire user actions) we **do not** panic — that would unwind the
/// actix worker and surface as a scary 500. Instead we log a warning and
/// return `T::default()`. Callers that care about distinguishing success
/// from a timed-out broker call should use [`try_run_on_broker`] instead.
pub fn run_on_broker<T, F>(f: F) -> T
where
    T: Default + Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    try_run_on_broker(f).unwrap_or_else(|| {
        tracing::warn!("fw_bus::run_on_broker: broker tick timed out (30 s), returning default");
        T::default()
    })
}

/// Same as [`run_on_broker`] but returns `None` on timeout instead of a
/// default value. Use when the caller needs to surface the timeout as a
/// proper error (e.g. an HTTP 503).
pub fn try_run_on_broker<T, F>(f: F) -> Option<T>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    let (tx, rx) = mpsc::sync_channel::<T>(1);
    send(FwCmd::Custom(Box::new(move || {
        let _ = tx.send(f());
    })));
    rx.recv_timeout(BROKER_TIMEOUT).ok()
}

/// Drain the channel and execute everything pending. Called once per
/// broker iteration. Non-blocking — returns when the queue is empty.
/// Must be called from the broker thread (a real Rockbox kernel thread).
pub fn drain(rx: &Receiver<FwCmd>) {
    use rockbox_sys as rb;
    loop {
        match rx.try_recv() {
            Ok(cmd) => {
                execute_on_broker(cmd, &|| true).unwrap_or_else(|e| {
                    tracing::warn!("fw_bus: broker exec failed: {e}");
                });
            }
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Disconnected) => {
                tracing::error!("fw_bus: sender disconnected — bus shutdown");
                return;
            }
        }
    }
}

fn execute_on_broker(cmd: FwCmd, _alive: &dyn Fn() -> bool) -> Result<(), &'static str> {
    use rockbox_sys as rb;
    macro_rules! reply {
        ($r:expr) => {
            if let Some(r) = $r {
                let _ = r.send(());
            }
        };
    }
    match cmd {
        FwCmd::Play {
            elapsed,
            offset,
            reply,
        } => {
            rb::playback::play(elapsed, offset);
            reply!(reply);
        }
        FwCmd::Pause { reply } => {
            rb::playback::pause();
            reply!(reply);
        }
        FwCmd::Resume { reply } => {
            rb::playback::resume();
            reply!(reply);
        }
        FwCmd::Next { reply } => {
            rb::playback::next();
            reply!(reply);
        }
        FwCmd::Prev { reply } => {
            rb::playback::prev();
            reply!(reply);
        }
        FwCmd::Stop { reply } => {
            rb::playback::hard_stop();
            reply!(reply);
        }
        FwCmd::FfRewind { newtime, reply } => {
            rb::playback::ff_rewind(newtime);
            reply!(reply);
        }
        FwCmd::FlushAndReloadTracks { reply } => {
            rb::playback::flush_and_reload_tracks();
            reply!(reply);
        }
        FwCmd::SetCrossfade { value, reply } => {
            rb::sound::audio_set_crossfade_safe(value);
            reply!(reply);
        }
        FwCmd::Custom(f) => f(),
    }
    Ok(())
}
