#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
wasm-pack build crates/po-wasm --release --target web --out-dir ../../web/src/wasm --out-name po_wasm --no-pack
rm -f web/src/wasm/.gitignore
scripts/wasm-hash.sh > web/src/wasm/source.hash
echo "wasm atualizado: $(cat web/src/wasm/source.hash)"
