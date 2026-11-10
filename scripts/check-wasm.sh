#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
expected="$(scripts/wasm-hash.sh)"
actual="$(cat web/src/wasm/source.hash 2>/dev/null || true)"
if [ "$expected" != "$actual" ]; then
  echo "web/src/wasm está desatualizado. Rode scripts/build-wasm.sh e faça commit do resultado."
  exit 1
fi
echo "web/src/wasm em dia com crates/."
