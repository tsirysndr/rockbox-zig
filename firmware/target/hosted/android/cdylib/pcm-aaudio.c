/* SPDX-License-Identifier: GPL-2.0-or-later
 *
 * AAudio PCM sink for the Android cdylib build. Models pcm-fifo.c —
 * push-with-completion-callback contract: the engine calls
 * sink_dma_start(addr, size) once, then a writer thread drains the
 * chunk into an AAudio output stream and asks the engine for the next
 * via pcm_play_dma_complete_callback().
 *
 * Write mode (not callback mode) — see project memory note for rationale.
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

#define BYTES_PER_FRAME 4   /* S16_LE stereo */

static AAudioStream *aa_stream      = NULL;
/* HW_FREQ_DEFAULT is an INDEX into hw_freq_sampr (not Hz). Until the first
 * codec calls set_freq, aa_sample_rate=0 means "let AAudio pick the device
 * default rate" — better than opening at "index Hz" (4, 6, …) which AAudio
 * silently rejects and falls back to the device rate anyway. set_freq will
 * resolve the index to a real rate (44100, 48000, …) on the first call. */
static int32_t       aa_sample_rate = 0;

static const void *pcm_data = NULL;
static size_t      pcm_size = 0;

static void  *aa_vol_buf     = NULL;
static size_t aa_vol_buf_cap = 0;

static pthread_mutex_t aa_mtx;
static pthread_t       aa_tid;
static volatile bool   aa_running      = false;
static volatile bool   aa_stop         = false;
static volatile bool   aa_disconnected = false;

/* 512 stereo S16LE frames of silence — written to the stream between tracks
 * so AAudio never underruns during the inter-track gap. */
static const int16_t s_silence[512 * 2] = {0};

static void on_error(AAudioStream *s, void *ud, aaudio_result_t err)
{
    (void)s; (void)ud;
    LOGW("AAudio error %d (%s) — flagging for reopen",
         err, AAudio_convertResultToText(err));
    aa_disconnected = true;
}

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
    AAudioStreamBuilder_setPerformanceMode(b, AAUDIO_PERFORMANCE_MODE_LOW_LATENCY);
    AAudioStreamBuilder_setFormat         (b, AAUDIO_FORMAT_PCM_I16);
    AAudioStreamBuilder_setChannelCount   (b, 2);
    AAudioStreamBuilder_setSampleRate     (b, freq);
    /* setUsage / setContentType are API 28+ — metadata only, skip on API 26
     * to keep the minSdk floor low. AAudio defaults to MEDIA / MUSIC anyway. */
    AAudioStreamBuilder_setErrorCallback  (b, on_error, NULL);

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
            /* No data yet (startup) or between tracks — write silence so the
             * stream stays alive and doesn't underrun while the firmware is
             * switching tracks. */
            if (aa_stream)
                AAudioStream_write(aa_stream, s_silence,
                                   sizeof(s_silence) / BYTES_PER_FRAME,
                                   200LL * 1000000LL);
            else {
                struct timespec t = { 0, 10000000 }; /* 10 ms */
                nanosleep(&t, NULL);
            }
            continue;
        }

        /* SW volume scaling skipped on Android — AAudio + Java MediaSession
         * handle volume system-side. Just write the raw PCM through. */
        const void *data = raw;
        (void)aa_vol_buf;
        (void)aa_vol_buf_cap;

        const uint8_t *p = (const uint8_t *)data;
        size_t bytes_left = size;
        while (bytes_left > 0 && !aa_stop && !aa_disconnected) {
            int32_t frames_left = (int32_t)(bytes_left / BYTES_PER_FRAME);
            int64_t timeout_ns = 200LL * 1000000LL;
            aaudio_result_t written =
                AAudioStream_write(aa_stream, p, frames_left, timeout_ns);
            if (written < 0) {
                LOGE("AAudio write failed: %d (%s)",
                     written, AAudio_convertResultToText(written));
                aa_disconnected = true;
                break;
            }
            if (written == 0) continue;
            size_t bytes_written = (size_t)written * BYTES_PER_FRAME;
            p          += bytes_written;
            bytes_left -= bytes_written;
        }

        if (aa_stop) break;

        pthread_mutex_lock(&aa_mtx);
        bool got_more = pcm_play_dma_complete_callback(PCM_DMAST_OK,
                                                        &pcm_data, &pcm_size);
        pthread_mutex_unlock(&aa_mtx);

        if (!got_more) {
            /* End of track: don't stop — fall into the silence loop above so
             * the AAudio stream keeps running until sink_dma_start() provides
             * the next track's data.  This eliminates the pause/requestStart
             * gap that causes audible cuts between tracks. */
            logf("pcm-aaudio: no more PCM data, entering silence loop");
            continue;
        }
        pcm_play_dma_status_callback(PCM_DMAST_STARTED);
    }

    if (aa_stream)
        AAudioStream_requestPause(aa_stream);

    aa_running = false;
    return NULL;
}

static void sink_dma_init(void)
{
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
    /* `freq_index` is an INDEX into sink->caps.samprs (== hw_freq_sampr),
     * NOT a sample rate in Hz. firmware/pcm.c::pcm_set_frequency stores
     * the index into pending_freq, then pcm_apply_settings forwards that
     * raw index to set_freq. We must look up the actual Hz rate via
     * hw_freq_sampr[idx] before passing it to AAudio — otherwise we open
     * the stream at "4 Hz" (or whatever the index is), AAudio falls back
     * to the device-default rate (48000), and 44100-Hz audio plays at
     * 48000 Hz → ~9% faster → chipmunks. Same gotcha as pcm-sdl.c:107. */
    int32_t hz = (int32_t)hw_freq_sampr[freq_index];

    pthread_mutex_lock(&aa_mtx);
    if (hz == aa_sample_rate && aa_stream) {
        pthread_mutex_unlock(&aa_mtx);
        return;
    }
    LOGI("set_freq idx=%u -> %d Hz (was %d Hz)", freq_index, hz, aa_sample_rate);
    aa_sample_rate = hz;
    close_stream();
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
    /* If aa_running is already true the thread is in the silence loop and
     * will pick up the new pcm_data on its next iteration — no new thread. */
}

static void sink_dma_stop(void)
{
    logf("pcm-aaudio: stop");
    /* Keep the writer thread alive — it transitions to the silence loop and
     * keeps the AAudio stream running until sink_dma_start() is called for
     * the next track.  Only null out the pending data so the thread sees no
     * new chunk to write. */
    pthread_mutex_lock(&aa_mtx);
    pcm_data = NULL;
    pcm_size = 0;
    pthread_mutex_unlock(&aa_mtx);
}

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
