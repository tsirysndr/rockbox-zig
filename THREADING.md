# Rockbox Threading Model & Rust Integration

## Overview

Rockboxd has two distinct classes of threads that must coexist:

1. **Rockbox kernel threads** — created via `create_thread()` in C, managed by the Rockbox cooperative scheduler, **must yield explicitly**.
2. **Rust OS threads** — created via `std::thread::spawn`, fully preemptive OS threads, completely independent of the Rockbox scheduler.

Understanding the boundary between these two classes is critical. Crossing it incorrectly (e.g. doing a plain OS-level block inside a Rockbox kernel thread) will silently starve every other Rockbox kernel thread.

All build targets — SDL hosted (simulator), headless macOS/Linux, and Android cdylib — use the **same POSIX-based scheduler** (`HAVE_POSIX_THREADS`), implemented in `firmware/target/hosted/headless/thread-posix.c`.

---

## The POSIX Cooperative Scheduler

Each Rockbox kernel thread is backed by a real `pthread`. Rockbox layers a **cooperative scheduler** on top of those OS threads using a single global mutex:

```c
// firmware/target/hosted/headless/thread-posix.c
static pthread_mutex_t g_mutex;   // THE global Rockbox scheduler mutex
```

**Only the thread that holds `g_mutex` is considered "running" by the Rockbox kernel.** All other Rockbox kernel threads are blocked waiting to acquire it.

`switch_thread()` is the scheduler's yield primitive:
```c
void switch_thread(void) {
    pthread_mutex_unlock(&g_mutex);   // release the token — let another thread run
    pthread_mutex_lock(&g_mutex);     // re-acquire when it's our turn again
}
```

Every Rockbox sleep / block call eventually calls `switch_thread()`, which is the only way to hand the scheduler token to another thread.

Per-thread blocking uses a custom counting semaphore built from `pthread_mutex_t + pthread_cond_t + int count` (because `sem_init()` is deprecated on macOS and `sem_timedwait()` is not available on all Darwin versions):

```c
typedef struct { pthread_mutex_t mtx; pthread_cond_t cond; int count; } posix_sem_t;
```

Blocked/sleeping threads wait on their per-thread `posix_sem_t` with a `pthread_cond_timedwait`-based timeout; wakeup posts to the same semaphore. `thread_exit()` uses `longjmp` (via `thread_jmpbufs`) to cleanly terminate, then releases `g_mutex` from `runthread`'s `else` branch.

### What happens without yielding

If a Rockbox kernel thread calls any long-running operation — including a plain Rust `thread::sleep`, `join()`, or a `tokio::block_on` — **without first releasing the mutex via `rb::system::sleep`**, the global mutex is never released. Every other Rockbox kernel thread is stuck forever.

### The yield contract

Any code running in a Rockbox kernel thread that may block for more than a few milliseconds must include a yield loop:

```rust
loop {
    thread::sleep(std::time::Duration::from_millis(100)); // OS-level rest
    rb::system::sleep(rb::HZ);                           // release Rockbox mutex
}
```

`rb::HZ` is 100 (ticks per second on the hosted target), so `sleep(rb::HZ)` sleeps for ~1 second and releases the mutex for that duration.

### Stack sizes

The `stack` and `stack_size` arguments to `create_thread()` are ignored (`(void)stack; (void)stack_size;`). The host OS assigns a default stack to each `pthread_create`'d thread.

### Rust OS thread integration — `rb_kernel_lock` / `rb_kernel_unlock`

`thread-posix.c` exposes two functions that let **any OS thread** (Rust or C) safely call Rockbox firmware FFI:

```c
void rb_kernel_lock(void);    // acquire g_mutex + set __running_self_entry
void rb_kernel_unlock(void);  // clear __running_self_entry + release g_mutex
```

`init_threads()` pre-allocates a `g_external_entry` — a `struct thread_entry` with `state = STATE_RUNNING` and name `"rb_external"`. `rb_kernel_lock()` installs this entry as `__running_self_entry()` while holding `g_mutex`, making the calling OS thread indistinguishable from a real Rockbox thread from the kernel's point of view:

