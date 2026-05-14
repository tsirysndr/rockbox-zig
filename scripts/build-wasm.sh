#!/usr/bin/env bash
# Build rockboxd as a WebAssembly module for browser embedding.
#
# What this does:
#   1. Creates and configures build-wasm/ (wasmapp target, target 207).
#   2. Builds all firmware .a files with emcc (static codecs, no SDL).
#   3. Compiles crates/wasm/ with cargo (wasm32-unknown-emscripten target).
#   4. Links everything via emcc into web/rockboxd.js + web/rockboxd.wasm.
#
# Prerequisites:
#   - Emscripten SDK 3.1.x installed and activated (emsdk activate latest)
#   - Rust wasm32-unknown-emscripten target (rustup target add wasm32-unknown-emscripten)
#   - llvm-objcopy (for CODECS_STATIC symbol renaming; Homebrew llvm on macOS)
#
# Usage:
#   source /path/to/emsdk/emsdk_env.sh
#   bash scripts/build-wasm.sh
#   bash scripts/build-wasm.sh --debug    # debug build
#
# Output:
#   web/rockboxd.js    — Emscripten JS loader (use with MODULARIZE=1)
#   web/rockboxd.wasm  — WebAssembly binary

set -euo pipefail

ROOTDIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOTDIR"

PROFILE="${PROFILE:-release}"
CARGO_FLAG="--release"
if [ "$PROFILE" = "debug" ] || [[ "${1:-}" == "--debug" ]]; then
    PROFILE="debug"
    CARGO_FLAG=""
    EMCC_OPT="${EMCC_OPT:--O0 -g}"
else
    EMCC_OPT="${EMCC_OPT:--O2}"
fi

# ── Toolchain checks ──────────────────────────────────────────────────────────

if ! command -v emcc &>/dev/null; then
    echo "ERROR: emcc not found. Activate the Emscripten SDK first:"
    echo "  source /path/to/emsdk/emsdk_env.sh"
    exit 1
fi

if ! rustup target list --installed | grep -q "wasm32-unknown-emscripten"; then
    echo "ERROR: wasm32-unknown-emscripten Rust target not installed."
    echo "  rustup target add wasm32-unknown-emscripten"
    exit 1
fi

EMCC_VERSION="$(emcc --version 2>&1 | head -1)"
echo "==> Using: $EMCC_VERSION"

# ── llvm-objcopy for CODECS_STATIC symbol renaming ────────────────────────────

LLVM_OBJCOPY=""
if [[ "$(uname)" == "Darwin" ]]; then
    # emcc bundles a compatible llvm-objcopy; prefer it.
    EMSDK_OC="$(dirname "$(command -v emcc)")/../bin/llvm-objcopy"
    if [ -x "$EMSDK_OC" ]; then
        LLVM_OBJCOPY="$EMSDK_OC"
    fi
    # Fall back to Homebrew llvm (prefer @21 over @22 — @22 segfaults on some codecs).
    if [ -z "$LLVM_OBJCOPY" ]; then
        for _oc in /opt/homebrew/opt/llvm@21/bin/llvm-objcopy \
                   /usr/local/opt/llvm@21/bin/llvm-objcopy \
                   /opt/homebrew/opt/llvm/bin/llvm-objcopy \
                   /usr/local/opt/llvm/bin/llvm-objcopy; do
            if [ -x "$_oc" ]; then
                LLVM_OBJCOPY="$_oc"
                break
            fi
        done
    fi
    if [ -z "$LLVM_OBJCOPY" ]; then
        echo "WARNING: llvm-objcopy not found; codec archives may be corrupted."
        echo "  Install via: brew install llvm"
        LLVM_OBJCOPY="llvm-objcopy"  # hope it's on PATH
    fi
else
    # Linux: emscripten bundles llvm-objcopy, or use system one.
    EMSDK_OC="$(dirname "$(command -v emcc)")/../bin/llvm-objcopy"
    if [ -x "$EMSDK_OC" ]; then
        LLVM_OBJCOPY="$EMSDK_OC"
    else
        LLVM_OBJCOPY="${LLVM_OBJCOPY:-llvm-objcopy}"
    fi
fi
echo "==> Using llvm-objcopy: $LLVM_OBJCOPY"

# ── Step 1: Configure build-wasm/ ─────────────────────────────────────────────

