/* SPDX-License-Identifier: GPL-2.0-or-later
 *
 * AAudio PCM sink for the Android cdylib build.
 *
 * Architecture (mirrors pcm-cpal.c / cpal-sink.rs exactly):
 *
 *   aa_thread  →  ring_push(data)  →  512 KB ring buffer  →  data_callback  →  speaker
 *
 * The ring buffer decouples Rockbox's chunk-based output from AAudio's
 * hardware callback timing.  When the ring is momentarily empty (e.g. during
 * an HTTP prefetch stall), the callback writes silence in real-time — the
 * firmware never injects silence itself.  This is identical to how
 * cpal-sink's VecDeque works on Linux/macOS, giving Android the same smooth
 * HTTP streaming behaviour.
 *
 * Uses AAudio data-callback mode (not blocking-write mode).
 * Requires AAudio (NDK API 26+).
 */

#include "autoconf.h"
#include "config.h"

#include <aaudio/AAudio.h>
#include <android/log.h>
#include <pthread.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#include "pcm.h"
#include "pcm-internal.h"
#include "pcm_mixer.h"
#include "pcm_sampr.h"
#include "pcm_sink.h"

#define LOGF_ENABLE
#include "logf.h"

#define TAG "rb-pcm-aaudio"
#define LOGI(fmt, ...) __android_log_print(ANDROID_LOG_INFO,  TAG, fmt, ##__VA_ARGS__)
#define LOGW(fmt, ...) __android_log_print(ANDROID_LOG_WARN,  TAG, fmt, ##__VA_ARGS__)
#define LOGE(fmt, ...) __android_log_print(ANDROID_LOG_ERROR, TAG, fmt, ##__VA_ARGS__)

#define BYTES_PER_FRAME 4   /* S16_LE stereo: 2 ch × 2 bytes */

/* ── Stop / disconnect flags (declared early so ring_push can read them) ──── */

static volatile bool   aa_stop         = false;
static volatile bool   aa_disconnected = false;

/* ── Ring buffer ─────────────────────────────────────────────────────────────
 *
 * 512 KB ≈ 2.9 s at 44100 Hz / 128 kbps.  Same capacity as the VecDeque in
 * crates/cpal-sink/src/lib.rs — chosen to absorb typical HTTP jitter without
 * perceptible startup latency.
 *
 * Producer: aa_thread (blocks on back-pressure via ring_space_cv).
 * Consumer: data_callback on AAudio's high-priority thread (non-blocking;
 *           fills silence for any shortfall).
 * ─────────────────────────────────────────────────────────────────────────── */

#define RING_SIZE (512 * 1024)

static uint8_t         ring_buf[RING_SIZE];
static size_t          ring_rd   = 0;   /* consumer read index  */
static size_t          ring_wr   = 0;   /* producer write index */
static size_t          ring_fill = 0;   /* bytes currently buffered */
static pthread_mutex_t ring_mtx;
static pthread_cond_t  ring_space_cv;   /* signalled when fill < RING_SIZE */

static void ring_reset(void)
{
    pthread_mutex_lock(&ring_mtx);
    ring_rd = ring_wr = ring_fill = 0;
    pthread_cond_broadcast(&ring_space_cv);
    pthread_mutex_unlock(&ring_mtx);
}

/* Push n bytes into the ring.  Blocks when full (back-pressure to aa_thread).
 * Returns early if aa_stop or aa_disconnected is set. */
static void ring_push(const void *src, size_t n)
{
    const uint8_t *p = (const uint8_t *)src;
    while (n > 0) {
        pthread_mutex_lock(&ring_mtx);
        while (ring_fill >= RING_SIZE) {
            if (aa_stop || aa_disconnected) {
                pthread_mutex_unlock(&ring_mtx);
                return;
            }
            pthread_cond_wait(&ring_space_cv, &ring_mtx);
        }
        if (aa_stop || aa_disconnected) {
            pthread_mutex_unlock(&ring_mtx);
            return;
        }

        size_t space = RING_SIZE - ring_fill;
        size_t chunk = n < space ? n : space;

        size_t tail = RING_SIZE - ring_wr;
        if (chunk <= tail) {
            memcpy(ring_buf + ring_wr, p, chunk);
        } else {
            memcpy(ring_buf + ring_wr, p,      tail);
            memcpy(ring_buf,           p + tail, chunk - tail);
        }
        ring_wr    = (ring_wr + chunk) % RING_SIZE;
        ring_fill += chunk;
        p += chunk;
        n -= chunk;
        pthread_mutex_unlock(&ring_mtx);
    }
}

/* Drain n bytes from the ring into dst.  Never blocks — fills any shortfall
 * with silence.  Called from the AAudio data callback. */
