#!/usr/bin/env bash
# Builds the Svelte wasm demo (demo/web) into the docmd output at /demo/.
# The vite build imports the wasm-bindgen bundle in demo/web/src/lib/pkg,
# which is committed to the repo; if it is missing we rebuild it when the
# toolchain (cargo wasm32 target + wasm-bindgen) is available, else fail.
set -euo pipefail
cd "$(dirname "$0")/.."

if [ ! -f demo/web/src/lib/pkg/intl_ai_wasm.js ]; then
  if command -v cargo >/dev/null 2>&1 && command -v wasm-bindgen >/dev/null 2>&1; then
    demo/web/scripts/sync-wasm.sh
  else
    echo "error: demo/web/src/lib/pkg missing; run demo/web/scripts/sync-wasm.sh" >&2
    exit 1
  fi
fi

pnpm --filter @intl-ai/demo-web build

rm -rf docs/.vitepress/dist/demo
mkdir -p docs/.vitepress/dist/demo
cp -r demo/web/dist/. docs/.vitepress/dist/demo/
echo "demo packaged at docs/.vitepress/dist/demo/"
