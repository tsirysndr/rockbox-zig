/* SPDX-License-Identifier: GPL-2.0-or-later
 *
 * Web Audio API PCM sink for the Rockbox WASM build.
 *
 * PCM data is written directly into a lock-free ring buffer that lives in
 * WASM linear memory.  Because the WASM module is compiled with -pthread,
 * Module.HEAP8.buffer is a SharedArrayBuffer.  The AudioWorklet therefore
 * needs no separate SAB — it creates Int16Array / Int32Array views directly
 * into WASM memory using the byte offsets returned by the rb_pcm_* exports.
 *
 * Exported accessor functions (called from rockbox.js after module init):
 *   rb_pcm_ring_ptr()        — byte offset of the int16 stereo sample array
 *   rb_pcm_ring_frames()     — ring capacity in frames
 *   rb_pcm_write_idx_ptr()   — byte offset of the int32 atomic write index
 *   rb_pcm_read_idx_ptr()    — byte offset of the int32 atomic read index
 *   rb_pcm_sample_rate_ptr() — byte offset of the int32 current sample rate
 */

#include "autoconf.h"
#include "config.h"

#include <emscripten.h>
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

#define BYTES_PER_FRAME  4           /* S16LE stereo: 2 ch × 2 bytes */
#define RING_FRAMES      65536       /* ~1.5 s at 44100 Hz */

/* ── Ring buffer in WASM linear memory (= SharedArrayBuffer) ───────────── */

static int16_t  s_ring[RING_FRAMES * 2]; /* interleaved stereo S16LE        */
static int32_t  s_write_idx = 0;         /* written by wa_thread             */
static int32_t  s_read_idx  = 0;         /* written by AudioWorklet (JS)     */
static int32_t  s_sample_rate_hz = 44100;

/* Balance: -100 (full left) … 0 (centre) … +100 (full right).
 * Applied in ring_push() so changes take effect on the next ring write,
 * bypassing the ~3 s pcmbuf pre-decode delay. */
static int32_t  s_balance = 0;

/* Set to 1 by rb_pcm_flush() to abort any in-progress ring_push() call.
 * Without this, wa_thread may have already dequeued a large chunk of old
 * audio into its local `raw` pointer and will write it back into the ring
 * after the flush resets s_write_idx, undoing the flush entirely. */
static int32_t  s_flush_pending = 0;

void rb_pcm_set_balance(int balance)
{
    if (balance < -100) balance = -100;
    if (balance >  100) balance =  100;
    __atomic_store_n(&s_balance, balance, __ATOMIC_RELEASE);
}

/* These are called from rockbox.js after the module loads.  They return WASM
 * linear-memory byte offsets so the AudioWorklet can build typed-array views
 * into Module.HEAP8.buffer (a SharedArrayBuffer in the pthread build). */
int32_t *rb_pcm_ring_ptr(void)        { return (int32_t *)s_ring;   }
int32_t  rb_pcm_ring_frames(void)     { return RING_FRAMES;         }
int32_t *rb_pcm_write_idx_ptr(void)   { return &s_write_idx;        }
int32_t *rb_pcm_read_idx_ptr(void)    { return &s_read_idx;         }
int32_t *rb_pcm_sample_rate_ptr(void) { return &s_sample_rate_hz;   }

/* Push n_frames stereo S16LE frames into the ring.
 * Blocks with 1ms nanosleep when the ring is full (AudioWorklet will catch up
 * before the sleep expires at normal playback rates).
 * Balance (-100…+100) is applied here so changes take effect immediately
 * without waiting for the Rockbox pcmbuf to drain (~3 s). */
static void ring_push(const int16_t *src, int n_frames)
{
    /* Read balance once before the loop to avoid per-sample atomic loads. */
    int32_t bal    = __atomic_load_n(&s_balance, __ATOMIC_ACQUIRE);
    int32_t l_gain = (bal > 0) ? (100 - bal) : 100; /* 0..100 */
    int32_t r_gain = (bal < 0) ? (100 + bal) : 100; /* 0..100 */

    for (int i = 0; i < n_frames; ) {
        /* Abort if rb_pcm_flush() was called while we hold old audio data.
         * The caller (wa_thread) will discard the remaining frames and enter
         * the silence/DMA-complete path so the decoder can refill from the
         * new seek position. */
        if (__atomic_load_n(&s_flush_pending, __ATOMIC_ACQUIRE))
            return;
        int32_t wi     = __atomic_load_n(&s_write_idx, __ATOMIC_SEQ_CST);
        int32_t ri     = __atomic_load_n(&s_read_idx,  __ATOMIC_SEQ_CST);
        int32_t nxt_wi = (wi + 1) % RING_FRAMES;
        if (nxt_wi == ri) {
            /* Ring full — yield for 1 ms and retry. */
            struct timespec t = {0, 1000000};
            nanosleep(&t, NULL);
            continue;
        }
        /* Integer balance: multiply by gain (0..100) and divide by 100.
         * When l_gain == r_gain == 100 the result is identical to src. */
        s_ring[wi * 2]     = (int16_t)((int32_t)src[i * 2]     * l_gain / 100);
        s_ring[wi * 2 + 1] = (int16_t)((int32_t)src[i * 2 + 1] * r_gain / 100);
        __atomic_store_n(&s_write_idx, nxt_wi, __ATOMIC_SEQ_CST);
        i++;
    }
}

