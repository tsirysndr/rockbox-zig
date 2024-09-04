![Rockbox Zig](https://www.rockbox.org/rockbox400.png)

# Rockbox Zig 🎵 ⚡

Rockbox Zig is an incremental enhancement of the [Rockbox](https://www.rockbox.org) firmware for portable audio players in the Zig programming language. It is a work in progress and is not yet ready for use.

## 🚀 Quickstart

Run the following commands to build the project:

```sh
. ./bin/activate-hermit
mkdir -p build && \
cd build && \
../tools/configure --target=sdlapp --type=N --lcdwidth=320 --lcdheight=480 && \
cd .. && \
zig build
```
