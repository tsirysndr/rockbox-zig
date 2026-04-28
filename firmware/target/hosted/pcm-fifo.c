/***************************************************************************
 *             __________               __   ___.
 *   Open      \______   \ ____   ____ |  | _\_ |__   _______  ___
 *   Source     |       _//  _ \_/ ___\|  |/ /| __ \ /  _ \  \/  /
 *   Jukebox    |    |   (  <_> )  \___|    < | \_\ (  <_> > <  <
 *   Firmware   |____|_  /\____/ \___  >__|_ \|___  /\____/__/\_ \
 *                     \/            \/     \/    \/            \/
 *
 * PCM sink that writes raw S16LE stereo PCM to a named FIFO or stdout.
 * Intended for use with Snapcast (pipe:// source) or similar consumers.
 *
 * Usage:
 *   pcm_fifo_set_path("/tmp/rockbox.fifo");  // or "-" for stdout
 *   pcm_switch_sink(PCM_SINK_FIFO);
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

#include <errno.h>
#include <fcntl.h>
#include <pthread.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <unistd.h>

#include "pcm.h"
#include "pcm-internal.h"
#include "pcm_mixer.h"
#include "pcm_normalizer.h"
#include "pcm_sampr.h"
#include "pcm_sink.h"

#define LOGF_ENABLE
#include "logf.h"

#define DEFAULT_FIFO_PATH "/tmp/rockbox.fifo"

static const char *fifo_path = DEFAULT_FIFO_PATH;
static int fifo_fd = -1;

static const void *pcm_data = NULL;
static size_t      pcm_size = 0;

/* Scratch buffer for SW volume scaling */
static void  *fifo_vol_buf     = NULL;
static size_t fifo_vol_buf_cap = 0;

static pthread_mutex_t fifo_mtx;
static pthread_t       fifo_tid;
static volatile bool   fifo_running = false;
static volatile bool   fifo_stop    = false;

/* When writing to stdout, we dup the real stdout fd so that any printf()
   inside Rockbox (which goes to fd 1) is redirected to stderr, keeping
   the PCM stream on the original stdout clean. */
static int stdout_pcm_fd = -1;

static void redirect_stdout_to_stderr(void)
{
    /* Save a copy of the real stdout */
    stdout_pcm_fd = dup(STDOUT_FILENO);
    /* Point fd 1 at stderr so printf/puts don't corrupt the PCM stream */
    dup2(STDERR_FILENO, STDOUT_FILENO);
}

void pcm_fifo_set_path(const char *path)
{
    fifo_path = path;

    if (strcmp(path, "-") == 0) {
        if (stdout_pcm_fd < 0)
            redirect_stdout_to_stderr();
        fifo_fd = stdout_pcm_fd;
        return;
    }

    /* Pre-create the FIFO so external readers (e.g. snapserver) can open it
     * before playback starts. Open with O_RDWR so we hold a permanent writer
     * reference: this prevents readers from ever seeing a premature EOF when
     * we're between tracks or haven't started playing yet. */
    if (mkfifo(path, 0666) < 0 && errno != EEXIST)
        logf("pcm-fifo: mkfifo(%s) failed: %s", path, strerror(errno));

    if (fifo_fd >= 0 && fifo_fd != stdout_pcm_fd) {
        close(fifo_fd);
        fifo_fd = -1;
    }

    /* O_RDWR|O_NONBLOCK: open succeeds immediately without a reader present.
     * Then clear O_NONBLOCK so writes in fifo_thread block naturally,
     * paced by the reader (snapcast/ffplay). */
    fifo_fd = open(path, O_RDWR | O_NONBLOCK);
    if (fifo_fd < 0) {
        logf("pcm-fifo: pre-open(%s) failed: %s", path, strerror(errno));
    } else {
        int flags = fcntl(fifo_fd, F_GETFL);
        if (flags >= 0)
            fcntl(fifo_fd, F_SETFL, flags & ~O_NONBLOCK);
        logf("pcm-fifo: pre-opened %s fd=%d (blocking)", path, fifo_fd);
    }
}

