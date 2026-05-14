/* SPDX-License-Identifier: GPL-2.0-or-later
 *
 * C-level JSON helpers for the WASM build. These let the Rust wasm crate
 * retrieve firmware state as JSON without needing to know the mp3entry or
 * playlist_info struct layouts.
 *
 * All returned strings are heap-allocated (malloc). Callers must free them
 * via rb_free_string() (implemented in crates/wasm).
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <stdbool.h>
#include <time.h>
#include <unistd.h>
#include <pthread.h>
#include <emscripten.h>

#include "config.h"
#include "audio.h"
#include "playlist.h"
#include "metadata.h"
#include "rtc.h"
#include "kernel.h"
#include "thread.h"
#include "settings.h"
#include "misc.h"
#include "sound.h"
#include "eq.h"
#include "pcmbuf.h"
#include "audio_thread.h"
#include "channel_mode.h"
#include "surround.h"
#include "crossfeed.h"
#include "tone_controls.h"
#include "afr.h"
#include "pbe.h"
#include "tdspeed.h"
#include "dsp_misc.h"

/* Declared here; defined in pcm-webapi.c. */
extern void rb_pcm_set_balance(int balance);
extern void rb_pcm_flush(void);

/* Escape a string for JSON: replace " → \" and \ → \\. */
static void json_escape(char *dst, size_t dsz, const char *src)
{
    size_t j = 0;
    for (size_t i = 0; src && src[i] && j + 2 < dsz; i++) {
        char c = src[i];
        if (c == '"' || c == '\\') {
            if (j + 3 >= dsz) break;
            dst[j++] = '\\';
        }
        dst[j++] = c;
    }
    dst[j] = '\0';
}

/* ── Track / playlist info cache ────────────────────────────────────────────
 *
 * audio_current_track() acquires id3_mutex, a Rockbox blocking mutex that
 * internally calls switch_thread() when contended.  switch_thread() reads
 * __cores[0].running (a global, not TLS) and expects to be called from a
 * registered Rockbox kernel thread.  The Emscripten main JS thread is NOT
 * a Rockbox kernel thread, so calling audio_current_track() from it
 * corrupts __cores[0].running and/or unlocks g_mutex (which the main thread
 * never held), causing "memory access out of bounds" in audio workers.
 *
 * Fix: rb_wasm_cmd_thread (a proper Rockbox kernel thread) refreshes the
 * cache every ~100 ms and after each command.  The main thread reads the
 * cache using pthread_mutex_trylock — no Rockbox scheduler involvement,
 * no blocking.
 */

#define TRACK_JSON_BUFSZ    1200
#define PLAYLIST_JSON_BUFSZ   64
#define SETTINGS_JSON_BUFSZ 3072

static char            s_track_json[TRACK_JSON_BUFSZ]       = "{\"error\":\"not ready\"}";
static char            s_playlist_json[PLAYLIST_JSON_BUFSZ]  = "{\"index\":0,\"amount\":0}";
static char            s_settings_json[SETTINGS_JSON_BUFSZ]  = "{}";
static pthread_mutex_t s_info_mtx = PTHREAD_MUTEX_INITIALIZER;

/* Build settings JSON into buf (must be SETTINGS_JSON_BUFSZ bytes). */
static void build_settings_json(char *buf, size_t sz)
{
    int n = snprintf(buf, sz,
        "{\"eq\":{\"enabled\":%s,\"precut\":%u,\"bands\":[",
        global_settings.eq_enabled ? "true" : "false",
        global_settings.eq_precut);

    for (int i = 0; i < EQ_NUM_BANDS && n < (int)sz - 2; i++) {
        n += snprintf(buf + n, sz - n,
            "%s{\"cutoff\":%d,\"q\":%d,\"gain\":%d}",
            i > 0 ? "," : "",
            global_settings.eq_band_settings[i].cutoff,
            global_settings.eq_band_settings[i].q,
            global_settings.eq_band_settings[i].gain);
    }
    n += snprintf(buf + n, sz - n, "]}");

#ifdef HAVE_CROSSFADE
    n += snprintf(buf + n, sz - n,
        ",\"crossfade\":{\"mode\":%d,\"fade_in_delay\":%d,\"fade_out_delay\":%d,"
        "\"fade_in_duration\":%d,\"fade_out_duration\":%d,\"mixmode\":%d}",
        global_settings.crossfade,
        global_settings.crossfade_fade_in_delay,
        global_settings.crossfade_fade_out_delay,
        global_settings.crossfade_fade_in_duration,
        global_settings.crossfade_fade_out_duration,
        global_settings.crossfade_fade_out_mixmode);
#endif

    n += snprintf(buf + n, sz - n,
        ",\"replaygain\":{\"noclip\":%s,\"type\":%d,\"preamp\":%d}",
        global_settings.replaygain_settings.noclip ? "true" : "false",
        global_settings.replaygain_settings.type,
        global_settings.replaygain_settings.preamp);

    n += snprintf(buf + n, sz - n,
        ",\"balance\":%d,\"channel_mode\":%d,\"stereo_width\":%d",
        global_settings.balance,
        global_settings.channel_config,
        global_settings.stereo_width);

    n += snprintf(buf + n, sz - n,
        ",\"crossfeed\":{\"type\":%d,\"direct_gain\":%d"
        ",\"cross_gain\":%d,\"hf_attenuation\":%d,\"hf_cutoff\":%d}",
        global_settings.crossfeed,
        global_settings.crossfeed_direct_gain,
        global_settings.crossfeed_cross_gain,
        global_settings.crossfeed_hf_attenuation,
        global_settings.crossfeed_hf_cutoff);

    n += snprintf(buf + n, sz - n,
        ",\"surround\":{\"enabled\":%d,\"balance\":%d"
        ",\"fx1\":%d,\"fx2\":%d,\"method2\":%s,\"mix\":%d}",
        global_settings.surround_enabled,
        global_settings.surround_balance,
        global_settings.surround_fx1,
        global_settings.surround_fx2,
        global_settings.surround_method2 ? "true" : "false",
        global_settings.surround_mix);

    n += snprintf(buf + n, sz - n,
        ",\"bass\":%d,\"treble\":%d"
        ",\"dithering\":%s,\"afr\":%d"
        ",\"pbe\":%d,\"pbe_precut\":%d"
        ",\"timestretch\":%s"
        ",\"repeat\":%d}",
        global_settings.bass,
        global_settings.treble,
        global_settings.dithering_enabled ? "true" : "false",
        global_settings.afr_enabled,
        global_settings.pbe,
        global_settings.pbe_precut,
        global_settings.timestretch_enabled ? "true" : "false",
        global_settings.repeat_mode);
    (void)n;
}