```c
void rb_kernel_lock(void) {
    pthread_mutex_lock(&g_mutex);
    __running_self_entry() = g_external_entry;
}

void rb_kernel_unlock(void) {
    __running_self_entry() = NULL;
    pthread_mutex_unlock(&g_mutex);
}
```

Rust handlers that need to call firmware-mutating FFI acquire the lock directly:

```rust
unsafe { rockbox_sys::rb_kernel_lock() };
rb::playback::play(elapsed, offset);
unsafe { rockbox_sys::rb_kernel_unlock() };
```

---

## Per-target differences

All targets share the same POSIX cooperative scheduler. What differs is the audio backend and boot path:

| Concern              | SDL hosted (simulator)                | Headless macOS/Linux                  | Android cdylib                                          |
| -------------------- | ------------------------------------- | ------------------------------------- | ------------------------------------------------------- |
| Autoconf define      | `HAVE_POSIX_THREADS`                  | `HAVE_POSIX_THREADS`                  | `HAVE_POSIX_THREADS`                                    |
| Backend file         | `thread-posix.c`                      | `thread-posix.c`                      | `thread-posix.c`                                        |
| Cooperative token    | `pthread_mutex_t g_mutex`             | `pthread_mutex_t g_mutex`             | `pthread_mutex_t g_mutex`                               |
| Rust OS thread FFI   | `rb_kernel_lock` / `rb_kernel_unlock` | `rb_kernel_lock` / `rb_kernel_unlock` | `rb_kernel_lock` / `rb_kernel_unlock`                   |
| Audio backend        | SDL2 audio                            | CPAL                                  | AAudio (API 26+)                                        |
| Boot path            | Zig entry → `main_c()`               | Zig entry → `main_c()`               | `rb_daemon_start` (JNI) spawns `rockbox-engine` pthread |
| Power-off            | `SDL_QUIT` event loop                 | `exit(0)` via `power_off()`           | `exit(0)` via `power_off()`                             |
| stdio                | terminal / controlled                 | stderr via `debug-headless.c`         | piped to logcat via `redirect_stdio_to_logcat`          |

### `__cores[0].running` — the critical invariant

On every Rockbox target the kernel tracks the currently-executing kernel thread via `__running_self_entry()`, which expands to `__cores[0].running` — a plain C global, **not TLS**. Any firmware function that calls `queue_send`, `wakeup_thread`, `pcmbuf_*`, or any kernel primitive reads this pointer to find its own thread entry.

`g_mutex` guarantees that only one thread writes this pointer at a time. `rb_kernel_lock()` extends this guarantee to arbitrary Rust OS threads: by acquiring `g_mutex` and installing `g_external_entry`, it ensures `__running_self_entry()` is always valid while any firmware FFI is in flight.

An OS thread calling firmware FFI without the correct `__running_self_entry()` will cause:
- `wakeup_thread_` to dereference a stale or null function pointer → **SIGSEGV at PC=0**
- Silent kernel scheduler corruption
- Symptoms appearing seconds later during a track switch or settings change

### Daemon boot sequence (Android)

`rb_daemon_start(configDir, musicDir, deviceName)` in `crates/expo/src/daemon.rs`:

1. Atomically transitions `STATE`: `STOPPED → STARTING` (returns `-114` if already running).
2. Installs the tracing-android logcat subscriber (idempotent).
3. Sets env vars: `HOME`, `ROCKBOX_LIBRARY`, `TMPDIR`, `ROCKBOX_PORT`, `ROCKBOX_GRAPHQL_PORT`, `ROCKBOX_TCP_PORT`, `ROCKBOX_MPD_PORT`.
4. Spawns `rockbox-engine` pthread (2 MB stack) that calls `main_c()` wrapped in `catch_unwind`.
5. Polls TCP `127.0.0.1:<port>` every 50 ms, up to 30 s, waiting for gRPC to bind.
6. On success: stores port in `LOCAL_PORT`, transitions to `RUNNING`, sets `SERVER_URL` if not already overridden by JS, spawns `rockbox-library-scan` thread.
7. Returns the bound gRPC port (positive) or a negative error code (`-110` = timeout, `-114` = already running).

---

## Thread Map

### Rockbox kernel threads (must yield)

