/* SPDX-License-Identifier: GPL-2.0-or-later
 *
 * No-op audio hardware driver for the WASM build. All output routing is
 * handled by the Web Audio API (pcm-webapi.c). These stubs satisfy the
 * audiohw_* surface that firmware/sound.c references.
 *
 * audiohw-swcodec.c (always compiled) provides:
 *   audiohw_set_channel, audiohw_set_stereo_width → channel_mode DSP
 *   audiohw_set_bass, audiohw_set_treble          → tone_controls DSP (HAVE_SW_TONE_CONTROLS)
 *   audiohw_set_prescaler                         → tone_controls prescaler
 *
 * We define those symbols here only when the swcodec file does NOT, to avoid
 * duplicate-symbol issues under --allow-multiple-definition.
 */
#include "config.h"

void audiohw_init(void)    { }
void audiohw_preinit(void) { }
void audiohw_postinit(void){ }
void audiohw_close(void)   { }

void audiohw_set_volume(int vol_l, int vol_r) { (void)vol_l; (void)vol_r; }
void audiohw_set_lineout_volume(int vol_l, int vol_r) { (void)vol_l; (void)vol_r; }
void audiohw_set_balance(int val)             { (void)val; }
void audiohw_set_bass_cutoff(int val)         { (void)val; }
void audiohw_set_treble_cutoff(int val)       { (void)val; }
void audiohw_set_eq_band_gain(unsigned band, int val)       { (void)band; (void)val; }
void audiohw_set_eq_band_frequency(unsigned band, int val)  { (void)band; (void)val; }
void audiohw_set_eq_band_width(unsigned band, int val)      { (void)band; (void)val; }
void audiohw_set_filter_roll_off(int value)   { (void)value; }
void audiohw_set_depth_3d(int val)            { (void)val; }
void audiohw_set_loudness(int val)            { (void)val; }
void audiohw_mute(int mute)                   { (void)mute; }
void audiohw_set_frequency(int fsel)          { (void)fsel; }

/* audiohw-swcodec.c owns these when HAVE_SW_TONE_CONTROLS is defined. */
#ifndef HAVE_SW_TONE_CONTROLS
void audiohw_set_bass(int val)         { (void)val; }
void audiohw_set_treble(int val)       { (void)val; }
void audiohw_set_prescaler(int val)    { (void)val; }
void audiohw_set_channel(int val)      { (void)val; }
void audiohw_set_stereo_width(int val) { (void)val; }
#endif
