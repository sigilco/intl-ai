#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
GUM="$(mise which -C "$REPO_ROOT" gum)"

"$GUM" style --border rounded --padding "1 2" --foreground 212 --bold -- \
  "npm install -D @intl-ai/unplugin" "github.com/sigilco/intl-ai"
echo "closing-ready"
