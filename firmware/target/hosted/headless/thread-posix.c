/* firmware/target/hosted/headless/thread-posix.c
 *
 * Cooperative Rockbox thread scheduler using POSIX pthreads.
 * Replaces the HAVE_SIGALTSTACK_THREADS backend for the headless host.
 *
 * One OS pthread is created per Rockbox thread.  A single global mutex
 * (g_mutex) enforces cooperative execution: only one Rockbox thread holds
 * the mutex at a time.  All blocking/sleeping releases g_mutex and waits on
 * a per-thread counting semaphore, then re-acquires g_mutex on wake-up.
 *
 * Why this exists: macOS 12 Monterey x86_64 returns EPERM from sigaltstack()
 * inside make_context(), causing an infinite retry loop.  This backend avoids
 * that syscall entirely and works identically on Monterey x86_64, Sequoia
 * arm64, and Linux.
 */

#include <stdlib.h>
#include <stdio.h>
#include <setjmp.h>
#include <time.h>
#include <errno.h>
#include <pthread.h>

/* Pull in struct thread_entry, thread_alloc/free, wait_queue_*, block_thread,
 * THREAD_ID_SLOT, MAXTHREADS, __running_self_entry, and the rest of the
 * kernel scheduler interface.  The path "../kernel-internal.h" resolves to
 * firmware/kernel/kernel-internal.h via -I$(FIRMDIR)/kernel/include. */
#include "../kernel-internal.h"
#include "core_alloc.h"
#include "panic.h"
#include "debug.h"

/* ── Counting semaphore ──────────────────────────────────────────────────────
 *
 * sem_init() is deprecated on macOS; named semaphores need boilerplate.
 * A mutex+condvar pair is portable and avoids both issues.
 */
typedef struct {
    pthread_mutex_t mtx;
    pthread_cond_t  cond;
    int             count;
} posix_sem_t;

static posix_sem_t *psem_create(int initial)
{
    posix_sem_t *s = malloc(sizeof(posix_sem_t));
    if (!s) return NULL;
    pthread_mutex_init(&s->mtx, NULL);
    pthread_cond_init(&s->cond, NULL);
    s->count = initial;
    return s;
}

static void psem_destroy(posix_sem_t *s)
{
    pthread_cond_destroy(&s->cond);
    pthread_mutex_destroy(&s->mtx);
    free(s);
}

static void psem_wait(posix_sem_t *s)
{
    pthread_mutex_lock(&s->mtx);
    while (s->count == 0)
        pthread_cond_wait(&s->cond, &s->mtx);
    s->count--;
    pthread_mutex_unlock(&s->mtx);
}

/* Returns 0 on success, ETIMEDOUT on timeout. */
static int psem_timedwait(posix_sem_t *s, unsigned ms)
{
    struct timespec abs;
    clock_gettime(CLOCK_REALTIME, &abs);
    abs.tv_sec  += ms / 1000;
    abs.tv_nsec += (long)(ms % 1000) * 1000000L;
    if (abs.tv_nsec >= 1000000000L) {
        abs.tv_sec++;
        abs.tv_nsec -= 1000000000L;
    }
    pthread_mutex_lock(&s->mtx);
    int rc = 0;
    while (s->count == 0) {
        rc = pthread_cond_timedwait(&s->cond, &s->mtx, &abs);
        if (rc == ETIMEDOUT)
            break;
        rc = 0; /* spurious wake — re-check count */
    }
    if (rc == 0)
        s->count--;
    pthread_mutex_unlock(&s->mtx);
    return rc;
}

/* Non-blocking decrement; returns EAGAIN if count was 0. */
static int psem_trywait(posix_sem_t *s)
{
    pthread_mutex_lock(&s->mtx);
    int rc = (s->count > 0) ? 0 : EAGAIN;
    if (rc == 0)
        s->count--;
    pthread_mutex_unlock(&s->mtx);
    return rc;
}

