#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
GUM="$(mise which -C "$REPO_ROOT" gum)"

"$GUM" style --border rounded --padding "1 2" --foreground 212 -- "AI Agent" "Translating with AI..."
"$GUM" spin --spinner dot --title "Calling the model..." -- sleep 2
"$GUM" style --foreground 244 -- "-> es, fr, de"
echo "hud-ready"
