#!/usr/bin/env bash
# Rebuild the wasm bundle into demo/web/src/lib/pkg.
# The compiler is pinned so the committed bundle is byte-reproducible:
# the CI freshness gate regenerates it and diffs. Bump TOOLCHAIN together
# with a regenerated pkg, and keep the wasm-bindgen CLI on the version in
# Cargo.lock.
set -euo pipefail
cd "$(dirname "$0")/../../.."

TOOLCHAIN=1.97.1
rustup toolchain install "$TOOLCHAIN" --target wasm32-unknown-unknown

cargo +"$TOOLCHAIN" build --release -p intl-ai-wasm --target wasm32-unknown-unknown
wasm-bindgen --target web --out-dir demo/web/src/lib/pkg \
  target/wasm32-unknown-unknown/release/intl_ai_wasm.wasm

echo "built demo/web/src/lib/pkg/"
