/***************************************************************************
 *             __________               __   ___.
 *   Open      \______   \ ____   ____ |  | _\_ |__   _______  ___
 *   Source     |       _//  _ \_/ ___\|  |/ /| __ \ /  _ \  \/  /
 *   Jukebox    |    |   (  <_> )  \___|    < | \_\ (  <_> > <  <
 *   Firmware   |____|_  /\____/ \___  >__|_ \|___  /\____/__/\_ \
 *                     \/            \/     \/    \/            \/
 *
 * Copyright (C) 2025 by Sho Tanimoto
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
#pragma once
#include <stddef.h>
#include <stdint.h>

struct pcm_sink_caps {
    const unsigned long* samprs;
    uint16_t             num_samprs;
    uint16_t             default_freq;
};

struct pcm_sink_ops {
    void (*init)(void);
    void (*postinit)(void);
    void (*set_freq)(uint16_t freq);
    void (*lock)(void);
    void (*unlock)(void);
    void (*play)(const void* addr, size_t size);
    void (*stop)(void);
};

struct pcm_sink {
    /* characteristics */
    const struct pcm_sink_caps caps;

    /* operations */
    const struct pcm_sink_ops ops;

    /* runtime states */
    unsigned long pending_freq;
    unsigned long configured_freq;
    unsigned long pcm_is_ready;
};

enum pcm_sink_ids {
    PCM_SINK_BUILTIN = 0,
#if (CONFIG_PLATFORM & PLATFORM_HOSTED)
    PCM_SINK_FIFO,
    PCM_SINK_AIRPLAY,
    PCM_SINK_SQUEEZELITE,
    PCM_SINK_UPNP,
    PCM_SINK_CHROMECAST,
    PCM_SINK_SNAPCAST_TCP,
#endif
    PCM_SINK_NUM
};

/* defined in each platform pcm source */
extern struct pcm_sink builtin_pcm_sink;

#if (CONFIG_PLATFORM & PLATFORM_HOSTED)
/* FIFO/pipe sink — writes raw S16LE stereo PCM to a named FIFO or stdout */
extern struct pcm_sink fifo_pcm_sink;
void pcm_fifo_set_path(const char *path);

/* AirPlay (RAOP) sink — streams ALAC-encoded audio over RTP */
extern struct pcm_sink airplay_pcm_sink;

/* Squeezelite (Slim Protocol) sink — serves PCM via HTTP to squeezelite */
extern struct pcm_sink squeezelite_pcm_sink;

/* UPnP/DLNA sink — streams WAV over HTTP to UPnP renderers */
extern struct pcm_sink upnp_pcm_sink;
void pcm_upnp_set_http_port(uint16_t port);
void pcm_upnp_set_renderer_url(const char *url);

/* Chromecast sink — streams WAV over HTTP and loads via Cast protocol */
extern struct pcm_sink chromecast_pcm_sink;
void pcm_chromecast_set_http_port(uint16_t port);
void pcm_chromecast_set_device_host(const char *host);
void pcm_chromecast_set_device_port(uint16_t port);

/* Snapcast TCP sink — streams raw S16LE PCM to snapserver's tcp:// source */
extern struct pcm_sink tcp_pcm_sink;
void pcm_tcp_set_host(const char *host);
void pcm_tcp_set_port(uint16_t port);
#endif
