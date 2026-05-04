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

/* THIS IS THE MAGIC FLAG. apps/main.c:486 gates server_init() (which spawns
 * the kernel thread that runs crates/server::start_server + start_servers,
 * binding gRPC/HTTP/GraphQL/MPD on 6061/6063/6062/6600) behind
 * #ifdef ROCKBOX_SERVER. Without it the firmware boots fine but nothing
 * ever binds. The desktop sdlapp build defines this via configure;
 * embedded-daemon needs the same. */
#define ROCKBOX_SERVER

/* Sister flag to ROCKBOX_SERVER. apps/SOURCES:314 gates server_thread.c
 * and broker_thread.c (which DEFINE server_init() and start_broker()) on
 * #ifdef CONFIG_SERVER — a separate macro from ROCKBOX_SERVER, both come
 * from configure on desktop (see build-lib/autoconf.h). Without this the
 * server_thread.o object never gets built into librockbox.a and the
 * cdylib link fails with "cannot locate symbol server_init". */
#define CONFIG_SERVER

/* `sigevent_t` (glibc-style typedef used by kernel-unix.c) is provided as
 * a -D macro in androidcdylibcc rather than typedef'd here — putting code
 * in config.h would leak into the output of `preprocess` (which uses
 * -include config.h to evaluate SOURCES files) and produce nonsense make
 * targets. -D makes the substitution at compile time only. */

/* pcm_copy_buffer is defined as memcpy when HAVE_SDL_AUDIO is set (see
 * firmware/export/pcm-internal.h). The hosted PCM sinks (pcm-fifo.c etc.)
 * call it; without the alias they fail to compile. We don't have SDL on
 * this build, so map it to memcpy directly here. The line starts with #
 * so `preprocess` filters it out of SOURCES expansion. */
#define pcm_copy_buffer memcpy

/* Enable firmware DEBUGF()/LDEBUGF() so the buffering, codec-loader, and
 * metadata paths actually log. Together with debug-android.c (added in
 * firmware/SOURCES under the CODECS_STATIC block) this routes debugf to
 * logcat under tag "Rockbox" via __android_log_print. Without this, every
 * DEBUGF call site expands to `do {} while(0)` and we get no firmware
 * diagnostics at all on play attempts.
 *
 * We CAN'T just `#define DEBUG` — libmad treats that as user-asserted
 * "build with assertions on" and errors out because we also have NDEBUG
 * (libmad enforces XOR). Instead we override DEBUGF directly. The macro
 * names are the ones debug.h would have set under DEBUG; pre-defining
 * them here makes debug.h's own gating skip its `do{}while(0)` fallback. */
#define DEBUGF  debugf
#define LDEBUGF debugf
