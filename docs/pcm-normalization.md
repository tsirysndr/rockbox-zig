# PCM Loudness Normalization

Rockbox implements a real-time PCM loudness normalizer that equalises the perceived volume across tracks and sources. It is similar in purpose to Spotify's "Normalize Volume" or Apple Music's "Sound Check", but operates at the raw audio buffer level rather than on pre-computed track metadata — so it works for any source including live streams, radio, and HTTP audio.

## Table of Contents

1. [Overview](#overview)
2. [How It Works](#how-it-works)
   - [Step 1 — RMS Measurement](#step-1--rms-measurement)
   - [Step 2 — Running RMS Estimate with Silence Gate](#step-2--running-rms-estimate-with-silence-gate)
   - [Step 3 — Gain Computation and Smoothing](#step-3--gain-computation-and-smoothing)
   - [Step 4 — Linear Gain Interpolation and Application](#step-4--linear-gain-interpolation-and-application)
3. [Asymmetric Attack / Release](#asymmetric-attack--release)
4. [Silence Gate](#silence-gate)
5. [Warm Start](#warm-start)
6. [Parameters Reference](#parameters-reference)
7. [Position in the Signal Chain](#position-in-the-signal-chain)
8. [Enabling the Normalizer](#enabling-the-normalizer)
9. [Comparison with ReplayGain](#comparison-with-replaygain)
10. [Known Limitations](#known-limitations)

---

## Overview

The normalizer targets a fixed RMS loudness level (−9 dBFS by default). Every PCM buffer that flows through a sink is analysed, a smoothed gain factor is computed, and the gain is applied in-place before the audio is written to the output device. The gain adjusts continuously and automatically — no track scanning, no metadata, no pre-processing required.

```
Decoded PCM  ──►  SW Volume scaling  ──►  Normalizer  ──►  Sink (SDL / FIFO / AirPlay / …)
                  (pcm_copy_buffer)        (pcm_normalizer_apply)
```

---

## How It Works

The algorithm runs once per PCM chunk. A "chunk" is the buffer delivered by the Rockbox audio engine to the DMA callback — typically 4 096–8 192 bytes (≈ 23–46 ms of stereo 44 100 Hz audio). The four steps below execute in order for every chunk.

### Step 1 — RMS Measurement

The Root Mean Square (RMS) amplitude of the current chunk is computed:

```
chunk_rms = sqrt( (1/N) × Σ (sᵢ / 32768)² )
```

where `sᵢ` are the raw S16LE sample values and dividing by 32 768 normalises them to the range `[−1, +1]`. RMS is used rather than peak amplitude because it correlates well with perceived loudness — a brief loud transient raises RMS only slightly, whereas sustained loud content raises it significantly.

The summation uses `double` precision to avoid accumulated rounding error over large chunk sizes.

### Step 2 — Running RMS Estimate with Silence Gate

A single chunk's RMS is noisy; averaging across many chunks gives a stable picture of the signal's loudness. A first-order Infinite Impulse Response (IIR) filter — also called an exponential moving average — is used:

```
rms_estimate = α × rms_estimate + (1 − α) × chunk_rms
```

The coefficient `α` controls how quickly the estimate tracks changes. Crucially, **two different coefficients** are used depending on the direction of change:

| Signal direction | Coefficient | Behaviour |
|---|---|---|
| `chunk_rms > rms_estimate` (getting louder) | `RMS_ATTACK = 0.3` | Tracks loud transients in 2–3 chunks (< 150 ms) |
| `chunk_rms < rms_estimate` (getting quieter) | `RMS_RELEASE = 0.99` | Takes ~7 s to settle on a quieter signal |

This asymmetry is essential. A fast attack means the estimate rises quickly when a loud section begins — preventing the normalizer from over-boosting and causing clipping. A slow release means the estimate falls slowly after a loud section ends — preventing the gain from shooting up during a brief quiet passage (the "pumping" or "breathing" artefact).

Chunks whose RMS falls below `GATE_THRESH` (−60 dBFS) are treated as silence: the RMS estimate and the gain are both held at their current values. This prevents the normalizer from amplifying the noise floor during pauses between tracks.

### Step 3 — Gain Computation and Smoothing

The desired gain is calculated as:

```
desired_gain = TARGET_RMS / rms_estimate
```

This is the factor that, if applied to the signal, would bring its estimated loudness to `TARGET_RMS`. The value is clamped to prevent extreme correction:

```
desired_gain = clamp(desired_gain, MIN_GAIN, MAX_GAIN)
               = clamp(desired_gain, 0.1, 10.0)       // −20 dB to +20 dB
```

The gain is not applied instantaneously — that would produce audible clicks at chunk boundaries whenever the gain changes significantly. Instead the applied gain `gain` moves toward `desired_gain` through another asymmetric IIR smoother:

```
gain = β × gain + (1 − β) × desired_gain
```

| Direction | Coefficient | Convergence |
|---|---|---|
| Gain decreasing (signal too loud) | `GAIN_ATTACK = 0.3` | Reaches target in ~3 chunks (< 150 ms) |
| Gain increasing (signal too quiet) | `GAIN_RELEASE = 0.98` | Reaches target in ~3 seconds |

The fast gain attack prevents over-shoot and clipping when a loud track suddenly follows a quiet one. The slow gain release prevents the loudness from rising abruptly during a quiet moment.

### Step 4 — Linear Gain Interpolation and Application

Applying a discontinuous gain at the start of each chunk would still produce a click if the gain changed significantly between chunks. The gain is therefore **linearly interpolated** from its value at the start of the chunk (`g_start`) to its new value at the end (`g_end`):

```
g(i) = g_start + (g_end − g_start) × (i / (N − 1))
```

Each sample is scaled by its per-sample gain and clamped to the S16 range to avoid integer overflow:

```c
float v = (float)s[i] * g(i);
v = clamp(v, -32768.0f, 32767.0f);
s[i] = (int16_t)v;
```

This ramp completely eliminates the inter-chunk click artefact, even at fast gain-attack rates.

---

## Asymmetric Attack / Release

The following diagram illustrates the asymmetric time constants on a hypothetical signal that starts quiet, becomes loud, and then returns to quiet:

```
RMS level
    │            ┌────────────────────────┐
    │            │  Loud section          │
    │            │                        │
    │  ──────────┘                        └─────────  Actual signal
    │         ↑ fast attack              ↑ slow release
    │
    │  ────────┐                              ┌────── rms_estimate
    │          └──────────────────────────────┘
    │
gain│  ────────┐                              ┌────── Applied gain
    │          └──────────────────────────────┘
    │          ↑ fast gain reduction     ↑ slow gain rise
    └──────────────────────────────────────────────── time
```

The fast attack on both the RMS estimator and the gain smoother means the normalizer reacts within ~150 ms when the signal becomes loud, preventing clipping. The slow release means it takes a few seconds to raise the gain again after a loud section, which avoids the pumping artefact that would otherwise be audible during the quiet parts of dynamic music or between tracks.

---

## Silence Gate

If `chunk_rms ≤ GATE_THRESH` (0.001 linear = −60 dBFS), the chunk is classified as silence and neither `rms_estimate` nor `gain` is updated. Without this gate, a pause between tracks would cause `rms_estimate` to decay toward zero, `desired_gain` to hit `MAX_GAIN`, and the next track to begin at full boost — creating a loud pop on playback resume.

The gate threshold of −60 dBFS is below any audible content but above the floating-point noise floor of a 16-bit signal.

---

## Warm Start

When the normalizer is first enabled (or re-enabled), state is reset to:

```c
gain         = 1.0f;   // no gain applied yet
rms_estimate = 0.1f;   // −20 dBFS: typical quiet-to-moderate music level
```

The `rms_estimate` warm-start at 0.1 (rather than at `TARGET_RMS`) means `desired_gain` starts above 1.0 for most content. This ensures the normalizer applies a boost from the very first chunk rather than waiting for the IIR filter to converge from the default. Without the warm start, the first several seconds of playback would sound un-normalised.

---

## Parameters Reference

All parameters are compile-time constants in `firmware/pcm_normalizer.c`.

| Constant | Value | dB equivalent | Description |
|---|---|---|---|
| `TARGET_RMS` | `0.35` | −9 dBFS | Target RMS loudness. Higher = louder output. |
| `RMS_ATTACK` | `0.3` | — | IIR coefficient for RMS rising (loud signal). Lower = faster. |
| `RMS_RELEASE` | `0.99` | — | IIR coefficient for RMS falling (quiet signal). Higher = slower. |
| `GAIN_ATTACK` | `0.3` | — | IIR coefficient for gain decreasing. Lower = faster. |
| `GAIN_RELEASE` | `0.98` | — | IIR coefficient for gain increasing. Higher = slower. |
| `MAX_GAIN` | `10.0` | +20 dB | Maximum boost applied to quiet tracks. |
| `MIN_GAIN` | `0.1` | −20 dB | Maximum cut applied to loud tracks. |
| `GATE_THRESH` | `0.001` | −60 dBFS | RMS below this → treat chunk as silence. |

### Choosing TARGET_RMS

`TARGET_RMS` is the most impactful parameter. A few reference points:

| Value | dBFS | Character |
|---|---|---|
| `0.071` | −23 dBFS | EBU R128 broadcast standard (very conservative) |
| `0.178` | −15 dBFS | Apple Music / AES streaming recommendation |
| `0.200` | −14 dBFS | Spotify / YouTube streaming target |
| `0.350` | −9 dBFS | **Current default** — loud and punchy |
| `0.500` | −6 dBFS | Very loud; risk of clipping on loud source material |

### Convergence Time Reference

IIR convergence depends on the chunk size. For a typical 4 096-byte chunk at 44 100 Hz stereo (46 ms per chunk):

| Parameter | Coefficient | ~Time to move 63% of the way to target |
|---|---|---|
| `RMS_ATTACK` | 0.3 | 1 chunk ≈ 46 ms |
| `RMS_RELEASE` | 0.99 | 100 chunks ≈ 4.6 s |
| `GAIN_ATTACK` | 0.3 | 1 chunk ≈ 46 ms |
| `GAIN_RELEASE` | 0.98 | 50 chunks ≈ 2.3 s |

Time constant τ = `−chunk_duration / ln(α)`. For `α = 0.98` and chunk = 46 ms: τ = −46 ms / ln(0.98) ≈ 2.3 s.

---

## Position in the Signal Chain

The normalizer runs **after** software volume scaling and **before** the audio is written to any output sink:

```
Rockbox audio engine
        │
        ▼ raw S16LE stereo PCM (read-only buffer from firmware)
┌───────────────────────┐
│  pcm_copy_buffer()    │  applies the user's SW volume setting
│  (pcm_sw_volume.c)    │  writes into a per-sink scratch buffer
└───────────────────────┘
        │
        ▼ volume-scaled PCM (writable scratch buffer)
┌───────────────────────┐
│ pcm_normalizer_apply()│  measures RMS, updates gain, applies in-place
│ (pcm_normalizer.c)    │
└───────────────────────┘
        │
        ▼ normalised PCM
┌───────────────────────┐
│    PCM sink            │  SDL / FIFO / AirPlay / Squeezelite /
│                        │  UPnP / Chromecast / Snapcast TCP
└───────────────────────┘
```

Because the normalizer runs after SW volume, it measures and targets the _post-volume-control_ signal level. If the user lowers the volume, the normalizer sees a quieter signal and raises its gain to compensate — partially offsetting the volume reduction. This is intentional: at any volume setting, loudness across tracks remains consistent.

---

## Enabling the Normalizer

Add to `~/.config/rockbox.org/settings.toml`:

```toml
normalize_volume = true
```

The setting is read at startup by `crates/settings/src/lib.rs` which calls `pcm_normalizer_enable(true)`. It is also persisted back to disk on `write_settings()` so the preference survives restarts.

The normalizer can also be toggled at runtime via the Rust FFI:

```rust
// crates/sys/src/sound/normalizer.rs
rockbox_sys::sound::normalizer::enable(true);
let on = rockbox_sys::sound::normalizer::is_enabled();
```

---

## Comparison with ReplayGain

Rockbox also supports ReplayGain, which is a pre-computed per-track gain stored in file tags. The two approaches are complementary:

| | ReplayGain | PCM Normalizer |
|---|---|---|
| **Requires track analysis** | Yes (offline scan) | No |
| **Works on streams / radio** | No | Yes |
| **Accuracy** | Very high (full-track analysis) | Moderate (real-time estimate) |
| **Artefacts** | None | Slight pumping on highly dynamic content |
| **Target** | Configurable per standard | `TARGET_RMS` compile constant |
| **Processing cost** | Zero at runtime | ~1–2% CPU (RMS + gain loop) |

For local music libraries, ReplayGain is generally preferred when tags are available. The PCM normalizer is the practical choice for streaming sources or when ReplayGain tags are missing.

Both can be active simultaneously. When ReplayGain is applied by the DSP engine (before the DMA stage), the PCM normalizer sees the already-normalised signal and applies only a small residual correction.

---

## Known Limitations

- **No lookahead.** The normalizer reacts to what has already been played. A sudden loud transient at the very start of a track will play at the previous gain for the first chunk (~46 ms) before the attack kicks in. In practice this is inaudible for most music.

- **RMS is not LUFS.** RMS amplitude correlates with perceived loudness but is not identical to the ITU-R BS.1770 Integrated Loudness (LUFS) metric used by broadcast standards. Content with heavy bass or aggressive dynamic compression may feel louder than its RMS suggests.

- **Chunk-size dependency.** The IIR coefficients produce the quoted time constants only at the assumed 46 ms chunk size. Smaller chunks (e.g., during low-latency mode) will make the attack and release feel slower in wall-clock time because more chunks are needed to advance the filter by the same amount. Chunk size is determined by the audio engine and is not directly configurable.

- **State is shared across tracks.** The `gain` and `rms_estimate` state variables are not reset between tracks. This is generally desirable — it prevents a jump in loudness at a track boundary — but means the normalizer's gain at the start of a new track reflects the previous track's loudness. Tracks that differ wildly in level may take a few seconds to settle.
