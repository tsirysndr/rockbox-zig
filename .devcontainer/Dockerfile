FROM rust:1.81-bookworm

RUN apt-get update && apt-get install -y build-essential \
  libusb-dev \
  libsdl1.2-dev \
  libfreetype6-dev \
  libunwind-dev \
  curl \
  zip \
  unzip \
  git \
  protobuf-compiler

RUN curl -Ssf https://pkgx.sh | sh

RUN pkgx install zig@0.13.0

RUN curl -s https://ohmyposh.dev/install.sh | bash -s -- -d /usr/local/bin

RUN git clone https://github.com/JanDeDobbeleer/oh-my-posh.git .oh-my-posh

RUN echo 'eval "$(oh-my-posh init bash --config ~/.cache/oh-my-posh/themes/tokyonight_storm.omp.json)"' >> /root/.bashrc

RUN curl --proto '=https' --tlsv1.2 -sSf https://setup.atuin.sh | bash

ENV BASH_ENV=/root/.bashrc
