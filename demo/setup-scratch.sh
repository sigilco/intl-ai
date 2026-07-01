#!/usr/bin/env bash
set -euo pipefail
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CLI_BIN="$REPO_ROOT/packages/cli/dist/index.mjs"
intl-ai() { node "$CLI_BIN" "$@"; }
export -f intl-ai

DEMO_DIR="$(mktemp -d -t intl-ai-demo)"
mkdir -p "$DEMO_DIR/locales"
cp "$REPO_ROOT/examples/vite/locales/en.json" "$DEMO_DIR/locales/en.json"
printf '{}' > "$DEMO_DIR/locales/es.json"
printf '{}' > "$DEMO_DIR/locales/fr.json"
printf '{}' > "$DEMO_DIR/locales/de.json"
printf '{"version":1,"entries":{}}' > "$DEMO_DIR/locales/intl-ai.lock.json"
cp "$REPO_ROOT/demo/intl-ai.demo.config.json" "$DEMO_DIR/intl-ai.config.json"
cd "$DEMO_DIR"

# This script is sourced into the live tape shell, not executed as a
# subprocess — `set -euo pipefail` above would otherwise leak into the
# interactive session and kill it the moment a command like
# `intl-ai check` exits non-zero on purpose (missing translations).
set +euo pipefail
