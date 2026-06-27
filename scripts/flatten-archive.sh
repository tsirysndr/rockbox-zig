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
# Delegates the actual parsing to flatten-archive.py — `ar xN <occ>` proved
# hopeless against archives whose member names are absolute paths (which is
# what llvm-ar stores when zig adds them via b.path(...)), so we parse the
# ar format directly. After extraction, this script just re-archives the
# resulting .o files with the system ar.
set -euo pipefail

AR="${AR:-ar}"

if [ "$#" -ne 1 ]; then
  echo "usage: $0 <archive.a>" >&2
  exit 64
fi

archive="$(realpath "$1")"
if [ ! -f "$archive" ]; then
  echo "flatten-archive: not found: $archive" >&2
  exit 1
fi

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

python3 "$script_dir/flatten-archive.py" "$archive" "$work/objs"

# Re-archive every .o into one flat archive
out="$work/flat.a"
find "$work/objs" -name '*.o' -print0 | xargs -0 "$AR" -rcs "$out"

mv "$out" "$archive"
echo "flatten-archive: $("$AR" t "$archive" | wc -l) object(s) -> $archive"
