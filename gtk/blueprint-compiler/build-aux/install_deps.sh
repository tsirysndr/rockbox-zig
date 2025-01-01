#!/bin/bash

set -e

echo "===== Install GTK ====="
git clone --depth=1 https://gitlab.gnome.org/GNOME/gtk.git
cd gtk
meson setup builddir \
  --prefix=/usr \
  -Ddocumentation=true \
  -Dbuild-demos=false \
  -Dbuild-examples=false \
  -Dbuild-tests=false \
  -Dbuild-testsuite=false
ninja -C builddir install
cd -
rm -rf gtk

echo "===== Install libadwaita ====="
git clone --depth=1 https://gitlab.gnome.org/GNOME/libadwaita.git
cd libadwaita
meson builddir \
  --prefix=/usr
ninja -C builddir install
cd -
rm -rf libadwaita
