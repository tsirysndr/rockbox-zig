/*
 * Rockbox tick / timer implementation for the headless macOS/Linux host target.
 *
 * kernel-unix.c uses POSIX interval timers (timer_create / timer_settime)
 * which are absent from macOS's SDK.  This replacement achieves identical
 * semantics with a dedicated pthread that calls nanosleep() in a loop.
 *
 * The firmware tick (HZ, default 100 Hz) drives call_tick_tasks() and
 * wakes the kernel scheduler via interrupt().  Plugin timers (timer_register
 * / timer_set_period / timer_unregister) use a second pthread.
 */

#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <errno.h>
#include <unistd.h>
#include <pthread.h>

#include "config.h"
#include "system.h"
#include "kernel.h"
#include "button.h"
#include "audio.h"
#include "panic.h"
#include "timer.h"

/* ── Scheduler sleep / wakeup ─────────────────────────────────────────────── */

static pthread_cond_t  wfi_cond = PTHREAD_COND_INITIALIZER;
static pthread_mutex_t wfi_mtx  = PTHREAD_MUTEX_INITIALIZER;

void wait_for_interrupt(void)
{
    pthread_cond_wait(&wfi_cond, &wfi_mtx);
}

void interrupt(void)
{
    pthread_cond_signal(&wfi_cond);
}

/* ── Firmware tick ─────────────────────────────────────────────────────────── */

static unsigned long tick_interval_ns;
static pthread_t tick_thread_tid;

static void *tick_thread(void *arg)
{
    (void)arg;
    struct timespec ts;
    ts.tv_sec  = 0;
    ts.tv_nsec = (long)tick_interval_ns;
    for (;;) {
        nanosleep(&ts, NULL);
        call_tick_tasks();
        interrupt();
    }
    return NULL;
}

void tick_start(unsigned int interval_in_ms)
{
    tick_interval_ns = (unsigned long)interval_in_ms * 1000000UL;
    pthread_mutex_lock(&wfi_mtx);
    int ret = pthread_create(&tick_thread_tid, NULL, tick_thread, NULL);
    if (ret != 0)
        panicf("tick_start(): pthread_create failed: %d\n", ret);
}

/* ── Plugin timers ────────────────────────────────────────────────────────── */

static pthread_t  ptimer_thread;
static pthread_t *ptimer_thread_p = NULL; /* non-NULL while timer is active */
static long       ptimer_interval_ns;
static int        timer_prio = -1;
void (*global_unreg_callback)(void);
void (*global_timer_callback)(void);

static void *ptimer_fn(void *arg)
{
    (void)arg;
    void (*cb)(void)  = global_timer_callback;
    long interval     = ptimer_interval_ns;
    struct timespec ts;
    ts.tv_sec  = 0;
    ts.tv_nsec = interval;
    for (;;) {
        nanosleep(&ts, NULL);
        /* Bail out if callback was unregistered while we were sleeping. */
        if (global_timer_callback != cb)
            break;
        cb();
    }
    return NULL;
}

#define cycles_to_microseconds(cycles) \
    ((long)(((long long)10000 * (cycles)) / (TIMER_FREQ / 100)))

bool timer_register(int reg_prio, void (*unregister_callback)(void),
                    long cycles, void (*timer_callback)(void))
{
    long in_us = cycles_to_microseconds(cycles);
    if (reg_prio <= timer_prio || in_us <= 0)
        return false;

    timer_unregister();

    ptimer_interval_ns   = in_us * 1000L;
    global_timer_callback = timer_callback;
    global_unreg_callback = unregister_callback;
    timer_prio            = reg_prio;

    if (pthread_create(&ptimer_thread, NULL, ptimer_fn, NULL) != 0)
        return false;

    ptimer_thread_p = &ptimer_thread;
    return true;
}

bool timer_set_period(long cycles)
{
    long in_us = cycles_to_microseconds(cycles);
    if (in_us <= 0)
        return false;
    /* The running thread reads ptimer_interval_ns without locking; on all
     * target architectures a 64-bit aligned store is atomic enough for a
     * timer whose accuracy is already limited to nanosleep granularity. */
    ptimer_interval_ns = in_us * 1000L;
    return true;
}

void timer_unregister(void)
{
    if (ptimer_thread_p) {
        /* Signal the thread to exit on its next wakeup by clearing the cb. */
        void (*saved_cb)(void) = global_timer_callback;
        global_timer_callback = NULL;
        if (global_unreg_callback)
            global_unreg_callback();
        /* Restore so caller can detect it was set before. */
        (void)saved_cb;
        pthread_join(ptimer_thread, NULL);
        ptimer_thread_p = NULL;
    }
    timer_prio            = -1;
    global_unreg_callback = NULL;
    global_timer_callback = NULL;
}
