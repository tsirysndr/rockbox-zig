/*
 * Real-time PCM loudness normalizer.
 *
 * Algorithm: RMS-based Automatic Gain Control with asymmetric attack/release
 * smoothing, similar to what Spotify's "Normalize Volume" does at the track
 * replay-gain level — but applied in real time at the PCM buffer level so it
 * works for any source (local files, streams, radio) without metadata.
 *
 * Data format: S16LE stereo, 44100 Hz (Rockbox hosted / sdlapp target).
 *
 * Copyright (C) 2026 Rockbox contributors
 *
 * This program is free software; you can redistribute it and/or
 * modify it under the terms of the GNU General Public License
 * as published by the Free Software Foundation; either version 2
 * of the License, or (at your option) any later version.
 */

#include "pcm_normalizer.h"

#include <math.h>
#include <stdint.h>

/* ── Tuning knobs ─────────────────────────────────────────────────────────────
 *
 * TARGET_RMS    Target output loudness, linear amplitude.
 *               -23 dBFS = 10^(-23/20) ≈ 0.0708  (EBU R128 / broadcast).
 *
 * RMS_ATTACK    Per-chunk IIR coefficient for the RMS estimator rising
 *               toward a LOUDER signal.  Smaller → faster attack.
 *               0.3 ≈ 2-3 chunks (< 150 ms) to track a loud transient.
 *
 * RMS_RELEASE   Per-chunk IIR coefficient for the RMS estimator falling
 *               toward a QUIETER signal.  Larger → slower release.
 *               0.997 ≈ 330 chunks (≈ 15 s) to fully settle.
 *
 * GAIN_ATTACK   How fast the applied gain is REDUCED (loud incoming signal).
 *               Fast: prevents clipping when switching to a loud track.
 *
 * GAIN_RELEASE  How slowly the applied gain is RAISED (quiet signal).
 *               Very slow: prevents the "pumping" artefact during pauses
 *               or quiet passages between tracks.
 *
 * MAX_GAIN      Hard upper bound on the boost, +18 dB.
 *               Prevents amplifying near-silence to noise.
 *
 * MIN_GAIN      Hard lower bound, -18 dB.
 *               Prevents crushing content that is louder than target.
 *
 * GATE_THRESH   RMS below this level → chunk is silence; neither the RMS
 *               estimate nor the gain are updated.  -60 dBFS ≈ 0.001.
 * ──────────────────────────────────────────────────────────────────────────── */
#define TARGET_RMS    0.50f     /* -6 dBFS — loud, punchy target */
#define RMS_ATTACK    0.3f      /* fast: track loud transients in ~2-3 chunks */
#define RMS_RELEASE   0.99f     /* ~7 s to settle on a quieter signal */
#define GAIN_ATTACK   0.3f      /* reduce gain quickly to prevent clipping */
#define GAIN_RELEASE  0.98f     /* ~3 s to raise gain — feels responsive without pumping */
#define MAX_GAIN      10.0f     /* +20 dB — handles very quiet classical / ambient */
#define MIN_GAIN      0.1f      /* -20 dB */
#define GATE_THRESH   0.001f

static bool  normalizer_enabled = false;
static float gain               = 1.0f;
static float rms_estimate       = 0.1f; /* warm-start below target → gain > 1 from the first chunk */

void pcm_normalizer_enable(bool enable)
{
    if (enable && !normalizer_enabled) {
        /* Reset state on each fresh enable so a stale gain from a previous
         * session doesn't immediately blast or mute the first chunk. */
        gain         = 1.0f;
        rms_estimate = 0.1f;
    }
    normalizer_enabled = enable;
}

bool pcm_normalizer_is_enabled(void)
{
    return normalizer_enabled;
}

void pcm_normalizer_apply(void *buf, size_t bytes)
{
    if (!normalizer_enabled || !buf || bytes < 2)
        return;

    int16_t *s = (int16_t *)buf;
    size_t   n = bytes / sizeof(int16_t);

    /* ── Step 1: Compute RMS of this chunk ──────────────────────────────── */
    double sum_sq = 0.0;
    for (size_t i = 0; i < n; i++) {
        double v = s[i] * (1.0 / 32768.0);
        sum_sq += v * v;
    }
    float chunk_rms = (n > 0) ? (float)sqrt(sum_sq / (double)n) : 0.0f;

    /* ── Step 2: Update RMS estimate (gate: skip silence) ───────────────── */
    if (chunk_rms > GATE_THRESH) {
        float rms_coeff = (chunk_rms > rms_estimate) ? RMS_ATTACK : RMS_RELEASE;
        rms_estimate = rms_coeff * rms_estimate + (1.0f - rms_coeff) * chunk_rms;
    }

    /* ── Step 3: Compute desired gain and smooth toward it ──────────────── */
    float desired_gain;
    if (rms_estimate > GATE_THRESH) {
        desired_gain = TARGET_RMS / rms_estimate;
        if (desired_gain > MAX_GAIN)  desired_gain = MAX_GAIN;
        if (desired_gain < MIN_GAIN)  desired_gain = MIN_GAIN;
    } else {
        desired_gain = gain; /* silence: hold current gain */
    }

    float g_coeff = (desired_gain < gain) ? GAIN_ATTACK : GAIN_RELEASE;
    float g_start = gain;
    gain = g_coeff * gain + (1.0f - g_coeff) * desired_gain;
    float g_end = gain;

    /* ── Step 4: Apply linearly-interpolated gain across the chunk ──────────
     *    Ramping from g_start → g_end avoids a click at chunk boundaries
     *    when the gain changes rapidly (e.g., a loud track follows a quiet one). */
    float n_inv = (n > 1) ? (1.0f / (float)(n - 1)) : 0.0f;
    for (size_t i = 0; i < n; i++) {
        float g = g_start + (g_end - g_start) * ((float)i * n_inv);
        float v = (float)s[i] * g;
        if (v >  32767.0f) v =  32767.0f;
        if (v < -32768.0f) v = -32768.0f;
        s[i] = (int16_t)v;
    }
}
