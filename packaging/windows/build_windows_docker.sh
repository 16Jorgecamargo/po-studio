#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

VERSION="$(grep -m1 '^version' Cargo.toml | cut -d '"' -f2)"
TARGET="x86_64-pc-windows-gnu"
mkdir -p dist

docker run --rm -v "$ROOT":/src -w /src -e CARGO_TARGET_DIR=/src/target/windows rust:1-bookworm bash -c "
    apt-get update -qq && apt-get install -y -qq gcc-mingw-w64-x86-64 binutils-mingw-w64-x86-64 >/dev/null &&
    rustup target add $TARGET >/dev/null &&
    cargo build --release --target $TARGET &&
    mkdir -p target/release &&
    cp target/windows/$TARGET/release/po-studio.exe target/release/po-studio.exe
"

cp target/release/po-studio.exe "dist/PO-Studio-$VERSION-windows-portable.exe"

docker run --rm --platform linux/amd64 -v "$ROOT":/work -w /work/packaging/windows amake/innosetup "/DAppVersion=$VERSION" po_studio.iss

ls -1 dist | grep windows
