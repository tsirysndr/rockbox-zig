/* SPDX-License-Identifier: GPL-2.0-or-later
 *
 * BINFMT_STATIC implementation of the codec/plugin loader for Android.
 * lc_open(name) → pointer into the compile-time codec table built when
 * codecs.make's CODECS_STATIC mode runs. No dlopen, no filesystem access.
 *
 * The lc_static_table below holds a stable pointer to each codec's
 * `__header_<name>` symbol. Because we take the address of every codec
 * header here, the linker keeps each codec's .text alive even with
 * --gc-sections enabled.
 */

#include <string.h>
#include <stddef.h>

#include "load_code.h"     /* pulls lc-static.h via CONFIG_BINFMT dispatch */
#include "lib/rbcodec/codecs/codecs.h"   /* full struct codec_header def */

/* ── extern decls for every codec's renamed __header_<name> symbol ────── */
/* These match the names emitted by codecs.make's CODECS_STATIC objcopy
 * --redefine-sym rule. If a codec is added to or removed from
 * lib/rbcodec/codecs/SOURCES, mirror the change here. */
extern const struct codec_header __header_a52;
extern const struct codec_header __header_a52_rm;
extern const struct codec_header __header_aac;
extern const struct codec_header __header_aac_bsf;
extern const struct codec_header __header_adx;
extern const struct codec_header __header_aiff;
extern const struct codec_header __header_alac;
extern const struct codec_header __header_ape;
extern const struct codec_header __header_asap;
extern const struct codec_header __header_atrac3_oma;
extern const struct codec_header __header_atrac3_rm;
extern const struct codec_header __header_au;
extern const struct codec_header __header_ay;
extern const struct codec_header __header_cook;
extern const struct codec_header __header_flac;
extern const struct codec_header __header_gbs;
extern const struct codec_header __header_hes;
extern const struct codec_header __header_kss;
extern const struct codec_header __header_mod;
extern const struct codec_header __header_mpa;
extern const struct codec_header __header_mpc;
extern const struct codec_header __header_nsf;
extern const struct codec_header __header_opus;
extern const struct codec_header __header_raac;
extern const struct codec_header __header_sgc;
extern const struct codec_header __header_shorten;
extern const struct codec_header __header_smaf;
extern const struct codec_header __header_spc;
extern const struct codec_header __header_speex;
extern const struct codec_header __header_tta;
extern const struct codec_header __header_vgm;
extern const struct codec_header __header_vorbis;
extern const struct codec_header __header_vox;
extern const struct codec_header __header_vtx;
extern const struct codec_header __header_wav;
extern const struct codec_header __header_wav64;
extern const struct codec_header __header_wavpack;
extern const struct codec_header __header_wma;
extern const struct codec_header __header_wmapro;

/* ── Codec table ──────────────────────────────────────────────────────── */
/* Names match the `<codec>.codec` filenames passed to lc_open by the
 * upper layer (apps/codec_thread.c via audio_formats[].codec_root_fn). */
const struct lc_static_entry lc_static_table[] = {
    { "a52.codec",          &__header_a52         },
    { "a52_rm.codec",       &__header_a52_rm      },
    { "aac.codec",          &__header_aac         },
    { "aac_bsf.codec",      &__header_aac_bsf     },
    { "adx.codec",          &__header_adx         },
    { "aiff.codec",         &__header_aiff        },
    { "alac.codec",         &__header_alac        },
    { "ape.codec",          &__header_ape         },
    { "asap.codec",         &__header_asap        },
    { "atrac3_oma.codec",   &__header_atrac3_oma  },
    { "atrac3_rm.codec",    &__header_atrac3_rm   },
    { "au.codec",           &__header_au          },
    { "ay.codec",           &__header_ay          },
    { "cook.codec",         &__header_cook        },
    { "flac.codec",         &__header_flac        },
    { "gbs.codec",          &__header_gbs         },
    { "hes.codec",          &__header_hes         },
    { "kss.codec",          &__header_kss         },
    { "mod.codec",          &__header_mod         },
    { "mpa.codec",          &__header_mpa         },
    { "mpc.codec",          &__header_mpc         },
    { "nsf.codec",          &__header_nsf         },
    { "opus.codec",         &__header_opus        },
    { "raac.codec",         &__header_raac        },
    { "sgc.codec",          &__header_sgc         },
    { "shorten.codec",      &__header_shorten     },
    { "smaf.codec",         &__header_smaf        },
    { "spc.codec",          &__header_spc         },
    { "speex.codec",        &__header_speex       },
    { "tta.codec",          &__header_tta         },
    { "vgm.codec",          &__header_vgm         },
    { "vorbis.codec",       &__header_vorbis      },
    { "vox.codec",          &__header_vox         },
    { "vtx.codec",          &__header_vtx         },
    { "wav.codec",          &__header_wav         },
    { "wav64.codec",        &__header_wav64       },
    { "wavpack.codec",      &__header_wavpack     },
    { "wma.codec",          &__header_wma         },
    { "wmapro.codec",       &__header_wmapro      },
    { (const char *)0,      (const struct codec_header *)0 }
};

/* ── lc_* implementations ─────────────────────────────────────────────── */

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
