/* SPDX-License-Identifier: GPL-2.0-or-later
 *
 * PCM sink that routes audio directly to libasound via snd_pcm_writei
 * (RWInterleaved access — the same path as `aplay`). Used for the
 * arm-linux-gnueabihf (ARMHFHOST) target to bypass cpal's ALSA backend which
 * crashes on some ARM builds via snd_pcm_status_get_htstamp / mmap.
 *
 * The Rust implementation lives in crates/alsa-sink/src/lib.rs.
 * This C file mirrors pcm-cpal.c exactly — only the function prefix changes.
 */

#include "autoconf.h"
#include "config.h"

#include <pthread.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <string.h>

#include "pcm.h"
#include "pcm-internal.h"
#include "pcm_mixer.h"
#include "pcm_sampr.h"
#include "pcm_sink.h"

#define LOGF_ENABLE
#include "logf.h"

/* ── Rust FFI — defined in crates/alsa-sink/src/lib.rs ────────────────────── */
extern void pcm_alsa_init(void);
extern void pcm_alsa_postinit(void);
extern void pcm_alsa_set_sample_rate(uint32_t rate_hz);
extern void pcm_alsa_start(void);
extern void pcm_alsa_push(const void *data, size_t size);
extern void pcm_alsa_stop(void);
extern void pcm_alsa_flush(void);
extern bool pcm_alsa_is_running(void);

/* ── Writer-thread state ────────────────────────────────────────────────── */

static const void     *pcm_data   = NULL;
static size_t          pcm_size   = 0;
static pthread_mutex_t alsa_mtx;
static pthread_t       alsa_tid;
static volatile bool   alsa_running  = false;
static volatile bool   alsa_stop     = false;
static volatile bool   alsa_draining = false;

static void *alsa_thread(void *arg)
{
    (void)arg;

    while (!alsa_stop) {
        pthread_mutex_lock(&alsa_mtx);
        const void *data = pcm_data;
        size_t      size = pcm_size;
        pcm_data = NULL;
        pcm_size = 0;
        pthread_mutex_unlock(&alsa_mtx);

        if (!data || !size) {
            alsa_stop = true;
            break;
        }

        pcm_alsa_push(data, size);

        if (alsa_stop || !pcm_alsa_is_running())
            break;

        alsa_draining = true;
        pthread_mutex_lock(&alsa_mtx);
        bool got_more = pcm_play_dma_complete_callback(PCM_DMAST_OK,
                                                        &pcm_data, &pcm_size);
        pthread_mutex_unlock(&alsa_mtx);
        alsa_draining = false;

        if (!got_more) {
            logf("pcm-alsa: no more PCM data, ring draining");
            break;
        }

        pcm_play_dma_status_callback(PCM_DMAST_STARTED);
    }

    alsa_running = false;
    return NULL;
}

/* ── Sink ops ───────────────────────────────────────────────────────────── */

static void sink_dma_init(void)
{
    pthread_mutexattr_t attr;
    pthread_mutexattr_init(&attr);
    pthread_mutexattr_settype(&attr, PTHREAD_MUTEX_RECURSIVE);
    pthread_mutex_init(&alsa_mtx, &attr);
    pthread_mutexattr_destroy(&attr);
    pcm_alsa_init();
}

static void sink_dma_postinit(void)
{
    pcm_alsa_postinit();
}

static void sink_set_freq(uint16_t freq_index)
{
    pcm_alsa_set_sample_rate((uint32_t)hw_freq_sampr[freq_index]);
}

static void sink_lock(void)   { pthread_mutex_lock(&alsa_mtx); }
static void sink_unlock(void) { pthread_mutex_unlock(&alsa_mtx); }

static void sink_dma_start(const void *addr, size_t size)
{
    logf("pcm-alsa: start (%p, %zu)", addr, size);

    pthread_mutex_lock(&alsa_mtx);
    alsa_stop    = false;
    alsa_running = true;
    pcm_data     = NULL;
    pcm_size     = 0;
    pthread_mutex_unlock(&alsa_mtx);

    pcm_alsa_start();
    pcm_alsa_push(addr, size);

    pthread_mutex_lock(&alsa_mtx);
    bool got_more = pcm_play_dma_complete_callback(PCM_DMAST_OK,
                                                    &pcm_data, &pcm_size);
    pthread_mutex_unlock(&alsa_mtx);

    if (!got_more) {
        logf("pcm-alsa: single-chunk track");
        alsa_running = false;
        return;
    }

    pcm_play_dma_status_callback(PCM_DMAST_STARTED);
    pthread_create(&alsa_tid, NULL, alsa_thread, NULL);
}

static void sink_dma_stop(void)
{
    logf("pcm-alsa: stop (draining=%d)", (int)alsa_draining);

    alsa_stop = true;
    pcm_alsa_stop();

    if (!alsa_draining)
        pcm_alsa_flush();

    if (alsa_running) {
        pthread_join(alsa_tid, NULL);
        alsa_running = false;
    }

    pthread_mutex_lock(&alsa_mtx);
    pcm_data      = NULL;
    pcm_size      = 0;
    alsa_draining = false;
    pthread_mutex_unlock(&alsa_mtx);
}

/* ── Sink structs ───────────────────────────────────────────────────────── */

struct pcm_sink builtin_pcm_sink = {
    .caps = {
        .samprs       = hw_freq_sampr,
        .num_samprs   = HW_NUM_FREQ,
        .default_freq = HW_FREQ_DEFAULT,
    },
    .ops = {
        .init     = sink_dma_init,
        .postinit = sink_dma_postinit,
        .set_freq = sink_set_freq,
        .lock     = sink_lock,
        .unlock   = sink_unlock,
        .play     = sink_dma_start,
        .stop     = sink_dma_stop,
    },
};

struct pcm_sink alsa_pcm_sink = {
    .caps = {
        .samprs       = hw_freq_sampr,
        .num_samprs   = HW_NUM_FREQ,
        .default_freq = HW_FREQ_DEFAULT,
    },
    .ops = {
        .init     = sink_dma_init,
        .postinit = sink_dma_postinit,
        .set_freq = sink_set_freq,
        .lock     = sink_lock,
        .unlock   = sink_unlock,
        .play     = sink_dma_start,
        .stop     = sink_dma_stop,
    },
};