| Thread          | C entry point                        | What it does                                                                                            |
| --------------- | ------------------------------------ | ------------------------------------------------------------------------------------------------------- |
| `server_thread` | `server_thread.c` → `start_server()` | Spawns the actix HTTP server in a Rust OS thread, then loops yielding to the Rockbox scheduler          |
| `broker_thread` | `broker_thread.c` → `start_broker()` | Event loop: publishes playback state to GraphQL subscriptions, scrobbles tracks, restores playlist on first tick. |

Both threads must call `rb::system::sleep(rb::HZ)` (or equivalent) on every loop iteration.

**`server_thread` pattern** (`crates/server/src/lib.rs`):
```rust
pub extern "C" fn start_server() {
    rockbox_settings::load_settings(None).ok();
    rockbox_upnp::init();

    // Spawn HTTP server in its own Rust OS thread — does NOT hold the
    // Rockbox mutex, so it can block freely in actix/tokio.
    thread::spawn(|| {
        match actix_rt::System::new().block_on(run_http_server()) { ... }
    });

    // Keep the Rockbox kernel thread alive and cooperative.
    loop {
        thread::sleep(std::time::Duration::from_millis(100));
        rb::system::sleep(rb::HZ);
    }
}
```

**`broker_thread` pattern** (`crates/server/src/lib.rs`):
```rust
pub extern "C" fn start_broker() {
    loop {
        // ... publish events, scrobble, restore playlist ...
        thread::sleep(std::time::Duration::from_millis(100));
        rb::system::sleep(rb::HZ);
    }
}
```

### Rust OS threads (no Rockbox scheduler involvement)

These are spawned with `std::thread::spawn` — they are pure OS threads and never interact with the Rockbox scheduler token.

| Thread / component                                                     | Spawned from                               | Runtime                                                   |
| ---------------------------------------------------------------------- | ------------------------------------------ | --------------------------------------------------------- |
| **HTTP server** (actix-web, port 6063)                                 | `start_server()` via `thread::spawn`       | `actix_rt::System` (single-thread + LocalSet per arbiter) |
| **gRPC server** (tonic, port 6061)                                     | `start_servers()` via `thread::spawn`      | `tokio::Builder::new_current_thread`                      |
| **GraphQL server** (async-graphql, port 6062)                          | `start_servers()` via `thread::spawn`      | `tokio::Builder::new_current_thread`                      |
| **MPD server** (port 6600)                                             | `start_servers()` via `thread::spawn`      | `tokio::Builder::new_current_thread`                      |
| **MPRIS server** (Linux, D-Bus)                                        | `start_servers()` via `thread::spawn`      | `async_std` task                                          |
| **UPnP runtime**                                                       | `rockbox_upnp::init()` (static `OnceLock`) | `tokio::runtime::Runtime::new()` (multi-thread)           |
| **Device scanners** (Chromecast, AirPlay, Snapcast, UPnP, Squeezelite) | `run_http_server()` via `thread::spawn`    | each creates its own `tokio::runtime::Runtime::new()`     |
| **Player event listener**                                              | `run_http_server()` via `thread::spawn`    | `tokio::runtime::Runtime::new()` (multi-thread)           |
| **Command relay**                                                      | `start_servers()` via `thread::spawn`      | `reqwest::blocking` (creates its own tokio internally)    |

### Android-only Rust OS threads

| Thread                               | Spawned from                                  | Purpose                                                                         |
| ------------------------------------ | --------------------------------------------- | ------------------------------------------------------------------------------- |
| **`rockbox-engine`** (2 MB stack)    | `rb_daemon_start`                             | Calls `main_c()`; owns all Rockbox kernel threads and the cooperative scheduler |
| **`rockbox-library-scan`**           | `spawn_library_scan()` after gRPC binds       | SQLite audio scan + Rocksky enrichment                                          |
| **`stdio-logcat-reader`** (detached) | `redirect_stdio_to_logcat()` at `system_init` | Reads the stdout/stderr pipe and writes lines to logcat tag `Rockbox`           |
| **`rockbox-rpc`** (× 2 workers)      | `RT` (`Lazy<Runtime>`) in `crates/expo`       | Tokio multi-thread runtime for all tonic gRPC client calls                      |

