---
layout: home

hero:
  name: "intl-ai"
  tagline: "AI i18n translation at build time. Zero runtime, any bundler, any model."
  actions:
    - theme: brand
      text: Get Started
      link: /guide/getting-started
    - theme: alt
      text: View on GitHub
      link: https://github.com/sigilco/intl-ai

features:
  - title: Any Bundler
    details: Vite, Webpack, Rollup, esbuild, Rspack, Rolldown, Farm, and Next.js. One plugin, every build tool.
  - title: Any Model
    details: OpenAI, Anthropic, Ollama, or any OpenAI-compatible API. Configure once, translate everywhere.
  - title: Zero Runtime
    details: All translation happens at build time. No AI SDK in production, no latency, no vendor lock-in.
  - title: Cross-Platform
    details: Native Expo, Flutter, SwiftUI, and .NET integrations. Ship translations into any mobile or desktop build.
---

<SponsorsHero />

## Quick Install

Install the standalone CLI with a single command on macOS or Linux:

```bash
curl -fsSL https://intl-ai.pages.dev/install.sh | bash
```

Or use it without installing via npm or Bun:

```bash
npx @intl-ai/cli fill
bunx @intl-ai/cli fill
```

Homebrew and mise support are also available; see the [installation guide](/guide/installation) for details.
