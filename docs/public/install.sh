#!/usr/bin/env bash
# Thin wrapper: delegates to the cargo-dist installer served from GitHub
# Releases, so this URL (intl-ai.pages.dev/install.sh) stays stable.
set -euo pipefail

INSTALLER="https://github.com/sigilco/intl-ai/releases/latest/download/intl-ai-installer.sh"

curl -fsSL "$INSTALLER" | bash