/* Called only from rb_wasm_cmd_thread — a proper Rockbox kernel thread that
 * may safely acquire id3_mutex via the cooperative scheduler. */
static void rb_wasm_refresh_cache(void)
{
    /* ── track ── */
    char track_buf[TRACK_JSON_BUFSZ];
    unsigned long elapsed = 0;
    struct mp3entry *e = audio_current_track();
    if (!e) {
        snprintf(track_buf, sizeof(track_buf), "{\"error\":\"no track\"}");
    } else {
        elapsed = (unsigned long)e->elapsed;
        char title[256]  = "";
        char artist[256] = "";
        char album[256]  = "";
        char path[512]   = "";
        if (e->title)  json_escape(title,  sizeof(title),  e->title);
        if (e->artist) json_escape(artist, sizeof(artist), e->artist);
        if (e->album)  json_escape(album,  sizeof(album),  e->album);
        json_escape(path, sizeof(path), e->path);
        snprintf(track_buf, sizeof(track_buf),
            "{\"title\":\"%s\",\"artist\":\"%s\",\"album\":\"%s\","
            "\"path\":\"%s\",\"duration_ms\":%lu,\"elapsed_ms\":%lu}",
            title, artist, album, path,
            (unsigned long)e->length, elapsed);
    }

    /* ── playlist (compact) ── */
    char playlist_buf[PLAYLIST_JSON_BUFSZ];
    snprintf(playlist_buf, sizeof(playlist_buf),
             "{\"index\":%d,\"amount\":%d}",
             playlist_get_display_index(), playlist_amount());

    /* ── settings ── */
    char settings_buf[SETTINGS_JSON_BUFSZ];
    build_settings_json(settings_buf, sizeof(settings_buf));

    /* ── atomic store ── */
    pthread_mutex_lock(&s_info_mtx);
    memcpy(s_track_json,    track_buf,    sizeof(track_buf));
    memcpy(s_playlist_json, playlist_buf, sizeof(playlist_buf));
    memcpy(s_settings_json, settings_buf, sizeof(settings_buf));
    pthread_mutex_unlock(&s_info_mtx);
}

/* Read functions — safe from the Emscripten main JS thread.
 * pthread_mutex_trylock never blocks; if the cache is being refreshed the
 * caller gets a "busy" placeholder and retries on the next poll cycle. */

char *rb_wasm_current_track_json(void)
{
    if (pthread_mutex_trylock(&s_info_mtx) != 0) {
        char *r = malloc(32);
        if (r) strcpy(r, "{\"error\":\"busy\"}");
        return r;
    }
    char *r = malloc(strlen(s_track_json) + 1);
    if (r) strcpy(r, s_track_json);
    pthread_mutex_unlock(&s_info_mtx);
    return r;
}

char *rb_wasm_playlist_json(void)
{
    if (pthread_mutex_trylock(&s_info_mtx) != 0) {
        char *r = malloc(32);
        if (r) strcpy(r, "{\"index\":0,\"amount\":0}");
        return r;
    }
    char *r = malloc(strlen(s_playlist_json) + 1);
    if (r) strcpy(r, s_playlist_json);
    pthread_mutex_unlock(&s_info_mtx);
    return r;
}