static void ring_pop(void *dst, size_t n)
{
    uint8_t *d = (uint8_t *)dst;
    pthread_mutex_lock(&ring_mtx);

    size_t avail = ring_fill < n ? ring_fill : n;
    if (avail > 0) {
        size_t tail = RING_SIZE - ring_rd;
        if (avail <= tail) {
            memcpy(d, ring_buf + ring_rd, avail);
        } else {
            memcpy(d,        ring_buf + ring_rd, tail);
            memcpy(d + tail, ring_buf,            avail - tail);
        }
        ring_rd    = (ring_rd + avail) % RING_SIZE;
        ring_fill -= avail;
        pthread_cond_signal(&ring_space_cv);
    }

    pthread_mutex_unlock(&ring_mtx);

    /* Underrun: fill remaining output with silence so the stream never
     * produces garbage.  This is the ONLY place silence enters the signal
     * chain — the firmware itself never injects silence. */
    if (avail < n)
        memset(d + avail, 0, n - avail);
}

/* ── AAudio stream ───────────────────────────────────────────────────────── */

static AAudioStream   *aa_stream      = NULL;
/* See set_freq comment: 0 means "let AAudio pick the device default rate". */
static int32_t         aa_sample_rate = 0;

static const void     *pcm_data = NULL;
static size_t          pcm_size = 0;

static pthread_mutex_t aa_mtx;
static pthread_t       aa_tid;
static volatile bool   aa_running      = false;

static void on_error(AAudioStream *s, void *ud, aaudio_result_t err)
{
    (void)s; (void)ud;
    LOGW("AAudio error %d (%s) — flagging for reopen",
         err, AAudio_convertResultToText(err));
    aa_disconnected = true;
    /* Unblock ring_push if it is waiting for space. */
    pthread_mutex_lock(&ring_mtx);
    pthread_cond_broadcast(&ring_space_cv);
    pthread_mutex_unlock(&ring_mtx);
}

/* ── Data callback (AAudio high-priority thread) ─────────────────────────── */

static aaudio_data_callback_result_t data_callback(
    AAudioStream *stream, void *userData,
    void *audioData, int32_t numFrames)
{
    (void)stream; (void)userData;
    ring_pop(audioData, (size_t)numFrames * BYTES_PER_FRAME);
    return AAUDIO_CALLBACK_RESULT_CONTINUE;
}

/* ── Stream open / close ─────────────────────────────────────────────────── */

static aaudio_result_t open_stream(int32_t freq)
{
    AAudioStreamBuilder *b;
    aaudio_result_t rc = AAudio_createStreamBuilder(&b);
    if (rc != AAUDIO_OK) {
        LOGE("createStreamBuilder failed: %d", rc);
        return rc;
    }
    AAudioStreamBuilder_setDirection      (b, AAUDIO_DIRECTION_OUTPUT);
    AAudioStreamBuilder_setSharingMode    (b, AAUDIO_SHARING_MODE_SHARED);
    /* NONE (default) instead of LOW_LATENCY: music streaming needs buffer
     * headroom for network jitter, not minimum latency. */
    AAudioStreamBuilder_setPerformanceMode(b, AAUDIO_PERFORMANCE_MODE_NONE);
    /* Request half a second of hardware buffer capacity.  The driver may
     * grant less; this is a hint. */
    {
        int32_t cap = freq > 0 ? (int32_t)(freq / 2) : 24000;
        AAudioStreamBuilder_setBufferCapacityInFrames(b, cap);
    }
    AAudioStreamBuilder_setFormat       (b, AAUDIO_FORMAT_PCM_I16);
    AAudioStreamBuilder_setChannelCount (b, 2);
    AAudioStreamBuilder_setSampleRate   (b, freq);
    AAudioStreamBuilder_setDataCallback (b, data_callback, NULL);
    AAudioStreamBuilder_setErrorCallback(b, on_error, NULL);

    rc = AAudioStreamBuilder_openStream(b, &aa_stream);
    AAudioStreamBuilder_delete(b);
    if (rc != AAUDIO_OK) {
        LOGE("openStream failed: %d (%s)", rc, AAudio_convertResultToText(rc));
        aa_stream = NULL;
        return rc;
    }
    int32_t actual = AAudioStream_getSampleRate(aa_stream);
    if (actual != freq) LOGW("AAudio gave %d Hz, requested %d", actual, freq);
    LOGI("AAudio open: %d Hz, %d-frame buffer",
         actual, AAudioStream_getBufferCapacityInFrames(aa_stream));
    return AAUDIO_OK;
}

static void close_stream(void)
{
    if (!aa_stream) return;
    AAudioStream_requestStop(aa_stream);
    AAudioStream_close(aa_stream);
    aa_stream = NULL;
}

/* ── Writer thread ───────────────────────────────────────────────────────── */

