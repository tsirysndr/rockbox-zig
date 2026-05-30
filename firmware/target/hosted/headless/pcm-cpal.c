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

#ifdef __ANDROID__
#include <sys/resource.h>     /* setpriority() */
#include <sys/syscall.h>      /* gettid() via SYS_gettid */
#include <unistd.h>
#endif

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
extern void pcm_cpal_set_volume(int vol_l, int vol_r);
extern void pcm_cpal_start(void);
extern void pcm_cpal_push(const void *data, size_t size);
extern void pcm_cpal_stop(void);
extern void pcm_cpal_flush(void);
extern bool pcm_cpal_is_running(void);

/* ── Writer-thread state ────────────────────────────────────────────────── */

static const void     *pcm_data   = NULL;
static size_t          pcm_size   = 0;
static pthread_mutex_t cpal_mtx;
static pthread_t       cpal_tid;
static volatile bool   cpal_running = false;
static volatile bool   cpal_stop    = false;
/* Set true while cpal_thread is inside pcm_play_dma_complete_callback so
 * sink_dma_stop knows whether it is being called because pcmbuf ran dry
 * (leave ring to drain) or because of an explicit user stop (flush ring). */
static volatile bool   cpal_draining = false;

static void *cpal_thread(void *arg)
{
    (void)arg;

#ifdef __ANDROID__
    /* Nice down to ANDROID_PRIORITY_AUDIO (-16).  Default priority (0) lets
     * any foreground app preempt this thread; if we don't refill the 512 KB
     * cpal ring within ~3 s the AAudio callback runs out of PCM and audible
     * gaps appear.  setpriority on a per-tid target requires SYS_gettid on
     * bionic — PRIO_PROCESS + tid is the documented Android pattern. */
    int tid = (int)syscall(SYS_gettid);
    if (setpriority(PRIO_PROCESS, tid, -16) != 0) {
        /* Non-fatal: kernel may deny if the audio_app group isn't joined.
         * Fall back to -8 (URGENT_DISPLAY), which is always permitted. */
        setpriority(PRIO_PROCESS, tid, -8);
    }
#endif

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

        /* Exit if an explicit stop was requested from outside the thread,
         * OR if the ring stopped draining (stream error / audio focus loss).
         * In the latter case pcm_cpal_push returned immediately without
         * writing to the ring; continuing would drain pcmbuf uselessly. */
        if (cpal_stop || !pcm_cpal_is_running()) break;

        /* Set drain flag before calling the completion callback.  If pcmbuf
         * is empty (HTTP stall) pcm_play_dma_complete_callback will call
         * pcm_play_stop_int → sink_dma_stop internally.  cpal_draining=true
         * tells sink_dma_stop NOT to flush the ring so the remaining decoded
         * audio can play out while the network reconnects. */
        cpal_draining = true;
        pthread_mutex_lock(&cpal_mtx);
        bool got_more = pcm_play_dma_complete_callback(PCM_DMAST_OK,
                                                        &pcm_data, &pcm_size);
        pthread_mutex_unlock(&cpal_mtx);
        cpal_draining = false;

        if (!got_more) {
            logf("pcm-cpal: no more PCM data, ring draining");
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

    pthread_mutex_lock(&cpal_mtx);
    cpal_stop    = false;
    cpal_running = true;
    pcm_data     = NULL;
    pcm_size     = 0;
    pthread_mutex_unlock(&cpal_mtx);

    /* Re-arm the ring so pcm_cpal_push() accepts new data. */
    pcm_cpal_start();

    /* Push the first chunk synchronously so the ring is pre-filled before
     * the writer thread starts.  Without this, thread-creation latency
     * (~1–5 ms on Linux) left the ring empty and produced an audible
     * silence gap at the beginning of every track. */
    pcm_cpal_push(addr, size);

    /* Ask firmware for the next chunk; the thread handles chunks 2+. */
    pthread_mutex_lock(&cpal_mtx);
    bool got_more = pcm_play_dma_complete_callback(PCM_DMAST_OK,
                                                    &pcm_data, &pcm_size);
    pthread_mutex_unlock(&cpal_mtx);

    if (!got_more) {
        logf("pcm-cpal: single-chunk track");
        cpal_running = false;
        return;
    }

    pcm_play_dma_status_callback(PCM_DMAST_STARTED);
    pthread_create(&cpal_tid, NULL, cpal_thread, NULL);
}

static void sink_dma_stop(void)
{
    logf("pcm-cpal: stop (draining=%d)", (int)cpal_draining);

    cpal_stop = true;
    /* Always mark running=false so pcm_cpal_push() unblocks and exits. */
    pcm_cpal_stop();

    /* cpal_draining is true when we reached here because pcmbuf ran dry
     * (HTTP stall): pcm_play_dma_complete_callback → pcm_play_stop_int →
     * sink_dma_stop.  In that case the ring still holds decoded audio that
     * should play through while the network reconnects — do NOT flush.
     * For an explicit user stop (cpal_draining=false) flush immediately. */
    if (!cpal_draining)
        pcm_cpal_flush();

    if (cpal_running) {
        pthread_join(cpal_tid, NULL);
        cpal_running = false;
    }

    pthread_mutex_lock(&cpal_mtx);
    pcm_data = NULL;
    pcm_size = 0;
    cpal_draining = false;
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
