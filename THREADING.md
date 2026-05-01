# Rockbox Threading Model & Rust Integration

## Overview

Rockboxd has two distinct classes of threads that must coexist:

1. **Rockbox kernel threads** — created via `create_thread()` in C, managed by the Rockbox cooperative scheduler, **must yield explicitly**.
2. **Rust OS threads** — created via `std::thread::spawn`, fully preemptive OS threads, completely independent of the Rockbox scheduler.

Understanding the boundary between these two classes is critical. Crossing it incorrectly (e.g. doing a plain OS-level block inside a Rockbox kernel thread) will silently starve every other Rockbox kernel thread.

---

## The SDL Cooperative Scheduler

On the `sdlapp` hosted target (macOS / Linux desktop), each Rockbox kernel thread is backed by a real SDL OS thread (`SDL_CreateThread`). However, Rockbox layers a **cooperative scheduler** on top of those OS threads using a single global SDL mutex:

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

## Thread Map

### Rockbox kernel threads (must yield)

| Thread | C entry point | What it does |
|--------|---------------|--------------|
| `server_thread` | `server_thread.c` → `start_server()` | Spawns the actix HTTP server in a Rust OS thread, then loops yielding to the Rockbox scheduler |
| `broker_thread` | `broker_thread.c` → `start_broker()` | Event loop: publishes playback state to GraphQL subscriptions, scrobbles tracks, restores playlist |

Both threads must call `rb::system::sleep(rb::HZ)` on every loop iteration.

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
    // ... setup ...
    loop {
        // ... do work (check playback, publish events) ...
        thread::sleep(std::time::Duration::from_millis(100));
        rb::system::sleep(rb::HZ);  // yield the Rockbox mutex
    }
}
```

### Rust OS threads (no Rockbox scheduler involvement)

These are spawned with `std::thread::spawn` — they are pure OS threads and never interact with the Rockbox mutex. They are free to block indefinitely.

| Thread / component | Spawned from | Runtime |
|--------------------|--------------|---------|
| **HTTP server** (actix-web, port 6063) | `start_server()` via `thread::spawn` | `actix_rt::System` (single-thread + LocalSet per arbiter) |
| **gRPC server** (tonic, port 6061) | `start_servers()` via `thread::spawn` | `tokio::Builder::new_current_thread` |
| **GraphQL server** (async-graphql, port 6062) | `start_servers()` via `thread::spawn` | `tokio::Builder::new_current_thread` |
| **MPD server** (port 6600) | `start_servers()` via `thread::spawn` | `tokio::Builder::new_current_thread` |
| **MPRIS server** (Linux, D-Bus) | `start_servers()` via `thread::spawn` | `async_std` task |
| **UPnP runtime** | `rockbox_upnp::init()` (static `OnceLock`) | `tokio::runtime::Runtime::new()` (multi-thread) |
| **Device scanners** (Chromecast, AirPlay, Snapcast, UPnP, Squeezelite) | `run_http_server()` via `thread::spawn` | each creates its own `tokio::runtime::Runtime::new()` |
| **Player event listener** | `run_http_server()` via `thread::spawn` | `tokio::runtime::Runtime::new()` (multi-thread) |
| **Command relay** | `start_servers()` via `thread::spawn` | `reqwest::blocking` (creates its own tokio internally) |

---

## Startup Sequence

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
    ├── thread::spawn(gRPC server)      ← Rust OS thread, new_current_thread runtime
    ├── thread::spawn(GraphQL server)   ← Rust OS thread, new_current_thread runtime
    ├── thread::spawn(MPD server)       ← Rust OS thread, new_current_thread runtime
    ├── thread::spawn(MPRIS, Linux)     ← Rust OS thread, async_std
    └── thread::spawn(command relay)    ← Rust OS thread, reqwest blocking

main.c: broker_init()
│
└── create_thread(broker_thread)        ← Rockbox kernel thread
    └── start_broker()
        └── loop { work + sleep + rb::system::sleep(HZ) }
```

---

## Tokio Runtime Layout

Multiple independent tokio runtimes coexist; they do not share thread pools or event loops.

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
