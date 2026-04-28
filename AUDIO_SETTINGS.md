# Rockbox Audio Settings — Complete Reference

All settings are stored in `global_settings` (`apps/settings.h`) and persisted to `~/.config/rockbox.org/settings.toml`. Playback-state fields (pitch, speed, volume) are also saved there via `global_status`.

Hardware settings flow through `firmware/sound.c → sound_set_*() → codec driver`.
DSP settings flow through `lib/rbcodec/dsp/` and are applied in the PCM pipeline before samples reach the sink.

> This document covers the **SDL (sdlapp) hosted target**. Hardware-only settings (hardware EQ, DAC filter roll-off, DAC power mode, hardware bass/treble tone controls, 3D enhancement, output select) are not available on this target and are omitted.

---

## Volume

**Storage:** `global_status.volume`

Rockbox uses a decibel scale where **0 dB** is a reference indicating the maximum output level the device can produce without distortion (clipping). All values below 0 dB yield progressively softer output. Values above 0 dB boost beyond the normal maximum and will ordinarily cause audible distortion, but may be useful for recordings with a low inherent volume.

On the SDL target, volume is software-controlled via the SDL audio mixer.

---

## Volume Limit

**Storage:** `global_settings.volume_limit`

Sets a hard ceiling on the maximum volume anywhere in the system. By default this equals the device's hardware maximum. Select a value from the list to cap volume at that level.

---

## Channel / Stereo Controls

### Balance

**Storage:** `global_settings.balance` | Range: −100..+100

Controls the balance between the left and right channels. The default (**0**) means both outputs are equal in volume. Negative values increase the left channel relative to the right; positive values increase the right channel relative to the left.

### Channels

**Storage:** `global_settings.channel_config`

Determines how the left and right channels of a stereo signal are routed to the output.

| Value | Behaviour |
|---|---|
| **Stereo** | Leave the audio signal unmodified. |
| **Mono** | Combine both channels; send the result to both outputs (monophonic). |
| **Custom** | Apply the stereo width specified by the **Stereo Width** setting. |
| **Mono Left** | Play the left channel in both stereo channels. |
| **Mono Right** | Play the right channel in both stereo channels. |
| **Karaoke** | Remove all sound common to both channels. Since vocals are typically equally present in both channels, this often (but not always) removes the voice track. May have other undesirable effects. |
| **Swap Left & Right** | Play the left channel on the right output and vice versa. |

Applied via `sound_set_channels()` in `firmware/sound.c`.

### Stereo Width

**Storage:** `global_settings.stereo_width` | Range: 0..255 %

Active when **Channels** is set to **Custom**. Controls the width of the stereo image:

- **0 %** — fully mono.
- **< 100 %** — progressively mixes one channel into the other, narrowing the stereo image.
- **100 %** — stereo field unaltered.
- **> 100 %** — progressively removes components present in both channels from each individual channel, widening the stereo field.

Applied via `sound_set_stereo_width()` in `firmware/sound.c`.

---

## Crossfeed

**Storage:** `global_settings.crossfeed` and related fields

When listening to music on headphones, each ear hears only its corresponding stereo channel. This lacks the natural spatial cues present when listening to loudspeakers, where each ear hears both speakers — but with a slight time delay and level difference for the farther speaker. The absence of these cues can make headphone listening sound unnatural, and is especially noticeable with older rock and jazz recordings where instruments are hard-panned to one side.

Crossfeed uses an algorithm to feed a delayed and filtered portion of the right channel into the left and vice versa, simulating loudspeaker spatial cues. The result is a more natural stereo image.

> **Warning:** crossfeed can cause output distortion if its settings result in a combined level that is too high.

