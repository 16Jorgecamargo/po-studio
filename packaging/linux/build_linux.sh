#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

VERSION="$(grep -m1 '^version' Cargo.toml | cut -d '"' -f2)"
ARCH="$(uname -m)"
case "$ARCH" in
    x86_64) DEB_ARCH="amd64" ;;
    aarch64) DEB_ARCH="arm64" ;;
    *) DEB_ARCH="$ARCH" ;;
esac
OUT="$ROOT/dist"
mkdir -p "$OUT"

cargo build --release
BIN="${CARGO_TARGET_DIR:-target}/release/po-studio"

STAGE="$OUT/linux/po-studio_${VERSION}_${DEB_ARCH}"
rm -rf "$OUT/linux"
mkdir -p "$STAGE/DEBIAN" "$STAGE/usr/bin" "$STAGE/usr/share/applications" "$STAGE/usr/share/icons/hicolor/256x256/apps" "$STAGE/usr/share/icons/hicolor/1024x1024/apps"

install -m 755 "$BIN" "$STAGE/usr/bin/po-studio"
install -m 644 packaging/linux/po-studio.desktop "$STAGE/usr/share/applications/po-studio.desktop"
install -m 644 assets/icon/icon_256.png "$STAGE/usr/share/icons/hicolor/256x256/apps/po-studio.png"
install -m 644 assets/icon/icon.png "$STAGE/usr/share/icons/hicolor/1024x1024/apps/po-studio.png"

cat > "$STAGE/DEBIAN/control" <<CONTROL
Package: po-studio
Version: $VERSION
Section: education
Priority: optional
Architecture: $DEB_ARCH
Depends: libc6, libgl1, libxkbcommon0
Maintainer: PO Studio
Description: PO Studio - pesquisa operacional
 Programacao linear, inteira, transporte, redes, projetos, filas e jogos.
CONTROL

if command -v dpkg-deb >/dev/null 2>&1; then
    dpkg-deb --build --root-owner-group "$STAGE" "$OUT/po-studio_${VERSION}_${DEB_ARCH}.deb"
fi

PORTABLE="$OUT/linux/PO-Studio-$VERSION-linux-$ARCH"
mkdir -p "$PORTABLE"
cp "$BIN" "$PORTABLE/po-studio"
cp packaging/linux/po-studio.desktop "$PORTABLE/"
cp assets/icon/icon_256.png "$PORTABLE/po-studio.png"
cp packaging/linux/install.sh "$PORTABLE/install.sh"
chmod +x "$PORTABLE/install.sh" "$PORTABLE/po-studio"
tar -C "$OUT/linux" -czf "$OUT/PO-Studio-$VERSION-linux-$ARCH.tar.gz" "$(basename "$PORTABLE")"

ls -1 "$OUT" | grep -E '\.(deb|tar\.gz)$'
