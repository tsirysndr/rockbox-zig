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
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>
#include <errno.h>
#include <stdbool.h>
#include <android/log.h>

#include "config.h"
#include "system.h"
#include "kernel.h"
#include "panic.h"
#include "debug.h"

#define TAG "rb-system-android"
#define LOGI(fmt, ...) __android_log_print(ANDROID_LOG_INFO,  TAG, fmt, ##__VA_ARGS__)
#define LOGE(fmt, ...) __android_log_print(ANDROID_LOG_ERROR, TAG, fmt, ##__VA_ARGS__)

/* ── Pipe stdout / stderr into logcat ─────────────────────────────────────
 *
 * Stock Rockbox C code uses raw printf() / fprintf(stderr, ...) for all the
 * `[metadata]`, `[streamfd]`, codec, buffering diagnostics. On Linux those
 * land on the controlling terminal. On Android stdout/stderr are wired to
 * /dev/null by Zygote so every print silently disappears.
 *
 * Solution: at process start, redirect both fds to the write end of a pipe
 * and spawn a reader thread that __android_log_writes each line. From
 * then on every printf in the firmware shows up under tag "Rockbox" in
 * logcat — same content the desktop terminal would have shown. */
static int  stdio_pipe[2] = { -1, -1 };
static pthread_t stdio_thread;

static void *stdio_logcat_reader(void *arg)
{
    (void)arg;
    char line[1024];
    size_t fill = 0;
    char buf[512];
    ssize_t n;
    while ((n = read(stdio_pipe[0], buf, sizeof(buf))) > 0) {
        for (ssize_t i = 0; i < n; ++i) {
            char c = buf[i];
            if (c == '\n' || fill + 1 >= sizeof(line)) {
                line[fill] = 0;
                if (fill > 0)
                    __android_log_write(ANDROID_LOG_INFO, "Rockbox", line);
                fill = 0;
                if (c != '\n') {
                    /* line was too long — keep this byte for the next line */
                    line[fill++] = c;
                }
            } else {
                line[fill++] = c;
            }
        }
    }
    if (fill > 0) {
        line[fill] = 0;
        __android_log_write(ANDROID_LOG_INFO, "Rockbox", line);
    }
    return NULL;
}

static void redirect_stdio_to_logcat(void)
{
    static bool installed = false;
    if (installed) return;

    /* line-buffer stdout, no buffering on stderr (matches POSIX terminal
     * behaviour so [metadata]…\n flushes promptly to logcat). */
    setvbuf(stdout, NULL, _IOLBF, 0);
    setvbuf(stderr, NULL, _IONBF, 0);

    if (pipe(stdio_pipe) != 0) {
        LOGE("redirect_stdio_to_logcat: pipe() failed: %s", strerror(errno));
        return;
    }
    if (dup2(stdio_pipe[1], STDOUT_FILENO) < 0
        || dup2(stdio_pipe[1], STDERR_FILENO) < 0) {
        LOGE("redirect_stdio_to_logcat: dup2 failed: %s", strerror(errno));
        return;
    }

    pthread_attr_t attr;
    pthread_attr_init(&attr);
    pthread_attr_setdetachstate(&attr, PTHREAD_CREATE_DETACHED);
    if (pthread_create(&stdio_thread, &attr, stdio_logcat_reader, NULL) != 0) {
        LOGE("redirect_stdio_to_logcat: pthread_create failed: %s", strerror(errno));
        pthread_attr_destroy(&attr);
        return;
    }
    pthread_attr_destroy(&attr);

    installed = true;
    LOGI("redirect_stdio_to_logcat: stdout+stderr now routed to logcat tag Rockbox");
}

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
    /* Wire stdout/stderr to logcat first so any prints emitted during the
     * rest of the boot path (codec init, kernel start, etc.) are captured. */
    redirect_stdio_to_logcat();

    int marker;
    stackbegin = stackend = (uintptr_t *)&marker;
    quitting = false;
    LOGI("system_init: headless cdylib boot");
}

/* ── Power off / reboot / exception wait ──────────────────────────────── */

void power_off(void)
{
    LOGI("power_off requested");
    pthread_mutex_lock(&shutdown_lock);
    quitting = true;
    pthread_cond_broadcast(&shutdown_cv);
    pthread_mutex_unlock(&shutdown_lock);
    /* Just exit the process — JNI's rb_daemon_stop pthread_joins the engine
     * thread and observes the exit. No SDL/sim cleanup needed. */
    exit(0);
}

void system_reboot(void)
{
    /* "Reboot" on Android = exit; JS layer calls rb_daemon_start to come back. */
    power_off();
}

void system_exception_wait(void)
{
    /* Block the calling thread until quitting is set. Used by the engine
     * after a fatal panicf() so we don't spin on the dead path. */
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
