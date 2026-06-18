/* SPDX-License-Identifier: GPL-2.0-or-later
 *
 * CMAF (HLS / DASH) sink stub for the Android cdylib build.
 *
 * pcm-cmaf.c depends on fdk-aac, which we do not cross-compile into
 * librockbox_expo.so — and serving HLS/DASH off a phone has no real
 * use case. crates/settings still references pcm_cmaf_* via crates/sys,
 * so without these stubs the cdylib fails to load with
 *   "cannot locate symbol pcm_cmaf_start".
 *
 * Selecting `audio_output = "cmaf"` is a no-op on Android; the daemon
 * stays on whatever sink was already active.
 */

#include <stdint.h>
#include <stddef.h>

void pcm_cmaf_set_http_port(uint16_t port)       { (void)port; }
void pcm_cmaf_set_bitrate(uint32_t bps)          { (void)bps; }
void pcm_cmaf_set_segment_dir(const char *path)  { (void)path; }
int  pcm_cmaf_start(void)                        { return 0; }
