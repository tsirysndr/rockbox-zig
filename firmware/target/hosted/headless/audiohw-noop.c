/* SPDX-License-Identifier: GPL-2.0-or-later
 *
 * No-op audio hardware driver for the headless host build.
 * Volume is handled by the OS audio mixer via cpal; the firmware does
 * not drive a codec chip directly on this target.
 */

void audiohw_init(void)                                       { }
void audiohw_preinit(void)                                    { }
void audiohw_postinit(void)                                   { }
void audiohw_close(void)                                      { }

void audiohw_set_volume(int vol_l, int vol_r)                 { (void)vol_l; (void)vol_r; }
void audiohw_set_lineout_volume(int vol_l, int vol_r)         { (void)vol_l; (void)vol_r; }
void audiohw_set_prescaler(int val)                           { (void)val; }
void audiohw_set_balance(int val)                             { (void)val; }
void audiohw_set_treble(int val)                              { (void)val; }
void audiohw_set_bass(int val)                                { (void)val; }
void audiohw_set_bass_cutoff(int val)                         { (void)val; }
void audiohw_set_treble_cutoff(int val)                       { (void)val; }
void audiohw_set_eq_band_gain(unsigned band, int val)         { (void)band; (void)val; }
void audiohw_set_eq_band_frequency(unsigned band, int val)    { (void)band; (void)val; }
void audiohw_set_eq_band_width(unsigned band, int val)        { (void)band; (void)val; }
void audiohw_set_filter_roll_off(int value)                   { (void)value; }
void audiohw_set_depth_3d(int val)                            { (void)val; }
void audiohw_set_loudness(int val)                            { (void)val; }
void audiohw_mute(int mute)                                   { (void)mute; }
void audiohw_set_frequency(int fsel)                          { (void)fsel; }
