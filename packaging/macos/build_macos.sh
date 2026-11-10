#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

APP_NAME="PO Studio"
BUNDLE_ID="app.postudio.desktop"
VERSION="$(grep -m1 '^version' Cargo.toml | cut -d '"' -f2)"
OUT="$ROOT/dist"
TARGETS="${TARGETS:-$(rustc -vV | sed -n 's/^host: //p')}"

rm -rf "$OUT/macos"
mkdir -p "$OUT/macos"

BINARIES=()
for target in $TARGETS; do
    rustup target add "$target" >/dev/null 2>&1 || true
    cargo build --release --target "$target"
    BINARIES+=("target/$target/release/po-studio")
done

APP="$OUT/macos/$APP_NAME.app"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"

if [ "${#BINARIES[@]}" -gt 1 ]; then
    lipo -create -output "$APP/Contents/MacOS/po-studio" "${BINARIES[@]}"
else
    cp "${BINARIES[0]}" "$APP/Contents/MacOS/po-studio"
fi
chmod +x "$APP/Contents/MacOS/po-studio"
cp assets/icon/icon.icns "$APP/Contents/Resources/AppIcon.icns"

cat > "$APP/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key><string>$APP_NAME</string>
    <key>CFBundleDisplayName</key><string>$APP_NAME</string>
    <key>CFBundleIdentifier</key><string>$BUNDLE_ID</string>
    <key>CFBundleVersion</key><string>$VERSION</string>
    <key>CFBundleShortVersionString</key><string>$VERSION</string>
    <key>CFBundleExecutable</key><string>po-studio</string>
    <key>CFBundleIconFile</key><string>AppIcon</string>
    <key>CFBundlePackageType</key><string>APPL</string>
    <key>CFBundleDevelopmentRegion</key><string>pt-BR</string>
    <key>LSMinimumSystemVersion</key><string>11.0</string>
    <key>LSApplicationCategoryType</key><string>public.app-category.education</string>
    <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
PLIST

codesign --force --deep --sign - "$APP" >/dev/null 2>&1 || true

STAGE="$OUT/macos/dmg"
mkdir -p "$STAGE"
cp -R "$APP" "$STAGE/"
ln -s /Applications "$STAGE/Applications"
DMG="$OUT/PO-Studio-$VERSION-macos.dmg"
rm -f "$DMG"
hdiutil create -volname "$APP_NAME" -srcfolder "$STAGE" -ov -format UDZO "$DMG" >/dev/null
rm -rf "$STAGE"

echo "$DMG"
