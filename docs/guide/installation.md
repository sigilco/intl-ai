---
title: Installation
---

# Installation

Choose the install method that fits your platform and workflow.

## Install script (macOS / Linux)

Download a pre-built binary from GitHub Releases:

```bash
curl -fsSL https://sigilco.github.io/intl-ai/install.sh | bash
```

Override the install directory or version with environment variables:

```bash
export INTL_AI_INSTALL_DIR="$HOME/.local/bin"
export INTL_AI_VERSION="v0.2.0"
curl -fsSL https://sigilco.github.io/intl-ai/install.sh | bash
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

```bash
brew install sigilco/tap/intl-ai
brew upgrade intl-ai
```

## mise (version pinning)

Add to your project's `.mise.toml`:

```toml
[tools]
intl-ai = "0.2.0"
```

Then run:

```bash
mise install
```

## Verify

```bash
intl-ai --help
intl-ai fill --help
```

## Next steps

- [Set up your AI model](/guide/ai-model)
- [Get started with a bundler](/guide/getting-started)