echo ""
echo "==> Step 1: Configure build-wasm/ (target 207 — wasmapp)"
mkdir -p build-wasm
if [ ! -f build-wasm/Makefile ]; then
    (cd build-wasm && printf "207\nN\n" | ../tools/configure)
else
    echo "    build-wasm/Makefile already exists, skipping configure"
fi

# ── Step 2: Build firmware static libs with emcc ──────────────────────────────

echo ""
echo "==> Step 2: Build firmware static libs (emcc)"

# Delete per-codec artefacts so Make rebuilds them with the correct objcopy.
CODEC_OBJ_DIR="$ROOTDIR/build-wasm/lib/rbcodec/codecs"
for name in a52 a52_rm aac aac_bsf adx aiff alac ape \
            atrac3_oma atrac3_rm au cook flac mod mpa mpc \
            opus raac shorten smaf speex tta vorbis vox \
            wav wav64 wavpack wma wmapro; do
    rm -f "$CODEC_OBJ_DIR/$name.o" "$CODEC_OBJ_DIR/$name.a" \
          "$CODEC_OBJ_DIR/$name-crt0.o"
done

# Ensure codec lib dir exists as a directory (not a stale empty file).
CODEC_LIB_DIR="$ROOTDIR/build-wasm/lib/rbcodec/codecs/lib"
if [ -f "$CODEC_LIB_DIR" ] && [ ! -d "$CODEC_LIB_DIR" ]; then
    rm -f "$CODEC_LIB_DIR"
fi

NCPU="$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)"
# -k: keep going; spc and asap crash llvm-objcopy on macOS (both codecs are unused).
(cd build-wasm && make -j"$NCPU" -k lib OC="$LLVM_OBJCOPY") || {
    if [[ "$(uname)" == "Darwin" ]]; then
        echo "    Note: spc/asap build failures are expected on macOS (llvm-objcopy crash — codecs unused)."
    else
        echo "ERROR: 'make lib' failed"; exit 1
    fi
}

# ── Step 2.1: Ensure libcodec.a is built ──────────────────────────────────────

echo ""
echo "==> Step 2.1: Build libcodec.a"
CODEC_DIR="$ROOTDIR/build-wasm/lib/rbcodec/codecs"
LIBCODEC="$CODEC_DIR/libcodec.a"
if [ ! -f "$LIBCODEC" ]; then
    (cd build-wasm && make -j"$NCPU" "$LIBCODEC" OC="$LLVM_OBJCOPY")
fi
echo "    libcodec.a: $(ls -lh "$LIBCODEC" | awk '{print $5}')"

# ── Step 3: Build Rust crate (wasm32-unknown-emscripten) ──────────────────────

echo ""
echo "==> Step 3: Build rockbox-wasm (wasm32-unknown-emscripten)"
RUSTFLAGS="-C target-feature=+atomics,+bulk-memory,+mutable-globals" \
    CARGO_TARGET_WASM32_UNKNOWN_EMSCRIPTEN_LINKER="emcc" \
    cargo build $CARGO_FLAG --target wasm32-unknown-emscripten -p rockbox-wasm
RUST_LIB="$ROOTDIR/target/wasm32-unknown-emscripten/$PROFILE/librockbox_wasm.a"
echo "    rockbox-wasm: $(ls -lh "$RUST_LIB" | awk '{print $5}')"

# ── Step 4: emcc link step ────────────────────────────────────────────────────

if [ "${SKIP_LINK:-0}" = "1" ]; then
    echo ""
    echo "==> Step 4: Skipping link step (SKIP_LINK=1)"
    echo "✔ Compile check complete (C firmware + Rust crate)."
    exit 0
fi

echo ""
echo "==> Step 4: Link rockboxd.{js,wasm} with emcc"

OUTPUT_DIR="$ROOTDIR/web"
mkdir -p "$OUTPUT_DIR"

BUILD="$ROOTDIR/build-wasm"

# Collect per-codec .a files (codec wrappers with renamed symbols).
CODEC_ARCHIVES=()
for name in a52 a52_rm aac aac_bsf adx aiff alac ape \
            atrac3_oma atrac3_rm au cook flac mod mpa mpc \
            opus raac shorten smaf speex tta vorbis vox \
            wav wav64 wavpack wma wmapro; do
    arc="$CODEC_DIR/$name.a"
    [ -f "$arc" ] && CODEC_ARCHIVES+=("$arc")
