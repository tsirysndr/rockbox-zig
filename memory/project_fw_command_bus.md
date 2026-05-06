---
name: fw_bus removal and ABBA deadlock fix
description: fw_bus channel removed; all handlers now use rb::with_kernel_lock; PLAYER_MUTEX removed; with_kernel_lock is panic-safe
type: project
---

## fw_bus is gone

`crates/server/src/fw_bus.rs` was deleted. All FFI calls from HTTP handlers, gRPC handlers, and GraphQL handlers now use `rb::with_kernel_lock(|| { ... })` directly (defined in `crates/sys/src/lib.rs`).

`rb::with_kernel_lock` acquires `g_mutex` (the cooperative-scheduling mutex in `thread-posix.c`), runs the closure, then releases `g_mutex`. Any OS thread can safely call Rockbox FFI this way.

## POSIX threads migration

`autoconf/autoconf-android.h`, `build-android-arm64/autoconf.h`, and `tools/configure` (`androidcdylibcc()` and `sdlconfig()` functions) were changed from `HAVE_SIGALTSTACK_THREADS` to `HAVE_POSIX_THREADS`. Android bionic fully supports the APIs used by `thread-posix.c` (mutex+condvar, `pthread_create`, `clock_gettime`, `setjmp/longjmp`).

## PLAYER_MUTEX removed (2026-05-06)

`PLAYER_MUTEX` was entirely removed from `crates/server/src/` (declaration in `lib.rs`, all usages in `handlers/player.rs`, `handlers/playlists.rs`, `handlers/smart_playlists.rs`, `handlers/saved_playlists.rs`, `handlers/settings.rs`, and the broker loop in `lib.rs`).

**Why it deadlocked in GPUI embedded but not SDL rockboxd:**
- In SDL builds, `rb_kernel_lock` is a no-op stub → `with_kernel_lock` provides zero locking → PLAYER_MUTEX was the only serializer
- In headless builds (GPUI embedded), `rb_kernel_lock` acquires `pthread_mutex_t g_mutex` for real → `with_kernel_lock` already provides exclusive mutual exclusion
- PLAYER_MUTEX inside `with_kernel_lock` was redundant in headless mode but created a deadlock surface:
  - Any code path where PLAYER_MUTEX was acquired WITHOUT `with_kernel_lock` (e.g., the broker loop at line 656, or the old `load` handler) could hold PLAYER_MUTEX while the broker held g_mutex → ABBA deadlock
  - Alternatively, if a handler panicked while holding PLAYER_MUTEX (before the panic-safe fix below), the mutex became poisoned and all subsequent `.lock().unwrap()` calls cascaded into panics

## with_kernel_lock is now panic-safe (2026-05-06)

`crates/sys/src/lib.rs` — `with_kernel_lock` now uses a RAII `KernelGuard`:

```rust
struct KernelGuard;
impl Drop for KernelGuard {
    fn drop(&mut self) { unsafe { rb_kernel_unlock() }; }
}

pub fn with_kernel_lock<T, F: FnOnce() -> T>(f: F) -> T {
    unsafe { rb_kernel_lock() };
    let _guard = KernelGuard;
    f()  // _guard drops here even on panic — g_mutex never leaked
}
```

Previously, if `f()` panicked, `rb_kernel_unlock` was never called and `g_mutex` was permanently locked, deadlocking all future `with_kernel_lock` callers (GPUI embedded only — SDL builds are no-ops).

## web::block is still used

`web::block`: `rb_kernel_lock` is a blocking mutex acquire; offloading to actix's blocking thread pool is required to avoid stalling the actix event loop.
