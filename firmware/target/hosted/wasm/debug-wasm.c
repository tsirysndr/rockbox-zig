/* SPDX-License-Identifier: GPL-2.0-or-later
 *
 * Debug/log shim for the WASM build. Routes firmware DEBUGF() to the
 * browser console via Emscripten's emscripten_log().
 *
 * wasmapp.h pre-defines DEBUGF to call debugf() (same trick as Android),
 * so this file's implementations are always active.
 */

#include <stdarg.h>
#include <stdio.h>
#include <emscripten.h>

#define LOG_TAG "Rockbox"

void debug_init(void) {}

void debugf(const char *fmt, ...)
{
    char buf[512];
    va_list ap;
    va_start(ap, fmt);
    vsnprintf(buf, sizeof(buf), fmt, ap);
    va_end(ap);
    emscripten_log(EM_LOG_CONSOLE, "[%s] %s", LOG_TAG, buf);
}

void ldebugf(const char *file, int line, const char *fmt, ...)
{
    char buf[512];
    va_list ap;
    va_start(ap, fmt);
    vsnprintf(buf, sizeof(buf), fmt, ap);
    va_end(ap);
    emscripten_log(EM_LOG_CONSOLE, "[%s] %s:%d %s", LOG_TAG, file, line, buf);
}
