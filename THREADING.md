# Rockbox Threading Model & Rust Integration

## Overview

Rockboxd has two distinct classes of threads that must coexist:

1. **Rockbox kernel threads** — created via `create_thread()` in C, managed by the Rockbox cooperative scheduler, **must yield explicitly**.
2. **Rust OS threads** — created via `std::thread::spawn`, fully preemptive OS threads, completely independent of the Rockbox scheduler.

Understanding the boundary between these two classes is critical. Crossing it incorrectly (e.g. doing a plain OS-level block inside a Rockbox kernel thread) will silently starve every other Rockbox kernel thread.

There are three scheduler backends, each selected at compile time by an autoconf define:

| Build target            | Autoconf define         | Backend file                                     |
| ----------------------- | ----------------------- | ------------------------------------------------ |
| SDL hosted (simulator)  | `HAVE_SDL_THREADS`      | `firmware/target/hosted/sdl/thread-sdl.c`        |
| Headless macOS / Linux  | `HAVE_POSIX_THREADS`    | `firmware/target/hosted/headless/thread-posix.c` |
| Android cdylib          | `HAVE_SIGALTSTACK_THREADS` | `firmware/asm/thread-unix.c` (signal-stack trick) |

All three share the same C-level `create_thread` / kernel-thread API and the same `struct regs` layout (`void *t, *told, *s; void (*start)(void)`). Only the mechanism for context-switching and blocking differs.

---

## The SDL Cooperative Scheduler (SDL hosted target — simulator)

On the `sdlapp` hosted target, each Rockbox kernel thread is backed by a real SDL OS thread (`SDL_CreateThread`). However, Rockbox layers a **cooperative scheduler** on top of those OS threads using a single global SDL mutex:

```c
// firmware/target/hosted/sdl/thread-sdl.c
static SDL_mutex *m;   // THE global Rockbox scheduler mutex
```

**Only the thread that holds `m` is considered "running" by the Rockbox kernel.** All other Rockbox kernel threads are blocked in `SDL_LockMutex(m)`.

`switch_thread()` is the scheduler's yield primitive:
```c
void switch_thread(void) {
    SDL_UnlockMutex(m);   // release the token — let another thread run
    SDL_LockMutex(m);     // re-acquire when it's our turn again
}
```

Every Rockbox sleep / block call eventually calls `switch_thread()`, which is the only way to hand the scheduler token to another thread.

### What happens without yielding

If a Rockbox kernel thread calls any long-running operation — including a plain Rust `thread::sleep`, `join()`, or a `tokio::block_on` — **without first releasing the mutex via `rb::system::sleep`**, the global mutex is never released. Every other Rockbox kernel thread is stuck forever in `SDL_LockMutex(m)`.

### The yield contract

Any code running in a Rockbox kernel thread that may block for more than a few milliseconds must include a yield loop:

```rust
loop {
    thread::sleep(std::time::Duration::from_millis(100)); // OS-level rest
    rb::system::sleep(rb::HZ);                           // release Rockbox mutex
}
```

`rb::HZ` is 100 (ticks per second on the hosted target), so `sleep(rb::HZ)` sleeps for ~1 second and releases the mutex for that duration.

### Stack size note

The `stack` and `stack_size` arguments to `create_thread()` are **ignored** on the SDL hosted target (see `thread-sdl.c`: `(void)stack; (void)stack_size;`). SDL threads receive the OS default stack size (typically 8 MB on macOS). Stack overflow is not a concern on hosted.

---

## The POSIX Cooperative Scheduler (headless macOS / Linux)

The headless host target (`build-headless/`, used by `zig build lib` to produce `librockboxd.a`) uses `HAVE_POSIX_THREADS`, implemented in `firmware/target/hosted/headless/thread-posix.c`.

### Why not `HAVE_SIGALTSTACK_THREADS`?

The previous backend (`firmware/asm/thread-unix.c`) used the "signal-stack trick": it called `sigaltstack()` + `SIGUSR1` to bootstrap cooperative thread contexts without ever calling `makecontext()`. On **macOS 12 Monterey x86_64** this fails — `sigaltstack()` returns `EPERM` — causing an infinite retry loop with the message:

