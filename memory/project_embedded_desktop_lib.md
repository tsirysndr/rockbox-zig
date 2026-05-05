---
name: Embeddable desktop library (librockboxd.a)
description: New crates/embed + zig build lib produces a fat static lib; gpui embeds daemon directly — no external rockboxd process needed
type: project
---

## Key facts

New build target `zig build lib` produces `zig/zig-out/lib/librockboxd.a` — a fat static archive
containing headless firmware + codecs + Rust gRPC servers + embed crate daemon boot code.

**Why:** Any desktop GUI (GPUI, Swift/AppKit, Qt) can embed the full Rockbox audio engine
in-process without spawning a subprocess. Same pattern as the Android cdylib (`crates/expo`
with `embedded-daemon` feature) but for desktop/headless.

**How to apply:** When working on gpui or a new desktop GUI, do NOT look for or spawn `rockboxd`.
The daemon boots via `rb_daemon_start()` (from `librockboxd.a`) at app startup.

## Files added / changed

- `crates/embed/` — new Rust `staticlib` crate (daemon boot + full gRPC client `rb_*` C ABI)
  - `src/lib.rs` — all `rb_*` gRPC client entry points (mirrors `crates/expo/src/lib.rs`)
  - `src/daemon.rs` — desktop daemon boot: `rb_daemon_start(music_dir, device_name)` (2 args, no config_dir since $HOME always exists on desktop)
  - `Cargo.toml` — `daemon` feature (default=on) pulls in headless stack; `fts5` search (no typesense)
  - `build.rs` — only proto compilation; firmware linking done by Zig
  - `proto` symlink → `../rpc/proto`
- `zig/src/lib.zig` — same `rb_*` firmware exports as `main.zig` but without `main()`
- `zig/build.zig` — added `lib` step; hoisted `codec_names`/`lib_names` to module scope
- `include/rockboxd.h` — public C header for all `rb_*` functions
- `gpui/build.rs` — links `librockboxd.a` + CoreAudio/etc frameworks automatically
- `gpui/src/startup.rs` — replaced with `extern "C"` declarations + `start_embedded()` fn
- `gpui/src/ui/startup_gate.rs` — removed error gate; now shows loading spinner while daemon boots

## Build order for librockboxd.a

1. `cd build-headless && make lib`
2. `cargo build --release -p rockbox-embed -p rockbox-server`
3. `cd zig && zig build lib` → `zig-out/lib/librockboxd.a`
4. `cd gpui && cargo build --release` (links against librockboxd.a automatically)

## rb_daemon_start signature (desktop — 2 args, not 3 like Android)

```c
int rb_daemon_start(const char *music_dir_ptr,    // NULL → $HOME/Music
                    const char *device_name_ptr);  // must be non-null
```

Android version has a third `config_dir_ptr` arg (sets $HOME on Android).
Desktop version omits it — $HOME already exists.
