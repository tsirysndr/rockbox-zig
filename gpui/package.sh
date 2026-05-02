#!/usr/bin/env bash
# package.sh — build a release binary and package for macOS or Linux
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

APP_NAME="Rockbox"
BUNDLE_ID="org.rockbox.Rockbox"
VERSION="0.1.0"
BINARY="target/release/${APP_NAME}"
APPICONSET="../macos/Rockbox/Assets.xcassets/AppIcon.appiconset"
OUT_DIR="dist"
APP_BUNDLE="${OUT_DIR}/${APP_NAME}.app"
DMG_STAGING="${OUT_DIR}/dmg_staging"
DMG_OUT="${OUT_DIR}/${APP_NAME}.dmg"

# ── Detect OS ─────────────────────────────────────────────────────────────────
OS="$(uname -s)"
case "$OS" in
    Linux*)  OS_TYPE="Linux";;
    Darwin*) OS_TYPE="macOS";;
    *)       echo "Unsupported OS: $OS" && exit 1;;
esac

echo "▸ Detected OS: $OS_TYPE"

# ── 1. Build ──────────────────────────────────────────────────────────────────
echo "▸ Building release binary…"
cargo build --release

if [ "$OS_TYPE" = "Linux" ]; then
    # ── Linux Package ─────────────────────────────────────────────────────────
    echo "▸ Creating Linux package structure…"
    rm -rf "$OUT_DIR"
    mkdir -p "${OUT_DIR}/usr/bin"
    mkdir -p "${OUT_DIR}/usr/share/applications"
    mkdir -p "${OUT_DIR}/usr/share/pixmaps"

    echo "▸ Copying binary to ${OUT_DIR}/usr/bin/…"
    cp "$BINARY" "${OUT_DIR}/usr/bin/rockbox-gpui"
    chmod +x "${OUT_DIR}/usr/bin/rockbox-gpui"

    echo "▸ Copying app icon…"
    cp "${APPICONSET}/256.png" "${OUT_DIR}/usr/share/pixmaps/rockbox-gpui.png"

    echo "▸ Creating desktop entry…"
    cat > "${OUT_DIR}/usr/share/applications/rockbox-gpui.desktop" <<DESKTOP
[Desktop Entry]
Name=Rockbox GPUI
Comment=Modern audio player with multi-room support
Exec=rockbox-gpui
Icon=rockbox-gpui
Terminal=false
Type=Application
Categories=AudioVideo;Audio;Player;
DESKTOP

    echo ""
    echo "✓  Linux package created in ${OUT_DIR}/"
    echo "   Binary: ${OUT_DIR}/usr/bin/rockbox-gpui"
    echo "   Desktop: ${OUT_DIR}/usr/share/applications/rockbox-gpui.desktop"
    echo "   Icon:   ${OUT_DIR}/usr/share/pixmaps/rockbox-gpui.png"
    exit 0
fi

# ── macOS Package ─────────────────────────────────────────────────────────────

# ── 2. Build .icns from the Xcode project's appiconset PNGs ──────────────────
echo "▸ Building app icon from Xcode appiconset…"
ICONSET="${OUT_DIR}/AppIcon.iconset"
rm -rf "$ICONSET" && mkdir -p "$ICONSET"

# sips resizes a source PNG to each required iconset size.
# The appiconset already has 16/32/64/128/256/512/1024 — map them directly.
resize_png() {
    local src="${APPICONSET}/${1}.png" name="$2" size="$3"
    sips -z "$size" "$size" "$src" --out "${ICONSET}/${name}.png" &>/dev/null
}

resize_png 16   icon_16x16        16
resize_png 32   icon_16x16@2x     32
resize_png 32   icon_32x32        32
resize_png 64   icon_32x32@2x     64
resize_png 128  icon_128x128      128
resize_png 256  icon_128x128@2x   256
resize_png 256  icon_256x256      256
resize_png 512  icon_256x256@2x   512
resize_png 512  icon_512x512      512
resize_png 1024 icon_512x512@2x   1024

iconutil -c icns "$ICONSET" -o "${OUT_DIR}/AppIcon.icns"
rm -rf "$ICONSET"

# ── 3. Assemble .app bundle ───────────────────────────────────────────────────
echo "▸ Assembling ${APP_BUNDLE}…"
rm -rf "$APP_BUNDLE"
mkdir -p "${APP_BUNDLE}/Contents/MacOS"
mkdir -p "${APP_BUNDLE}/Contents/Resources"

cp "$BINARY" "${APP_BUNDLE}/Contents/MacOS/${APP_NAME}"
cp "${OUT_DIR}/AppIcon.icns" "${APP_BUNDLE}/Contents/Resources/AppIcon.icns"

cat > "${APP_BUNDLE}/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>      <string>${APP_NAME}</string>
    <key>CFBundleIdentifier</key>      <string>${BUNDLE_ID}</string>
    <key>CFBundleName</key>            <string>${APP_NAME}</string>
    <key>CFBundleDisplayName</key>     <string>${APP_NAME}</string>
    <key>CFBundleIconFile</key>        <string>AppIcon</string>
    <key>CFBundleVersion</key>         <string>${VERSION}</string>
    <key>CFBundleShortVersionString</key> <string>${VERSION}</string>
    <key>CFBundlePackageType</key>     <string>APPL</string>
    <key>NSHighResolutionCapable</key> <true/>
    <key>LSMinimumSystemVersion</key>  <string>13.0</string>
    <key>NSSupportsAutomaticGraphicsSwitching</key> <true/>
</dict>
</plist>
PLIST

# ── 4. Ad-hoc code sign (allows Gatekeeper to run without a paid dev account) ─
echo "▸ Signing ad-hoc…"
codesign --force --deep --sign - "$APP_BUNDLE"

# ── 5. Build DMG ──────────────────────────────────────────────────────────────
echo "▸ Creating DMG…"
rm -rf "$DMG_STAGING" "$DMG_OUT"
mkdir -p "$DMG_STAGING"
cp -r "$APP_BUNDLE" "$DMG_STAGING/"
ln -s /Applications "${DMG_STAGING}/Applications"

hdiutil create \
    -volname "${APP_NAME}" \
    -srcfolder "$DMG_STAGING" \
    -ov \
    -format UDZO \
    "$DMG_OUT"

rm -rf "$DMG_STAGING" "${OUT_DIR}/AppIcon.icns"

echo ""
echo "✓  ${DMG_OUT}"
