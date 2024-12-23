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
#define CIRCULAR_BUFFER_SIZE (2 * 1024 * 1024) // 2MB

bool pcm_thread_is_initialized = false;

/* PCM thread stack */
static long pcm_stack[(DEFAULT_STACK_SIZE * 4) / sizeof(long)];
static const char pcm_thread_name[] = "pcm";
unsigned int pcm_thread_id = 0;

extern void process_pcm_buffer(Uint8 *data, size_t size);
extern void debugfn(const char *args, int value);

/**
 * Circular buffer structure.
 */
typedef struct {
    Uint8 buffer[CIRCULAR_BUFFER_SIZE];
    size_t head;
    size_t tail;
    size_t size;
} CircularBuffer;

static CircularBuffer circular_buffer = { .head = 0, .tail = 0, .size = 0 };

/**
 * Write data to the circular buffer.
 */
static void circular_buffer_write(CircularBuffer *cb, const Uint8 *data, size_t size) {
    for (size_t i = 0; i < size; i++) {
        cb->buffer[cb->head] = data[i];
        cb->head = (cb->head + 1) % CIRCULAR_BUFFER_SIZE;
        if (cb->size < CIRCULAR_BUFFER_SIZE) {
            cb->size++;
        } else {
            cb->tail = (cb->tail + 1) % CIRCULAR_BUFFER_SIZE; // Overwrite oldest data
        }
    }
}

/**
 * Read data from the circular buffer.
 */
static size_t circular_buffer_read(CircularBuffer *cb, Uint8 *data, size_t size) {
    size_t bytes_read = 0;
    while (bytes_read < size && cb->size > 0) {
        data[bytes_read++] = cb->buffer[cb->tail];
        cb->tail = (cb->tail + 1) % CIRCULAR_BUFFER_SIZE;
        cb->size--;
    }
    return bytes_read;
}

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

        circular_buffer_write(&circular_buffer, conv_buffer, converted_size);

        curr_data += chunk_size;
        remaining -= chunk_size;
    }

    free(stream);
    free(conv_buffer);
}

/**
 * Pull audio data and process it for playback.
 */
void pull_audio_data() {
    const size_t THRESHOLD = 2 *1024 * 1024; // 2MB 
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

    size_t bytes_to_read = data_size;
    while (bytes_to_read > 0) {
        size_t chunk_size = circular_buffer_read(&circular_buffer, data, THRESHOLD);
        if (chunk_size > 0) {
            process_pcm_buffer(data, chunk_size);
            bytes_to_read -= chunk_size;
        } else {
            break; // No more data to read
        }
    }

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
