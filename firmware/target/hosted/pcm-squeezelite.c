/***************************************************************************
 * PCM sink that streams raw S16LE stereo PCM to squeezelite via the Slim
 * Protocol (port 3483) + a plain HTTP audio endpoint (port 9999).
 *
 * Usage:
 *   pcm_squeezelite_set_slim_port(3483);   // optional, this is the default
 *   pcm_squeezelite_set_http_port(9999);   // optional, this is the default
 *   pcm_switch_sink(PCM_SINK_SQUEEZELITE);
 *   // Then run: squeezelite -s localhost
 *
 * Copyright (C) 2026 Rockbox contributors
 *
 * This program is free software; you can redistribute it and/or
 * modify it under the terms of the GNU General Public License
 * as published by the Free Software Foundation; either version 2
 * of the License, or (at your option) any later version.
 *
 * This software is distributed on an "AS IS" basis, WITHOUT WARRANTY OF ANY
 * KIND, either express or implied.
 *
 ****************************************************************************/

#include "autoconf.h"
#include "config.h"

#include <pthread.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include <time.h>
#include <unistd.h>

#include "pcm.h"
#include "pcm-internal.h"
#include "pcm_mixer.h"
#include "pcm_normalizer.h"
#include "pcm_sampr.h"
#include "pcm_sink.h"

#define LOGF_ENABLE
#include "logf.h"

/* Rust C API — symbols provided by the rockbox-slim crate via librockbox_cli.a */
extern void pcm_squeezelite_set_slim_port(uint16_t port);
extern void pcm_squeezelite_set_http_port(uint16_t port);
extern int  pcm_squeezelite_start(void);
extern int  pcm_squeezelite_write(const uint8_t *data, size_t len);
extern void pcm_squeezelite_stop(void);
extern void pcm_squeezelite_close(void);

static const void *pcm_data = NULL;
static size_t      pcm_size = 0;

/* Scratch buffer for SW volume scaling */
static void  *squeezelite_vol_buf     = NULL;
static size_t squeezelite_vol_buf_cap = 0;

static pthread_mutex_t squeezelite_mtx;
static pthread_t       squeezelite_tid;
static volatile bool   squeezelite_running = false;
static volatile bool   squeezelite_stop    = false;

/* Real-time pacing state — reset on every sink_dma_start(). */
static struct timespec play_start;
static uint64_t        play_bytes;

/* Actual sample rate set by sink_set_freq(); defaults to 44100.
 * bytes_per_sec = sample_rate * 2 channels * 2 bytes/sample. */
static unsigned long current_sample_rate = 44100;

static void *squeezelite_thread(void *arg)
{
    (void)arg;

    while (!squeezelite_stop) {
        pthread_mutex_lock(&squeezelite_mtx);
        const void *raw  = pcm_data;
        size_t      size = pcm_size;
        pcm_data = NULL;
        pcm_size = 0;
        pthread_mutex_unlock(&squeezelite_mtx);

        /* Apply SW volume scaling */
        if (size > squeezelite_vol_buf_cap) {
            free(squeezelite_vol_buf);
            squeezelite_vol_buf     = malloc(size);
            squeezelite_vol_buf_cap = squeezelite_vol_buf ? size : 0;
        }
        const void *data = (squeezelite_vol_buf && size > 0)
            ? (pcm_copy_buffer(squeezelite_vol_buf, raw, size), squeezelite_vol_buf)
            : raw;
        if (data == squeezelite_vol_buf)
            pcm_normalizer_apply(squeezelite_vol_buf, size);

        if (data && size > 0) {
            if (pcm_squeezelite_write((const uint8_t *)data, size) < 0) {
                logf("pcm-squeezelite: write error");
                squeezelite_stop = true;
                break;
            }

            /* Pace to real-time so the DMA loop does not drain the entire
             * track instantly.  We track total bytes written and sleep
             * whenever we are ahead of the expected wall-clock position.
             *
             * bytes_per_sec = sample_rate * 2 channels * 2 bytes/sample */
            play_bytes += size;
            uint64_t bps        = (uint64_t)current_sample_rate * 4;
            uint64_t expected_us = play_bytes * 1000000ULL / bps;

            struct timespec now;
            clock_gettime(CLOCK_MONOTONIC, &now);
            /* Use signed arithmetic to avoid uint64_t wrap when tv_nsec
             * rolls over (happens every second on a monotonic clock). */
            int64_t elapsed_us =
                (int64_t)(now.tv_sec  - play_start.tv_sec)  * 1000000LL +
                ((int64_t)now.tv_nsec - (int64_t)play_start.tv_nsec) / 1000LL;

            if (elapsed_us >= 0 && expected_us > (uint64_t)elapsed_us) {
                usleep((useconds_t)(expected_us - (uint64_t)elapsed_us));
            }
        }

        if (squeezelite_stop)
            break;

        pthread_mutex_lock(&squeezelite_mtx);
        bool got_more = pcm_play_dma_complete_callback(PCM_DMAST_OK,
                                                        &pcm_data, &pcm_size);
        pthread_mutex_unlock(&squeezelite_mtx);

        if (!got_more) {
            logf("pcm-squeezelite: no more PCM data");
            break;
        }

        pcm_play_dma_status_callback(PCM_DMAST_STARTED);
    }

    squeezelite_running = false;
    return NULL;
}

static void sink_dma_init(void)
{
    pthread_mutexattr_t attr;
    pthread_mutexattr_init(&attr);
    pthread_mutexattr_settype(&attr, PTHREAD_MUTEX_RECURSIVE);
    pthread_mutex_init(&squeezelite_mtx, &attr);
    pthread_mutexattr_destroy(&attr);
}

static void sink_dma_postinit(void)
{
}

static void sink_set_freq(uint16_t freq)
{
    current_sample_rate = hw_freq_sampr[freq];
    logf("pcm-squeezelite: sample rate %lu Hz", current_sample_rate);
}

static void sink_lock(void)
{
    pthread_mutex_lock(&squeezelite_mtx);
}

static void sink_unlock(void)
{
    pthread_mutex_unlock(&squeezelite_mtx);
}

static void sink_dma_start(const void *addr, size_t size)
{
    logf("pcm-squeezelite: start (%p, %zu)", addr, size);

    if (pcm_squeezelite_start() < 0) {
        logf("pcm-squeezelite: server start failed");
        return;
    }

    /* Reset real-time pacing for this track. */
    clock_gettime(CLOCK_MONOTONIC, &play_start);
    play_bytes = 0;

    pthread_mutex_lock(&squeezelite_mtx);
    pcm_data = addr;
    pcm_size = size;
    pthread_mutex_unlock(&squeezelite_mtx);

    squeezelite_stop    = false;
    squeezelite_running = true;
    pthread_create(&squeezelite_tid, NULL, squeezelite_thread, NULL);
}

static void sink_dma_stop(void)
{
    logf("pcm-squeezelite: stop");

    squeezelite_stop = true;

    if (squeezelite_running) {
        pthread_join(squeezelite_tid, NULL);
        squeezelite_running = false;
    }

    pthread_mutex_lock(&squeezelite_mtx);
    pcm_data = NULL;
    pcm_size = 0;
    pthread_mutex_unlock(&squeezelite_mtx);

    pcm_squeezelite_stop();
}

struct pcm_sink squeezelite_pcm_sink = {
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
