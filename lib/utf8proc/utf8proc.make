#             __________               __   ___.
#   Open      \______   \ ____   ____ |  | _\_ |__   _______  ___
#   Source     |       _//  _ \_/ ___\|  |/ /| __ \ /  _ \  \/  /
#   Jukebox    |    |   (  <_> )  \___|    < | \_\ (  <_> > <  <
#   Firmware   |____|_  /\____/ \___  >__|_ \|___  /\____/__/\_ \
#                     \/            \/     \/    \/            \/
#

UTF8PROC_DIR := $(ROOTDIR)/lib/utf8proc
UTF8PROC_SRC := $(UTF8PROC_DIR)/utf8proc.c
UTF8PROC_OBJ := $(call c2obj, $(UTF8PROC_SRC))

OTHER_SRC += $(UTF8PROC_SRC)

LIBUTF8PROC = $(BUILDDIR)/lib/libutf8proc.a
CORE_LIBS += $(LIBUTF8PROC)

INCLUDES += -I$(UTF8PROC_DIR)

DEFINES += -DUTF8PROC_EXPORTS

$(LIBUTF8PROC): $(UTF8PROC_OBJ)
	$(SILENT)$(shell rm -f $@)
	$(call PRINTS,AR $(@F))$(AR) rcs $@ $^ >/dev/null
