/* SPDX-License-Identifier: GPL-2.0-or-later
 *
 * Headless system shim for the Android cdylib build of Rockbox. Provides
 * the same surface as firmware/target/hosted/sdl/system-sdl.c using
 * pthreads + clock_gettime + plain POSIX I/O. No SDL, no LCD, no event
 * loop, no button polling.
 *
 * Lifecycle is owned by JNI: rb_daemon_start (in crates/expo) calls
 * system_init() once after spawning the engine thread; rb_daemon_stop
 * calls sys_poweroff().
 */

#include <pthread.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>
#include <errno.h>
#include <stdbool.h>
#include <android/log.h>

#include "system.h"
#include "kernel.h"
#include "panic.h"
#include "debug.h"

#define TAG "rb-system-android"
#define LOGI(fmt, ...) __android_log_print(ANDROID_LOG_INFO,  TAG, fmt, ##__VA_ARGS__)
#define LOGE(fmt, ...) __android_log_print(ANDROID_LOG_ERROR, TAG, fmt, ##__VA_ARGS__)

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
    LOGI("system_init: headless cdylib boot");
}

/* ── Power off / reboot / exception wait ──────────────────────────────── */

void sdl_sys_quit(void)   /* kept for symbol compatibility with apps/ callers */
{
    quitting = true;
    sys_poweroff();
}

void power_off(void)
{
    LOGI("power_off requested");
    pthread_mutex_lock(&shutdown_lock);
    quitting = true;
    pthread_cond_broadcast(&shutdown_cv);
    pthread_mutex_unlock(&shutdown_lock);
    sim_do_exit();
}

void sim_do_exit(void)
{
    sim_kernel_shutdown();
}

void system_reboot(void)
{
    sim_thread_exception_wait();
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
    /* No CLI on Android — JNI passes config through env vars set by
     * rb_daemon_start before the engine thread spawns. */
    (void)argc; (void)argv;
}