char *rb_wasm_settings_json(void)
{
    if (pthread_mutex_trylock(&s_info_mtx) != 0) {
        char *r = malloc(4);
        if (r) strcpy(r, "{}");
        return r;
    }
    char *r = malloc(strlen(s_settings_json) + 1);
    if (r) strcpy(r, s_settings_json);
    pthread_mutex_unlock(&s_info_mtx);
    return r;
}

char *rb_wasm_playlist_state_json(void)
{
    static const char stub[] = "{\"urls\":[],\"index\":0,\"elapsed\":0,\"amount\":0}";
    char *r = malloc(sizeof(stub));
    if (r) memcpy(r, stub, sizeof(stub));
    return r;
}

/* audio_status() bitmask → {0=stopped, 1=playing, 2=paused} */
int rb_wasm_audio_status(void)
{
    int s = audio_status();
    /* AUDIO_STATUS_PLAY=1, AUDIO_STATUS_PAUSE=2 */
    if (s & 1) return (s & 2) ? 2 : 1;
    return 0;
}

/* ── Kernel lock — no-op for WASM ────────────────────────────────────────── */

/* thread-posix.c provides mutex-based rb_kernel_lock/unlock for headless
 * builds, but Emscripten's main JS thread cannot block on pthread_mutex_lock
 * (it throws an "unwind" exception).  The firmware's worker threads run
 * Rockbox's cooperative scheduler independently; JS FFI calls are
 * short-latency reads that tolerate occasional torn reads. */
void rb_kernel_lock(void)   { /* no-op on WASM main thread */ }
void rb_kernel_unlock(void) { /* no-op on WASM main thread */ }

/* ── Firmware ready flag ─────────────────────────────────────────────────── */

/* Set to 1 by the firmware once the audio subsystem is up.  The Rust crate
 * polls this before reporting daemon-state == 2 to JS. */
static volatile int rb_firmware_ready_flag = 0;

/* ── WASM command dispatcher ─────────────────────────────────────────────── */
/*
 * audio_next/prev/pause/resume/stop/seek all go through id3_mutex, a
 * Rockbox blocking mutex whose lock path calls switch_thread().
 * switch_thread() reads __running_self_entry() = __cores[0].running, which
 * is only valid inside a proper Rockbox kernel thread.  Calling these
 * functions from the Emscripten main JS thread (which has no Rockbox thread
 * entry) corrupts the scheduler and causes "memory access out of bounds" in
 * audio worker threads.
 *
 * Fix: a tiny dedicated Rockbox thread ("wasm_cmd") receives commands via a
 * kernel event queue and calls the real audio functions from that safe
 * context.  queue_post() uses only corelock (a spinlock) plus a condvar
 * signal — no switch_thread(), safe to call from any OS thread including the
 * Emscripten main thread.
 */

#define WASM_CMD_NEXT            0L
#define WASM_CMD_PREV            1L
#define WASM_CMD_PAUSE           2L
#define WASM_CMD_RESUME          3L
#define WASM_CMD_STOP            4L
#define WASM_CMD_SEEK            5L
#define WASM_CMD_PLAY_AT         6L  /* data = playlist index (0-based) */
#define WASM_CMD_PLAY_URL        7L  /* data = malloc'd NUL-terminated URL; thread frees */
#define WASM_CMD_ENQUEUE_URL     8L  /* data = malloc'd NUL-terminated URL; thread frees */
#define WASM_CMD_CLEAR_QUEUE     9L
#define WASM_CMD_SHUFFLE         10L
#define WASM_CMD_SET_EQ_ENABLED  11L /* data = 0 or 1 */
#define WASM_CMD_SET_EQ_PRECUT   12L /* data = precut value (0..240) */
#define WASM_CMD_SET_EQ_BAND     13L /* data = malloc'd wasm_eq_band_cmd_t*; thread frees */
#define WASM_CMD_SET_CROSSFADE   14L /* data = malloc'd wasm_crossfade_cmd_t*; thread frees */
#define WASM_CMD_SET_REPLAYGAIN  15L /* data = malloc'd wasm_replaygain_cmd_t*; thread frees */
#define WASM_CMD_SAVE_SETTINGS   16L /* calls settings_save() */
#define WASM_CMD_SET_BALANCE     17L /* data = int -100..100 */
#define WASM_CMD_SET_CHANNEL_MODE 18L /* data = int 0..6 */
#define WASM_CMD_SET_STEREO_WIDTH 19L /* data = int 0..250 (percent) */
#define WASM_CMD_SET_CROSSFEED   20L /* data = malloc'd wasm_crossfeed_dsp_cmd_t*; thread frees */
#define WASM_CMD_SET_SURROUND    21L /* data = malloc'd wasm_surround_cmd_t*; thread frees */
#define WASM_CMD_SET_BASS        22L /* data = int (whole dB) */
#define WASM_CMD_SET_TREBLE      23L /* data = int (whole dB) */
#define WASM_CMD_SET_DITHERING   24L /* data = 0 or 1 */
#define WASM_CMD_SET_AFR         25L /* data = int (0=off, 1-3=mode) */
#define WASM_CMD_SET_PBE         26L /* data = malloc'd wasm_pbe_cmd_t*; thread frees */
#define WASM_CMD_SET_TIMESTRETCH 27L /* data = int (0=off, else stretch% * PITCH_SPEED_PRECISION) */
#define WASM_CMD_SET_REPEAT 28L