```
thread creation failed.Retrying
make_context(): Operation not permitted
```

The root cause is that macOS 12 Monterey returns `EPERM` from `sigaltstack()` when a thread already has an alternate signal stack installed (indicated by `SS_ONSTACK`). `HAVE_POSIX_THREADS` avoids `sigaltstack()` entirely.

### How `HAVE_POSIX_THREADS` works

The design is the same as `HAVE_SDL_THREADS` — one OS thread per Rockbox thread, global mutex for cooperative exclusion — but uses only POSIX primitives:

```c
// firmware/target/hosted/headless/thread-posix.c
static pthread_mutex_t g_mutex;   // THE global Rockbox scheduler mutex
```

Per-thread blocking uses a custom counting semaphore built from `pthread_mutex_t + pthread_cond_t + int count` (because `sem_init()` is deprecated on macOS and `sem_timedwait()` is not available on all Darwin versions):

```c
typedef struct { pthread_mutex_t mtx; pthread_cond_t cond; int count; } posix_sem_t;
```

`switch_thread()` is structurally identical to the SDL version:

```c
void switch_thread(void) {
    pthread_mutex_unlock(&g_mutex);   // release the token
    pthread_mutex_lock(&g_mutex);     // re-acquire when it's our turn
}
```

Blocked/sleeping threads wait on their per-thread `posix_sem_t` with a `pthread_cond_timedwait`-based timeout; wakeup posts to the same semaphore. `thread_exit()` uses `longjmp` (via the same `thread_jmpbufs` pattern as SDL) to cleanly terminate, then releases `g_mutex` from `runthread`'s `else` branch.

### Stack sizes

Like SDL threads, the `stack` / `stack_size` arguments are ignored (`(void)stack; (void)stack_size;`). The host OS assigns a default stack to each `pthread_create`'d thread.

---

## The Headless Scheduler (Android cdylib)

The Android cdylib target (`firmware/target/hosted/android/cdylib/`) replaces SDL entirely with plain pthreads and POSIX clock. There is **no SDL mutex**, no event loop, no LCD, no button polling.

### Three-way comparison

| Concern              | SDL hosted                           | Headless macOS/Linux                           | Android cdylib                                          |
| -------------------- | ------------------------------------ | ---------------------------------------------- | ------------------------------------------------------- |
| Autoconf define      | `HAVE_SDL_THREADS`                   | `HAVE_POSIX_THREADS`                           | `HAVE_SIGALTSTACK_THREADS`                              |
| Backend file         | `thread-sdl.c`                       | `thread-posix.c`                               | `thread-unix.c`                                         |
| Cooperative token    | `SDL_mutex *m` (single global)       | `pthread_mutex_t g_mutex` (single global)      | `__cores[0].running` (global current-thread pointer)    |
| Per-thread blocking  | `SDL_sem *` (SDL semaphore)          | `posix_sem_t *` (cond+mutex+count)             | `setjmp`/`longjmp` + signal alt-stack                   |
| Thread creation      | `SDL_CreateThread`                   | `pthread_create` + `pthread_detach`            | Bootstrapped via `sigaltstack()` + `SIGUSR1`            |
| Scheduler yield      | `SDL_UnlockMutex` / `SDL_LockMutex`  | `pthread_mutex_unlock` / `pthread_mutex_lock`  | `longjmp` to next thread's `jmp_buf`                    |
| Boot path            | SDL event thread initialises audio   | Zig linker entry → `main_c()`                  | `rb_daemon_start` (JNI) spawns `rockbox-engine` pthread |
| Power-off            | `SDL_QUIT` event loop                | `exit(0)` via `power_off()`                    | `exit(0)` via `power_off()`                             |
| Stack size respected | No (SDL default, typically 8 MB)     | No (pthread default)                           | Yes — passed to signal alt-stack                        |
| macOS 12 x86_64      | Works                                | Works                                          | **Broken** — `sigaltstack()` returns `EPERM`            |
| stdio                | terminal / controlled                | stderr via `debug-headless.c`                  | piped to logcat via `redirect_stdio_to_logcat`          |

### `__cores[0].running` — the critical invariant

