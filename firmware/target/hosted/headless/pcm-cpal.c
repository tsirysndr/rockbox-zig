/* SPDX-License-Identifier: GPL-2.0-or-later
 *
 * PCM sink that routes audio through the Rust cpal crate (cross-platform
 * audio I/O). Models pcm-fifo.c exactly — push-with-completion-callback
 * contract — but instead of writing to a file descriptor, the writer thread
 * calls pcm_cpal_push() which enqueues data into a ring buffer consumed by
 * cpal's audio callback.
 *
 * This sink is wired as PCM_SINK_BUILTIN on the headless host build so that
 * rockboxd produces audio out of the box without any settings.toml entry.
 *
 * The Rust implementation lives in crates/cpal-sink/src/lib.rs.
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

/* ── Rust FFI — defined in crates/cpal-sink/src/lib.rs ────────────────────── */
extern void pcm_cpal_init(void);
extern void pcm_cpal_postinit(void);
extern void pcm_cpal_set_sample_rate(uint32_t rate_hz);
extern void pcm_cpal_start(void);
extern void pcm_cpal_push(const void *data, size_t size);
extern void pcm_cpal_stop(void);

/* ── Writer-thread state ────────────────────────────────────────────────── */

static const void     *pcm_data   = NULL;
static size_t          pcm_size   = 0;
static pthread_mutex_t cpal_mtx;
static pthread_t       cpal_tid;
static volatile bool   cpal_running = false;
static volatile bool   cpal_stop    = false;

static void *cpal_thread(void *arg)
{
    (void)arg;

    while (!cpal_stop) {
        pthread_mutex_lock(&cpal_mtx);
        const void *data = pcm_data;
        size_t      size = pcm_size;
        pcm_data = NULL;
        pcm_size = 0;
        pthread_mutex_unlock(&cpal_mtx);

        if (!data || !size) {
            cpal_stop = true;
            break;
        }

        /* Push current chunk into cpal ring (blocks on back-pressure). */
        pcm_cpal_push(data, size);

        if (cpal_stop) break;

        /* Ask firmware for next chunk; updates pcm_data / pcm_size. */
        pthread_mutex_lock(&cpal_mtx);
        bool got_more = pcm_play_dma_complete_callback(PCM_DMAST_OK,
                                                        &pcm_data, &pcm_size);
        pthread_mutex_unlock(&cpal_mtx);

        if (!got_more) {
            logf("pcm-cpal: no more PCM data");
            break;
        }

        pcm_play_dma_status_callback(PCM_DMAST_STARTED);
    }

    cpal_running = false;
    return NULL;
}

/* ── Sink ops ───────────────────────────────────────────────────────────── */

static void sink_dma_init(void)
{
    pthread_mutexattr_t attr;
    pthread_mutexattr_init(&attr);
    pthread_mutexattr_settype(&attr, PTHREAD_MUTEX_RECURSIVE);
    pthread_mutex_init(&cpal_mtx, &attr);
    pthread_mutexattr_destroy(&attr);
    pcm_cpal_init();
}

static void sink_dma_postinit(void)
{
    pcm_cpal_postinit();
}

static void sink_set_freq(uint16_t freq_index)
{
    pcm_cpal_set_sample_rate((uint32_t)hw_freq_sampr[freq_index]);
}

static void sink_lock(void)
{
    pthread_mutex_lock(&cpal_mtx);
}

static void sink_unlock(void)
{
    pthread_mutex_unlock(&cpal_mtx);
}

static void sink_dma_start(const void *addr, size_t size)
{
    logf("pcm-cpal: start (%p, %zu)", addr, size);

    /* Re-arm the ring so pcm_cpal_push() doesn't silently discard data after
     * a previous pcm_cpal_stop() set running=false. */
    pcm_cpal_start();

    pthread_mutex_lock(&cpal_mtx);
    pcm_data    = addr;
    pcm_size    = size;
    cpal_stop   = false;
    cpal_running = true;
    pthread_mutex_unlock(&cpal_mtx);

    pthread_create(&cpal_tid, NULL, cpal_thread, NULL);
}

static void sink_dma_stop(void)
{
    logf("pcm-cpal: stop");

    cpal_stop = true;
    pcm_cpal_stop();

    if (cpal_running) {
        pthread_join(cpal_tid, NULL);
        cpal_running = false;
    }

    pthread_mutex_lock(&cpal_mtx);
    pcm_data = NULL;
    pcm_size = 0;
    pthread_mutex_unlock(&cpal_mtx);
}

/* ── Sink struct ────────────────────────────────────────────────────────── */

/* On the headless host, cpal is the built-in sink. pcm.c references
 * builtin_pcm_sink via extern; we satisfy that here. */
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

/* Also expose under the named PCM_SINK_CPAL slot so settings.toml can
 * switch to it explicitly with audio_output = "cpal". */
struct pcm_sink cpal_pcm_sink = {
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
