#!/usr/bin/env bash
# Build the rockbox-expo Rust static lib for iOS device + simulator and pack
# them into an .xcframework consumed by the Pod.
#
# Required Rust targets (install once):
#   rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios
#
# Output: expo/modules/rockbox-rpc/ios/RockboxExpo.xcframework
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MODULE_DIR="$(dirname "$SCRIPT_DIR")"
WORKSPACE_ROOT="$(cd "$MODULE_DIR/../../.." && pwd)"

LIB_NAME="librockbox_expo.a"
PROFILE=${PROFILE:-release}
CARGO_FLAGS=()
if [[ "$PROFILE" == "release" ]]; then
  CARGO_FLAGS+=(--release)
fi

# 1. Build for each iOS target
( cd "$WORKSPACE_ROOT" && cargo build -p rockbox-expo "${CARGO_FLAGS[@]}" --target aarch64-apple-ios )
( cd "$WORKSPACE_ROOT" && cargo build -p rockbox-expo "${CARGO_FLAGS[@]}" --target aarch64-apple-ios-sim )
( cd "$WORKSPACE_ROOT" && cargo build -p rockbox-expo "${CARGO_FLAGS[@]}" --target x86_64-apple-ios )

DEVICE_LIB="$WORKSPACE_ROOT/target/aarch64-apple-ios/$PROFILE/$LIB_NAME"
SIM_ARM_LIB="$WORKSPACE_ROOT/target/aarch64-apple-ios-sim/$PROFILE/$LIB_NAME"
SIM_X86_LIB="$WORKSPACE_ROOT/target/x86_64-apple-ios/$PROFILE/$LIB_NAME"

# 2. Create a fat simulator slice (arm64 + x86_64)
SIM_FAT_DIR="$WORKSPACE_ROOT/target/ios-sim-fat"
mkdir -p "$SIM_FAT_DIR"
SIM_FAT_LIB="$SIM_FAT_DIR/$LIB_NAME"
lipo -create "$SIM_ARM_LIB" "$SIM_X86_LIB" -output "$SIM_FAT_LIB"

# 3. Pack into an xcframework with a tiny module map so Swift can `import` it
HEADERS_DIR="$MODULE_DIR/ios/headers"
mkdir -p "$HEADERS_DIR"
cat > "$HEADERS_DIR/RockboxExpo.h" <<'HDR'
#pragma once
#include <stdint.h>

int32_t  rb_set_server_url(const char *url);
int32_t  rb_ping(void);
int32_t  rb_play(void);
int32_t  rb_pause(void);
int32_t  rb_play_pause(void);
int32_t  rb_next(void);
int32_t  rb_prev(void);
int32_t  rb_seek(int32_t position_ms);
char    *rb_status_json(void);
char    *rb_current_track_json(void);
int32_t  rb_like_track(const char *id);
int32_t  rb_unlike_track(const char *id);
void     rb_free_string(char *ptr);
HDR
cat > "$HEADERS_DIR/module.modulemap" <<'MODMAP'
module RockboxExpo {
  header "RockboxExpo.h"
  export *
}
MODMAP

OUT="$MODULE_DIR/ios/RockboxExpo.xcframework"
rm -rf "$OUT"
xcodebuild -create-xcframework \
  -library "$DEVICE_LIB"  -headers "$HEADERS_DIR" \
  -library "$SIM_FAT_LIB" -headers "$HEADERS_DIR" \
  -output "$OUT"

echo
echo "Built $OUT"
