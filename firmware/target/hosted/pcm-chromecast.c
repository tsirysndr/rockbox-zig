/***************************************************************************
 * PCM sink that streams raw S16LE stereo PCM to Chromecast devices as a
 * continuous WAV stream over HTTP (port 7881 by default), and tells the
 * Chromecast to load that URL via the Cast Media protocol.
 *
 * Usage:
 *   pcm_chromecast_set_http_port(7881);          // optional, this is the default
 *   pcm_chromecast_set_device_host("192.168.1.x"); // Chromecast device IP
 *   pcm_chromecast_set_device_port(8009);         // optional, default 8009
 *   pcm_switch_sink(PCM_SINK_CHROMECAST);
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

/* Rust C API — symbols provided by the rockbox-chromecast crate via librockbox_cli.a */
extern void pcm_chromecast_set_http_port(uint16_t port);
extern void pcm_chromecast_set_device_host(const char *host);
extern void pcm_chromecast_set_device_port(uint16_t port);
extern void pcm_chromecast_set_sample_rate(uint32_t rate);
extern int  pcm_chromecast_start(void);
extern int  pcm_chromecast_write(const uint8_t *data, size_t len);
extern void pcm_chromecast_stop(void);
extern void pcm_chromecast_close(void);

static const void *pcm_data = NULL;
static size_t      pcm_size = 0;

/* Scratch buffer for SW volume scaling */
static void  *chromecast_vol_buf     = NULL;
static size_t chromecast_vol_buf_cap = 0;

static pthread_mutex_t chromecast_mtx;
static pthread_t       chromecast_tid;
static volatile bool   chromecast_running = false;
static volatile bool   chromecast_stop    = false;

/* Real-time pacing — reset on every sink_dma_start(). */
static struct timespec play_start;
static uint64_t        play_bytes;

/* Actual sample rate set by sink_set_freq(); defaults to 44100.
 * bytes_per_sec = sample_rate * 2 channels * 2 bytes/sample */
static unsigned long current_sample_rate = 44100;

static void *chromecast_thread(void *arg)
{
    (void)arg;

    while (!chromecast_stop) {
        pthread_mutex_lock(&chromecast_mtx);
        const void *raw  = pcm_data;
        size_t      size = pcm_size;
        pcm_data = NULL;
        pcm_size = 0;
        pthread_mutex_unlock(&chromecast_mtx);

        /* Apply SW volume scaling */
        if (size > chromecast_vol_buf_cap) {
            free(chromecast_vol_buf);
            chromecast_vol_buf     = malloc(size);
            chromecast_vol_buf_cap = chromecast_vol_buf ? size : 0;
        }
        const void *data = (chromecast_vol_buf && size > 0)
            ? (pcm_copy_buffer(chromecast_vol_buf, raw, size), chromecast_vol_buf)
            : raw;
        if (data == chromecast_vol_buf)
            pcm_normalizer_apply(chromecast_vol_buf, size);

        if (data && size > 0) {
            if (pcm_chromecast_write((const uint8_t *)data, size) < 0) {
                logf("pcm-chromecast: write error");
                chromecast_stop = true;
                break;
            }

            /* Pace to real-time so the DMA loop does not drain the entire
             * track instantly.  Same technique as pcm-upnp.c — use
             * signed int64_t for the nanosecond diff to avoid uint wrap. */
            play_bytes += size;
            uint64_t bps        = (uint64_t)current_sample_rate * 4;
            uint64_t expected_us = play_bytes * 1000000ULL / bps;

            struct timespec now;
            clock_gettime(CLOCK_MONOTONIC, &now);
            int64_t elapsed_us =
                (int64_t)(now.tv_sec  - play_start.tv_sec)  * 1000000LL +
                ((int64_t)now.tv_nsec - (int64_t)play_start.tv_nsec) / 1000LL;

            if (elapsed_us >= 0 && expected_us > (uint64_t)elapsed_us) {
                usleep((useconds_t)(expected_us - (uint64_t)elapsed_us));
            }
        }

        if (chromecast_stop)
            break;

        pthread_mutex_lock(&chromecast_mtx);
        bool got_more = pcm_play_dma_complete_callback(PCM_DMAST_OK,
                                                        &pcm_data, &pcm_size);
        pthread_mutex_unlock(&chromecast_mtx);

        if (!got_more) {
            logf("pcm-chromecast: no more PCM data");
            break;
        }

        pcm_play_dma_status_callback(PCM_DMAST_STARTED);
    }

    chromecast_running = false;
    return NULL;
}

static void sink_dma_init(void)
{
    pthread_mutexattr_t attr;
    pthread_mutexattr_init(&attr);
    pthread_mutexattr_settype(&attr, PTHREAD_MUTEX_RECURSIVE);
    pthread_mutex_init(&chromecast_mtx, &attr);
    pthread_mutexattr_destroy(&attr);
}

static void sink_dma_postinit(void)
{
}

static void sink_set_freq(uint16_t freq)
{
    current_sample_rate = hw_freq_sampr[freq];
    pcm_chromecast_set_sample_rate((uint32_t)current_sample_rate);
    logf("pcm-chromecast: sample rate %lu Hz", current_sample_rate);
}

static void sink_lock(void)
{
    pthread_mutex_lock(&chromecast_mtx);
}

static void sink_unlock(void)
{
    pthread_mutex_unlock(&chromecast_mtx);
}

static void sink_dma_start(const void *addr, size_t size)
{
    logf("pcm-chromecast: start (%p, %zu)", addr, size);

    if (pcm_chromecast_start() < 0) {
        logf("pcm-chromecast: server start failed");
        return;
    }

    clock_gettime(CLOCK_MONOTONIC, &play_start);
    play_bytes = 0;

    pthread_mutex_lock(&chromecast_mtx);
    pcm_data = addr;
    pcm_size = size;
    pthread_mutex_unlock(&chromecast_mtx);

    chromecast_stop    = false;
    chromecast_running = true;
    pthread_create(&chromecast_tid, NULL, chromecast_thread, NULL);
}

static void sink_dma_stop(void)
{
    logf("pcm-chromecast: stop");

    chromecast_stop = true;

    if (chromecast_running) {
        pthread_join(chromecast_tid, NULL);
        chromecast_running = false;
    }

    pthread_mutex_lock(&chromecast_mtx);
    pcm_data = NULL;
    pcm_size = 0;
    pthread_mutex_unlock(&chromecast_mtx);

    pcm_chromecast_stop();
}

struct pcm_sink chromecast_pcm_sink = {
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
