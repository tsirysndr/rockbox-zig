/* SPDX-License-Identifier: GPL-2.0-or-later
 *
 * Static-link variant of the codec/plugin loader. Used by the Android cdylib
 * build (CONFIG_BINFMT=BINFMT_STATIC) where dlopen of arbitrary shared libs
 * from app data dirs is blocked by the OS. All codecs are linked into the
 * parent binary; lc_open() returns a pointer into a compile-time table built
 * by codecs.make when CODECS_STATIC=1 is set, then implemented by
 * firmware/target/hosted/android/cdylib/lc-android.c.
 */
#ifndef __LC_STATIC_H__
#define __LC_STATIC_H__

#include <stddef.h>

struct codec_header;   /* forward; full def in lib/rbcodec/codecs/codecs.h */

struct lc_static_entry {
    const char                  *fname;   /* "flac.codec" etc. */
    const struct codec_header   *hdr;
};

/* Generated table; defined in firmware/target/hosted/android/cdylib/
 * codec_table.c by the codecs.make CODECS_STATIC mode. NULL-terminated. */
extern const struct lc_static_entry lc_static_table[];

#endif /* __LC_STATIC_H__ */