On the Android cdylib (and every non-SDL Rockbox target), the kernel tracks the currently-executing kernel thread via the global `__cores[0].running` pointer. **Any firmware function that calls `queue_send`, `wakeup_thread`, `pcmbuf_*`, or any kernel primitive reads this pointer to find its own thread entry.**

On the headless POSIX target (`HAVE_POSIX_THREADS`) the same invariant holds: `__running_self_entry()` expands to `__cores[0].running` and is updated by every thread at the top of `runthread()` and inside `switch_thread()`. The global `g_mutex` guarantees only one Rockbox kernel thread writes this pointer at a time — but OS threads that bypass `g_mutex` (actix workers, gRPC handlers) will still see a stale value.

If such a function is called from a non-Rockbox pthread (e.g. an actix worker or a tonic gRPC handler) the pointer resolves to the wrong thread entry. Consequences:
- `wakeup_thread_` dereferences a stale or null function pointer → **SIGSEGV at PC=0**
- Kernel scheduler state is silently corrupted
- Symptoms may appear seconds later, during a track switch or settings change

This is why **all firmware-mutating calls from Rust handlers must go through the firmware-command bus** (see below).

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

## The Firmware-Command Bus (`crates/server/src/fw_bus.rs`)

### Problem

gRPC / HTTP handlers run on actix workers or tonic tasks — plain Rust OS threads. Calling firmware FFI directly from these threads violates the `__cores[0].running` invariant and causes the SIGSEGV / scheduler corruption described above. This affects both the Android cdylib and any future hosted-pthread build.

### Solution

All kernel-mutating calls are serialised through a single `std::sync::mpsc` channel. **Only the broker thread drains this channel.** The broker IS a real Rockbox kernel thread (spawned by `apps/broker_thread.c::create_thread`), so its firmware calls always run with a valid `__running_self_entry()`.

```
actix worker / tonic task
  → fw_bus::send(FwCmd::Play { elapsed, offset, reply })
      → mpsc channel
          → broker thread (real Rockbox kernel thread)
              → rb::playback::play(elapsed, offset)
              → reply_tx.send(())
  ← fw_bus::send_and_wait blocks until reply arrives (≤ 30 s)
```

### API

| Function                       | Use case                                                             |
| ------------------------------ | -------------------------------------------------------------------- |
| `fw_bus::init()`               | Call once at startup, before broker thread spawns                    |
| `fw_bus::send(cmd)`            | Fire-and-forget (no reply needed)                                    |
| `fw_bus::send_and_wait(make)`  | Send + block until broker confirms (for actix `web::block` handlers) |
| `fw_bus::run_on_broker(f)`     | Run arbitrary closure on broker; returns `T::default()` on timeout   |
| `fw_bus::try_run_on_broker(f)` | Same but returns `Option<T>` — `None` on timeout                     |
| `fw_bus::drain(rx)`            | Called once per broker iteration to execute pending commands         |

### `FwCmd` variants

`Play`, `Pause`, `Resume`, `Next`, `Prev`, `Stop`, `FfRewind`, `FlushAndReloadTracks`, `SetCrossfade`, and `Custom(Box<dyn FnOnce() + Send>)` (escape hatch for anything not enumerated).

### Timeout behaviour

The broker tick period is ~10 ms at idle (the `rb::system::sleep(HZ/10)` in the broker loop). `BROKER_TIMEOUT` is 30 s — generous because building a large playlist can take several seconds. On timeout `run_on_broker` logs a warning and returns `T::default()` rather than panicking; `try_run_on_broker` returns `None`.

### Where fw_bus is NOT needed

- Read-only status queries (`rb::playback::status()`, `rb::playback::current_track()`) — these only read atomics or copy structs; no kernel primitive is called.
- Anything running inside the broker thread itself (it already owns `__running_self_entry()`).
- Desktop SDL builds — the SDL mutex serialises everything anyway. The bus is still compiled in and works correctly; it just adds one channel hop of latency.

---

## Thread Map

### Rockbox kernel threads (must yield)

| Thread          | C entry point                        | What it does                                                                                                        |
| --------------- | ------------------------------------ | ------------------------------------------------------------------------------------------------------------------- |
| `server_thread` | `server_thread.c` → `start_server()` | Spawns the actix HTTP server in a Rust OS thread, then loops yielding to the Rockbox scheduler                      |
| `broker_thread` | `broker_thread.c` → `start_broker()` | Event loop: drains `fw_bus`, publishes playback state to GraphQL subscriptions, scrobbles tracks, restores playlist |

