/***************************************************************************
 * PCM sink that AAC-LC encodes the audio stream and exposes it as both
 * HLS and DASH from the same in-memory CMAF (fragmented MP4) segment ring.
 *
 * Usage:
 *   pcm_cmaf_set_http_port(7882);   // optional, this is the default
 *   pcm_cmaf_set_bitrate(128000);   // optional, this is the default
 *   pcm_switch_sink(PCM_SINK_CMAF);
 *   // Then point any HLS/DASH player at:
 *   //   http://<host>:7882/hls/master.m3u8
 *   //   http://<host>:7882/dash/manifest.mpd
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
#include "pcm_sampr.h"
#include "pcm_sink.h"

#define LOGF_ENABLE
#include "logf.h"

/* Rust C API — symbols provided by the rockbox-cmaf crate via librockbox_cli.a */
extern void pcm_cmaf_set_http_port(uint16_t port);
extern void pcm_cmaf_set_bitrate(uint32_t bps);
extern int  pcm_cmaf_start(void);
extern int  pcm_cmaf_write(const uint8_t *data, size_t len);
extern void pcm_cmaf_stop(void);
extern void pcm_cmaf_close(void);

static const void *pcm_data = NULL;
static size_t      pcm_size = 0;

/* Scratch buffer for SW volume scaling */
static void  *cmaf_vol_buf     = NULL;
static size_t cmaf_vol_buf_cap = 0;

static pthread_mutex_t cmaf_mtx;
static pthread_t       cmaf_tid;
static volatile bool   cmaf_running = false;
static volatile bool   cmaf_stop    = false;

/* Real-time pacing state — reset on every sink_dma_start(). */
static struct timespec play_start;
static uint64_t        play_bytes;

/* Actual sample rate set by sink_set_freq(); defaults to 44100.
 * bytes_per_sec = sample_rate * 2 channels * 2 bytes/sample. */
static unsigned long current_sample_rate = 44100;

static void *cmaf_thread(void *arg)
{
    (void)arg;

    while (!cmaf_stop) {
        pthread_mutex_lock(&cmaf_mtx);
        const void *raw  = pcm_data;
        size_t      size = pcm_size;
        pcm_data = NULL;
        pcm_size = 0;
        pthread_mutex_unlock(&cmaf_mtx);

        /* Apply SW volume scaling */
        if (size > cmaf_vol_buf_cap) {
            free(cmaf_vol_buf);
            cmaf_vol_buf     = malloc(size);
            cmaf_vol_buf_cap = cmaf_vol_buf ? size : 0;
        }
        const void *data = (cmaf_vol_buf && size > 0)
            ? (pcm_copy_buffer(cmaf_vol_buf, raw, size), cmaf_vol_buf)
            : raw;
        if (data && size > 0) {
            if (pcm_cmaf_write((const uint8_t *)data, size) < 0) {
                logf("pcm-cmaf: write error");
                cmaf_stop = true;
                break;
            }

            /* Pace to real-time so the DMA loop does not drain the whole
             * track instantly; otherwise the encoder would consume the
             * entire file before any HTTP listener can connect.  We track
             * total bytes written and sleep whenever we're ahead of the
             * expected wall-clock position.
             *
             * bytes_per_sec = sample_rate * 2 channels * 2 bytes/sample */
            play_bytes += size;
            uint64_t bps         = (uint64_t)current_sample_rate * 4;
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

        if (cmaf_stop)
            break;

        pthread_mutex_lock(&cmaf_mtx);
        bool got_more = pcm_play_dma_complete_callback(PCM_DMAST_OK,
                                                       &pcm_data, &pcm_size);
        pthread_mutex_unlock(&cmaf_mtx);

        if (!got_more) {
            logf("pcm-cmaf: no more PCM data");
            break;
        }

        pcm_play_dma_status_callback(PCM_DMAST_STARTED);
    }

    cmaf_running = false;
    return NULL;
}

static void sink_dma_init(void)
{
    pthread_mutexattr_t attr;
    pthread_mutexattr_init(&attr);
    pthread_mutexattr_settype(&attr, PTHREAD_MUTEX_RECURSIVE);
    pthread_mutex_init(&cmaf_mtx, &attr);
    pthread_mutexattr_destroy(&attr);
}

static void sink_dma_postinit(void)
{
}

static void sink_set_freq(uint16_t freq)
{
    current_sample_rate = hw_freq_sampr[freq];
    logf("pcm-cmaf: sample rate %lu Hz", current_sample_rate);
}

static void sink_lock(void)
{
    pthread_mutex_lock(&cmaf_mtx);
}

static void sink_unlock(void)
{
    pthread_mutex_unlock(&cmaf_mtx);
}

static void sink_dma_start(const void *addr, size_t size)
{
    logf("pcm-cmaf: start (%p, %zu)", addr, size);

    if (pcm_cmaf_start() < 0) {
        logf("pcm-cmaf: server start failed");
        return;
    }

    /* Reset real-time pacing for this track. */
    clock_gettime(CLOCK_MONOTONIC, &play_start);
    play_bytes = 0;

    pthread_mutex_lock(&cmaf_mtx);
    pcm_data = addr;
    pcm_size = size;
    pthread_mutex_unlock(&cmaf_mtx);

    cmaf_stop    = false;
    cmaf_running = true;
    pthread_create(&cmaf_tid, NULL, cmaf_thread, NULL);
}

static void sink_dma_stop(void)
{
    logf("pcm-cmaf: stop");

    cmaf_stop = true;

    if (cmaf_running) {
        pthread_join(cmaf_tid, NULL);
        cmaf_running = false;
    }

    pthread_mutex_lock(&cmaf_mtx);
    pcm_data = NULL;
    pcm_size = 0;
    pthread_mutex_unlock(&cmaf_mtx);

    pcm_cmaf_stop();
}

struct pcm_sink cmaf_pcm_sink = {
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
