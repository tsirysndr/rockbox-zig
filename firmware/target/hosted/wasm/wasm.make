#             __________               __   ___.
#   Open      \______   \ ____   ____ |  | _\_ |__   _______  ___
#   Source     |       _//  _ \_/ ___\|  |/ /| __ \ /  _ \  \/  /
#   Jukebox    |    |   (  <_> )  \___|    < | \_\ (  <_> > <  <
#   Firmware   |____|_  /\____/ \___  >__|_ \|___  /\____/__/\_ \
#                     \/            \/     \/    \/            \/
#
# Build glue for the WASM browser target. The final .wasm + .js are
# produced by scripts/build-wasm.sh (emcc link step), NOT by this make.
# Our job:
#
#   1. Pull in the WASM target-tree sources (system-wasm.c, pcm-webapi.c,
#      lc-wasm.c, rb_zig_compat.c, wasm-bridge.c, noops, …) so they end up
#      in libfirmware.a (a WASM object archive compiled by emcc).
#   2. Build a "static-libs.stamp" sentinel once all required archives have
#      been (re)built — $(BINARY) points at it via tools/configure.

WASM_DIR := $(ROOTDIR)/firmware/target/hosted/wasm

INCLUDES  += -I$(WASM_DIR)
OTHER_SRC += $(call preprocess, $(WASM_DIR)/SOURCES)

# ── Sentinel target ──────────────────────────────────────────────────────────
# $(BINARY) = "static-libs.stamp" (set by tools/configure for this target).
# Real artifacts are the WASM .a files consumed by scripts/build-wasm.sh.
WASM_LIBS := \
    $(ROCKBOXLIB)   \
    $(FIRMLIB)      \
    $(RBCODECLIB)   \
    $(CORE_LIBS)    \
    $(CODECLIB)     \
    $(CODECS)

$(BUILDDIR)/$(BINARY): $(WASM_LIBS)
	$(call PRINTS,STAMP $(@F))
	$(SILENT) mkdir -p $(dir $@)
	$(SILENT) { \
	    echo "# Built $$(date -u +%Y-%m-%dT%H:%M:%SZ)"; \
	    echo "# Target: WASM browser (Emscripten, Web Audio API)"; \
	    echo "# Archives:"; \
	    for f in $^; do \
	        sz=$$(stat -f%z "$$f" 2>/dev/null || stat -c%s "$$f" 2>/dev/null); \
	        echo "#   $$f  ($$sz bytes)"; \
	    done; \
	} > $@
	$(SILENT) echo "✔ WASM static libs ready for: scripts/build-wasm.sh"

$(RBINFO): $(BUILDDIR)/$(BINARY)
	$(SILENT) touch $@
