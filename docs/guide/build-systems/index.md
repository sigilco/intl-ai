---
title: Build systems
description: intl-ai supports Vite, Webpack, Rollup, esbuild, Rspack, Rolldown, and Farm via unplugin. Pick your bundler.
---

# Build systems

intl-ai runs at build time via `@intl-ai/unplugin`, a thin shim that spawns the `intl-ai` binary (installed automatically as a dependency; see [Getting started](/guide/getting-started/) for other install channels and `intl-ai.toml` setup). It supports every major bundler through [unjs/unplugin](https://github.com/unjs/unplugin). Choose your bundler below.

Plugin options mirror the CLI's quality controls: `validate` (fill-time gate), `judgeThreshold`, `failOn` (`check --fail-on` severities), `dev`, and `strict`. See the [package README](https://www.npmjs.com/package/@intl-ai/unplugin) for the full table.

- [Vite](/guide/build-systems/vite) - Modern, fast build tool
- [Webpack](/guide/build-systems/webpack) - Industry standard bundler
- [Rollup](/guide/build-systems/rollup) - Flexible module bundler
- [esbuild](/guide/build-systems/esbuild) - Extremely fast JavaScript bundler
- [Rspack](/guide/build-systems/rspack) - Rust-based, webpack-compatible bundler
- [Rolldown](/guide/build-systems/rolldown) - Rust-powered Rollup-compatible bundler
- [Farm](/guide/build-systems/farm) - Rust-based web build tool
- [Next.js](/guide/build-systems/next-js) - React framework with Turbopack bridge

If you use a framework, wire up the bundler here and pair it with an i18n library from [i18n libraries](/guide/i18n-libraries/).