done

# Collect codec helper libraries (lib*.a in the codecs dir).
CODEC_HELPER_LIBS=()
for arc in "$CODEC_DIR"/lib*.a; do
    [ -f "$arc" ] && CODEC_HELPER_LIBS+=("$arc")
done

# All rb_* exports (note: Emscripten expects leading underscore on native-arch
# but WASM exports use the plain name; emcc handles the mapping automatically).
EXPORTED_FUNCTIONS='["_malloc","_free",
    "_rb_daemon_start","_rb_daemon_state","_rb_free_string",
    "_rb_play_url","_rb_enqueue_url",
    "_rb_play","_rb_pause","_rb_play_pause",
    "_rb_next","_rb_prev","_rb_seek","_rb_stop",
    "_rb_clear_queue","_rb_shuffle_queue","_rb_jump_to_queue_position",
    "_rb_adjust_volume","_rb_sound_current",
    "_rb_status_json","_rb_current_track_json","_rb_playlist_json",
    "_rb_settings_json",
    "_rb_set_eq_enabled","_rb_set_eq_precut","_rb_set_eq_band",
    "_rb_set_crossfade","_rb_set_replaygain","_rb_save_settings",
    "_rb_set_balance","_rb_set_channel_mode","_rb_set_stereo_width",
    "_rb_set_crossfeed","_rb_set_surround",
    "_rb_set_bass","_rb_set_treble",
    "_rb_set_dithering","_rb_set_afr","_rb_set_pbe","_rb_set_timestretch",
    "_rb_set_repeat",
    "_rb_pcm_ring_ptr","_rb_pcm_ring_frames",
    "_rb_pcm_write_idx_ptr","_rb_pcm_read_idx_ptr","_rb_pcm_sample_rate_ptr"]'
# Flatten to a single line (no whitespace inside the JSON array).
EXPORTED_FUNCTIONS="$(echo "$EXPORTED_FUNCTIONS" | tr -d '\n ' | sed 's/,/ ,/g' | tr -d ' ')"

emcc \
    -o "$OUTPUT_DIR/rockboxd.js" \
    $EMCC_OPT \
    -pthread \
    -sPTHREAD_POOL_SIZE=8 \
    -sINITIAL_MEMORY=335544320 \
    -sALLOW_MEMORY_GROWTH=1 \
    -sSTACK_SIZE=2097152 \
    -sDEFAULT_PTHREAD_STACK_SIZE=524288 \
    -sMODULARIZE=1 \
    -sEXPORT_NAME=RockboxModule \
    -sNO_EXIT_RUNTIME=1 \
    "-sEXPORTED_RUNTIME_METHODS=[\"UTF8ToString\",\"lengthBytesUTF8\",\"stringToUTF8\",\"HEAP8\",\"HEAPU8\",\"FS\"]" \
    "-sEXPORTED_FUNCTIONS=$EXPORTED_FUNCTIONS" \
    -sENVIRONMENT=web,worker \
    -Wl,--allow-multiple-definition \
    -Wl,--no-check-features \
    "$RUST_LIB" \
    "$BUILD/firmware/libfirmware.a" \
    "$BUILD/librockbox.a" \
    "$BUILD/lib/librbcodec.a" \
    "$BUILD/lib/libfixedpoint.a" \
    "$BUILD/lib/libtlsf.a" \
    "$BUILD/lib/libskin_parser.a" \
    -Wl,--whole-archive \
    "${CODEC_ARCHIVES[@]}" \
    -Wl,--no-whole-archive \
    "${CODEC_HELPER_LIBS[@]}"
# Note: -lidbfs.js is intentionally omitted — it injects C-level FS init code into
# every pthread worker at startup, which crashes in Rockbox's thread bootstrap.
# Persistence is handled entirely in JS via native indexedDB + MEMFS file I/O.

echo ""
echo "✔ Build complete:"
echo "    $OUTPUT_DIR/rockboxd.js"
echo "    $OUTPUT_DIR/rockboxd.wasm"
echo ""
echo "Serve web/ with COOP/COEP headers (required for SharedArrayBuffer):"
echo "  Cross-Origin-Opener-Policy:   same-origin"
echo "  Cross-Origin-Embedder-Policy: require-corp"
echo ""
echo "Minimal dev server (Node):"
echo "  node scripts/wasm-dev-server.mjs"
