#             __________               __   ___.
#   Open      \______   \ ____   ____ |  | _\_ |__   _______  ___
#   Source     |       _//  _ \_/ ___\|  |/ /| __ \ /  _ \  \/  /
#   Jukebox    |    |   (  <_> )  \___|    < | \_\ (  <_> > <  <
#   Firmware   |____|_  /\____/ \___  >__|_ \|___  /\____/__/\_ \
#                     \/            \/     \/    \/            \/
#
# Build glue for the headless host target (macOS / Linux, no SDL, cpal PCM).
# The final binary (rockboxd) is produced by `zig build -Dheadless=true`,
# NOT by this make. Our job:
#
#   1. Pull in the headless target-tree sources (system-headless.c,
#      pcm-cpal.c, lc-headless.c, rb_zig_compat.c, …) so they end up
#      in libfirmware.a.
#   2. Build a "static-libs.stamp" sentinel once all required archives
#      have been (re)built — $(BINARY) points at it via tools/configure.

HEADLESS_DIR := $(ROOTDIR)/firmware/target/hosted/headless

INCLUDES  += -I$(HEADLESS_DIR)
OTHER_SRC += $(call preprocess, $(HEADLESS_DIR)/SOURCES)

# ── Sentinel target ──────────────────────────────────────────────────────────
# $(BINARY) = "static-libs.stamp" (set by tools/configure for this target).
# Real artifacts are the .a files consumed by `zig build -Dheadless=true`.
HEADLESS_LIBS := \
    $(ROCKBOXLIB)   \
    $(FIRMLIB)      \
    $(RBCODECLIB)   \
    $(CORE_LIBS)    \
    $(CODECS)

$(BUILDDIR)/$(BINARY): $(HEADLESS_LIBS)
	$(call PRINTS,STAMP $(@F))
	$(SILENT) mkdir -p $(dir $@)
	$(SILENT) { \
	    echo "# Built $$(date -u +%Y-%m-%dT%H:%M:%SZ)"; \
	    echo "# Target: headless host (macOS / Linux, cpal PCM)"; \
	    echo "# Archives:"; \
	    for f in $^; do \
	        sz=$$(stat -f%z "$$f" 2>/dev/null || stat -c%s "$$f" 2>/dev/null); \
	        echo "#   $$f  ($$sz bytes)"; \
	    done; \
	} > $@
	$(SILENT) echo "✔ headless static libs ready for: zig build -Dheadless=true"

$(RBINFO): $(BUILDDIR)/$(BINARY)
	$(SILENT) touch $@
