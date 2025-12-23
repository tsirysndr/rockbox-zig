#!/bin/bash

set -e -o pipefail

mkdir -p /tmp/rockbox-pkg/usr/local/{bin,lib,share}

cp /usr/local/bin/rockbox* /tmp/rockbox-pkg/usr/local/bin
cp -r /usr/local/lib/rockbox /tmp/rockbox-pkg/usr/local/lib
cp -r /usr/local/share/rockbox /tmp/rockbox-pkg/usr/local/share

export ARCH=$(uname -m)
export VERSION=$(git describe --tags --abbrev=0)

pkgbuild \
  --identifier "com.github.rockbox-zig" \
  --version "0.1.0" \
  --root "/tmp/rockbox-pkg" \
  --install-location "/" \
  "rockbox-installer-${ARCH}.pkg"

rm -rf /tmp/rockbox-pkg