---

## Startup Sequences

### SDL hosted and headless (`rockboxd` binary / `librockboxd.a`)

```
Zig entry → main_c()
│
├── server_init() → create_thread(server_thread)     ← Rockbox kernel thread
│   └── start_server()
│       ├── load_settings()
│       ├── rockbox_upnp::init()
│       ├── thread::spawn(actix HTTP server)          ← Rust OS thread (free to block)
│       └── loop { sleep + rb::system::sleep(HZ) }   ← yields g_mutex
│
├── sleep(HZ)
│
└── start_servers()                                   ← called on main C thread
    ├── thread::spawn(gRPC server)                    ← Rust OS thread
    │   └── firmware FFI via rb_kernel_lock/unlock
    ├── thread::spawn(GraphQL server)
    ├── thread::spawn(MPD server)
    ├── thread::spawn(MPRIS, Linux only)
    └── thread::spawn(command relay)

broker_init() → create_thread(broker_thread)         ← Rockbox kernel thread
└── start_broker()
    └── loop { publish events + scrobble + sleep + rb::system::sleep(HZ) }
```

### Android cdylib

```
JNI: RockboxRpcModule.OnCreate
│
└── rb_daemon_start(configDir, musicDir, deviceName)   ← crates/expo/src/daemon.rs
    ├── STATE: STOPPED → STARTING
    ├── install_logcat_subscriber()
    ├── configure_environment()         ← sets HOME, ROCKBOX_LIBRARY, TMPDIR, ports
    │
    ├── thread::spawn("rockbox-engine", stack=2MB)
    │   └── main_c()                    ← apps/main.c
    │       ├── system_init()           ← redirect_stdio_to_logcat + stackbegin
    │       ├── server_init()           ← create_thread(server_thread)
    │       │   └── start_server()
    │       │       ├── load_settings()
    │       │       ├── thread::spawn(actix HTTP server)
    │       │       └── loop { sleep + rb::system::sleep(HZ) }
    │       ├── sleep(HZ)
    │       ├── start_servers()
    │       │   ├── thread::spawn(gRPC server)
    │       │   ├── thread::spawn(GraphQL server)
    │       │   ├── thread::spawn(MPD server)
    │       │   └── thread::spawn(command relay)
    │       └── broker_init()           ← create_thread(broker_thread)
    │           └── start_broker()
    │               └── loop { work + sleep + rb::system::sleep(HZ) }
    │
    ├── wait_for_grpc(:6061, 30s)       ← polls TCP connect every 50 ms
    ├── STATE: STARTING → RUNNING
    ├── rb_set_server_url("http://127.0.0.1:6061")  ← if JS hasn't overridden
    └── spawn_library_scan(force=false) ← own OS thread + current_thread tokio runtime
```

---

## Tokio Runtime Layout

Multiple independent tokio runtimes coexist; they do not share thread pools or event loops.

### Desktop

```
┌─────────────────────────────────────────────────────┐
│  UPnP Runtime (static, multi-thread)                │
│  Owns: mDNS/SSDP, UPnP HTTP, UPnP renderer          │
└─────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────┐
│  actix-rt System (current-thread + LocalSet)        │
│  Owns: HTTP REST API handlers (port 6063)           │
│  Workers: actix arbiters, each on their own thread  │
└─────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────┐
│  gRPC Runtime (current-thread)                      │
│  Owns: tonic server (port 6061)                     │
└─────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────┐
│  GraphQL Runtime (current-thread)                   │
│  Owns: async-graphql + subscriptions (port 6062)    │
└─────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────┐
│  MPD Runtime (current-thread)                       │
│  Owns: MPD protocol server (port 6600)              │
└─────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────┐
│  Scanner / player-event runtimes (multi-thread each)│
│  Short-lived, one per background scanning thread    │
└─────────────────────────────────────────────────────┘
```

### Android (additional runtimes)

