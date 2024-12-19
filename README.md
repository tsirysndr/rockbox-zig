<div>
  <img src="https://www.rockbox.org/rockbox400.png" />
  <img src="https://ziglang.org/ziggy.svg" height="150"/>
</div>


# Rockbox Zig 🎵 ⚡

[![GPL-2.0 licensed](https://img.shields.io/badge/License-GPL-blue.svg)](./LICENSE)
[![ci](https://github.com/tsirysndr/rockbox-zig/actions/workflows/ci.yml/badge.svg)](https://github.com/tsirysndr/rockbox-zig/actions/workflows/ci.yml)
[![Docker Pulls](https://img.shields.io/docker/pulls/tsiry/rockbox)](https://hub.docker.com/r/tsiry/rockbox)
![GitHub Downloads (all assets, all releases)](https://img.shields.io/github/downloads/tsirysndr/rockbox-zig/total)
[![discord](https://img.shields.io/discord/1292855167921815715?label=discord&logo=discord&color=5865F2)](https://discord.gg/tXPrgcPKSt)
[![storybook](https://raw.githubusercontent.com/storybooks/brand/master/badge/badge-storybook.svg)](https://master--670ceec25af685dcdc87c0df.chromatic.com/?path=/story/components-albums--default)


![Rockbox UI](./gtk/data/screenshots/1.png)

A modern take on the [Rockbox](https://www.rockbox.org) open-source firmware with enhancements in Zig and Rust. This project offers:

- gRPC & GraphQL APIs for seamless interaction and control
- Chromecast support for streaming to your TV
- [MPD](https://mpd.readthedocs.io/en/stable/protocol.html) server for compatibility with existing clients
- [MPRIS](https://specifications.freedesktop.org/mpris-spec/) support for desktop integration
- TypeScript support for building powerful extensions

Take advantage of modern tooling while preserving the core functionality of Rockbox.

> [!NOTE]
**🐲 It is a work in progress and is not yet ready for use. 🏗️🚧**

![Preview](./docs/preview.png)

## 🚀 Quickstart

To quickly get started, you can run the following docker command:
```sh
docker run \
    --device /dev/snd \
    --privileged \
    -p 6061:6061 \
    -p 6062:6062 \
    -p 6063:6063 \
    -p 6600:6600 \
    -v $HOME/Music:/root/Music \
    tsiry/rockbox:latest
```

Run the following commands to build the project:

```sh
sudo apt-get install libusb-dev libsdl1.2-dev libfreetype6-dev libunwind-dev zip protobuf-compiler
mkdir -p build && cd build
../tools/configure --target=sdlapp --type=N --lcdwidth=320 --lcdheight=240 --prefix=$HOME/.local
make zig
```

## 🚚 Installation

with `curl` (Ubuntu/Debian):

```sh
curl -fsSL https://raw.githubusercontent.com/tsirysndr/rockbox-zig/HEAD/install.sh | bash
```

MacOS, currently not supported, but you can run in a docker container.

## 📦 Downloads

- `Linux`: intel: [rockbox_2024.11.18_x86_64-linux.tar.gz](https://github.com/tsirysndr/rockbox-zig/releases/download/2024.11.18/rockbox_2024.11.18_x86_64-linux.tar.gz) arm64: [rockbox_2024.11.18_aarch64-linux.tar.gz](https://github.com/tsirysndr/rockbox-zig/releases/download/2024.11.18/rockbox_2024.11.18_aarch64-linux.tar.gz)


## ✨ Features

- [x] Zig Build System
- [x] Rockbox API FFI for Rust
- [x] [gRPC API](https://buf.build/tsiry/rockboxapis/docs/main:rockbox.v1alpha1)
- [x] GraphQL API
- [x] HTTP API
- [x] Web Client (React)
- [x] Fast search engine, built with [Tantivy](https://github.com/quickwit-oss/tantivy)
- [x] Desktop Client (Electron/Gtk)
- [ ] Terminal Client (TUI)
- [ ] Android Library
- [ ] Mobile version (React Native)
- [ ] Stream from Youtube (audio only)
- [ ] Stream from Spotify
- [ ] Stream from Tidal
- [x] Stream to Chromecast
- [ ] Stream to Kodi
- [ ] TuneIn Radio
- [x] MPD Server
- [x] MPRIS
- [ ] UPnP/DLNA
- [ ] Airplay
- [ ] TypeScript ([Deno](https://deno.com)) API (for writing plugins)
- [ ] Wasm extensions

## 🧑‍🔬 Architecture

![architecture](./docs/rockbox-arch.png)
  
## 📚 GraphQL API

Open [http://localhost:6062/graphiql](http://localhost:6062/graphiql) in your browser.

<p style="margin-top: 20px; margin-bottom: 20px;">
 <img src="./docs/graphql.png" width="100%" />
</p>
  
## 📚 HTTP API

Open [http://localhost:6063](http://localhost:6063) in your browser.

<p style="margin-top: 20px; margin-bottom: 20px;">
 <img src="./docs/http-api.png" width="100%" />
</p>

## 📚 gRPC API

[https://buf.build/tsiry/rockboxapis/docs/main:rockbox.v1alpha1](https://buf.build/tsiry/rockboxapis/docs/main:rockbox.v1alpha1)

Try Rockbox gRPC API using [Buf Studio](https://buf.build/studio/tsiry/rockboxapis/rockbox.v1alpha1.LibraryService/GetAlbums?target=http%3A%2F%2Flocalhost%3A6061&selectedProtocol=grpc-web).

<p style="margin-top: 20px; margin-bottom: 20px;">
 <img src="./docs/grpc.png" width="100%" />
</p>
