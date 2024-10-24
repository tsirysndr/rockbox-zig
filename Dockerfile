FROM rust:1.81-bookworm AS builder

RUN apt-get update && apt-get install -y build-essential \
  libusb-dev \
  libsdl1.2-dev \
  libfreetype6-dev \
  libunwind-dev \
  curl \
  zip \
  unzip \
  protobuf-compiler

RUN curl -Ssf https://pkgx.sh | sh

RUN pkgx install zig@0.13.0 node bun@1.1.30 protoc buf

COPY . /app

WORKDIR /app

RUN mkdir -p build /root/.local/lib/rockbox

WORKDIR /app/webui/rockbox

RUN bun install

RUN bun run build

WORKDIR /app/build

RUN ../tools/configure --target=sdlapp --type=N --lcdwidth=320 --lcdheight=240 --prefix=$HOME/.local

RUN make ziginstall -j$(nproc)

FROM debian:bookworm

RUN apt-get update && apt-get install -y \
  libusb-dev \
  libsdl1.2-dev \
  libfreetype6-dev \
  libunwind-dev \
  alsa-utils \
  libasound2 \
  pulseaudio

COPY --from=builder /root/.local /root/.local

COPY --from=builder /root/.local/bin/rockbox /usr/bin/rockbox

ENV SDL_VIDEODRIVER=dummy

EXPOSE 6061
EXPOSE 6062
EXPOSE 6063

CMD ["rockbox"]
