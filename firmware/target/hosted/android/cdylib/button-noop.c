/* SPDX-License-Identifier: GPL-2.0-or-later
 *
 * No-op button driver for the Android cdylib build. Engine never sees
 * any input; control comes through gRPC.
 */

#include <stdbool.h>
#include "config.h"
#include "button.h"

void button_init_device(void)           { }
int  button_read_device(void)           { return 0; }
void button_set_flip(bool flip)         { (void)flip; }