static void psem_post(posix_sem_t *s)
{
    pthread_mutex_lock(&s->mtx);
    s->count++;
    pthread_cond_signal(&s->cond);
    pthread_mutex_unlock(&s->mtx);
}

static int psem_value(posix_sem_t *s)
{
    pthread_mutex_lock(&s->mtx);
    int v = s->count;
    pthread_mutex_unlock(&s->mtx);
    return v;
}

/* ── Global state ─────────────────────────────────────────────────────────── */

static jmp_buf         thread_jmpbufs[MAXTHREADS];
static pthread_mutex_t g_mutex;
static struct thread_entry *g_external_entry = NULL;

#define THREADS_RUN  0
#define THREADS_EXIT 1
static volatile int threads_status = THREADS_RUN;

/* Milliseconds since CLOCK_MONOTONIC epoch — used for sleep alignment. */
static unsigned long posix_get_ms(void)
{
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (unsigned long)ts.tv_sec * 1000UL +
           (unsigned long)ts.tv_nsec / 1000000UL;
}

/* ── Scheduler ────────────────────────────────────────────────────────────── */

void switch_thread(void)
{
    struct thread_entry *current = __running_self_entry();
    enable_irq();

    switch (current->state)
    {
    case STATE_RUNNING:
        pthread_mutex_unlock(&g_mutex);
        pthread_mutex_lock(&g_mutex);
        break;

    case STATE_BLOCKED:
    {
        posix_sem_t *s = (posix_sem_t *)current->context.s;
        pthread_mutex_unlock(&g_mutex);
        psem_wait(s);
        pthread_mutex_lock(&g_mutex);
        int oldlevel = disable_irq_save();
        current->state = STATE_RUNNING;
        restore_irq(oldlevel);
        break;
    }

    case STATE_BLOCKED_W_TMO:
    {
        posix_sem_t *s = (posix_sem_t *)current->context.s;
        pthread_mutex_unlock(&g_mutex);
        int result = psem_timedwait(s, current->tmo_tick);
        pthread_mutex_lock(&g_mutex);
        int oldlevel = disable_irq_save();
        current->state = STATE_RUNNING;
        if (result == ETIMEDOUT) {
            /* Drain any posts that raced with the timeout. */
            while (psem_value(s) > 0)
                psem_trywait(s);
        }
        restore_irq(oldlevel);
        break;
    }

    case STATE_SLEEPING:
    {
        posix_sem_t *s = (posix_sem_t *)current->context.s;
        pthread_mutex_unlock(&g_mutex);
        psem_timedwait(s, current->tmo_tick);
        pthread_mutex_lock(&g_mutex);
        current->state = STATE_RUNNING;
        break;
    }
    }

    __running_self_entry() = current;

    if (threads_status != THREADS_RUN)
        thread_exit();
}

void sleep_thread(int ticks)
{
    struct thread_entry *current = __running_self_entry();
    current->state = STATE_SLEEPING;
    /* Align to next tick boundary — same formula as thread-sdl.c. */
    int rem = (int)(posix_get_ms() % (1000 / HZ));
    if (rem < 0) rem = 0;
    current->tmo_tick = (1000 / HZ) * ticks + ((1000 / HZ) - 1) - rem;
}

void block_thread_(struct thread_entry *current, int ticks)
{
    if (ticks < 0)
        current->state = STATE_BLOCKED;
    else {
        current->state = STATE_BLOCKED_W_TMO;
        current->tmo_tick = (1000 / HZ) * ticks;
    }
    wait_queue_register(current);
}

unsigned int wakeup_thread_(struct thread_entry *thread)
{
    switch (thread->state)
    {
    case STATE_BLOCKED:
    case STATE_BLOCKED_W_TMO:
        wait_queue_remove(thread);
        thread->state = STATE_RUNNING;
        psem_post((posix_sem_t *)thread->context.s);
        return THREAD_OK;
    }
    return THREAD_NONE;
}

