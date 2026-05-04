/* SPDX-License-Identifier: GPL-2.0-or-later
 *
 * No-button target for the Android cdylib. Control comes from JS via gRPC,
 * not hardware buttons. We still need to define BUTTON_* enum values
 * because the upper layers reference them in switch statements; the values
 * are arbitrary since button_read_device() in button-noop.c always returns 0.
 */

#ifndef _BUTTON_TARGET_H_
#define _BUTTON_TARGET_H_

/* Define the standard Rockbox button set so apps/action.c switch statements
 * compile. None ever fire — button-noop.c returns 0 unconditionally. */
#define BUTTON_NONE         0x00000000

#define BUTTON_LEFT         0x00000001
#define BUTTON_RIGHT        0x00000002
#define BUTTON_UP           0x00000004
#define BUTTON_DOWN         0x00000008
#define BUTTON_SELECT       0x00000010
#define BUTTON_PLAY         0x00000020
#define BUTTON_POWER        0x00000040
#define BUTTON_VOL_UP       0x00000080
#define BUTTON_VOL_DOWN     0x00000100
#define BUTTON_BACK         0x00000200
#define BUTTON_MENU         0x00000400

#define BUTTON_MAIN (BUTTON_LEFT | BUTTON_RIGHT | BUTTON_UP | BUTTON_DOWN | \
                     BUTTON_SELECT | BUTTON_PLAY | BUTTON_POWER |           \
                     BUTTON_VOL_UP | BUTTON_VOL_DOWN | BUTTON_BACK |        \
                     BUTTON_MENU)

#define BUTTON_REMOTE       0

/* Touch / wheel features intentionally absent. */

#endif /* _BUTTON_TARGET_H_ */
