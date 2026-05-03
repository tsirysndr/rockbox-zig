/*
 * Configuration for Rockbox built as a static library set, linked into a
 * single Android cdylib (librockbox_expo.so) by the Expo native module.
 *
 * Differs from android.h (the upstream Java-shell-app target) in:
 *   - Headless: no LCD/touchscreen/input — UI lives in React Native.
 *   - Codecs are statically linked, not dlopen'd (BINFMT_STATIC).
 *   - PCM sink is AAudio, not the Java AudioTrack JNI bridge.
 *   - No buttons/keymap; control surface is gRPC over loopback.
 */

/* Hosted on Linux (bionic), targeting Android. */
#define CONFIG_PLATFORM (PLATFORM_HOSTED|PLATFORM_ANDROID)

/* Static-link all codecs into the parent binary. Picked up by
 * firmware/export/load_code.h and lib/rbcodec/codecs/codecs.h. */
#define CONFIG_BINFMT   BINFMT_STATIC

#define HAVE_FPU

#define MODEL_NUMBER 125
#define MODEL_NAME   "Rockbox (Android cdylib)"

#define USB_NONE

/* Headless — but upper layers want an LCD surface to write to. We provide
 * minimal RGB565 dimensions; lcd-noop.c (in cdylib/) discards every paint. */
#define HAVE_LCD_COLOR
#define HAVE_ALBUMART
#define HAVE_BMP_SCALING
#define HAVE_JPEG

#ifndef LCD_WIDTH
#define LCD_WIDTH  320
#endif
#ifndef LCD_HEIGHT
#define LCD_HEIGHT 480
#endif
#define LCD_DEPTH  16
#define LCD_PIXELFORMAT RGB565

#define HAVE_TAGCACHE

/* No touchscreen/buttons — control comes from JS via gRPC. The keypad
 * symbol still has to exist so upper layers compile; routes are stubbed
 * in button-noop.c (added later, alongside lcd-noop.c). */
#define CONFIG_KEYPAD ANDROID_PAD

#define CONFIG_RTC      RTC_HOSTED
#define CONFIG_STORAGE  STORAGE_HOSTFS
#define HAVE_STORAGE_FLUSH
#define BOOTDIR "/.rockbox"

/* With BINFMT_STATIC the "codec buffer" no longer holds loaded code
 * (codecs are in .text), but some upper layers still size buffers from
 * these. Keep modest values. */
#define CODEC_SIZE          0x100000
#define PLUGIN_BUFFER_SIZE  0x80000

#define PLATFORM_HAS_VOLUME_CHANGE
#define HAVE_SW_TONE_CONTROLS

#define CONFIG_BATTERY_MEASURE PERCENTAGE_MEASURE
#define NO_LOW_BATTERY_SHUTDOWN

#define AB_REPEAT_ENABLE

/* bionic only declares `struct sigevent` (POSIX). Rockbox's kernel-unix.c
 * uses the glibc-style `sigevent_t` typedef. Provide it once here so we
 * don't have to patch shared firmware code. */
#ifndef __ASSEMBLER__
#include <signal.h>
typedef struct sigevent sigevent_t;
#endif