void thread_thaw(unsigned int thread_id)
{
    struct thread_entry *thread = __thread_id_entry(thread_id);
    if (thread->id == thread_id && thread->state == STATE_FROZEN)
    {
        thread->state = STATE_RUNNING;
        psem_post((posix_sem_t *)thread->context.s);
    }
}

/* ── Thread lifecycle ─────────────────────────────────────────────────────── */

static void *runthread(void *data)
{
    pthread_mutex_lock(&g_mutex);

    struct thread_entry *current = (struct thread_entry *)data;
    __running_self_entry() = current;

    jmp_buf *jb = &thread_jmpbufs[THREAD_ID_SLOT(current->id)];

    if (setjmp(*jb) == 0)
    {
        if (current->state == STATE_FROZEN)
        {
            posix_sem_t *s = (posix_sem_t *)current->context.s;
            pthread_mutex_unlock(&g_mutex);
            psem_wait(s);
            pthread_mutex_lock(&g_mutex);
            __running_self_entry() = current;
        }

        if (threads_status == THREADS_RUN)
            current->context.start();

        thread_exit();
    }
    else
    {
        /* thread_exit() longjmp'd here — release mutex and terminate. */
        pthread_mutex_unlock(&g_mutex);
    }
    return NULL;
}

unsigned int create_thread(void (*function)(void),
                           void *stack, size_t stack_size,
                           unsigned flags, const char *name)
{
    struct thread_entry *thread = thread_alloc();
    if (!thread) {
        DEBUGF("posix: no free thread slot\n");
        return 0;
    }

    posix_sem_t *s = psem_create(0);
    if (!s) {
        DEBUGF("posix: semaphore alloc failed\n");
        return 0;
    }

    thread->name          = name;
    thread->state         = (flags & CREATE_THREAD_FROZEN) ?
                             STATE_FROZEN : STATE_RUNNING;
    thread->context.start = function;
    thread->context.s     = s;
    thread->context.t     = NULL;
    thread->context.told  = NULL;

    pthread_t tid;
    if (pthread_create(&tid, NULL, runthread, thread) != 0) {
        DEBUGF("posix: pthread_create failed\n");
        psem_destroy(s);
        return 0;
    }
    pthread_detach(tid);

    /* pthread_t stored as void* — valid on all 64-bit targets we support. */
    thread->context.t = (void *)(uintptr_t)tid;

    return thread->id;
    (void)stack; (void)stack_size;
}

void thread_exit(void)
{
    struct thread_entry *current = __running_self_entry();
    int oldlevel = disable_irq_save();

    posix_sem_t *s = (posix_sem_t *)current->context.s;

    current->context.t    = NULL;
    current->context.s    = NULL;
    current->context.told = NULL;

    unsigned int id = current->id;
    new_thread_id(current);
    current->state = STATE_KILLED;
    wait_queue_wake(&current->queue);

    psem_destroy(s);
    restore_irq(oldlevel);
    thread_free(current);

    longjmp(thread_jmpbufs[THREAD_ID_SLOT(id)], 1);
    while (1); /* unreachable */
}

void thread_wait(unsigned int thread_id)
{
    struct thread_entry *current = __running_self_entry();
    struct thread_entry *thread  = __thread_id_entry(thread_id);

    if (thread->id == thread_id && thread->state != STATE_KILLED)
    {
        block_thread(current, TIMEOUT_BLOCK, &thread->queue);
        switch_thread();
    }
}

/* ── Initialisation ───────────────────────────────────────────────────────── */

