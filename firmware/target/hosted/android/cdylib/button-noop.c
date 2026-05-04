/* SPDX-License-Identifier: GPL-2.0-or-later
 *
 * No-op button driver for the Android cdylib build. Engine never sees
 * any input; control comes through gRPC.
 */

#include <stddef.h>
#include <stdbool.h>
#include "config.h"
#include "button.h"

void button_init_device(void)           { }
int  button_read_device(void)           { return 0; }
void button_set_flip(bool flip)         { (void)flip; }

/* get_context_mapping is normally defined per-target in apps/keymaps/
 * keymap-<target>.c (we gated out keymap-android.c because it references
 * BUTTON_DPAD_* constants from the Java-shell button-target.h). The
 * callers in apps/action.c expect a pointer to a button_mapping array,
 * iterating until it sees BUTTON_NONE — returning NULL is safe (action.c
 * checks for null and falls through to default behaviour). */
struct button_mapping;
const struct button_mapping* get_context_mapping(int context)
{
    (void)context;
    return NULL;
}
