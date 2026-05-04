/* SPDX-License-Identifier: GPL-2.0-or-later
 *
 * Routes firmware debugf() / logf() / panicf() to stderr so that
 * DEBUGF / LDEBUGF macros (defined in headlesshost.h to call debugf)
 * produce visible output on the terminal. Matches the role of
 * debug-android.c in the Android cdylib build.
 */

#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>

#include "config.h"
#include "debug.h"

void debugf(const char *fmt, ...)
{
    va_list ap;
    va_start(ap, fmt);
    vfprintf(stderr, fmt, ap);
    va_end(ap);
}

void ldebugf(const char *file, int line, const char *fmt, ...)
{
    va_list ap;
    fprintf(stderr, "%s:%d: ", file, line);
    va_start(ap, fmt);
    vfprintf(stderr, fmt, ap);
    va_end(ap);
}
