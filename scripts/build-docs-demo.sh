#!/usr/bin/env bash
# Copies the wasm demo into the docmd output so it is served at /demo/.
# The wasm bundle needs cargo (wasm32 target) + wasm-bindgen; where either
# is missing the static shell still ships, and the page reports the error.
set -euo pipefail
cd "$(dirname "$0")/.."

if command -v cargo >/dev/null 2>&1 && command -v wasm-bindgen >/dev/null 2>&1; then
  demo/wasm/build.sh
else
  echo "warning: cargo or wasm-bindgen missing; copying demo without pkg/" >&2
fi

mkdir -p docs/.vitepress/dist/demo
cp -r demo/wasm/index.html demo/wasm/main.mjs demo/wasm/mock-provider.mjs \
  demo/wasm/worker.mjs docs/.vitepress/dist/demo/
[ -d demo/wasm/pkg ] && cp -r demo/wasm/pkg docs/.vitepress/dist/demo/ || true
echo "demo packaged at docs/.vitepress/dist/demo/"
