#!/usr/bin/env bash
# Rebuild the wasm bundle into demo/web/src/lib/pkg.
set -euo pipefail
cd "$(dirname "$0")/../../.."

cargo build --release -p intl-ai-wasm --target wasm32-unknown-unknown
wasm-bindgen --target web --out-dir demo/web/src/lib/pkg \
  target/wasm32-unknown-unknown/release/intl_ai_wasm.wasm

echo "built demo/web/src/lib/pkg/"
