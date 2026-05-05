# ── WebUI ──────────────────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS webui-builder

RUN apt-get update && apt-get install -y curl unzip && rm -rf /var/lib/apt/lists/*

RUN curl -fsSL https://deno.land/install.sh | sh
ENV DENO_INSTALL="/root/.deno"
ENV PATH="${DENO_INSTALL}/bin:${PATH}"

WORKDIR /app/webui/rockbox
COPY webui/rockbox/package.json webui/rockbox/package-lock.json ./
RUN deno install --allow-scripts
COPY webui/rockbox/ ./
RUN deno task build

# ── Rockbox daemon ─────────────────────────────────────────────────────────────
FROM rust:1.94-bookworm AS builder

ARG TARGETARCH
ARG TAG

ENV TAG=${TAG}

# Runtime deps for cpal (ALSA) and other libs
RUN apt-get update && apt-get install -y \
  build-essential \
  libunwind-dev \
  libasound2-dev \
  libdbus-1-dev \
  protobuf-compiler \
  curl \
  wget \
  zip \
  unzip \
  cmake

# Install Zig 0.16.0
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

COPY . /app
WORKDIR /app
COPY --from=webui-builder /app/webui/rockbox/dist/ /app/webui/rockbox/dist/

# Build rockboxd via the headless script (configure + make + cargo + zig)
RUN bash scripts/build-headless.sh

# Build the rockbox CLI binary
RUN cargo build -p rockbox --release

# ── Runtime image ──────────────────────────────────────────────────────────────
FROM typesense/typesense:30.1 AS typesense

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
  libunwind8 \
  libasound2 \
  libdbus-1-3 \
  && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/zig/zig-out/bin/rockboxd /usr/local/bin/rockboxd
COPY --from=builder /app/target/release/rockbox   /usr/local/bin/rockbox
COPY --from=typesense /opt/typesense-server        /usr/local/bin/typesense-server

EXPOSE 6061
EXPOSE 6062
EXPOSE 6063
EXPOSE 6600

CMD ["rockboxd"]