Both threads must call `rb::system::sleep(rb::HZ)` (desktop) or equivalent on every loop iteration.

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
    let fw_rx = fw_bus::take_receiver();
    loop {
        // Drain firmware commands from actix/gRPC handlers first.
        if let Some(rx) = &fw_rx {
            fw_bus::drain(rx);
        }
        // ... publish events, scrobble, restore playlist ...
        thread::sleep(std::time::Duration::from_millis(100));
        rb::system::sleep(rb::HZ);
    }
}
```

### Rust OS threads (no Rockbox scheduler involvement)

These are spawned with `std::thread::spawn` — they are pure OS threads and never interact with the Rockbox scheduler token or `__cores[0].running`.

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

### Desktop (SDL hosted)

```
main.c: server_init()
│
├── create_thread(server_thread)        ← Rockbox kernel thread
│   └── start_server()
│       ├── load_settings()
│       ├── rockbox_upnp::init()        ← creates static UPnP multi-thread runtime
│       ├── thread::spawn(actix)        ← Rust OS thread (free to block)
│       └── loop { sleep + rb::system::sleep(HZ) }   ← yields Rockbox mutex
│
├── sleep(HZ)                           ← Rockbox scheduler sleep on main thread (~1 s)
│
└── start_servers()                     ← called on main C thread after 1 s
    ├── fw_bus::init()                  ← create mpsc channel before broker spawns
    ├── thread::spawn(gRPC server)      ← Rust OS thread, new_current_thread runtime
    ├── thread::spawn(GraphQL server)   ← Rust OS thread, new_current_thread runtime
    ├── thread::spawn(MPD server)       ← Rust OS thread, new_current_thread runtime
    ├── thread::spawn(MPRIS, Linux)     ← Rust OS thread, async_std
    └── thread::spawn(command relay)    ← Rust OS thread, reqwest blocking

main.c: broker_init()
│
└── create_thread(broker_thread)        ← Rockbox kernel thread
    └── start_broker()
        ├── fw_bus::take_receiver()     ← claim the mpsc Receiver
        └── loop { fw_bus::drain + work + sleep + rb::system::sleep(HZ) }
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
    │       │       ├── fw_bus::init()
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
    │               ├── fw_bus::take_receiver()
    │               └── loop { fw_bus::drain + work + sleep + rb::system::sleep(HZ) }
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

### Rule 6: All firmware-mutating FFI from Rust handlers must go through `fw_bus`

On the headless pthread target (Android cdylib) calling `rb::playback::play`, `rb::sound::set_volume`, or any function that reaches `queue_send` / `wakeup_thread` from a non-Rockbox pthread corrupts `__cores[0].running` and causes a SIGSEGV. Route every such call through `fw_bus::run_on_broker(|| ...)` from inside a `web::block(...)` closure.

Read-only queries (status, current track) that only touch atomics or copy structs are safe to call directly.

### Rule 7: `fw_bus::init()` must be called before any handler sends a command

`fw_bus::send()` silently drops commands if the bus hasn't been initialised. `init()` is called inside `start_servers()`, which runs before the HTTP/gRPC servers start accepting requests. On Android, the boot sequence guarantees `start_servers()` (inside `main_c()`) completes before `rb_daemon_start` returns.

### Rule 8: Never re-enable `HAVE_SIGALTSTACK_THREADS` for the headless host

The headless macOS/Linux build uses `HAVE_POSIX_THREADS` (defined in `build-headless/autoconf.h`). Do not replace it with `HAVE_SIGALTSTACK_THREADS` — `sigaltstack()` returns `EPERM` on macOS 12 Monterey x86_64, causing an infinite retry loop at startup:

```
thread creation failed.Retrying
make_context(): Operation not permitted
```

If you re-run `tools/configure` to regenerate `build-headless/autoconf.h`, check that the threading line reads:

```c
#define HAVE_POSIX_THREADS
```

and restore it if configure has reverted it to `HAVE_SIGALTSTACK_THREADS`.
