/* SPDX-License-Identifier: GPL-2.0-or-later
 *
 * BINFMT_STATIC implementation of the codec/plugin loader for Android.
 * lc_open(name) → pointer into the compile-time codec table built by
 * codecs.make when CODECS_STATIC=1. No dlopen, no filesystem access.
 */

#include <string.h>
#include <stddef.h>

#include "load_code.h"     /* pulls lc-static.h via CONFIG_BINFMT dispatch */

void *lc_open(const char *filename, unsigned char *buf, size_t buf_size)
{
    (void)buf;
    (void)buf_size;

    if (!filename) return NULL;

    /* Caller may pass a bare name ("flac.codec") or a full path
     * ("/codecs/flac.codec"). Match on basename only. */
    const char *base = strrchr(filename, '/');
    base = base ? base + 1 : filename;

    for (const struct lc_static_entry *e = lc_static_table; e->fname; e++) {
        if (strcmp(base, e->fname) == 0)
            return (void *)e;
    }
    return NULL;
}

void *lc_get_header(void *handle)
{
    if (!handle) return NULL;
    return (void *)((const struct lc_static_entry *)handle)->hdr;
}

void lc_close(void *handle)
{
    /* Codec lives in .text for the process lifetime — nothing to free. */
    (void)handle;
}

/* Open-from-buffer is meaningless for static-linked codecs; the upper
 * layer falls back to lc_open() if this returns NULL. */
void *lc_open_from_mem(void *addr, size_t blob_size)
{
    (void)addr;
    (void)blob_size;
    return NULL;
}