| Setting | Storage field | Range | Default | Description |
|---|---|---|---|---|
| **Type** | `global_settings.crossfeed` | off, meier, custom | off | `meier` uses fixed sensible defaults; `custom` exposes the four parameters below |
| **Direct Gain** | `global_settings.crossfeed_direct_gain` | −60..0 dB (step 5) | −15 dB | How much to decrease the level of the signal travelling the direct path from a speaker to the corresponding ear |
| **Cross Gain** | `global_settings.crossfeed_cross_gain` | −120..−30 dB (step 5) | −60 dB | How much to decrease the level of the signal travelling the cross path from a speaker to the opposite ear |
| **HF Attenuation** | `global_settings.crossfeed_hf_attenuation` | −240..−60 dB (step 5) | −160 dB | How much the upper frequencies of the cross-path signal are dampened. Total high-frequency level is a combination of this setting and Cross Gain |
| **HF Cutoff** | `global_settings.crossfeed_hf_cutoff` | 500..2000 Hz (step 100) | 700 Hz | Frequency at which the cross-path signal begins to be cut by the HF Attenuation amount |

Applied via `dsp_set_crossfeed_type()` and `dsp_set_crossfeed_cross_params()` in `lib/rbcodec/dsp/crossfeed.h`.

---

## Equalizer

**Storage:** `global_settings.eq_enabled`, `global_settings.eq_precut`, `global_settings.eq_band_settings[]`

Rockbox features a **parametric EQ** — unlike non-parametric (graphic) equalizers, a parametric EQ allows independent adjustment of centre frequency, gain, and bandwidth for each band. This provides precise control with fewer bands than would otherwise be needed.

> **Note:** using more bands than necessary wastes battery and introduces additional rounding noise. Use the fewest number of bands required.

### EQ bands

| Band | Filter type | Default centre / cutoff | Q recommendation |
|---|---|---|---|
| Band 0 | Low shelf filter | 32 Hz | 0.7 (higher values add an unwanted boost near cutoff) |
| Bands 1–8 | Peaking (bell) filters | 64 / 125 / 250 / 500 / 1k / 2k / 4k / 8k Hz | Higher Q = narrower range; lower Q = wider range |
| Band 9 | High shelf filter | 16 000 Hz | 0.7 |

**Band parameters (per band):**
- **Cutoff / Centre frequency** — where the shelving starts (shelf bands) or the centre of the affected range (peak bands).
- **Gain** — positive numbers boost, negative numbers cut. Unit: dB.
- **Q** — controls bandwidth for peak filters (higher = narrower). Should be 0.7 for shelf filters.

### EQ sub-settings

| Setting | Storage field | Type | Description |
|---|---|---|---|
| **Enable EQ** | `global_settings.eq_enabled` | bool | Master on/off switch for the software EQ |
| **Precut** | `global_settings.eq_precut` | 0..24.0 dB | Global negative gain applied to decoded audio before the EQ. Prevents distortion when boosting bands. Can also be used as a volume cap. Not applied when EQ is disabled |
| **Graphical EQ** | — | screen | Graphical interface for adjusting gain, centre frequency, and Q for each band |
| **Simple EQ** | — | screen | Simplified view: only gain is adjustable per band |
| **Advanced EQ** | — | submenu | Same parameters as Graphical EQ, via text menus |
| **Save EQ Preset** | — | action | Saves current EQ configuration to a `.cfg` file |
| **Browse EQ Presets** | — | screen | Lists built-in presets and any saved configurations |

Applied via `dsp_set_eq_precut()` and `dsp_set_eq_coefs()` in `lib/rbcodec/dsp/eq.h`.

---

## ReplayGain

**Storage:** `global_settings.replaygain_settings`

| Setting | Storage field | Values / Range | Default | Description |
|---|---|---|---|---|
| **Type** | `.type` | track, album, track shuffle, off | shuffle | Which RG tag to read for normalization |
| **No-Clip** | `.noclip` | bool | false | Scale down if RG adjustment would cause clipping |
| **Preamp** | `.preamp` | −120..+120 dB (step 5) | 0 dB | Additional gain applied on top of the RG value |

Applied via `dsp_replaygain_set_settings()` in `lib/rbcodec/dsp/dsp_misc.h`.

---

## Crossfade

**Storage:** `global_settings.crossfade` and related fields | Condition: `HAVE_CROSSFADE`

Overlaps the end of one track with the beginning of the next.