/* Payload structs for complex settings commands. */
typedef struct { int band; int cutoff; int q; int gain; } wasm_eq_band_cmd_t;
typedef struct {
    int mode; int fi_delay; int fo_delay; int fi_dur; int fo_dur; int mixmode;
} wasm_crossfade_cmd_t;
typedef struct { int noclip; int type; int preamp; } wasm_replaygain_cmd_t;
typedef struct {
    int type_; int direct_gain; int cross_gain; int hf_attenuation; int hf_cutoff;
} wasm_crossfeed_dsp_cmd_t;
typedef struct {
    int enabled; int balance; int fx1; int fx2; int method2; int mix;
} wasm_surround_cmd_t;
typedef struct { int pbe; int precut; } wasm_pbe_cmd_t;

static struct event_queue rb_wasm_cmd_q;
static long               rb_wasm_cmd_stack[8192 / sizeof(long)];

static void rb_wasm_cmd_thread(void)
{
    struct queue_event ev;
    for (;;) {
        /* Wake every ~100 ms even when idle so elapsed_ms stays current. */
        queue_wait_w_tmo(&rb_wasm_cmd_q, &ev, HZ / 10);
        if (ev.id != SYS_TIMEOUT) {
            switch (ev.id) {
            case WASM_CMD_NEXT:    audio_next();                              break;
            case WASM_CMD_PREV:    audio_prev();                              break;
            case WASM_CMD_PAUSE:   audio_pause();                             break;
            case WASM_CMD_RESUME:  audio_resume();                            break;
            case WASM_CMD_STOP:    audio_hard_stop();                         break;
            case WASM_CMD_SEEK: {
                /* audio_ff_rewind() triggers codec_seek_buffer_callback which
                 * calls bufseek().  When the seek target is outside the
                 * currently-buffered window bufseek posts Q_REBUFFER_HANDLE
                 * (blocking) to the buffering thread, which then walks the
                 * HTTP stream chunk-by-chunk to the new position.  For seeks
                 * deep into a file this takes several seconds.
                 *
                 * Fix: use playlist_start() with a byte offset calculated from
                 * the elapsed/duration ratio.  The buffering layer starts
                 * fetching from that byte offset directly, so the codec never
                 * needs to seek through the buffer — it just decodes from
                 * the new position. */
                long elapsed_ms = (long)ev.data;
                int cur_idx = playlist_get_display_index() - 1;
                if (cur_idx < 0) cur_idx = 0;

                unsigned long byte_offset = 0;
                struct mp3entry *id3 = audio_current_track();
                if (id3 && id3->length > 0 && id3->filesize > 0) {
                    byte_offset = (unsigned long)(
                        (uint64_t)elapsed_ms * (uint64_t)id3->filesize
                        / (uint64_t)id3->length
                    );
                }

                playlist_start(cur_idx, (unsigned long)elapsed_ms, byte_offset);
                rb_pcm_flush();
                break;
            }
            case WASM_CMD_PLAY_AT:
                playlist_start((int)ev.data, 0UL, 0UL);
                break;
            case WASM_CMD_PLAY_URL: {
                char *url = (char *)(uintptr_t)ev.data;
                playlist_remove_all_tracks(playlist_get_current());
                playlist_insert_track(playlist_get_current(), url,
                                      PLAYLIST_INSERT_LAST, false, false);
                playlist_start(0, 0UL, 0UL);
                free(url);
                break;
            }
            case WASM_CMD_ENQUEUE_URL: {
                char *url = (char *)(uintptr_t)ev.data;
                playlist_insert_track(playlist_get_current(), url,
                                      PLAYLIST_INSERT_LAST, false, false);
                free(url);
                break;
            }
            case WASM_CMD_CLEAR_QUEUE:
                playlist_remove_all_tracks(playlist_get_current());
                break;
            case WASM_CMD_SHUFFLE:
                playlist_shuffle(0, 0);
                break;
            case WASM_CMD_SET_EQ_ENABLED: {
                global_settings.eq_enabled = (bool)(int)ev.data;
                dsp_eq_enable(global_settings.eq_enabled);
                break;
            }
            case WASM_CMD_SET_EQ_PRECUT: {
                global_settings.eq_precut = (unsigned int)(int)ev.data;
                dsp_set_eq_precut((int)global_settings.eq_precut);
                break;
            }
            case WASM_CMD_SET_EQ_BAND: {
                wasm_eq_band_cmd_t *cmd =
                    (wasm_eq_band_cmd_t *)(uintptr_t)ev.data;
                if (cmd && cmd->band >= 0 && cmd->band < EQ_NUM_BANDS) {
                    global_settings.eq_band_settings[cmd->band].cutoff = cmd->cutoff;
                    global_settings.eq_band_settings[cmd->band].q      = cmd->q;
                    global_settings.eq_band_settings[cmd->band].gain   = cmd->gain;
                    dsp_set_eq_coefs(cmd->band,
                                     &global_settings.eq_band_settings[cmd->band]);
                }
                free(cmd);
                break;
            }
            case WASM_CMD_SET_CROSSFADE: {
                wasm_crossfade_cmd_t *cmd =
                    (wasm_crossfade_cmd_t *)(uintptr_t)ev.data;
#ifdef HAVE_CROSSFADE
                if (cmd) {
                    global_settings.crossfade                   = cmd->mode;
                    global_settings.crossfade_fade_in_delay     = cmd->fi_delay;
                    global_settings.crossfade_fade_out_delay    = cmd->fo_delay;
                    global_settings.crossfade_fade_in_duration  = cmd->fi_dur;
                    global_settings.crossfade_fade_out_duration = cmd->fo_dur;
                    global_settings.crossfade_fade_out_mixmode  = cmd->mixmode;
                }
                /* Register the new crossfade mode.  audio_set_crossfade()
                 * calls audio_queue_send() (blocking) which deadlocks when
                 * called while playback is stopped (audio thread never replies).
                 *
                 * Strategy:
                 * - Always call pcmbuf_request_crossfade_enable() so crossfade_enable_request
                 *   is up to date.
                 * - When PLAYING: post REMAKE so the audio thread calls pcmbuf_play_stop +
                 *   pcmbuf_init with the new request, taking effect immediately.
                 * - When STOPPED: do NOT post REMAKE.  audio_fill_file_buffer() checks
                 *   pcmbuf_is_same_size() when the next track starts and calls
                 *   audio_reset_buffer() → pcmbuf_init() naturally.
                 *   Calling pcmbuf_is_same_size() from this thread is avoided because it
                 *   modifies crossfade_setting (via pcmbuf_finish_crossfade_enable) without
                 *   holding the audio-thread lock, creating a race. */
                pcmbuf_request_crossfade_enable(
                    cmd ? cmd->mode : CROSSFADE_ENABLE_OFF);
                if (audio_status() & AUDIO_STATUS_PLAY) {
                    audio_queue_post(Q_AUDIO_REMAKE_AUDIO_BUFFER, 0);
                }
#endif
                free(cmd);
                break;
            }
            case WASM_CMD_SET_REPLAYGAIN: {
                wasm_replaygain_cmd_t *cmd =
                    (wasm_replaygain_cmd_t *)(uintptr_t)ev.data;
                if (cmd) {
                    global_settings.replaygain_settings.noclip = (bool)cmd->noclip;
                    global_settings.replaygain_settings.type   = cmd->type;
                    global_settings.replaygain_settings.preamp = cmd->preamp;
                }
                free(cmd);
                replaygain_update();
                break;
            }
            case WASM_CMD_SAVE_SETTINGS:
                settings_save();
                break;
            case WASM_CMD_SET_BALANCE:
                global_settings.balance = (int)ev.data;
                sound_set_balance(global_settings.balance);
                /* sound_set_balance() → audiohw_set_volume() is a no-op in
                 * the WASM build.  rb_pcm_set_balance() applies the balance
                 * at the ring_push() level; change takes effect on the next
                 * pcmbuf chunk (~46 ms) with no gap or cut. */
                rb_pcm_set_balance(global_settings.balance);
                break;
            case WASM_CMD_SET_CHANNEL_MODE:
                global_settings.channel_config = (int)ev.data;
                channel_mode_set_config(global_settings.channel_config);
                break;
            case WASM_CMD_SET_STEREO_WIDTH:
                global_settings.stereo_width = (int)ev.data;
                channel_mode_custom_set_width(global_settings.stereo_width);
                break;
            case WASM_CMD_SET_CROSSFEED: {
                wasm_crossfeed_dsp_cmd_t *cmd =
                    (wasm_crossfeed_dsp_cmd_t *)(uintptr_t)ev.data;
                if (cmd) {
                    global_settings.crossfeed               = cmd->type_;
                    global_settings.crossfeed_direct_gain   = cmd->direct_gain;
                    global_settings.crossfeed_cross_gain    = cmd->cross_gain;
                    global_settings.crossfeed_hf_attenuation = cmd->hf_attenuation;
                    global_settings.crossfeed_hf_cutoff     = cmd->hf_cutoff;
                    dsp_set_crossfeed_type(cmd->type_);
                    dsp_set_crossfeed_direct_gain(cmd->direct_gain);
                    dsp_set_crossfeed_cross_params(cmd->cross_gain,
                                                   cmd->hf_attenuation,
                                                   cmd->hf_cutoff);
                }
                free(cmd);
                break;
            }
            case WASM_CMD_SET_SURROUND: {
                wasm_surround_cmd_t *cmd =
                    (wasm_surround_cmd_t *)(uintptr_t)ev.data;
                if (cmd) {
                    global_settings.surround_enabled = cmd->enabled;
                    global_settings.surround_balance = cmd->balance;
                    global_settings.surround_fx1     = cmd->fx1;
                    global_settings.surround_fx2     = cmd->fx2;
                    global_settings.surround_method2 = (bool)cmd->method2;
                    global_settings.surround_mix     = cmd->mix;
                    dsp_surround_set_balance(cmd->balance);
                    dsp_surround_set_cutoff(cmd->fx1, cmd->fx2);
                    dsp_surround_side_only((bool)cmd->method2);
                    dsp_surround_mix(cmd->mix);
                    dsp_surround_enable(cmd->enabled);
                }
                free(cmd);
                break;
            }
            case WASM_CMD_SET_BASS:
                global_settings.bass = (int)ev.data;
                /* sound_set_bass() calls audiohw_set_bass (→ tone_set_bass) AND
                 * set_prescaled_volume (→ tone_set_prescale → update_filter +
                 * dsp_proc_enable).  Direct tone_set_bass() alone never triggers
                 * update_filter, so the DSP proc would stay off. */
                sound_set_bass(global_settings.bass);
                break;
            case WASM_CMD_SET_TREBLE:
                global_settings.treble = (int)ev.data;
                sound_set_treble(global_settings.treble);
                break;
            case WASM_CMD_SET_DITHERING:
                global_settings.dithering_enabled = (bool)(int)ev.data;
                dsp_dither_enable(global_settings.dithering_enabled);
                break;
            case WASM_CMD_SET_AFR:
                global_settings.afr_enabled = (int)ev.data;
                dsp_afr_enable(global_settings.afr_enabled);
                break;
            case WASM_CMD_SET_PBE: {
                wasm_pbe_cmd_t *cmd = (wasm_pbe_cmd_t *)(uintptr_t)ev.data;
                if (cmd) {
                    global_settings.pbe        = cmd->pbe;
                    global_settings.pbe_precut = cmd->precut;
                    dsp_pbe_precut(cmd->precut);
                    dsp_pbe_enable(cmd->pbe);
                }
                free(cmd);
                break;
            }
            case WASM_CMD_SET_TIMESTRETCH:
                if ((int)ev.data == 0) {
                    global_settings.timestretch_enabled = false;
                    dsp_timestretch_enable(false);
                } else {
                    global_settings.timestretch_enabled = true;
                    dsp_timestretch_enable(true);
                    dsp_set_timestretch((int32_t)ev.data);
                }
                break;
            case WASM_CMD_SET_REPEAT:
                global_settings.repeat_mode = (int)ev.data;
                if (audio_status() & AUDIO_STATUS_PLAY)
                    audio_flush_and_reload_tracks();
                break;
            }
        }

        rb_wasm_refresh_cache();
    }
}

