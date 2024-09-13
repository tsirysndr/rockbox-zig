<div>
  <img src="https://www.rockbox.org/rockbox400.png" />
  <img src="https://ziglang.org/ziggy.svg" height="150"/>
</div>

# Rockbox Zig 🎵 ⚡

Rockbox Zig is an incremental enhancement of the [Rockbox](https://www.rockbox.org) firmware for portable audio players in Zig and Rust.

> [!NOTE]
**🐲 It is a work in progress and is not yet ready for use. 🏗️🚧**

![Preview](./docs/preview.png)

## 🚀 Quickstart

Run the following commands to build the project:

```sh
sudo apt-get install libusb-dev libsdl1.2-dev libfreetype6-dev
mkdir -p build && cd build
../tools/configure --target=sdlapp --type=N --lcdwidth=320 --lcdheight=240 --prefix=$HOME/.local
make zig
```

## ✨ Features

- [x] Zig Build System
- [x] Rockbox API FFI for Rust
- [x] gRPC API
- [x] GraphQL API
- [ ] Web Client (React)
- [ ] Desktop Client (Electron/Gtk)
- [ ] Terminal Client (TUI)
- [ ] Android Library
- [ ] Mobile version (React Native)
- [ ] Stream from Youtube (audio only)
- [ ] Stream from Spotify
- [ ] Stream from Tidal
- [ ] Stream to Chromecast
- [ ] TuneIn Radio
- [ ] MPD Server
- [ ] MPRIS
- [ ] Upnp Player
- [ ] Airplay
- [ ] TypeScript ([Deno](https://deno.com)) API (for writing plugins)

## 🧑‍🔬 Architecture

![architecture](./docs/rockbox-server-architecture.jpg)
  
