#!/bin/bash
set -euo pipefail

TMP=/tmp/rockbox-pkg

case "$(uname -m)" in
  x86_64)
    ARCH="x86_64"
    ;;
  arm64)
    ARCH="aarch64"
    ;;
  *)
    echo "Unsupported architecture: $(uname -m)"
    exit 1
    ;;
esac

VERSION=$(git describe --tags --abbrev=0)

mkdir -p "$TMP/usr/local"/{bin,lib,share}
mkdir -p "/tmp/scripts"

cp target/release/rockbox "$TMP/usr/local/bin"
cp zig/zig-out/bin/rockboxd "$TMP/usr/local/bin"
cp ./packaging/macos/postinstall "/tmp/scripts"

pkgbuild \
  --identifier "com.github.rockboxd" \
  --version "$VERSION" \
  --root "$TMP" \
  --scripts "/tmp/scripts" \
  --install-location "/" \
  "rockbox-${VERSION}-${ARCH}.pkg"

rm -rf "$TMP" "/tmp/scripts"