static void rb_wasm_cmd_init(void)
{
    queue_init(&rb_wasm_cmd_q, true);
    create_thread(rb_wasm_cmd_thread,
                  rb_wasm_cmd_stack, sizeof(rb_wasm_cmd_stack),
                  0, "wasm_cmd"
                  IF_PRIO(, PRIORITY_BACKGROUND)
                  IF_COP(, CPU));
}

void rb_wasm_cmd_post(long id, intptr_t data)
{
    queue_post(&rb_wasm_cmd_q, id, data);
}

void rb_signal_firmware_ready(void)
{
    /* settings_apply() already applied all loaded settings (including EQ).
     * pcmbuf uses a 1 s base buffer (WASM-specific MIN_BUFFER_SIZE in
     * pcmbuf.c) so EQ/DSP changes are audible in ~1 s without disabling
     * crossfade (low_latency_mode is never set). */
    rb_wasm_cmd_init();
    rb_firmware_ready_flag = 1;
}

int rb_is_firmware_ready(void)
{
    return rb_firmware_ready_flag;
}

/* ── RTC ─────────────────────────────────────────────────────────────────── */

int rtc_read_datetime(struct tm *tm)
{
    time_t t = time(NULL);
    struct tm *lt = localtime(&t);
    if (lt) *tm = *lt;
    return lt ? 0 : -1;
}
int rtc_write_datetime(const struct tm *tm) { (void)tm; return -1; }

