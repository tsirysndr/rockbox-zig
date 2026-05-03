#             __________               __   ___.
#   Open      \______   \ ____   ____ |  | _\_ |__   _______  ___
#   Source     |       _//  _ \_/ ___\|  |/ /| __ \ /  _ \  \/  /
#   Jukebox    |    |   (  <_> )  \___|    < | \_\ (  <_> > <  <
#   Firmware   |____|_  /\____/ \___  >__|_ \|___  /\____/__/\_ \
#                     \/            \/     \/    \/            \/
#
# Build glue for the Android cdylib target. The final .so (librockbox_expo.so)
# is produced by cargo via crates/expo/build.rs, NOT by this make. Our job:
#
#   1. Pull in the Android target-tree sources (system-android.c,
#      pcm-aaudio.c, lc-android.c, rb_zig_compat.c, lcd-noop.c, button-noop.c)
#      so they end up in libfirmware.a.
#   2. Build a "static-libs.stamp" sentinel after all required archives have
#      been (re)built — $(BINARY) points at it via tools/configure.

ANDROID_CDYLIB_DIR := $(ROOTDIR)/firmware/target/hosted/android/cdylib

INCLUDES  += -I$(ANDROID_CDYLIB_DIR)
OTHER_SRC += $(call preprocess, $(ANDROID_CDYLIB_DIR)/SOURCES)

# ── Sentinel target ─────────────────────────────────────────────────────────
# $(BINARY) = "static-libs.stamp" (set by tools/configure for this target).
# Real artifacts are the .a files in $(BUILDDIR)/{firmware,lib,...}.
ANDROID_CDYLIB_LIBS := \
    $(ROCKBOXLIB)            \
    $(FIRMLIB)               \
    $(RBCODECLIB)            \
    $(CORE_LIBS)             \
    $(CODECS)

$(BUILDDIR)/$(BINARY): $(ANDROID_CDYLIB_LIBS)
	$(call PRINTS,STAMP $(@F))
	$(SILENT) mkdir -p $(dir $@)
	$(SILENT) { \
	    echo "# Built $$(date -u +%Y-%m-%dT%H:%M:%SZ)"; \
	    echo "# ABI:        $(ANDROID_ABI)"; \
	    echo "# API level:  $(ANDROID_API_LEVEL)"; \
	    echo "# Archives:"; \
	    for f in $^; do \
	        sz=$$(stat -f%z "$$f" 2>/dev/null || stat -c%s "$$f" 2>/dev/null); \
	        echo "#   $$f  ($$sz bytes)"; \
	    done; \
	} > $@
	$(SILENT) echo "✔ $(MODELNAME) static libs ready for cargo link"

# RBINFO is a runtime info file consumed by the on-device firmware boot
# path. The cdylib has no on-device boot path — JNI handles init — so the
# upstream rule's mkinfo.pl invocation is replaced with a no-op touch.
$(RBINFO): $(BUILDDIR)/$(BINARY)
	$(SILENT) touch $@