/* ── State ────────────────────────────────────────────────────────────── */

static const void   *pcm_data = NULL;
static size_t        pcm_size = 0;

static pthread_mutex_t wa_mtx;
static pthread_t       wa_tid;
static volatile bool   wa_running = false;
static volatile bool   wa_stop    = false;

static const int16_t s_silence[512 * 2] = {0};

/* ── Writer thread ────────────────────────────────────────────────────── */

static void *wa_thread(void *arg)
{
    (void)arg;

    while (!wa_stop) {
        pthread_mutex_lock(&wa_mtx);
        const void *raw  = pcm_data;
        size_t      size = pcm_size;
        pcm_data = NULL;
        pcm_size = 0;
        pthread_mutex_unlock(&wa_mtx);

        if (size == 0) {
            ring_push(s_silence, 512);
            /* Drive the mixer's idle counter so pcm_play_stop_int() eventually
             * fires, allowing mixer_start_pcm() to re-arm the sink on resume.
             * Both callbacks MUST be called under wa_mtx so that
             * mixer_buffer_callback() runs with pcm_play_lock() held.
             * Without the lock, mixer_buffer_callback iterates active_channels
             * while Rockbox audio/mixer threads can concurrently add/remove
             * entries (via mixer_channel_play_data/stop/pause which all call
             * pcm_play_lock = sink_lock = wa_mtx), causing a race that
             * produces invalid pointers and triggers a WASM OOB trap. */
            pthread_mutex_lock(&wa_mtx);
            bool got = pcm_play_dma_complete_callback(PCM_DMAST_OK,
                                                       &pcm_data, &pcm_size);
            if (got)
                pcm_play_dma_status_callback(PCM_DMAST_STARTED);
            pthread_mutex_unlock(&wa_mtx);
            continue;
        }

        ring_push((const int16_t *)raw, (int)(size / BYTES_PER_FRAME));

        if (wa_stop) break;

        pthread_mutex_lock(&wa_mtx);
        bool got_more = pcm_play_dma_complete_callback(PCM_DMAST_OK,
                                                        &pcm_data, &pcm_size);
        if (got_more)
            pcm_play_dma_status_callback(PCM_DMAST_STARTED);
        pthread_mutex_unlock(&wa_mtx);

        if (!got_more) {
            logf("pcm-webapi: no more data, entering silence loop");
            continue;
        }
    }

    wa_running = false;
    return NULL;
}

/* Discard all buffered audio so the AudioWorklet immediately picks up any DSP
 * setting change (EQ, replaygain, …) or seek rather than waiting up to ~1.5 s
 * for the ring to drain.
 *
 * The flush has two parts:
 *   1. Set s_flush_pending so that any in-progress ring_push() call in
 *      wa_thread exits early and discards the remaining old-audio frames.
 *      Without this, wa_thread would re-fill the ring with those frames
 *      immediately after we reset the indices, undoing the flush.
 *   2. Atomically reset s_write_idx = s_read_idx so the AudioWorklet sees
 *      an empty ring on its next process() call.
 * After clearing the flag, wa_thread's next ring_push (silence or new audio)
 * proceeds normally. */
void rb_pcm_flush(void)
{
    __atomic_store_n(&s_flush_pending, 1, __ATOMIC_RELEASE);
    int32_t ri = __atomic_load_n(&s_read_idx, __ATOMIC_SEQ_CST);
    __atomic_store_n(&s_write_idx, ri, __ATOMIC_SEQ_CST);
    __atomic_store_n(&s_flush_pending, 0, __ATOMIC_RELEASE);
}

/* ── pcm_sink ops ─────────────────────────────────────────────────────── */

static void sink_dma_init(void)
{
    pthread_mutexattr_t attr;
    pthread_mutexattr_init(&attr);
    pthread_mutexattr_settype(&attr, PTHREAD_MUTEX_RECURSIVE);
    pthread_mutex_init(&wa_mtx, &attr);
    pthread_mutexattr_destroy(&attr);
}

static void sink_dma_postinit(void) { /* AudioContext managed by JS */ }

static void sink_set_freq(uint16_t freq_index)
{
    int32_t hz = (int32_t)hw_freq_sampr[freq_index];
    __atomic_store_n(&s_sample_rate_hz, hz, __ATOMIC_RELEASE);
}

static void sink_lock  (void) { pthread_mutex_lock  (&wa_mtx); }
static void sink_unlock(void) { pthread_mutex_unlock(&wa_mtx); }

static void sink_dma_start(const void *addr, size_t size)
{
    logf("pcm-webapi: start (%p, %zu)", addr, size);

    pthread_mutex_lock(&wa_mtx);
    pcm_data = addr;
    pcm_size = size;
    pthread_mutex_unlock(&wa_mtx);

    if (!wa_running) {
        wa_stop    = false;
        wa_running = true;
        pthread_create(&wa_tid, NULL, wa_thread, NULL);
    }
}

static void sink_dma_stop(void)
{
    logf("pcm-webapi: stop");
    pthread_mutex_lock(&wa_mtx);
    pcm_data = NULL;
    pcm_size = 0;
    pthread_mutex_unlock(&wa_mtx);
}

struct pcm_sink webapi_pcm_sink = {
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
