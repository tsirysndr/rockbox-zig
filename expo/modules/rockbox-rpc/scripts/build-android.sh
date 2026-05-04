#!/usr/bin/env bash
# Build the rockbox-expo Rust cdylib for Android via cargo-ndk and drop the
# resulting .so files under android/src/main/jniLibs/<abi>/.
#
# Prereqs:
#   cargo install cargo-ndk
#   rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
#   ANDROID_NDK_HOME must point at a valid NDK r25+ install.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MODULE_DIR="$(dirname "$SCRIPT_DIR")"
WORKSPACE_ROOT="$(cd "$MODULE_DIR/../../.." && pwd)"

PROFILE=${PROFILE:-release}
JNILIBS="$MODULE_DIR/android/src/main/jniLibs"
mkdir -p "$JNILIBS"

cd "$WORKSPACE_ROOT"

# embedded-daemon links the full C firmware + Rust server into the cdylib
# so the phone can play audio in-process and bind gRPC/HTTP/GraphQL/MPD
# locally. Without it the .so is a thin tonic gRPC client only.
FEATURES=${FEATURES:-embedded-daemon}

cargo ndk \
  -t arm64-v8a \
  --platform 26 \
  -o "$JNILIBS" \
  build -p rockbox-expo \
        --features "$FEATURES" \
        $( [[ "$PROFILE" == "release" ]] && echo "--release" )

echo
echo "Built $JNILIBS/<abi>/librockbox_expo.so"
