/* SPDX-License-Identifier: GPL-2.0-or-later
 *
 * No-op button driver for the headless host build.
 * Control arrives via gRPC; no hardware input is ever polled.
 */

#include <stddef.h>
#include <stdbool.h>
#include "config.h"
#include "button.h"

void button_init_device(void)   { }
int  button_read_device(void)   { return 0; }
void button_set_flip(bool flip) { (void)flip; }

struct button_mapping;
const struct button_mapping *get_context_mapping(int context)
{
    (void)context;
    return NULL;
}
