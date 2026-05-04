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

/* Platform-driver LCD frame-out functions. Without these the .so links
 * with undefined refs and dlopen fails at runtime. lcd_clear_display /
 * lcd_set_foreground / lcd_set_background / lcd_get_foreground /
 * lcd_get_background are NOT here — they're provided by
 * firmware/drivers/lcd-color-common.c which IS linked. */
void lcd_update(void)                          { }
void lcd_update_rect(int x, int y, int w, int h)
{ (void)x; (void)y; (void)w; (void)h; }
int lcd_get_dpi(void)                          { return 160; }

/* Backlight — usually short symbol surface */
void backlight_hw_on(void)              { }
void backlight_hw_off(void)             { }
int  backlight_hw_brightness(int b)     { (void)b; return 0; }
