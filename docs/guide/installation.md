---
title: Installation
description: Install intl-ai via curl, npm, npx, or brew. Works on macOS, Linux, and any Node.js 22+ environment.
---

# Installation

Choose the install method that fits your platform and workflow.

## Install script (macOS / Linux)

Download a pre-built binary from GitHub Releases:

```bash
curl -fsSL https://intl-ai.pages.dev/install.sh | bash
```

Override the install directory or version with environment variables:

```bash
export INTL_AI_INSTALL_DIR="$HOME/.local/bin"
export INTL_AI_VERSION="v0.3.0"
curl -fsSL https://intl-ai.pages.dev/install.sh | bash
```

## npm / npx (any platform with Node.js 22+)

Run without installing:

```bash
npx @intl-ai/cli fill
```

Or add to your project:

```bash
npm install -D @intl-ai/cli
pnpm add -D @intl-ai/cli
bun add -D @intl-ai/cli
```

## Homebrew (macOS / Linux)

The tap formula is populated automatically as part of the release, so `brew install sigilco/tap-intl-ai/intl-ai` works immediately after each release.

```bash
brew tap sigilco/tap-intl-ai
brew install intl-ai
brew upgrade intl-ai
```

## mise (version pinning)

Add to your project's `.mise.toml`:

```toml
[tools]
intl-ai = "0.3.0"
```

Then run:

```bash
mise install
```

Note: until the registry entry is merged into mise's official registry, use `mise use npm:intl-ai@0.3.0` or add `packaging/mise/intl-ai.toml` from this repo to your custom registry.

## Verify

```bash
intl-ai --help
intl-ai fill --help
```

## Next steps

- [Set up your AI model](/guide/ai-model)
- [Get started with a bundler](/guide/getting-started)
