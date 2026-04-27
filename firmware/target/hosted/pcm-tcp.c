/***************************************************************************
 *             __________               __   ___.
 *   Open      \______   \ ____   ____ |  | _\_ |__   _______  ___
 *   Source     |       _//  _ \_/ ___\|  |/ /| __ \ /  _ \  \/  /
 *   Jukebox    |    |   (  <_> )  \___|    < | \_\ (  <_> > <  <
 *   Firmware   |____|_  /\____/ \___  >__|_ \|___  /\____/__/\_ \
 *                     \/            \/     \/    \/            \/
 *
 * PCM sink that writes raw S16LE stereo PCM over a TCP socket.
 * Intended for use with Snapcast (tcp:// source) or similar consumers.
 *
 * Usage:
 *   pcm_tcp_set_host("192.168.1.x");   // snapserver host
 *   pcm_tcp_set_port(4953);            // snapserver tcp source port (default)
 *   pcm_switch_sink(PCM_SINK_SNAPCAST_TCP);
 *
 * Snapserver config:
 *   source = tcp://0.0.0.0:4953?name=default&sampleformat=44100:16:2
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

#include <arpa/inet.h>
#include <errno.h>
#include <netdb.h>
#include <pthread.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <unistd.h>

#include "pcm.h"
#include "pcm-internal.h"
#include "pcm_mixer.h"
#include "pcm_sampr.h"
#include "pcm_sink.h"

#define LOGF_ENABLE
#include "logf.h"

#define DEFAULT_TCP_HOST "127.0.0.1"
#define DEFAULT_TCP_PORT 4953

static char     tcp_host[256] = DEFAULT_TCP_HOST;
static uint16_t tcp_port      = DEFAULT_TCP_PORT;
static int      tcp_fd        = -1;

static const void    *pcm_data = NULL;
static size_t         pcm_size = 0;

static pthread_mutex_t tcp_mtx;
static pthread_t       tcp_tid;
static volatile bool   tcp_running = false;
static volatile bool   tcp_stop    = false;

void pcm_tcp_set_host(const char *host)
{
    strncpy(tcp_host, host, sizeof(tcp_host) - 1);
    tcp_host[sizeof(tcp_host) - 1] = '\0';
}

void pcm_tcp_set_port(uint16_t port)
{
    tcp_port = port;
}

static int tcp_connect_once(void)
{
    struct addrinfo hints, *res, *rp;
    char portstr[8];
    int fd = -1;

    memset(&hints, 0, sizeof(hints));
    hints.ai_family   = AF_UNSPEC;
    hints.ai_socktype = SOCK_STREAM;

    snprintf(portstr, sizeof(portstr), "%u", (unsigned)tcp_port);
    if (getaddrinfo(tcp_host, portstr, &hints, &res) != 0) {
        logf("pcm-tcp: getaddrinfo(%s:%s) failed: %s",
             tcp_host, portstr, strerror(errno));
        return -1;
    }

    for (rp = res; rp != NULL; rp = rp->ai_next) {
        fd = socket(rp->ai_family, rp->ai_socktype, rp->ai_protocol);
        if (fd < 0)
            continue;
        if (connect(fd, rp->ai_addr, rp->ai_addrlen) == 0)
            break;
        close(fd);
        fd = -1;
    }
    freeaddrinfo(res);

    if (fd < 0)
        logf("pcm-tcp: connect to %s:%u failed: %s",
             tcp_host, (unsigned)tcp_port, strerror(errno));
    else
        logf("pcm-tcp: connected to %s:%u fd=%d",
             tcp_host, (unsigned)tcp_port, fd);

    return fd;
}

static void *tcp_thread(void *arg)
{
    (void)arg;

    while (!tcp_stop) {
        pthread_mutex_lock(&tcp_mtx);
        const void *data = pcm_data;
        size_t      size = pcm_size;
        pcm_data = NULL;
        pcm_size = 0;
        pthread_mutex_unlock(&tcp_mtx);

        /* Write current chunk in pieces so stop() can interrupt promptly */
        while (size > 0 && !tcp_stop) {
            ssize_t n = write(tcp_fd, data, size);
            if (n < 0) {
                if (errno == EINTR || errno == EAGAIN)
                    continue;
                logf("pcm-tcp: write error: %s", strerror(errno));
                /* Mark connection dead; reconnect on next sink_dma_start */
                close(tcp_fd);
                tcp_fd = -1;
                tcp_stop = true;
                break;
            }
            data = (const char *)data + n;
            size -= n;
        }

        if (tcp_stop)
            break;

        /* Request next buffer */
        pthread_mutex_lock(&tcp_mtx);
        bool got_more = pcm_play_dma_complete_callback(PCM_DMAST_OK,
                                                        &pcm_data, &pcm_size);
        pthread_mutex_unlock(&tcp_mtx);

        if (!got_more) {
            logf("pcm-tcp: no more PCM data");
            break;
        }

        pcm_play_dma_status_callback(PCM_DMAST_STARTED);
    }

    tcp_running = false;
    return NULL;
}

static void sink_dma_init(void)
{
    pthread_mutexattr_t attr;
    pthread_mutexattr_init(&attr);
    pthread_mutexattr_settype(&attr, PTHREAD_MUTEX_RECURSIVE);
    pthread_mutex_init(&tcp_mtx, &attr);
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
    pthread_mutex_lock(&tcp_mtx);
}

static void sink_unlock(void)
{
    pthread_mutex_unlock(&tcp_mtx);
}

static void sink_dma_start(const void *addr, size_t size)
{
    logf("pcm-tcp: start (%p, %zu) fd=%d", addr, size, tcp_fd);

    /* Connect on first call or after a write error closed the socket */
    if (tcp_fd < 0) {
        tcp_fd = tcp_connect_once();
        if (tcp_fd < 0) {
            logf("pcm-tcp: dropping audio (no connection to %s:%u)",
                 tcp_host, (unsigned)tcp_port);
            return;
        }
    }

    pthread_mutex_lock(&tcp_mtx);
    pcm_data = addr;
    pcm_size = size;
    pthread_mutex_unlock(&tcp_mtx);

    tcp_stop    = false;
    tcp_running = true;
    pthread_create(&tcp_tid, NULL, tcp_thread, NULL);
}

static void sink_dma_stop(void)
{
    logf("pcm-tcp: stop");

    tcp_stop = true;

    if (tcp_running) {
        pthread_join(tcp_tid, NULL);
        tcp_running = false;
    }

    pthread_mutex_lock(&tcp_mtx);
    pcm_data = NULL;
    pcm_size = 0;
    pthread_mutex_unlock(&tcp_mtx);

    /* Keep tcp_fd open between tracks so snapserver doesn't see a disconnect.
     * If the write loop closed it on error, tcp_fd is already -1 and the
     * next sink_dma_start will reconnect automatically. */
}

struct pcm_sink tcp_pcm_sink = {
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
