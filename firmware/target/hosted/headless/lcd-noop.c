/* SPDX-License-Identifier: GPL-2.0-or-later
 *
 * No-op LCD driver for the headless host build. Satisfies the LCD
 * function-pointer surface that apps/gui/* references at link time;
 * everything is discarded since there is no display.
 */

#include <stdbool.h>
#include "config.h"
#include "lcd.h"

void lcd_init_device(void)              { }
void lcd_shutdown(void)                 { }
void lcd_update(void)                   { }
void lcd_update_rect(int x, int y, int w, int h)
{ (void)x; (void)y; (void)w; (void)h; }
int  lcd_get_dpi(void)                  { return 160; }
void backlight_hw_on(void)              { }
void backlight_hw_off(void)             { }
int  backlight_hw_brightness(int b)     { (void)b; return 0; }
