---
layout: home

hero:
  name: "intl-ai"
  tagline: "AI-powered i18n translation. Zero runtime dependencies."
  actions:
    - theme: brand
      text: Get Started
      link: /guide/getting-started
    - theme: alt
      text: View on GitHub
      link: https://github.com/sigilco/intl-ai

features:
  - title: Bundler Plugins
    details: Universal unplugin for Vite, Webpack, Rollup, esbuild, Rspack, and Next.js.
  - title: Mobile Ready
    details: Native Expo and Flutter example integrations ship translations into mobile builds.
  - title: Standalone Binary
    details: Zero-dependency compiled binaries for CI, servers, and any platform.
  - title: Zero Runtime
    details: Translations happen at build time, with no AI SDK overhead in production.
---

## Quick Install

Install the standalone CLI with a single command on macOS or Linux:

```bash
curl -fsSL https://sigilco.github.io/intl-ai/install.sh | bash
```

Or use it without installing via npm or Bun:

```bash
npx @intl-ai/cli fill
bunx @intl-ai/cli fill
```

Homebrew and mise support are also available; see the [installation guide](/guide/installation) for details.
