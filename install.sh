#!/bin/bash

set -e -o pipefail

readonly MAGENTA="$(tput setaf 5 2>/dev/null || echo '')"
readonly GREEN="$(tput setaf 2 2>/dev/null || echo '')"
readonly CYAN="$(tput setaf 6 2>/dev/null || echo '')"
readonly ORANGE="$(tput setaf 3 2>/dev/null || echo '')"
readonly NO_COLOR="$(tput sgr0 2>/dev/null || echo '')"

if ! command -v curl >/dev/null 2>&1; then
    echo "Error: curl is required to install Rockbox."
    exit 1
fi

if ! command -v tar >/dev/null 2>&1; then
    echo "Error: tar is required to install Rockbox."
    exit 1
fi

RELEASE_URL="https://api.github.com/repos/tsirysndr/rockboxd/releases/latest"
INSTALL_DIR="/usr/local/bin"

# ── Detect OS / architecture ──────────────────────────────────────────────────

detect_platform() {
    OS=$(uname -s)
    ARCH=$(uname -m)

    case "$OS" in
        Linux)
            case "$ARCH" in
                aarch64) PLATFORM="aarch64-linux" ;;
                x86_64)  PLATFORM="x86_64-linux"  ;;
                *) echo "Unsupported architecture: $ARCH" && exit 1 ;;
            esac
            ;;
        Darwin)
            case "$ARCH" in
                arm64)  PLATFORM="aarch64-darwin" ;;
                x86_64) PLATFORM="x86_64-darwin"  ;;
                *) echo "Unsupported architecture: $ARCH" && exit 1 ;;
            esac
            ;;
        *)
            echo "Unsupported operating system: $OS"
            exit 1
            ;;
    esac
}

detect_platform

# ── Install runtime dependencies ──────────────────────────────────────────────

install_deps() {
    if command -v apt-get >/dev/null 2>&1; then
        _sudo apt-get install -y libunwind-dev libasound2 libdbus-1-3
    elif command -v brew >/dev/null 2>&1; then
        : # no extra runtime deps on macOS
    elif command -v pacman >/dev/null 2>&1; then
        _sudo pacman -S --noconfirm libunwind alsa-lib
    elif command -v dnf >/dev/null 2>&1; then
        _sudo dnf install -y libunwind alsa-lib
    elif command -v zypper >/dev/null 2>&1; then
        _sudo zypper install -y libunwind alsa
    elif command -v apk >/dev/null 2>&1; then
        _sudo apk add libunwind alsa-lib
    fi
}

_sudo() {
    if command -v sudo >/dev/null 2>&1; then
        sudo "$@"
    else
        "$@"
    fi
}

install_deps

# ── Download & install ────────────────────────────────────────────────────────

# The release tarball contains both `rockbox` (CLI) and `rockboxd` (daemon).
DOWNLOAD_URL=$(curl -sSL "$RELEASE_URL" \
    | grep -o "browser_download_url.*rockbox_.*_${PLATFORM}\\.tar\\.gz\"" \
    | head -1 \
    | cut -d ' ' -f 2 \
    | tr -d '"')

if [ -z "$DOWNLOAD_URL" ]; then
    echo "Error: could not find a release asset for platform '${PLATFORM}'."
    echo "Check https://github.com/tsirysndr/rockboxd/releases for available builds."
    exit 1
fi

ASSET_NAME=$(basename "$DOWNLOAD_URL")

echo "Downloading ${ASSET_NAME} ..."
curl -SL "$DOWNLOAD_URL" -o "/tmp/${ASSET_NAME}"

echo "Extracting ..."
tar -xzf "/tmp/${ASSET_NAME}" -C /tmp

chmod +x /tmp/rockbox /tmp/rockboxd

echo "Installing to ${INSTALL_DIR} ..."
_sudo mv /tmp/rockbox   "${INSTALL_DIR}/rockbox"
_sudo mv /tmp/rockboxd  "${INSTALL_DIR}/rockboxd"

rm -f "/tmp/${ASSET_NAME}"

cat <<EOF
${ORANGE}
              __________               __   ___.
    Open      \______   \ ____   ____ |  | _\_ |__   _______  ___
    Source     |       _//  _ \_/ ___\|  |/ /| __ \ /  _ \  \/  /
    Jukebox    |    |   (  <_> )  \___|    < | \_\ (  <_> > <  <
    Firmware   |____|_  /\____/ \___  >__|_ \|___  /\____/__/\_ \\
                      \/            \/     \/    \/            \/
${NO_COLOR}
Welcome to Rockbox Daemon! 🚀
A fork of the original Rockbox project, with a focus on modernization and more features.

${GREEN}https://github.com/tsirysndr/rockboxd${NO_COLOR}

Please file an issue if you encounter any problems!

===============================================================================

Installation completed! 🎉

Installed:
  ${CYAN}${INSTALL_DIR}/rockbox${NO_COLOR}   — CLI client
  ${CYAN}${INSTALL_DIR}/rockboxd${NO_COLOR}  — daemon (headless audio server)

To get started, run:

  ${CYAN}rockboxd${NO_COLOR}          (start the daemon)
  ${CYAN}rockbox start${NO_COLOR}     (or use the CLI to launch it)

Stuck? Join our Discord ${MAGENTA}https://discord.gg/tXPrgcPKSt${NO_COLOR}

EOF