void init_threads(void)
{
    pthread_mutex_init(&g_mutex, NULL);
    pthread_mutex_lock(&g_mutex);

    thread_alloc_init();

    struct thread_entry *thread = thread_alloc();
    if (!thread) {
        fprintf(stderr, "posix: main thread alloc failed\n");
        return;
    }

    thread->name         = __main_thread_name;
    thread->state        = STATE_RUNNING;
    thread->context.s    = psem_create(0);
    thread->context.t    = NULL;
    thread->context.told = NULL;
    __running_self_entry() = thread;

    if (!thread->context.s) {
        fprintf(stderr, "posix: main semaphore alloc failed\n");
        return;
    }

    /* Allocate a stub entry for external (Rust OS thread) callers.
     * rb_kernel_lock() sets __running_self_entry() to this entry while the
     * caller holds g_mutex, so the kernel always sees a valid STATE_RUNNING
     * thread_entry without needing a dedicated broker thread. */
    g_external_entry = thread_alloc();
    if (!g_external_entry) {
        fprintf(stderr, "posix: external entry alloc failed\n");
        return;
    }
    g_external_entry->name         = "rb_external";
    g_external_entry->state        = STATE_RUNNING;
    g_external_entry->context.s    = psem_create(0);
    g_external_entry->context.t    = NULL;
    g_external_entry->context.told = NULL;

    if (!g_external_entry->context.s) {
        fprintf(stderr, "posix: external semaphore alloc failed\n");
        return;
    }

    /* setjmp here is the exit trampoline: if thread_exit() longjmps here
     * for the main thread we fall through and exit the process. */
    if (setjmp(thread_jmpbufs[THREAD_ID_SLOT(thread->id)]) == 0)
        return; /* normal init path */

    pthread_mutex_unlock(&g_mutex);
    exit(0);
}

/* ── Rust OS thread kernel-lock API ───────────────────────────────────────────
 *
 * Any OS thread (e.g. a tokio/actix worker) may call Rockbox firmware FFI
 * safely by bracketing the call with rb_kernel_lock() / rb_kernel_unlock():
 *
 *   rb_kernel_lock();
 *   audio_play(elapsed, offset);
 *   rb_kernel_unlock();
 *
 * rb_kernel_lock() acquires the global cooperative mutex (g_mutex) so no
 * Rockbox kernel thread can run concurrently, then installs g_external_entry
 * as the current thread so __running_self_entry() always returns a valid
 * STATE_RUNNING entry.  rb_kernel_unlock() reverses both steps.
 *
 * These replace the fw_bus broker channel: direct in-line calls with O(1)
 * overhead instead of mpsc round-trips with a 30 s timeout safety net.
 *
 * macOS: pthread_mutex_lock/unlock work identically on Monterey x86_64 and
 * Sequoia arm64 — same as the rest of this file.
 */

/* WASM: the JS main thread cannot block on pthread_mutex_lock (Emscripten
 * would throw an "unwind" exception).  The wasm-bridge.c file provides
 * no-op stubs; let the weak stubs in broker_thread.c win for all other
 * hosted builds that don't need the cooperative scheduler handshake. */
#if !(CONFIG_PLATFORM & PLATFORM_WASM)
void rb_kernel_lock(void)
{
    pthread_mutex_lock(&g_mutex);
    __running_self_entry() = g_external_entry;
}

void rb_kernel_unlock(void)
{
    __running_self_entry() = NULL;
    pthread_mutex_unlock(&g_mutex);
}
#endif /* !PLATFORM_WASM */

/* ── Priority scheduling stub ─────────────────────────────────────────────────
 *
 * HAVE_PRIORITY_SCHEDULING is not defined for the headless host, so thread.c
 * does not compile thread_set_priority().  The rockbox-sys Rust crate
 * unconditionally declares it as extern "C", which creates an undefined
 * reference that the Xcode linker cannot satisfy.  Provide a no-op here so
 * the symbol is always available; priority adjustment is a best-effort
 * optimisation and is safe to ignore on hosted targets. */
#ifndef HAVE_PRIORITY_SCHEDULING
int thread_set_priority(unsigned int thread_id, int priority)
{
    (void)thread_id;
    (void)priority;
    return 0;
}
#endif
