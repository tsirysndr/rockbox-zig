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

export PATH="$HOME/.local/bin:$PATH"

# Define the release information
RELEASE_URL="https://api.github.com/repos/tsirysndr/rockbox-zig/releases/latest"

ASSET_NAME=""

function detect_os() {
  # Determine the operating system
  OS=$(uname -s)
  if [ "$OS" = "Linux" ]; then
      # Determine the CPU architecture
      ARCH=$(uname -m)
      if [ "$ARCH" = "aarch64" ]; then
          ASSET_NAME="_aarch64-linux.tar.gz"
      elif [ "$ARCH" = "x86_64" ]; then
          ASSET_NAME="_x86_64-linux.tar.gz"
      else
          echo "Unsupported architecture: $ARCH"
          exit 1
      fi
  else
      echo "Unsupported operating system: $OS"
      exit 1
  fi;
}

# Install Rockbox CLI

detect_os

# Retrieve the download URL for the desired asset
DOWNLOAD_URL=$(curl -sSL "$RELEASE_URL" | grep -o "browser_download_url.*rockbox_.*$ASSET_NAME\"" | cut -d ' ' -f 2)

ASSET_NAME=$(basename $DOWNLOAD_URL)

INSTALL_DIR="$HOME/.local/bin"

DOWNLOAD_URL=`echo $DOWNLOAD_URL | tr -d '\"'`

# Download the asset
curl -SL $DOWNLOAD_URL -o /tmp/$ASSET_NAME

# Extract the asset
tar -xzf /tmp/$ASSET_NAME -C /tmp

# Set the correct permissions for the binary
chmod +x /tmp/rockbox

mv /tmp/rockbox $INSTALL_DIR

if command -v apt-get >/dev/null 2>&1; then
    if command -v sudo >/dev/null 2>&1; then
        sudo apt-get install -y libusb-dev \
            libsdl1.2-dev \
            libfreetype6 \
            libunwind-dev \
            alsa-utils \
            libasound2
    else
        apt-get install -y libusb-dev \
            libsdl1.2-dev \
            libfreetype6 \
            libunwind-dev \
            alsa-utils \
            libasound2
    fi
elif command -v pacman >/dev/null 2>&1; then
    if command -v sudo >/dev/null 2>&1; then
        sudo pacman -S --noconfirm libusb \
            sdl \
            freetype2 \
            libunwind \
            alsa-lib
    else
        pacman -S --noconfirm libusb \
            sdl \
            freetype2 \
            libunwind \
            alsa-lib
    fi
elif command -v dnf >/dev/null 2>&1; then
    if command -v sudo >/dev/null 2>&1; then
        sudo dnf install -y libusb \
            SDL \
            freetype \
            libunwind \
            alsa-lib
    else
        dnf install -y libusb \
            SDL \
            freetype \
            libunwind \
            alsa-lib
    fi
elif command -v zypper >/dev/null 2>&1; then
    if command -v sudo >/dev/null 2>&1; then
        sudo zypper install -y libusb-1_0-0 \
            libSDL-1_2-0 \
            freetype2 \
            libunwind \
            alsa
    else
        zypper install -y libusb-1_0-0 \
            libSDL-1_2-0 \
            freetype2 \
            libunwind \
            alsa
    fi
elif command -v apk >/dev/null 2>&1; then
    if command -v sudo >/dev/null 2>&1; then
        sudo apk add libusb \
            sdl \
            freetype \
            libunwind \
            alsa-lib
    else
        apk add libusb \
            sdl \
            freetype \
            libunwind \
            alsa-lib
    fi
fi

# Install Rockbox daemon

detect_os

DOWNLOAD_URL=$(curl -sSL $RELEASE_URL | grep -o "browser_download_url.*rockboxd_.*$ASSET_NAME\"" | cut -d ' ' -f 2)

ASSET_NAME=$(basename $DOWNLOAD_URL)

DOWNLOAD_URL=`echo $DOWNLOAD_URL | tr -d '\"'`

# Download the asset
curl -SL $DOWNLOAD_URL -o /tmp/$ASSET_NAME

# Extract the asset
tar -xzf /tmp/$ASSET_NAME -C /tmp

# Set the correct permissions for the binary
chmod +x /tmp/rockboxd

mv /tmp/rockboxd $INSTALL_DIR

# Install Rockbox assets

detect_os

ASSET_NAME=$(echo $ASSET_NAME | sed 's/_x86_64/-x86_64/')
ASSET_NAME=$(echo $ASSET_NAME | sed 's/_aarch64/-aarch64/')

DOWNLOAD_URL=$(curl -sSL $RELEASE_URL | grep -o "browser_download_url.*rockbox-assets.*$ASSET_NAME\"" | cut -d ' ' -f 2)

ASSET_NAME=$(basename $DOWNLOAD_URL)

DOWNLOAD_URL=`echo $DOWNLOAD_URL | tr -d '\"'`

# Download the asset
curl -SL $DOWNLOAD_URL -o /tmp/$ASSET_NAME


# Extract the asset
mkdir -p /tmp/rockbox-assets
tar -xzf /tmp/$ASSET_NAME -C /tmp/rockbox-assets

mkdir -p $INSTALL_DIR/../share/rockbox
cp -r /tmp/rockbox-assets/* $INSTALL_DIR/../share/rockbox

# Install Rockbox Codecs

detect_os

ASSET_NAME=$(echo $ASSET_NAME | sed 's/_x86_64/-x86_64/')
ASSET_NAME=$(echo $ASSET_NAME | sed 's/_aarch64/-aarch64/')

DOWNLOAD_URL=$(curl -sSL $RELEASE_URL | grep -o "browser_download_url.*rockbox-codecs.*$ASSET_NAME\"" | cut -d ' ' -f 2)

ASSET_NAME=$(basename $DOWNLOAD_URL)

DOWNLOAD_URL=`echo $DOWNLOAD_URL | tr -d '\"'`

# Download the asset
curl -SL $DOWNLOAD_URL -o /tmp/$ASSET_NAME

# Extract the asset
tar -xzf /tmp/$ASSET_NAME -C /tmp

mkdir -p $INSTALL_DIR/../lib/rockbox
cp -r /tmp/codecs $INSTALL_DIR/../lib/rockbox
cp -r /tmp/rocks $INSTALL_DIR/../lib/rockbox

# detect if user can run rockboxd, if not add $HOME/.local/bin to PATH
if ! command -v rockboxd >/dev/null 2>&1; then
    echo "export PATH=\"\$HOME/.local/bin:\$PATH\"" >> $HOME/.bashrc
    export PATH="$HOME/.local/bin:$PATH"
fi

cat <<EOF
${ORANGE}
              __________               __   ___.
    Open      \______   \ ____   ____ |  | _\_ |__   _______  ___
    Source     |       _//  _ \_/ ___\|  |/ /| __ \ /  _ \  \/  /
    Jukebox    |    |   (  <_> )  \___|    < | \_\ (  <_> > <  <
    Firmware   |____|_  /\____/ \___  >__|_ \|___  /\____/__/\_ \\
                      \/            \/     \/    \/            \/
${NO_COLOR}
Welcome to Rockbox Zig! 🚀
A fork of the original Rockbox project, with a focus on modernization and more features.

${GREEN}https://github.com/tsirysndr/rockbox-zig${NO_COLOR}

Please file an issue if you encounter any problems!

===============================================================================

Installation completed! 🎉

To get started, run:

${CYAN}rockbox start${NO_COLOR}

Stuck? Join our Discord ${MAGENTA}https://discord.gg/tXPrgcPKSt${NO_COLOR}

EOF
