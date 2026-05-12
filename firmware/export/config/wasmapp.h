/*
 * Configuration for Rockbox built as a WebAssembly module (wasmapp target).
 *
 * Analogous to androidcdylib.h / headlesshost.h.  Differences:
 *   - No SDL, no gRPC/HTTP server — JS calls rb_* exports directly.
 *   - Codecs statically linked (BINFMT_STATIC, same as Android cdylib).
 *   - PCM sink is Web Audio API (pcm-webapi.c calls EM_JS glue).
 *   - No ROCKBOX_SERVER / CONFIG_SERVER — server stack is not needed.
 *   - All media paths are HTTP URLs; netstream handles them via fd aliasing.
 */

/* Hosted on the browser via Emscripten, targeting WASM. */
#define CONFIG_PLATFORM (PLATFORM_HOSTED|PLATFORM_WASM)

/* Static-link all codecs into the .wasm binary. */
#define CONFIG_BINFMT   BINFMT_STATIC

#define HAVE_FPU

#define MODEL_NUMBER 127
#define MODEL_NAME   "Rockbox (WASM)"

#define USB_NONE

/* Headless — upper layers need an LCD surface. lcd-noop.c discards paints. */
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

/* No physical buttons — control comes from JS via rb_* exports. */
#define CONFIG_KEYPAD ANDROID_PAD

#define CONFIG_RTC      RTC_HOSTED
#define CONFIG_STORAGE  STORAGE_HOSTFS
#define HAVE_STORAGE_FLUSH
#define BOOTDIR "/.rockbox"

/* Codec/plugin buffer sizes (codecs are in .text; values kept for sizing). */
#define CODEC_SIZE          0x100000
#define PLUGIN_BUFFER_SIZE  0x80000

#define PLATFORM_HAS_VOLUME_CHANGE
#define HAVE_SW_TONE_CONTROLS

#define CONFIG_BATTERY_MEASURE PERCENTAGE_MEASURE
#define NO_LOW_BATTERY_SHUTDOWN

#define AB_REPEAT_ENABLE

/* No gRPC/HTTP server in the WASM build. JS calls rb_* exports directly
 * so ROCKBOX_SERVER and CONFIG_SERVER are intentionally NOT defined here. */

/* pcm_copy_buffer: no SDL so map directly to memcpy (same as Android). */
#define pcm_copy_buffer memcpy

/* Route DEBUGF to emscripten_log / console (see debug-wasm.c). */
#define DEBUGF  debugf
#define LDEBUGF debugf
