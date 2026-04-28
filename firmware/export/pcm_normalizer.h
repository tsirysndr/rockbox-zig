#ifndef PCM_NORMALIZER_H
#define PCM_NORMALIZER_H

#include <stdbool.h>
#include <stddef.h>

/* Real-time PCM loudness normalizer — Spotify-style "Normalize Volume".
 *
 * Tracks the RMS energy of the audio stream with asymmetric attack/release
 * smoothing and applies a per-chunk gain so all tracks play at roughly the
 * same perceived loudness (target: -23 dBFS RMS ≈ EBU R128).
 *
 * Called by each PCM sink after SW-volume scaling (pcm_copy_buffer).
 * Operates in-place on S16LE stereo PCM. */

void pcm_normalizer_enable(bool enable);
bool pcm_normalizer_is_enabled(void);
void pcm_normalizer_apply(void *buf, size_t bytes);

#endif /* PCM_NORMALIZER_H */
