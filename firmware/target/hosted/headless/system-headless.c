/* SPDX-License-Identifier: GPL-2.0-or-later
 *
 * Headless system shim for the macOS / Linux headless build of Rockbox.
 * Same surface as firmware/target/hosted/sdl/system-sdl.c, implemented
 * with pthreads + plain POSIX I/O. No SDL, no event loop, no button polling.
 *
 * Lifecycle is owned by the Rust cli entry point: start() calls main_c()
 * which calls system_init(); SIGTERM/SIGINT are caught by Rust which calls
 * exit(0) directly, bypassing power_off().
 */

#include <pthread.h>
#include <signal.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

#include "config.h"
#include "system.h"
#include "kernel.h"
#include "panic.h"
#include "debug.h"

/* ── Globals required by the kernel stack accounting ─────────────────────── */

uintptr_t *stackbegin;
uintptr_t *stackend;

/* ── Lifecycle state ──────────────────────────────────────────────────────── */

static volatile bool quitting;
static pthread_mutex_t shutdown_lock = PTHREAD_MUTEX_INITIALIZER;
static pthread_cond_t  shutdown_cv   = PTHREAD_COND_INITIALIZER;

/* ── system_init ──────────────────────────────────────────────────────────── */

void system_init(void)
{
    /* Line-buffer stdout; no buffering on stderr so prints flush immediately. */
    setvbuf(stdout, NULL, _IOLBF, 0);
    setvbuf(stderr, NULL, _IONBF, 0);

    int marker;
    stackbegin = stackend = (uintptr_t *)&marker;
    quitting = false;
}

/* ── Power off / reboot / exception wait ─────────────────────────────────── */

void power_off(void)
{
    pthread_mutex_lock(&shutdown_lock);
    quitting = true;
    pthread_cond_broadcast(&shutdown_cv);
    pthread_mutex_unlock(&shutdown_lock);
    exit(0);
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

/* ── Hosted filesystem hooks ─────────────────────────────────────────────── */

int hostfs_init(void)
{
    return 0;
}

#ifdef HAVE_STORAGE_FLUSH
int hostfs_flush(void)
{
    sync();
    return 0;
}
#endif

void sys_handle_argv(int argc, char *argv[])
{
    (void)argc; (void)argv;
}