| Setting | Storage field | Range | Default | Description |
|---|---|---|---|---|
| **Mode** | `global_settings.crossfade` | off, auto track change, manual skip, shuffle, shuffle+manual, always | off | When crossfading is triggered |
| **Fade-In Delay** | `global_settings.crossfade_fade_in_delay` | 0..7 s | 0 s | Silence before the fade-in begins |
| **Fade-Out Delay** | `global_settings.crossfade_fade_out_delay` | 0..7 s | 0 s | Silence before the fade-out begins |
| **Fade-In Duration** | `global_settings.crossfade_fade_in_duration` | 0..15 s | 2 s | Length of the fade-in ramp |
| **Fade-Out Duration** | `global_settings.crossfade_fade_out_duration` | 0..15 s | 2 s | Length of the fade-out ramp |
| **Fade-Out Mode** | `global_settings.crossfade_fade_out_mixmode` | crossfade, mix | crossfade | Whether the outgoing track fades out or mixes at a flat level |

---

## Dithering

**Storage:** `global_settings.dithering_enabled`

Most Rockbox audio decoders work at a higher bit depth than the 16 bits used for output. The simplest approach is to discard the surplus bits, which adds time-varying distortion correlated with the desired signal.

**Dithering** adds low-level noise to the signal *before* discarding the surplus bits. The result is a uniform, signal-independent noise floor that most listeners find preferable to the time-varying distortion of simple truncation.

**Noise shaping** is applied after dithering: it redistributes the dithering noise toward frequencies humans hear less easily — in Rockbox's case, above ~10 kHz.

This setting is most beneficial with dynamic material that has frequent quiet passages (e.g. classical music). The effect is subtle and not easily noticeable.

- Dithering algorithm: **highpass triangular distribution (HPTPDF)**
- Noise shaper order: **third-order**

Applied via `dsp_dither_enable()` in `lib/rbcodec/dsp/dsp_misc.h`.

---

## Pitch & Time-Stretch

Condition: `HAVE_PITCHCONTROL`. Playback state is persisted in `global_status`.

Enabling Timestretch allows playback speed to be changed independently of pitch. Intended primarily for speech playback — may noticeably degrade the listening experience with complex music.

| Setting | Storage field | Range | Default | Description |
|---|---|---|---|---|
| **Pitch** | `global_status.resume_pitch` | ~50..200 % | 100 % | Pitch shift without changing tempo |
| **Speed** | `global_status.resume_speed` | ~35..250 % | 100 % | Playback speed without changing pitch |
| **Timestretch Enable** | `global_settings.timestretch_enabled` | bool | false | Enables the TDHS time-domain algorithm; accessible via the Pitch Screen after reboot |

Applied via `sound_set_pitch()` and `dsp_timestretch_enable()` in `lib/rbcodec/dsp/tdspeed.h`.

---

## Output Sample Rate

**Storage:** `global_settings.play_frequency` | Condition: `HAVE_PLAY_FREQ`

| Setting | Values | Description |
|---|---|---|
| **Play Frequency** | auto, 44.1 kHz, 48 kHz, 88.2 kHz, 96 kHz | Output sample rate. `auto` matches the source file's native rate |

---

## Haas Surround

**Storage:** `global_settings.surround_enabled` and related fields

Implements the **Haas effect** with an adjustable delay time to enhance the stereo image. A full-range Haas effect creates the impression that sound starts from one channel and ends in the other. Four additional controls move the perceived stage back toward the centre:

| Setting | Storage field | Range | Default | Description |
|---|---|---|---|---|
| **Enable** | `global_settings.surround_enabled` | 0, 5, 8, 10, 15, 30 ms | 0 (off) | Delay time for the Haas effect; 0 disables it |
| **Balance** | `global_settings.surround_balance` | 0..99 % | 35 % | Left/right channel output ratio to re-centre the stage |
| **f(x1) — HF Cutoff** | `global_settings.surround_fx1` | 600..8000 Hz (step 200) | 3400 Hz | Upper boundary of a bypass band for frequencies (mostly vocals) that are not affected by the surround processing |
| **f(x2) — LF Cutoff** | `global_settings.surround_fx2` | 40..400 Hz (step 40) | 320 Hz | Lower boundary of the bypass band |
| **Side Only** | `global_settings.surround_method2` | bool | false | Uses mid-side processing to apply the effect to the side channel only, leaving the centre image unmodified |
| **Dry/Wet Mix** | `global_settings.surround_mix` | 0..100 % (step 5) | 50 % | Proportion of original (dry) vs effected (wet) signal in the final output |

