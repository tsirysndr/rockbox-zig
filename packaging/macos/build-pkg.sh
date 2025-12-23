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

cp /usr/local/bin/rockbox* "$TMP/usr/local/bin"
cp -R /usr/local/lib/rockbox "$TMP/usr/local/lib"
cp -R /usr/local/share/rockbox "$TMP/usr/local/share"

pkgbuild \
  --identifier "com.github.rockbox-zig" \
  --version "$VERSION" \
  --root "$TMP" \
  --install-location "/" \
  "rockbox-${VERSION}-${ARCH}.pkg"

rm -rf "$TMP"