/* ── HTTP streaming via Range requests ───────────────────────────────────── */
/*
 * Synchronous XHR is allowed in Web Workers (pthreads) but not on the main
 * JS thread.  The Rockbox buffering thread runs as a pthread, so blocking
 * inside a Range request is fine.
 *
 * Instead of downloading the full file upfront, we fetch CHUNK_SIZE-aligned
 * 256 KB slices on demand.  rb_net_open fetches only the first chunk, so
 * playback can begin as soon as that arrives.  Subsequent reads fetch the
 * next aligned chunk when the decoder advances past the cached window.
 * rb_net_lseek is purely positional; the fetch happens lazily at the next
 * rb_net_read call.
 *
 * Servers must honour "Range: bytes=START-END" (HTTP 206).  Servers that
 * return 200 (no Range support) still work — we extract the correct slice
 * from the full response body — but every chunk request re-downloads the
 * entire file over the network.
 *
 * File sizes above 2 GB are not supported (range offsets are int32_t).
 */

#define CHUNK_SIZE     (256 * 1024)
#define NET_HANDLE_MAX 8

typedef struct {
    char     url[2048];
    char     content_type[128];
    int64_t  total_size; /* from Content-Range or Content-Length; -1 = unknown */
    int64_t  pos;        /* current logical read position */
    uint8_t *chunk_buf;  /* CHUNK_SIZE bytes, malloc'd on open */
    int64_t  chunk_off;  /* file offset of chunk_buf[0]; -1 = invalid */
    int64_t  chunk_len;  /* valid bytes in chunk_buf */
    bool     used;
} wasm_stream_t;