Applied via `dsp_surround_enable()` and related functions in `lib/rbcodec/dsp/surround.h`.

---

## Perceptual Bass Enhancement (PBE)

**Storage:** `global_settings.pbe`, `global_settings.pbe_precut`

Implements a group delay correction and an additional biophonic EQ to boost bass perception.

| Setting | Storage field | Range | Default | Description |
|---|---|---|---|---|
| **PBE** | `global_settings.pbe` | 0..100 % (step 25) | 0 % (off) | Strength of the bass enhancement effect |
| **PBE Precut** | `global_settings.pbe_precut` | −4.5..0 dB (step 0.1) | −2.5 dB | Negative overall gain applied to prevent audio distortion caused by the EQ gain. Stacks with any other EQ applied |

Applied via `dsp_pbe_enable()` and `dsp_pbe_precut()` in `lib/rbcodec/dsp/pbe.h`.

---

## Auditory Fatigue Reduction (AFR)

**Storage:** `global_settings.afr_enabled`

Human hearing is more sensitive to certain frequency bands. AFR applies additional equalization and bi-shelf filtering to reduce energy in those bands, minimising the risk of temporary threshold shift (auditory fatigue) during extended listening sessions.

| Setting | Values | Default |
|---|---|---|
| **AFR Enable** | off, weak, moderate, strong | off |

Applied via `dsp_afr_enable()` in `lib/rbcodec/dsp/afr.h`.

---

## Compressor

**Storage:** `global_settings.compressor_settings`

The compressor reduces the dynamic range of the audio signal by progressively reducing the gain of louder signals. When the compressed signal is subsequently amplified (via makeup gain), the quiet sections become louder while the loud sections stay below clipping. This is useful for listening to dynamic material in noisy environments.

| Setting | Storage field | Values / Range | Default | Description |
|---|---|---|---|---|
| **Threshold** | `.threshold` | off, −3, −6, −9, −12, −15, −18, −21, −24 dB | off | Input level above which compression begins. The maximum compression (minimum operating level) is −24 dB |
| **Makeup Gain** | `.makeup_gain` | off, auto | auto | **Off:** no re-amplification after compression. **Auto:** amplifies so the loudest post-compression signal is just below clipping, restoring perceived loudness |
| **Ratio** | `.ratio` | 2:1, 4:1, 6:1, 10:1, limit | 2:1 | For every N dB above threshold, the output rises by only 1 dB. **Limit** = ∞:1 — the output cannot exceed the threshold at all |
| **Knee** | `.knee` | hard, soft | soft | **Hard knee:** transition occurs precisely at the threshold. **Soft knee:** transition is smoothed over ±3 dB around the threshold |
| **Attack Time** | `.attack_time` | 0..30 ms (step 5) | 5 ms | Delay between the input signal exceeding the threshold and the compressor acting on it |
| **Release Time** | `.release_time` | 100..1000 ms (step 100) | 500 ms | Time for the gain to recover by 10 dB after the signal drops below the threshold. A longer release time reduces "pumping" artefacts |

Applied via `dsp_set_compressor()` in `lib/rbcodec/dsp/compressor.h`.

---

## UI Audio Feedback

| Setting | Storage field | Values | Default | Description |
|---|---|---|---|---|
| **Beep** | `global_settings.beep` | off, weak, moderate, strong | off | Audible tone on track change or key events |
| **Keyclick** | `global_settings.keyclick` | off, weak, moderate, strong | off | Audio click on every key press |
| **Keyclick Repeats** | `global_settings.keyclick_repeats` | bool | false | Whether held keys also produce a click |

---

## Summary

| Category | Setting count |
|---|---|
| Volume & limit | 2 |
| Channel / stereo | 3 |
| Crossfeed | 5 |
| Software EQ (10 bands) | 12 |
| ReplayGain | 3 |
| Crossfade | 6 |
| Dithering | 1 |
| Pitch / Time-Stretch | 3 |
| Output sample rate | 1 |
| Haas Surround | 6 |
| PBE | 2 |
| AFR | 1 |
| Compressor | 6 |
| UI audio feedback | 3 |
| **Total** | **~54** |
