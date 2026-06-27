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
#
# Requires GNU binutils `ar` for the `xN <occurrence>` modifier — Linux ships
# this as `/usr/bin/ar`. macOS has BSD ar which lacks `-N`, but the macOS
# build path uses `libtool -static` and never calls this script. Override
# with `AR=/path/to/gnu-ar` to test on macOS.
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

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

# Recursive extraction. Two collision modes have to be handled:
#   1. Cross-archive: different nested archives both contain `cpuinfo-noop.o`.
#      Solved by prefixing each renamed .o with its archive lineage.
#   2. Within-archive: a single archive lists two members with the same
#      name. apps/SOURCES adds both `gui/list.o` and `gui/bitmap/list.o` to
#      librockbox.a; both end up as basename `list.o`. A bulk `ar x` lets
#      the second overwrite the first and silently drops a whole .o worth of
#      symbols (gui_synclist_*, simplelist_*, list_do_action…). Solved by
#      walking `ar t` in order, tracking the occurrence count per name, and
#      using `ar xN <n>` to pull the n-th occurrence into its own temp dir.
extract() {
  local src="$1" outdir="$2" prefix="$3"
  # Loop variables must be `local` — without this, the recursive call would
  # clobber the outer frame's `nested` (and leave it empty on return).
  local name nested i=0 idx=0 occ td
  local -A occ_count=()
  mkdir -p "$outdir"

  while IFS= read -r name; do
    idx=$((idx + 1))
    occ_count[$name]=$((${occ_count[$name]:-0} + 1))
    occ=${occ_count[$name]}
    td="$outdir/.ext.${idx}"
    mkdir -p "$td"
    ( cd "$td" && "$AR" xN "$occ" "$src" "$name" ) || true
    if [ -f "$td/$name" ]; then
      mv "$td/$name" "$outdir/${prefix}__${idx}__$(basename "$name")"
    fi
    rm -rf "$td"
  done < <("$AR" t "$src")

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
find "$work/objs" -name '*.o' -print0 | xargs -0 "$AR" -rcs "$out"

mv "$out" "$archive"
echo "flatten-archive: $("$AR" t "$archive" | wc -l) object(s) -> $archive"
