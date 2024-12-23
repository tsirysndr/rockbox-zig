/***************************************************************************
 *             __________               __   ___.
 *   Open      \______   \ ____   ____ |  | _\_ |__   _______  ___
 *   Source     |       _//  _ \_/ ___\|  |/ /| __ \ /  _ \  \/  /
 *   Jukebox    |    |   (  <_> )  \___|    < | \_\ (  <_> > <  <
 *   Firmware   |____|_  /\____/ \___  >__|_ \|___  /\____/__/\_ \
 *                     \/            \/     \/    \/            \/
 * $Id$
 *
 * Copyright (C) 2024 - Tsiry Sandratraina
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

#include "config.h"
#include "kernel.h"
#include "logf.h"
#include "system.h"

#include "pcm.h"
#include "pcm-internal.h"
#include "pcm_sampr.h"
#include <stdint.h>
#include <stdbool.h>
#include <SDL.h>
#include <stdlib.h>
#include <string.h>

#define BUFFER_SIZE 4096

bool pcm_thread_is_initialized = false;

/* PCM thread stack */
static long pcm_stack[(DEFAULT_STACK_SIZE * 4) / sizeof(long)];
static const char pcm_thread_name[] = "pcm";
unsigned int pcm_thread_id = 0;

extern void process_pcm_buffer(Uint8 *data, size_t size);
extern void debugfn(const char *args, int value);

/**
 * Convert audio format using SDL_AudioCVT.
 */
static void convert_audio_format(const void *input, uint8_t *output, size_t size, SDL_AudioCVT *cvt) {
    if (cvt && cvt->needed) {
        memcpy(cvt->buf, input, size);
        cvt->len = (int)size;

        if (SDL_ConvertAudio(cvt) != 0) {
            logf("Audio conversion failed: %s", SDL_GetError());
            return;
        }

        memcpy(output, cvt->buf, (size_t)cvt->len_cvt);
    } else {
        memcpy(output, input, size);
    }
}

/**
 * Process audio data, handling conversion and buffering.
 */
static void process_audio(SDL_AudioCVT *cvt, Uint8 *data, size_t *data_size) {
    const void *pcm_data = NULL;
    size_t pcm_data_size = 0;
    bool new_buffer = false;

    uint8_t *stream = (uint8_t *)malloc(BUFFER_SIZE);
    uint8_t *conv_buffer = (uint8_t *)malloc(BUFFER_SIZE * (cvt ? cvt->len_mult : 1));

    if (!stream || !conv_buffer) {
        logf("Memory allocation failed in process_audio");
        free(stream);
        free(conv_buffer);
        return;
    }

    *data_size = 0;

    new_buffer = pcm_play_dma_complete_callback(PCM_DMAST_OK, &pcm_data, &pcm_data_size);

    if (!new_buffer || pcm_data_size == 0) {
        free(stream);
        free(conv_buffer);
        return;
    }

    pcm_play_dma_status_callback(PCM_DMAST_STARTED);

    size_t remaining = pcm_data_size;
    const uint8_t *curr_data = (const uint8_t *)pcm_data;

    while (remaining > 0) {
        size_t chunk_size = (remaining < BUFFER_SIZE) ? remaining : BUFFER_SIZE;

        memcpy(stream, curr_data, chunk_size);

        size_t converted_size = 0;
        if (cvt && cvt->needed) {
            convert_audio_format(stream, conv_buffer, chunk_size, cvt);
            converted_size = (size_t)cvt->len_cvt;
        } else {
            memcpy(conv_buffer, stream, chunk_size);
            converted_size = chunk_size;
        }

        memcpy(data + *data_size, conv_buffer, converted_size);
        *data_size += converted_size;

        curr_data += chunk_size;
        remaining -= chunk_size;

        debugfn("Processed chunk size", converted_size);
        debugfn("Processed total data size", *data_size);
    }

    free(stream);
    free(conv_buffer);
}

/**
 * Pull audio data and process it for playback.
 */
void pull_audio_data() {
    Uint8 *data = (Uint8 *)malloc(BUFFER_SIZE * 2); // Allocate enough space for output audio
    if (!data) {
        logf("Memory allocation failed in pull_audio_data");
        return;
    }

    SDL_AudioCVT cvt;
    int cvt_status = SDL_BuildAudioCVT(&cvt, AUDIO_S16SYS, 2, pcm_sampr, AUDIO_S16SYS, 2, 44100);

    if (cvt_status > 0) {
        cvt.buf = (uint8_t *)malloc(BUFFER_SIZE * cvt.len_mult);
        if (!cvt.buf) {
            logf("Memory allocation failed for SDL_AudioCVT buffer");
            free(data);
            return;
        }
    }

    size_t data_size = 0;
    process_audio(cvt_status > 0 ? &cvt : NULL, data, &data_size);

    if (data_size == 0)  {
      free(data);
      return;
    }
    process_pcm_buffer(data, data_size);

    if (cvt_status > 0 && cvt.buf) {
        free(cvt.buf);
    }

    free(data);
}

/**
 * PCM thread main loop.
 */
static void pcm_thread(void) {
    while (true) {
        pull_audio_data();
        sleep(HZ / 2); 
    }
}

/**
 * Initialize the PCM thread.
 */
void INIT_ATTR pcm_thread_init(void) {
    if (pcm_thread_is_initialized) {
        logf("PCM thread already initialized");
        return;
    }

    logf("Initializing PCM thread");

    pcm_thread_id = create_thread(
        pcm_thread, pcm_stack, sizeof(pcm_stack), 0,
        pcm_thread_name IF_PRIO(, PRIORITY_USER_INTERFACE) IF_COP(, CPU));

    sleep(HZ);

    pcm_thread_is_initialized = true;
}

