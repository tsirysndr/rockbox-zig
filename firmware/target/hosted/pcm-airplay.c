/***************************************************************************
 * PCM sink that encodes S16LE stereo PCM as ALAC escape frames and streams
 * them over RAOP (AirPlay 1) via UDP/RTP to a compatible receiver.
 *
 * Usage:
 *   pcm_airplay_set_host("192.168.1.x", 5000);
 *   pcm_switch_sink(PCM_SINK_AIRPLAY);
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
#include <string.h>

#include "pcm.h"
#include "pcm-internal.h"
#include "pcm_mixer.h"
#include "pcm_sampr.h"
#include "pcm_sink.h"

#define LOGF_ENABLE
#include "logf.h"

/* Rust C API — symbols are provided by the rockbox-airplay crate via
 * librockbox_cli.a. */
extern void    pcm_airplay_set_host(const char *host, uint16_t port);
extern void    pcm_airplay_add_receiver(const char *host, uint16_t port);
extern void    pcm_airplay_clear_receivers(void);
extern int     pcm_airplay_connect(void);
extern int     pcm_airplay_write(const uint8_t *data, size_t len);
extern void    pcm_airplay_stop(void);
extern void    pcm_airplay_close(void);

static const void *pcm_data = NULL;
static size_t      pcm_size = 0;

static pthread_mutex_t airplay_mtx;
static pthread_t       airplay_tid;
static volatile bool   airplay_running = false;
static volatile bool   airplay_stop    = false;

static void *airplay_thread(void *arg)
{
    (void)arg;

    while (!airplay_stop) {
        pthread_mutex_lock(&airplay_mtx);
        const void *data = pcm_data;
        size_t      size = pcm_size;
        pcm_data = NULL;
        pcm_size = 0;
        pthread_mutex_unlock(&airplay_mtx);

        if (data && size > 0) {
            if (pcm_airplay_write((const uint8_t *)data, size) < 0) {
                logf("pcm-airplay: write error");
                airplay_stop = true;
                break;
            }
        }

        if (airplay_stop)
            break;

        pthread_mutex_lock(&airplay_mtx);
        bool got_more = pcm_play_dma_complete_callback(PCM_DMAST_OK,
                                                        &pcm_data, &pcm_size);
        pthread_mutex_unlock(&airplay_mtx);

        if (!got_more) {
            logf("pcm-airplay: no more PCM data");
            break;
        }

        pcm_play_dma_status_callback(PCM_DMAST_STARTED);
    }

    airplay_running = false;
    return NULL;
}

static void sink_dma_init(void)
{
    pthread_mutexattr_t attr;
    pthread_mutexattr_init(&attr);
    pthread_mutexattr_settype(&attr, PTHREAD_MUTEX_RECURSIVE);
    pthread_mutex_init(&airplay_mtx, &attr);
    pthread_mutexattr_destroy(&attr);
}

static void sink_dma_postinit(void)
{
}

static void sink_set_freq(uint16_t freq)
{
    (void)freq;
}

static void sink_lock(void)
{
    pthread_mutex_lock(&airplay_mtx);
}

static void sink_unlock(void)
{
    pthread_mutex_unlock(&airplay_mtx);
}

static void sink_dma_start(const void *addr, size_t size)
{
    logf("pcm-airplay: start (%p, %zu)", addr, size);

    /* Connect if not already connected */
    if (pcm_airplay_connect() < 0) {
        logf("pcm-airplay: connect failed");
        return;
    }

    pthread_mutex_lock(&airplay_mtx);
    pcm_data = addr;
    pcm_size = size;
    pthread_mutex_unlock(&airplay_mtx);

    airplay_stop    = false;
    airplay_running = true;
    pthread_create(&airplay_tid, NULL, airplay_thread, NULL);
}

static void sink_dma_stop(void)
{
    logf("pcm-airplay: stop");

    airplay_stop = true;

    if (airplay_running) {
        pthread_join(airplay_tid, NULL);
        airplay_running = false;
    }

    pthread_mutex_lock(&airplay_mtx);
    pcm_data = NULL;
    pcm_size = 0;
    pthread_mutex_unlock(&airplay_mtx);

    pcm_airplay_stop();
}

struct pcm_sink airplay_pcm_sink = {
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
