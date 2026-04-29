FROM rust:1.94-bookworm

ARG TARGETARCH

RUN apt-get update && apt-get install -y apt-utils && \
  apt-get install -y build-essential \
  libusb-dev \
  libsdl2-dev \
  libfreetype6-dev \
  libunwind-dev \
  curl \
  zip \
  unzip \
  git \
  protobuf-compiler \
  locales \
  vim \
  cmake \
  libdbus-1-dev \
  flatpak

RUN locale-gen en_US.UTF-8

RUN curl -Ssf https://pkgx.sh | sh

RUN case "${TARGETARCH}" in \
      amd64) ZIG_ARCH="x86_64" ;; \
      arm64) ZIG_ARCH="aarch64" ;; \
      *) echo "Unsupported arch: ${TARGETARCH}" && exit 1 ;; \
    esac && \
    export VERSION=0.16.0 && \
    wget "https://ziglang.org/download/${VERSION}/zig-${ZIG_ARCH}-linux-${VERSION}.tar.xz" && \
    tar -xf "zig-${ZIG_ARCH}-linux-${VERSION}.tar.xz" && \
    mv "zig-${ZIG_ARCH}-linux-${VERSION}" /usr/local/zig && \
    ln -s /usr/local/zig/zig /usr/local/bin/zig

RUN curl -s https://ohmyposh.dev/install.sh | bash -s -- -d /usr/local/bin

RUN git clone https://github.com/JanDeDobbeleer/oh-my-posh.git .oh-my-posh

RUN echo 'eval "$(oh-my-posh init bash --config ~/.cache/oh-my-posh/themes/tokyonight_storm.omp.json)"' >> /root/.bashrc

RUN curl --proto '=https' --tlsv1.2 -sSf https://setup.atuin.sh | bash

ENV BASH_ENV=/root/.bashrc

RUN curl -fsSL https://bun.sh/install | bash

ENV PATH=/root/.bun/bin:$PATH

RUN pkgm install node protoc buf deno

ENV LANG=en_US.UTF-8

ENV LANGUAGE=en_US.UTF-8