static wasm_stream_t   g_net[NET_HANDLE_MAX];
static pthread_mutex_t g_net_mtx = PTHREAD_MUTEX_INITIALIZER;

/*
 * Synchronous HEAD — returns total Content-Length, or -1 on error.
 * Content-Length is a CORS-safe header, always readable without
 * Access-Control-Expose-Headers, so this works even when Content-Range
 * is blocked.  Called once per rb_net_open to get the true file size.
 */
EM_JS(int32_t, wasm_xhr_head_size, (const char *url_c), {
    var url = UTF8ToString(url_c);
    var xhr = new XMLHttpRequest();
    xhr.open('HEAD', url, false);
    try { xhr.send(); } catch(e) { return -1; }
    if (xhr.status < 200 || xhr.status >= 300) return -1;
    var cl = xhr.getResponseHeader('Content-Length');
    return cl ? parseInt(cl, 10) : -1;
});

/*
 * Synchronous Range GET — must be called from a pthread (Web Worker).
 *
 * Fetches [range_start, range_end] into the pre-allocated *buf.
 * range_end < 0 means open-ended ("bytes=START-").
 * *out_total receives the total file size from Content-Range; -1 if unknown.
 * ct_buf/ct_n are optional; receive Content-Type without parameters.
 *
 * Returns bytes written to buf, or < 0 on error:
 *   -1  network / send error
 *   -2  HTTP status not 200 or 206
 */
EM_JS(int32_t, wasm_xhr_range, (
    const char *url_c,
    int32_t range_start, int32_t range_end,
    void *buf, int32_t buf_size,
    int32_t *out_total,
    char *ct_buf, int32_t ct_n
), {
    var url = UTF8ToString(url_c);
    var xhr = new XMLHttpRequest();
    xhr.open('GET', url, false);
    var hdr = 'bytes=' + range_start + '-';
    if (range_end >= 0) hdr += range_end;
    xhr.setRequestHeader('Range', hdr);
    xhr.responseType = 'arraybuffer';
    try { xhr.send(null); } catch(e) { console.error('[net] xhr send error', e); return -1; }
    if (xhr.status !== 200 && xhr.status !== 206) {
        console.error('[net] Range request failed status=' + xhr.status + ' url=' + url.substring(0, 80));
        return -2;
    }

    var total = -1;
    if (xhr.status === 206) {
        /* Content-Range: bytes START-END/TOTAL — extract after the last '/' */
        var cr = xhr.getResponseHeader('Content-Range');
        if (cr) {
            var slash = cr.lastIndexOf('/');
            if (slash >= 0) total = parseInt(cr.substring(slash + 1), 10);
        }
    }
    if (total < 0) {
        var cl = xhr.getResponseHeader('Content-Length');
        if (cl) total = parseInt(cl, 10);
    }
    /* out_total and ct_buf are pointers into the static g_net[] array, which
     * lives at a low (pre-growth) address — HEAP32/HEAPU8 cover them fine. */
    if (out_total) HEAP32[out_total >> 2] = total;

    if (ct_buf && ct_n > 0) {
        var ct = xhr.getResponseHeader('Content-Type') || '';
        var semi = ct.indexOf(';');
        if (semi >= 0) ct = ct.slice(0, semi).trim();
        stringToUTF8(ct, ct_buf, ct_n);
    }

    /* For 200 responses (server ignored Range), skip to range_start in body */
    var body_start = (xhr.status === 200 && range_start > 0) ? range_start : 0;
    var ab        = xhr.response;
    var available = ab.byteLength - body_start;
    if (available < 0) available = 0;
    var n = Math.min(available, buf_size);
    /* buf is a malloc'd pointer that may lie in grown heap — create a fresh
     * Uint8Array view from the raw SharedArrayBuffer to avoid stale bounds. */
    if (n > 0) (new Uint8Array(HEAPU8.buffer)).set(new Uint8Array(ab, body_start, n), buf);
    return n;
});

/*
 * Ensure the chunk cache covers byte [pos].
 * Aligns the fetch to CHUNK_SIZE boundaries.
 * Returns 0 on success, -1 on fetch error.
 */
