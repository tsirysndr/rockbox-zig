/* SPDX-License-Identifier: GPL-2.0-or-later
 *
 * No-op LCD driver for the Android cdylib build. Satisfies the LCD
 * function-pointer surface that apps/gui/* references at link time;
 * everything's discarded since there's no display.
 *
 * This file is grown iteratively as the linker reveals more unresolved
 * lcd_ / backlight_ symbols. Start tight, add as needed.
 */

#include <stdbool.h>
#include "config.h"
#include "lcd.h"

/* Core LCD lifecycle */
void lcd_init_device(void)              { }
void lcd_shutdown(void)                 { }

/* lcd_update / lcd_update_rect / lcd_clear_display are provided by
 * firmware/drivers/lcd-bitmap-common.c (or 16-bit variant) — no need
 * to redefine. */

/* lcd_set_foreground / lcd_set_background / lcd_get_foreground /
 * lcd_get_background are already provided by firmware/drivers/lcd-bitmap-common.c
 * (or similar). No need to redefine. */

/* Backlight — usually short symbol surface */
void backlight_hw_on(void)              { }
void backlight_hw_off(void)             { }
int  backlight_hw_brightness(int b)     { (void)b; return 0; }
