/*
 * Configuration for a headless macOS / Linux build of Rockbox: no SDL,
 * no GUI, statically-linked codecs, cpal PCM sink, single rockboxd binary
 * produced by the Zig linker.
 *
 * Mirrors androidcdylib.h in structure; key differences:
 *   - PLATFORM_HOSTED only (no PLATFORM_ANDROID)
 *   - PCM sink is cpal (Rust), not AAudio
 *   - DEBUGF routes to stderr via debug-headless.c
 *   - Final link is done by `zig build -Dheadless=true`, not cargo/JNI
 */

#define CONFIG_PLATFORM (PLATFORM_HOSTED)

/* Static-link all codecs into the binary. */
#define CONFIG_BINFMT BINFMT_STATIC

#define HAVE_FPU

#define MODEL_NUMBER 126
#define MODEL_NAME   "Rockbox (headless host)"

#define USB_NONE

/* Headless — upper layers still need an LCD surface for buffer sizing. */
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

/* No hardware buttons — control arrives via gRPC. */
#define CONFIG_KEYPAD ANDROID_PAD

#define CONFIG_RTC      RTC_HOSTED
#define CONFIG_STORAGE  STORAGE_HOSTFS
#define HAVE_STORAGE_FLUSH
#define BOOTDIR "/.rockbox"

#define CODEC_SIZE         0x100000
#define PLUGIN_BUFFER_SIZE 0x80000

#define PLATFORM_HAS_VOLUME_CHANGE
#define HAVE_SW_TONE_CONTROLS

#define CONFIG_BATTERY_MEASURE PERCENTAGE_MEASURE
#define NO_LOW_BATTERY_SHUTDOWN

#define AB_REPEAT_ENABLE

/* Mandatory for the gRPC / HTTP / GraphQL / MPD servers to start. */
#define ROCKBOX_SERVER
#define CONFIG_SERVER

/* pcm_copy_buffer: no SDL on this build, map directly to memcpy. */
#define pcm_copy_buffer memcpy

/* Route firmware DEBUGF/LDEBUGF to stderr via debug-headless.c. */
#define DEBUGF  debugf
#define LDEBUGF debugf
