#!/usr/bin/env bash
# Builds the wasm bindings for the demo: cargo release build for
# wasm32-unknown-unknown, then wasm-bindgen web-target glue into pkg/.
set -euo pipefail
cd "$(dirname "$0")/../.."

cargo build --release -p intl-ai-wasm --target wasm32-unknown-unknown
wasm-bindgen --target web --out-dir demo/wasm/pkg \
  target/wasm32-unknown-unknown/release/intl_ai_wasm.wasm

echo "built demo/wasm/pkg/ (serve demo/wasm/ over http, e.g. python3 -m http.server)"