```
┌─────────────────────────────────────────────────────┐
│  rockbox-rpc Runtime (multi-thread, 2 workers)      │
│  Lazy<Runtime> in crates/expo/src/lib.rs            │
│  Owns: all outbound tonic gRPC client calls         │
│  (rb_play, rb_pause, rb_status_json, streams, …)    │
└─────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────┐
│  library-scan Runtime (current-thread, ephemeral)   │
│  Created per spawn_library_scan() call              │
│  Owns: SQLite connection pool, audio_scan,          │
│        save_audio_metadata, Rocksky enrichment      │
└─────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────┐
│  save_remote_track_metadata Runtime (current-thread)│
│  Created per call from C firmware (streamfd.c)      │
│  Safe: called from a Rockbox kernel thread, which   │
│  is never inside an existing async context          │
└─────────────────────────────────────────────────────┘
```

All runtimes share a single **SQLite database** (`~/.config/rockbox.org/rockbox-library.db`). The connection pool is configured with:
- WAL journal mode (concurrent readers + one writer)
- `busy_timeout = 30 s` (serialize concurrent writers instead of failing)

---

## Rules & Pitfalls

### Rule 1: Never do a plain OS block inside a Rockbox kernel thread

Wrong — starves every other Rockbox thread:
```rust
pub extern "C" fn start_server() {
    let rt = tokio::runtime::Builder::new_current_thread().build().unwrap();
    rt.block_on(run_server());  // WRONG: holds Rockbox mutex, nothing else runs
}
```

Wrong — same problem:
```rust
    let handle = thread::spawn(|| { run_server() });
    handle.join().unwrap();  // WRONG: OS-level block, Rockbox mutex never released
```

Correct — spawn work out, yield in the Rockbox thread:
```rust
    thread::spawn(|| { run_server() });
    loop {
        thread::sleep(Duration::from_millis(100));
        rb::system::sleep(rb::HZ);  // releases Rockbox mutex
    }
```

### Rule 2: Use `actix_rt::System` for the HTTP server, not a raw tokio runtime

Wrong — actix detects "existing Tokio runtime" and collapses all workers onto one thread:
```rust
let rt = tokio::runtime::Builder::new_current_thread().build().unwrap();
rt.block_on(HttpServer::new(...).run()); // "starting in existing Tokio runtime"
```

Correct:
```rust
actix_rt::System::new().block_on(run_http_server());
```

### Rule 3: Never call `tokio::runtime::Runtime::new()` from inside an active `block_on`

Calling `Runtime::new()` from within a `block_on` context (e.g. inside an async actix handler) panics in tokio 1.27+. This is why `rockbox_upnp::init()` is called **before** any runtime is started in `start_server()`.

### Rule 4: `reqwest::blocking` creates its own tokio runtime internally

The command-relay thread in `start_servers()` uses `reqwest::blocking::Client`. This internally creates a multi-thread tokio runtime. It must run in a plain Rust OS thread, never inside an existing async context.

### Rule 5: `SimpleBroker` is runtime-agnostic

`rockbox_graphql::simplebroker::SimpleBroker` uses `futures_channel::mpsc::UnboundedSender` for pub/sub. `publish()` is a synchronous call — it does not require or interact with any tokio runtime. It is safe to call from any thread, including Rockbox kernel threads and scanner threads with their own runtimes.

### Rule 6: Use `rb_kernel_lock` / `rb_kernel_unlock` for firmware-mutating FFI from Rust OS threads

Calling `rb::playback::play`, `rb::sound::set_volume`, or any function that reaches `queue_send` / `wakeup_thread` from a Rust OS thread corrupts `__cores[0].running` without the lock, causing a SIGSEGV. Always wrap such calls:

```rust
unsafe { rockbox_sys::rb_kernel_lock() };
rb::playback::play(elapsed, offset);
unsafe { rockbox_sys::rb_kernel_unlock() };
```

This acquires `g_mutex` and installs `g_external_entry` — O(1) overhead, no round-trip. Read-only queries (status, current track) that only touch atomics or copy structs are safe to call directly without the lock.

### Rule 7: Do not nest `rb_kernel_lock` calls

`g_mutex` is a non-recursive `pthread_mutex_t`. Calling `rb_kernel_lock()` from a thread that already holds it will deadlock. Flatten the calls: acquire the lock once, do all the firmware work, release it.
