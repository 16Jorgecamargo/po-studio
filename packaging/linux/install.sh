#!/usr/bin/env bash
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
PREFIX="${PREFIX:-$HOME/.local}"

mkdir -p "$PREFIX/bin" "$PREFIX/share/applications" "$PREFIX/share/icons/hicolor/256x256/apps"
install -m 755 "$HERE/po-studio" "$PREFIX/bin/po-studio"
install -m 644 "$HERE/po-studio.png" "$PREFIX/share/icons/hicolor/256x256/apps/po-studio.png"
sed "s|^Exec=.*|Exec=$PREFIX/bin/po-studio|" "$HERE/po-studio.desktop" > "$PREFIX/share/applications/po-studio.desktop"
command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$PREFIX/share/applications" || true

echo "PO Studio instalado em $PREFIX/bin/po-studio"
