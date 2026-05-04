# Headless Build (`rockboxd` without SDL)

This document describes the headless macOS/Linux build of `rockboxd`: what it is, how to build it, and the non-obvious implementation decisions made along the way.

---

## Overview

The standard `rockboxd` binary links against SDL2 for audio output and windowing. The **headless build** removes that dependency so the daemon can run on servers, CI machines, or any host without a display:

- Audio is output through [cpal](https://github.com/RustAudio/cpal) (CoreAudio on macOS, ALSA on Linux) via the `cpal-sink` Rust crate.
- The Rockbox C codecs are **statically linked** (no `dlopen`) so the binary self-contains all format support.
- All other APIs (gRPC, GraphQL, HTTP, MPD) are identical to the SDL build.

The resulting binary lives at `zig/zig-out/bin/rockboxd`.

---

## Quick Start

```sh
# Install prerequisites (macOS)
brew install llvm@21   # llvm@22 crashes on Mach-O redefine-sym — see below

# Full build (configure + Make + Cargo + Zig)
bash scripts/build-headless.sh

# Run
./zig/zig-out/bin/rockboxd
RUST_LOG=info ./zig/zig-out/bin/rockboxd
```

On a clean tree `build-headless.sh` does everything. On subsequent runs it skips the configure step and only rebuilds what changed.

---

## Build Pipeline

### Step 1 — Configure (`tools/configure`, target 206)

```sh
mkdir -p build-headless
(cd build-headless && printf "206\nN\n" | ../tools/configure)
```

Target 206 is `headlesshost`. `tools/configure` sets:

| Variable | Value | Why |
|---|---|---|
| `TARGET` | `-DHEADLESSHOST` | Selects headless firmware path |
| `APP_TYPE` | `headless_host` | Headless Make rules |
| `CODECS_STATIC` | `1` | Static codec linking (see below) |
| `EXTRA_DEFINES` | `-DCODECS_STATIC -DZIG_APP -DAPPLICATION` | Propagated into C flags |
| `OC` | `llvm-objcopy` (from llvm@21) | Safe Mach-O symbol renaming (see below) |

The configure script searches for `llvm@21` first, then falls back to the generic `llvm` formula. **It explicitly avoids `llvm@22`** due to a crash described below.

### Step 2 — C firmware (`make lib`)

```sh
cd build-headless && make -j$(nproc) lib OC=/opt/homebrew/opt/llvm@21/bin/llvm-objcopy
```

Builds:
- `librockbox.a` — core application layer
- `firmware/libfirmware.a` — audio engine, playback, buffering
- `lib/librbcodec.a` — metadata parsers, DSP
- `lib/libfixedpoint.a`, `lib/libskin_parser.a`, `lib/libtlsf.a`
- `lib/rbcodec/codecs/libspeex-voice.a` — voice codec
- **Per-codec archives** (`a52.a`, `flac.a`, `opus.a`, … 29 total) — see CODECS_STATIC section
- **Support libraries** (`liba52.a`, `libffmpegFLAC.a`, `libtremor.a`, … 22 total)
- `lib/rbcodec/codecs/libcodec.a` — codec runtime helpers (see below)

### Step 2.1 — Build `libcodec.a` explicitly

`libcodec.a` provides `codec_init`, `codec_malloc`, `codec_free`, `codec_realloc`, `codec_set_replaygain`, `bs_clz_tab`, `bs_log2_tab`, `ff_copy_bits`, `ff_fft_calc_c`, and other symbols that every codec calls unconditionally.

In `CODECS_STATIC` mode the Make rule that archives per-codec `.a` files **intentionally omits** `$(CODEC_LIBS)` (see `codecs.make` line 240–242: `ifndef CODECS_STATIC / $(CODECS): $(CODEC_LIBS) / endif`). This means `libcodec.a` is never built as a side-effect of `make lib` when `CODECS_STATIC=1`. It must be built explicitly:

```sh
make "$ROOTDIR/build-headless/lib/rbcodec/codecs/libcodec.a" OC=...
```

`build-headless.sh` does this automatically if the file is absent.

**Important pitfall**: on the very first build run, Make's compile step for `lib/rbcodec/codecs/lib/codeclib.c` tries to `mkdir` the output directory. If a previous aborted run left an empty *file* at that path (instead of a directory), `mkdir` fails with "File exists". The script removes the stale file before invoking Make.

### Step 2.5 — Extract codec `.o` files

```sh
for name in a52 flac opus ...; do
    mkdir -p codec-objects/$name
    (cd codec-objects/$name && ar x ../codecs/$name.a)
done
```

This extracts the renamed `.o` and `-crt0.o` pairs from each per-codec archive for **direct linking** by Zig (see "Zig MachO archive scanning" below).

### Step 3 — Rust crates

```sh
cargo build --release --features cpal-sink -p rockbox-cli
cargo build --release -p rockbox-server
```

The `cpal-sink` feature activates `crates/cpal-sink/` and wires up the `pcm_cpal_*` symbols that the C firmware calls at runtime.

### Step 4 — Zig link

```sh
cd zig && zig build -Dheadless=true -Doptimize=ReleaseFast
```

Links all the `.a` files plus the extracted codec objects into a single `zig-out/bin/rockboxd` Mach-O 64-bit (ARM64) or ELF64 executable.

---

## CODECS_STATIC — How Static Codec Linking Works

On desktop Rockbox, each codec (`flac`, `opus`, …) is compiled as a shared library (`.codec` file) and loaded at runtime via `dlopen`. The Android cdylib target can't use `dlopen` from app directories on modern Android, so a static-linking mode was introduced.

In `CODECS_STATIC` mode, `codecs.make` adds `-DCODECS_STATIC` to the codec C flags. This changes the `CODEC_HEADER` macro in `lib/rbcodec/codecs/codecs.h`:

```c
// Normal (dlopen) mode:
const struct codec_header __header
    __attribute__ ((section (".header"))) = { ... };

// CODECS_STATIC mode:
const struct codec_header __header
    __attribute__((visibility("default"))) = { ... };
```

Without the `.header` section, the symbol is a regular exported symbol. The `codecs.make` build then runs `objcopy --redefine-sym` to rename `__header` → `__header_<name>` (and likewise for `codec_main`, `codec_run`, `codec_start`) so that all 29 codecs can coexist in one binary without symbol collisions.

The runtime codec dispatcher (`lc-headless.c`) has a table:

```c
static const struct lc_entry lc_static_table[] = {
    { "a52",   &__header_a52   },
    { "flac",  &__header_flac  },
    ...
};
```

When the firmware needs to play a file, it looks up the codec by name in this table and calls `entry_point` directly instead of loading a `.codec` file.

---

## Non-Obvious Problems and Their Fixes

### 1. GNU binutils `objcopy` corrupts Mach-O 64-bit objects

**Symptom**: `zig build` fails with `failed to parse TBD file` or Zig reports the object files as invalid.

**Root cause**: GNU `objcopy --redefine-sym` on macOS Mach-O 64-bit files silently converts them to Mach-O 32-bit format:
- The magic bytes change from `0xCFFAEDFE` (`MH_MAGIC_64`) to `0xCEFAEDFE` (`MH_MAGIC`).
- The 4-byte `reserved` field is stripped from `mach_header_64`, making the binary 4 bytes shorter and misaligning all load commands.

This is a known GNU binutils bug — the tool does not fully understand the 64-bit Mach-O format.

**Fix**: Use `llvm-objcopy` instead. `tools/configure` (the `headlesshostcc()` function) now searches for it at configure time and writes the path into `build-headless/Makefile` as `OC=`. `build-headless.sh` also passes `OC=` on the `make` command line.

Affected path in Makefile: `OC=/opt/homebrew/opt/binutils/bin/objcopy` (wrong) → `OC=/opt/homebrew/opt/llvm@21/bin/llvm-objcopy` (correct).

### 2. LLVM 22 crashes in `setSymbolInRelocationInfo`

**Symptom**: `make lib` crashes with a segfault inside `llvm-objcopy` during the `asap` and `spc` codec archives:

```
#3 llvm::objcopy::macho::MachOReader::setSymbolInRelocationInfo(...)
Segmentation fault: 11
```

**Root cause**: LLVM 22.x has a bug in its Mach-O section-relative relocation handling. The crash only affects two codecs (`asap`, `spc`) — neither of which is in our required codec set.

**Fix**: Prefer `llvm@21` over the default `llvm` formula (which resolves to 22.x). Both `tools/configure` and `build-headless.sh` now search for `llvm@21` first:

```sh
for _oc in /opt/homebrew/opt/llvm@21/bin/llvm-objcopy \
           /usr/local/opt/llvm@21/bin/llvm-objcopy \
           /opt/homebrew/opt/llvm/bin/llvm-objcopy \
           ...
```

### 3. Zig's MachO linker does not scan archives from data-section relocations

**Symptom**: Zig build fails with 29 undefined symbols `___header_a52`, `___header_flac`, etc.

**Root cause**: `lc_static_table` in `lc-headless.c` is a data array of pointers to codec header structs. Each pointer is a data-section relocation pointing to `__header_<name>`. Zig's internal MachO linker uses **code-section undefined-symbol references** to decide which archive members to pull in; it does not process data-section relocations for archive scanning. Consequently the codec `.a` files are never opened even though they are listed on the link command.

Using `--force_undefined` (Zig's `forceUndefinedSymbol` API) does not help because LLD still does not scan the codec archives once the symbols are forced-defined as "undefined" — the force flag only prevents stripping, not loading.

**Fix**: Extract each codec's `.o` files directly from their `.a` archives using `ar x`, then pass the individual `.o` files to Zig as `addObjectFile`. Unconditionally included object files are always linked regardless of relocation type.

```zig
// build.zig
for (codec_names) |name| {
    const dir = b.pathJoin(&.{ obj_base, name });
    exe.root_module.addObjectFile(b.path(b.pathJoin(&.{ dir, b.fmt("{s}.o",      .{name}) })));
    exe.root_module.addObjectFile(b.path(b.pathJoin(&.{ dir, b.fmt("{s}-crt0.o", .{name}) })));
}
```

The support libraries (`liba52.a`, `libtremor.a`, etc.) are still passed as archives because the codec `.o` files reference their symbols via **code** relocations, which drives normal archive scanning correctly.

### 4. `libcodec.a` not built in CODECS_STATIC mode

**Symptom**: Zig build fails with undefined symbols `_codec_init`, `_codec_malloc`, `_codec_free`, `_codec_set_replaygain`, `_bs_clz_tab`, `_bs_log2_tab`, `_ff_copy_bits`, `_ff_fft_calc_c`.

**Root cause**: `lib/rbcodec/codecs/lib/codeclib.c` (and `ffmpeg_bitstream.c`, `fft-ffmpeg.c`, `mdct.c`) define the above symbols. They compile into `libcodec.a` (also called `CODECLIB` in the Make world). In the normal (non-static) codec build, `$(CODECS): $(CODEC_LIBS)` ensures `libcodec.a` is built as a dependency. In CODECS_STATIC mode, `codecs.make` explicitly guards this dependency away:

```makefile
ifndef CODECS_STATIC
$(CODECS): $(CODEC_LIBS)   # this must be last in codec dependency list
endif
```

So `libcodec.a` is never produced as a side-effect of `make lib` when `CODECS_STATIC=1`. It must be built as a separate explicit target.

**Fix**:
1. `build-headless.sh` (Step 2.1) builds `libcodec.a` explicitly if absent.
2. `build.zig` links it directly before the support libraries:

```zig
// build.zig — headless block
exe.root_module.addObjectFile(b.path(b.pathJoin(&.{ codec_dir, "libcodec.a" })));
```

### 5. cpal stream fails with "stream configuration not supported"

**Symptom**:
```
ERROR rockbox_cpal_sink: pcm-cpal: failed to open stream at 44100 Hz:
    The requested stream configuration is not supported by the device.
```

**Root cause**: The original implementation used a hardcoded `StreamConfig` with `sample_rate: 44100 Hz` and the `i16` format. On macOS, CoreAudio devices typically only advertise `f32` output, and many built-in audio devices only support 48000 Hz (not 44100 Hz).

**Fix**: `crates/cpal-sink/src/lib.rs` now:
1. Calls `device.supported_output_configs()` to enumerate all supported ranges.
2. Picks the first stereo range that covers the firmware rate (preferring `f32`).
3. Falls back to `device.default_output_config()` if no exact match is found.
4. If the device rate differs from the firmware rate, runs a **linear interpolation resampler** inline in the cpal callback.

**Resampler design**: The resampler is stateful — it persists a fractional phase accumulator between callback invocations. For each output stereo frame it advances the phase by `in_rate / out_rate` (e.g., `44100 / 48000 ≈ 0.9188`) and linearly interpolates between the previous and current input frame. This produces correct-pitch audio with no additional crate dependencies.

```
device: 48000 Hz F32  ←  resampler (step=0.9188)  ←  firmware ring: 44100 Hz S16LE
```

---

## File Map

| File | Purpose |
|---|---|
| `scripts/build-headless.sh` | Full build script — configure, make, cargo, zig |
| `tools/configure` | Rockbox configure script; `headlesshostcc()` sets up the headless target and finds `llvm@21` |
| `firmware/export/config/headlesshost.h` | C config header for the headless target |
| `firmware/target/hosted/headless/` | Headless-specific C sources (PCM sink, codec loader, etc.) |
| `firmware/target/hosted/headless/lc-headless.c` | `lc_static_table[]` — maps codec names to `__header_*` pointers |
| `firmware/target/hosted/headless/pcm-cpal.c` | C side of the cpal PCM sink; calls `pcm_cpal_push()` / `pcm_cpal_set_sample_rate()` |
| `lib/rbcodec/codecs/codecs.make` | Per-codec build rules; `CODECS_STATIC` block (line 273+) handles symbol renaming |
| `lib/rbcodec/codecs/lib/codeclib.c` | `codec_init`, `codec_malloc`, `bs_clz_tab`, etc. → compiled into `libcodec.a` |
| `crates/cpal-sink/src/lib.rs` | Rust cpal backend — ring buffer, resampler, stream negotiation |
| `zig/build.zig` | Zig linker script — `headless` block lists all `.o` files and `.a` archives |

---

## Rebuild After Changes

| Changed | Command |
|---|---|
| Any C firmware file | `cd build-headless && make lib OC=...` then `cd zig && zig build -Dheadless=true` |
| `lc-headless.c` or codec C files | Same as above |
| `crates/cpal-sink/` | `cargo build --release --features cpal-sink -p rockbox-cli` then `zig build` |
| Any other Rust crate | `cargo build --release -p rockbox-cli -p rockbox-server` then `zig build` |
| `zig/build.zig` | `cd zig && zig build -Dheadless=true` |
| Everything | `bash scripts/build-headless.sh` |

> **Stale binary pitfall**: `zig build` only re-links if the `.a` files are newer than the binary. Always rebuild Make/Cargo before running `zig build` after changing C or Rust code.