static int net_fetch_chunk(wasm_stream_t *s, int64_t pos)
{
    if (s->chunk_off >= 0 &&
        pos >= s->chunk_off &&
        pos <  s->chunk_off + s->chunk_len)
        return 0; /* cache hit */

    int64_t off = (pos / CHUNK_SIZE) * CHUNK_SIZE;
    int64_t end = off + CHUNK_SIZE - 1;
    if (s->total_size > 0 && end >= s->total_size)
        end = s->total_size - 1;

    int32_t total = -1;
    int32_t n = wasm_xhr_range(s->url,
                                (int32_t)off, (int32_t)end,
                                s->chunk_buf, CHUNK_SIZE,
                                &total, NULL, 0);
    if (n < 0) return -1;

    if (total > 0 && s->total_size < 0)
        s->total_size = (int64_t)total;

    s->chunk_off = off;
    s->chunk_len = (int64_t)n;
    return 0;
}

int rb_net_open(const char *url)
{
    if (!url) return -1;

    pthread_mutex_lock(&g_net_mtx);
    int h = -1;
    for (int i = 0; i < NET_HANDLE_MAX; i++) {
        if (!g_net[i].used) { h = i; break; }
    }
    if (h < 0) { pthread_mutex_unlock(&g_net_mtx); return -1; }

    wasm_stream_t *s = &g_net[h];
    memset(s, 0, sizeof(*s));
    s->chunk_off  = -1;
    s->total_size = -1;
    s->used       = true;
    pthread_mutex_unlock(&g_net_mtx);

    s->chunk_buf = malloc(CHUNK_SIZE);
    if (!s->chunk_buf) { s->used = false; return -1; }
    snprintf(s->url, sizeof(s->url), "%s", url);

    /* HEAD request: Content-Length is a CORS-safe header, always readable.
     * Use it as the authoritative total_size so we don't depend on
     * Content-Range (which browsers block without Access-Control-Expose-Headers). */
    int32_t head_size = wasm_xhr_head_size(url);
    s->total_size = head_size > 0 ? (int64_t)head_size : -1;

    /* Fetch first chunk; content_type comes from this response */
    int32_t ignored_total = -1;
    int32_t n = wasm_xhr_range(url,
                                0, CHUNK_SIZE - 1,
                                s->chunk_buf, CHUNK_SIZE,
                                &ignored_total,
                                s->content_type,
                                (int32_t)sizeof(s->content_type));
    if (n < 0) {
        free(s->chunk_buf);
        s->chunk_buf = NULL;
        s->used      = false;
        return -1;
    }

    /* If HEAD didn't give us a size (server doesn't support HEAD or no
     * Content-Length), fall back to whatever the Range response provided. */
    if (s->total_size < 0 && ignored_total > 0)
        s->total_size = (int64_t)ignored_total;

    s->chunk_off  = 0;
    s->chunk_len  = (int64_t)n;
    s->pos        = 0;
    return h;
}

int64_t rb_net_read(int32_t h, void *buf, size_t count)
{
    if (h < 0 || h >= NET_HANDLE_MAX || !g_net[h].used) return -1;
    wasm_stream_t *s = &g_net[h];

    if (s->total_size >= 0 && s->pos >= s->total_size) return 0;

    int64_t  total_read = 0;
    uint8_t *out        = (uint8_t *)buf;

    while (count > 0) {
        if (s->total_size >= 0 && s->pos >= s->total_size) break;

        if (net_fetch_chunk(s, s->pos) < 0) break;

        int64_t in_chunk = s->pos - s->chunk_off;
        int64_t avail    = s->chunk_len - in_chunk;
        if (avail <= 0) break;

        int64_t n = ((int64_t)count < avail) ? (int64_t)count : avail;
        memcpy(out, s->chunk_buf + in_chunk, (size_t)n);
        s->pos     += n;
        out        += n;
        total_read += n;
        count      -= (size_t)n;
    }

    return total_read;
}

int64_t rb_net_lseek(int32_t h, int64_t offset, int32_t whence)
{
    if (h < 0 || h >= NET_HANDLE_MAX || !g_net[h].used) return -1;
    wasm_stream_t *s = &g_net[h];

    int64_t np;
    if      (whence == SEEK_SET) np = offset;
    else if (whence == SEEK_CUR) np = s->pos + offset;
    else if (whence == SEEK_END) {
        if (s->total_size < 0) return -1;
        np = s->total_size + offset;
    }
    else return -1;

    if (np < 0) return -1;
    if (s->total_size >= 0 && np > s->total_size) return -1;

    s->pos = np;
    return s->pos;
}

void rb_net_close(int32_t h)
{
    if (h < 0 || h >= NET_HANDLE_MAX || !g_net[h].used) return;
    free(g_net[h].chunk_buf);
    g_net[h].chunk_buf = NULL;
    g_net[h].used      = false;
}

int64_t rb_net_len(int32_t h)
{
    if (h < 0 || h >= NET_HANDLE_MAX || !g_net[h].used) return -1;
    return g_net[h].total_size;
}

int64_t rb_net_content_type(int32_t h, char *dst, size_t n)
{
    if (h < 0 || h >= NET_HANDLE_MAX || !g_net[h].used || !dst) return -1;
    snprintf(dst, n, "%s", g_net[h].content_type);
    return (int64_t)strlen(g_net[h].content_type);
}
