#!/usr/bin/env bash
# Flatten a Zig-produced static archive into one containing only .o members.
#
# `zig build lib` packs each input static archive as a member of the output
# archive — librockboxd.a ends up with nested `.a` (and even `.so`) files as
# members instead of the underlying .o objects. macOS works around this by
# repacking via `libtool -static`. GNU/Linux linkers (including rust-lld)
# don't unpack nested archive members, so consumers see "is neither ET_REL
# nor LLVM bitcode" warnings and undefined symbols from the nested archives.
#
# This script extracts all object files (recursively through nested archives)
# and re-archives them flat. Dynamic library refs (.so) are dropped — they're
# expected to be linked via -l flags at the consumer's link step, not bundled.
set -euo pipefail

if [ "$#" -ne 1 ]; then
  echo "usage: $0 <archive.a>" >&2
  exit 64
fi

archive="$(realpath "$1")"
if [ ! -f "$archive" ]; then
  echo "flatten-archive: not found: $archive" >&2
  exit 1
fi

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

# Extract recursively, prefixing each .o with a unique slug so colliding
# basenames across nested archives don't overwrite each other.
extract() {
  local src="$1" outdir="$2" prefix="$3"
  # Loop variables must be `local` — without this, the recursive call would
  # clobber the outer frame's `nested` (and leave it empty on return).
  local obj nested i=0
  mkdir -p "$outdir"
  ( cd "$outdir" && ar x "$src" )

  while IFS= read -r -d '' obj; do
    mv "$obj" "$outdir/${prefix}__$(basename "$obj")"
  done < <(find "$outdir" -maxdepth 1 -name '*.o' -print0)

  find "$outdir" -maxdepth 1 -name '*.so*' -delete

  while IFS= read -r -d '' nested; do
    extract "$(realpath "$nested")" "$outdir/sub_${i}" \
            "${prefix}__$(basename "$nested" .a)_${i}"
    rm "$nested"
    i=$((i + 1))
  done < <(find "$outdir" -maxdepth 1 -name '*.a' -print0)
}

extract "$archive" "$work/objs" "root"

out="$work/flat.a"
find "$work/objs" -name '*.o' -print0 | xargs -0 ar -rcs "$out"

mv "$out" "$archive"
echo "flatten-archive: $(ar t "$archive" | wc -l) object(s) -> $archive"
