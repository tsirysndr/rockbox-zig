/* SPDX-License-Identifier: GPL-2.0-or-later
 *
 * Headless system shim for the WASM build of Rockbox. Provides the same
 * surface as system-android.c / system-headless.c but targets Emscripten.
 * No SDL, no LCD, no event loop, no button polling.
 *
 * Lifecycle: rb_daemon_start (crates/wasm) calls main_c() in a pthread
 * spawned by Emscripten. system_init() runs once at firmware boot.
 */

#include <pthread.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>
#include <stdbool.h>
#include <sys/stat.h>

#include <emscripten.h>

#include "config.h"
#include "system.h"
#include "kernel.h"
#include "panic.h"
#include "debug.h"

/* ── Globals required by the kernel stack accounting ──────────────────── */

uintptr_t *stackbegin;
uintptr_t *stackend;

/* ── Lifecycle state ──────────────────────────────────────────────────── */

static volatile bool quitting;
static pthread_mutex_t shutdown_lock = PTHREAD_MUTEX_INITIALIZER;
static pthread_cond_t  shutdown_cv   = PTHREAD_COND_INITIALIZER;

/* ── system_init ──────────────────────────────────────────────────────── */

void system_init(void)
{
    int marker;
    stackbegin = stackend = (uintptr_t *)&marker;
    quitting = false;

    /* Line-buffer stdout so firmware diagnostic prints flush promptly to the
     * browser console via Emscripten's stdout→console.log shim. */
    setvbuf(stdout, NULL, _IOLBF, 0);
    setvbuf(stderr, NULL, _IONBF, 0);

    /* Ensure HOME is visible to the C library's getenv() so that
     * paths_init() (called next) can resolve ROCKBOX_DIR paths.
     * Rust's std::env::set_var uses a separate env table on Emscripten
     * that the C library cannot see. */
    if (!getenv("HOME") || getenv("HOME")[0] == '\0')
        setenv("HOME", "/config", 1);

    EM_ASM({ console.log("[Rockbox] system_init: WASM boot"); });
}

/* ── Power off / reboot / exception wait ──────────────────────────────── */

void power_off(void)
{
    EM_ASM({ console.log("[Rockbox] power_off requested"); });
    pthread_mutex_lock(&shutdown_lock);
    quitting = true;
    pthread_cond_broadcast(&shutdown_cv);
    pthread_mutex_unlock(&shutdown_lock);
    /* Exit the pthread — JS layer observes via rb_daemon_state(). */
    pthread_exit(NULL);
}

void system_reboot(void)
{
    power_off();
}

void system_exception_wait(void)
{
    pthread_mutex_lock(&shutdown_lock);
    while (!quitting)
        pthread_cond_wait(&shutdown_cv, &shutdown_lock);
    pthread_mutex_unlock(&shutdown_lock);
    system_reboot();
}

/* ── Hosted filesystem hooks ──────────────────────────────────────────── */

int hostfs_init(void)
{
    /* Create the /.rockbox directory tree in MEMFS. */
    mkdir("/.rockbox",        0755);
    mkdir("/.rockbox/codecs", 0755);
    mkdir("/.rockbox/rocks",  0755);
    mkdir("/.rockbox/eqs",    0755);
    mkdir("/.rockbox/themes", 0755);
    mkdir("/.rockbox/fonts",  0755);
    mkdir("/music",           0755);

    /* filesystem-app.c:handle_special_dirs() rewrites "/.rockbox/..." paths to
     * "$HOME/.config/rockbox.org/..." at runtime.  Create each directory in
     * the chain: $HOME itself may not exist in Emscripten MEMFS. */
    const char *home = getenv("HOME");
    if (!home || home[0] == '\0')
        home = "/config"; /* fallback matching rb_daemon_start default */

    /* Create $HOME and every .config/rockbox.org sub-tree under it.
     * In Emscripten MEMFS only "/" exists at startup so we must
     * explicitly mkdir each component. */
    char buf[256];
    mkdir(home, 0755); /* e.g. /config */

    snprintf(buf, sizeof(buf), "%s/.config", home);
    mkdir(buf, 0755);
    snprintf(buf, sizeof(buf), "%s/.config/rockbox.org", home);
    mkdir(buf, 0755);
    snprintf(buf, sizeof(buf), "%s/.config/rockbox.org/rocks.data", home);
    mkdir(buf, 0755);
    return 0;
}

#ifdef HAVE_STORAGE_FLUSH
int hostfs_flush(void)
{
    return 0;
}
#endif

void sys_handle_argv(int argc, char *argv[])
{
    (void)argc; (void)argv;
}