static void *aa_thread(void *arg)
{
    (void)arg;

    if (!aa_stream && open_stream(aa_sample_rate) != AAUDIO_OK) {
        aa_running = false;
        return NULL;
    }

    aaudio_result_t rc = AAudioStream_requestStart(aa_stream);
    if (rc != AAUDIO_OK) {
        LOGE("requestStart failed: %d", rc);
        aa_running = false;
        return NULL;
    }

    while (!aa_stop) {

        if (aa_disconnected) {
            LOGI("recovering from disconnect");
            close_stream();
            aa_disconnected = false;
            ring_reset();
            if (open_stream(aa_sample_rate) != AAUDIO_OK ||
                AAudioStream_requestStart(aa_stream) != AAUDIO_OK) {
                LOGE("recovery failed — exiting writer");
                break;
            }
        }

        pthread_mutex_lock(&aa_mtx);
        const void *raw  = pcm_data;
        size_t      size = pcm_size;
        pcm_data = NULL;
        pcm_size = 0;
        pthread_mutex_unlock(&aa_mtx);

        if (size == 0) {
            /* No data yet or between tracks.  The ring buffer + data callback
             * keep the stream alive with silence automatically — no explicit
             * silence injection needed here. */
            struct timespec t = { 0, 1000000 }; /* 1 ms */
            nanosleep(&t, NULL);
            continue;
        }

        /* Push chunk into ring (blocks on back-pressure, never on hardware).
         * The data callback drains the ring at hardware rate in parallel. */
        ring_push(raw, size);
        if (aa_stop) break;

        /* Ask the firmware for the next PCM chunk. */
        pthread_mutex_lock(&aa_mtx);
        bool got_more = pcm_play_dma_complete_callback(PCM_DMAST_OK,
                                                        &pcm_data, &pcm_size);
        pthread_mutex_unlock(&aa_mtx);

        if (!got_more) {
            logf("pcm-aaudio: no more PCM data, entering idle loop");
            continue;
        }
        pcm_play_dma_status_callback(PCM_DMAST_STARTED);
    }

    if (aa_stream)
        AAudioStream_requestPause(aa_stream);

    aa_running = false;
    return NULL;
}

/* ── Sink ops ────────────────────────────────────────────────────────────── */

static void sink_dma_init(void)
{
    pthread_mutex_init(&ring_mtx, NULL);
    pthread_cond_init (&ring_space_cv, NULL);

    pthread_mutexattr_t attr;
    pthread_mutexattr_init(&attr);
    pthread_mutexattr_settype(&attr, PTHREAD_MUTEX_RECURSIVE);
    pthread_mutex_init(&aa_mtx, &attr);
    pthread_mutexattr_destroy(&attr);
    LOGI("init");
}

static void sink_dma_postinit(void)
{
    pthread_mutex_lock(&aa_mtx);
    if (!aa_stream) open_stream(aa_sample_rate);
    pthread_mutex_unlock(&aa_mtx);
}

static void sink_set_freq(uint16_t freq_index)
{
    /* `freq_index` is an INDEX into hw_freq_sampr, not Hz — look up the rate. */
    int32_t hz = (int32_t)hw_freq_sampr[freq_index];

    pthread_mutex_lock(&aa_mtx);
    if (hz == aa_sample_rate && aa_stream) {
        pthread_mutex_unlock(&aa_mtx);
        return;
    }
    LOGI("set_freq idx=%u -> %d Hz (was %d Hz)", freq_index, hz, aa_sample_rate);
    aa_sample_rate = hz;
    close_stream();
    ring_reset();   /* flush audio encoded at the old sample rate */
    open_stream(aa_sample_rate);
    pthread_mutex_unlock(&aa_mtx);
}

static void sink_lock  (void) { pthread_mutex_lock  (&aa_mtx); }
static void sink_unlock(void) { pthread_mutex_unlock(&aa_mtx); }

static void sink_dma_start(const void *addr, size_t size)
{
    logf("pcm-aaudio: start (%p, %zu)", addr, size);

    pthread_mutex_lock(&aa_mtx);
    pcm_data = addr;
    pcm_size = size;
    pthread_mutex_unlock(&aa_mtx);

    if (!aa_running) {
        /* First start or after a true shutdown — spawn the writer thread. */
        aa_stop    = false;
        aa_running = true;
        pthread_create(&aa_tid, NULL, aa_thread, NULL);
    }
    /* If already running the thread is idle-looping and will pick up
     * pcm_data on its next 1 ms tick. */
}

static void sink_dma_stop(void)
{
    logf("pcm-aaudio: stop");
    /* Keep the writer thread and AAudio stream alive so the next track
     * starts without a requestStart gap.  Just clear the pending chunk;
     * the thread idles at 1 ms ticks until sink_dma_start() is called. */
    pthread_mutex_lock(&aa_mtx);
    pcm_data = NULL;
    pcm_size = 0;
    pthread_mutex_unlock(&aa_mtx);
}

/* ── Sink descriptor ─────────────────────────────────────────────────────── */

struct pcm_sink aaudio_pcm_sink = {
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
