<div>
  <img src="https://www.rockbox.org/rockbox400.png" />
  <img src="https://ziglang.org/ziggy.svg" height="150"/>
</div>

# Rockbox Zig 🎵 ⚡

Rockbox Zig is an incremental enhancement of the [Rockbox](https://www.rockbox.org) firmware for portable audio players in the Zig programming language.

> [!NOTE]
**🐲 It is a work in progress and is not yet ready for use. 🏗️🚧**

## 🚀 Quickstart

Run the following commands to build the project:

```sh
sudo apt-get install libusb-dev libsdl1.2-dev libfreetype6-dev
. ./bin/activate-hermit
mkdir -p build && cd build
../tools/configure --target=sdlapp --type=N --lcdwidth=320 --lcdheight=240 --prefix=$HOME/.local
make zig
```

Or with Nix:
```sh
nix develop
mkdir -p build && cd build
../tools/configure --target=sdlapp --type=N --lcdwidth=320 --lcdheight=240 --prefix=$HOME/.local
make zig
```
