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
#include "system.h"
#include "pcm.h"
#include "pcm-internal.h"
#include "pcm_sampr.h"
#include <stdint.h>
#include <stdbool.h>
#include <SDL.h>

#define BYTES_PER_SAMPLE 4 

extern void debugfn(const char *args, int value);

extern void process_pcm_buffer(Uint8 *data, size_t size);

static void convert_audio_format(const void *input, uint8_t *output, 
                               size_t size, SDL_AudioCVT *cvt)
{
    if (cvt && cvt->needed) {
        // Copy input to conversion buffer
        memcpy(cvt->buf, input, size);
        cvt->len = size;

        // Convert the audio
        SDL_ConvertAudio(cvt);

        // Copy converted audio to output
        memcpy(output, cvt->buf, cvt->len_cvt);
    } else {
        // No conversion needed, direct copy
        memcpy(output, input, size);
    }
}

static void process_audio(SDL_AudioCVT *cvt, int sample_rate, int channels)
{
    const void *pcm_data;
    size_t pcm_data_size;
    bool new_buffer = false;
    uint8_t *stream;
    uint8_t *conv_buffer;  // Buffer for converted audio
    int len = 8192;

    // Allocate buffers for audio processing
    stream = (uint8_t *)malloc(len);
    conv_buffer = (uint8_t *)malloc(len * (cvt ? cvt->len_mult : 1));

    if (!stream || !conv_buffer) {
        free(stream);
        free(conv_buffer);
        return;
    }

    while (1) 
    {
        // Get new buffer from callback
        new_buffer = pcm_play_dma_complete_callback(PCM_DMAST_OK, &pcm_data, 
                                                  &pcm_data_size);

        if (!new_buffer || pcm_data_size == 0) {
            // No more data available
            break;
        }

        // Notify that we started processing
        if (new_buffer) {
            pcm_play_dma_status_callback(PCM_DMAST_STARTED);
        }

        // Process the buffer in chunks
        size_t remaining = pcm_data_size;
        const uint8_t *curr_data = pcm_data;

        while (remaining > 0) {
            // Determine how much to process this iteration
            size_t chunk_size = (remaining < (size_t)len) ? remaining : (size_t)len;

            // Copy data to processing buffer
            memcpy(stream, curr_data, chunk_size);

            // Convert audio format
            size_t converted_size;
            if (cvt && cvt->needed) {
                convert_audio_format(stream, conv_buffer, chunk_size, cvt);
                converted_size = (size_t)(chunk_size * cvt->len_ratio);
            } else {
                memcpy(conv_buffer, stream, chunk_size);
                converted_size = chunk_size;
            }

            // Process the converted audio data
            process_pcm_buffer(conv_buffer, converted_size);

            // Update pointers/counters
            curr_data += chunk_size;
            remaining -= chunk_size;
        }

         
    }

    free(stream);
    free(conv_buffer);
}

void pcm_callback(void)
{
    // Initialize audio format conversion
    SDL_AudioCVT cvt;
    int cvt_status = SDL_BuildAudioCVT(&cvt, 
        AUDIO_S16SYS,      // Source format
        2,                 // Source channels
        pcm_sampr,         // Source rate
        AUDIO_S16SYS,      // Destination format
        2,                 // Destination channels
        44100);           // Destination rate

    if (cvt_status > 0) {
        cvt.buf = (uint8_t *)malloc(8192 * cvt.len_mult); // Adjust size as needed
        if (!cvt.buf) {
            // Handle allocation failure
            return;
        }
    }

    // Start the processing
    process_audio(cvt_status > 0 ? &cvt : NULL, pcm_sampr, 2);


    // Cleanup
    if (cvt_status > 0 && cvt.buf) {
        free(cvt.buf);
    }
}