static void *fifo_thread(void *arg)
{
    (void)arg;

    while (!fifo_stop) {
        pthread_mutex_lock(&fifo_mtx);
        const void *raw  = pcm_data;
        size_t      size = pcm_size;
        pcm_data = NULL;
        pcm_size = 0;
        pthread_mutex_unlock(&fifo_mtx);

        /* Apply SW volume scaling before writing */
        if (size > fifo_vol_buf_cap) {
            free(fifo_vol_buf);
            fifo_vol_buf     = malloc(size);
            fifo_vol_buf_cap = fifo_vol_buf ? size : 0;
        }
        const void *data = (fifo_vol_buf && size > 0)
            ? (pcm_copy_buffer(fifo_vol_buf, raw, size), fifo_vol_buf)
            : raw;
        if (data == fifo_vol_buf)
            pcm_normalizer_apply(fifo_vol_buf, size);

        /* Write current chunk in pieces so stop() can interrupt promptly */
        while (size > 0 && !fifo_stop) {
            ssize_t n = write(fifo_fd, data, size);
            if (n < 0) {
                if (errno == EINTR || errno == EAGAIN)
                    continue;
                logf("pcm-fifo: write error: %s", strerror(errno));
                fifo_stop = true;
                break;
            }
            data = (const char *)data + n;
            size -= n;
        }

        if (fifo_stop)
            break;

        /* Ask for the next buffer */
        pthread_mutex_lock(&fifo_mtx);
        bool got_more = pcm_play_dma_complete_callback(PCM_DMAST_OK,
                                                        &pcm_data, &pcm_size);
        pthread_mutex_unlock(&fifo_mtx);

        if (!got_more) {
            logf("pcm-fifo: no more PCM data");
            break;
        }

        pcm_play_dma_status_callback(PCM_DMAST_STARTED);
    }

    fifo_running = false;
    return NULL;
}

static void sink_dma_init(void)
{
    pthread_mutexattr_t attr;
    pthread_mutexattr_init(&attr);
    pthread_mutexattr_settype(&attr, PTHREAD_MUTEX_RECURSIVE);
    pthread_mutex_init(&fifo_mtx, &attr);
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
    pthread_mutex_lock(&fifo_mtx);
}

static void sink_unlock(void)
{
    pthread_mutex_unlock(&fifo_mtx);
}

static void sink_dma_start(const void *addr, size_t size)
{
    logf("pcm-fifo: start (%p, %zu) fd=%d", addr, size, fifo_fd);

    /* fifo_fd should already be open (set by pcm_fifo_set_path).
     * Fallback: try a blocking open so we wait for a reader. */
    if (fifo_fd < 0) {
        if (strcmp(fifo_path, "-") == 0) {
            fifo_fd = stdout_pcm_fd;
        } else {
            if (mkfifo(fifo_path, 0666) < 0 && errno != EEXIST)
                logf("pcm-fifo: mkfifo(%s) failed: %s", fifo_path, strerror(errno));
            fifo_fd = open(fifo_path, O_RDWR | O_NONBLOCK);
        }
        if (fifo_fd < 0) {
            logf("pcm-fifo: no reader on %s — dropping audio", fifo_path);
            return;
        }
    }

    pthread_mutex_lock(&fifo_mtx);
    pcm_data = addr;
    pcm_size = size;
    pthread_mutex_unlock(&fifo_mtx);

    fifo_stop    = false;
    fifo_running = true;
    pthread_create(&fifo_tid, NULL, fifo_thread, NULL);
}

static void sink_dma_stop(void)
{
    logf("pcm-fifo: stop");

    fifo_stop = true;

    if (fifo_running) {
        pthread_join(fifo_tid, NULL);
        fifo_running = false;
    }

    pthread_mutex_lock(&fifo_mtx);
    pcm_data = NULL;
    pcm_size = 0;
    pthread_mutex_unlock(&fifo_mtx);

    /* Keep fifo_fd open between tracks so the reader (e.g. snapserver) never
     * sees EOF and doesn't disconnect.  The fd is only ever closed if
     * pcm_fifo_set_path() is called with a new path. */
}

struct pcm_sink fifo_pcm_sink = {
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
