/* SPDX-License-Identifier: GPL-2.0-or-later
 *
 * No-op LCD driver for the Android cdylib build. Satisfies the LCD
 * function-pointer surface that apps/gui/* references at link time;
 * everything's discarded since there's no display.
 *
 * This file is grown iteratively as the linker reveals more unresolved
 * lcd_*/backlight_* symbols. Start tight, add as needed.
 */

#include <stdbool.h>
#include "config.h"
#include "lcd.h"

/* Core LCD lifecycle */
void lcd_init_device(void)              { }
void lcd_shutdown(void)                 { }

/* Frame ops — apps/gui calls these unconditionally */
void lcd_update(void)                   { }
void lcd_update_rect(int x, int y, int w, int h)
{ (void)x;(void)y;(void)w;(void)h; }
void lcd_clear_display(void)            { }

/* Color setters — used by the WPS/menu code paths */
void lcd_set_foreground(unsigned col)   { (void)col; }
void lcd_set_background(unsigned col)   { (void)col; }
unsigned lcd_get_foreground(void)       { return 0; }
unsigned lcd_get_background(void)       { return 0; }

/* Backlight — usually short symbol surface */
void backlight_hw_on(void)              { }
void backlight_hw_off(void)             { }
int  backlight_hw_brightness(int b)     { (void)b; return 0; }
