#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
find crates/po-core crates/po-wasm -type f \( -name '*.rs' -o -name 'Cargo.toml' \) -print0 \
  | LC_ALL=C sort -z \
  | xargs -0 cat \
  | { command -v sha256sum >/dev/null && sha256sum || shasum -a 256; } \
  | cut -d' ' -f1
