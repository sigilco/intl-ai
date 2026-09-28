---
title: Community integrations
description: Tier-1 supported shims versus community-supported plugins, and how to build your own.
---

# Community integrations

intl-ai's contract is small: a stack runs `intl-ai fill` at the lifecycle point where translations should exist, and optionally `intl-ai check --fail-on` as a gate. Everything else (which build tool, which i18n library, which language) is interchangeable. That makes community-maintained integrations a natural fit.

## Support tiers

**Tier 1, maintained in this repo** and versioned in lockstep with the binary:

| Integration         | Stack                                        | Shape                                  |
| ------------------- | -------------------------------------------- | -------------------------------------- |
| `intl-ai`           | any                                          | the binary itself                      |
| `@intl-ai/unplugin` | Vite, Webpack, Rollup, esbuild, Rspack, Farm | unplugin adapter, runs on `buildStart` |
| `@intl-ai/next`     | Next.js (webpack and Turbopack)              | `withIntlAi()` `next.config` wrapper   |
| `@intl-ai/expo`     | Expo / React Native                          | config plugin, runs during prebuild    |

Tier-1 shims get released fixes and the same option surface (`fill`, `failOn`, `validate`, `judgeThreshold`, `dev`, `strict`, `config`, `cwd`, `bin`, `args`).

**Community-supported**: everything else, maintained by its author. A Flutter `intl_ai` package on pub.dev, a Gradle task, an Xcode build-phase script, a WordPress or Rails plugin, a Metro wrapper: all are welcome. We review and link them here and in the README, but they are owned by their authors for fixes and releases.

## Building a shim

A good shim does three things:

1. **Locate the binary.** For npm packages, depend on the `intl-ai` package (it installs the platform binary on postinstall) and resolve it like [`@intl-ai/unplugin` does](https://github.com/sigilco/intl-ai/blob/develop/packages/unplugin/src/run.ts): `bin` option, `INTL_AI_BIN` env, the npm package, then `PATH`. Outside npm, document the install channels (install script, brew, mise) and resolve the binary the same way.
2. **Spawn at the right lifecycle.** `intl-ai fill` where translations should exist (build start, prebuild, asset pipeline), plus `intl-ai check --fail-on <kinds>` if the shim acts as a build gate.
3. **Expose the quality controls.** Map the CLI's quality surface to your platform's conventions: the `[fill].validate` gate, `judge` threshold via `--judge-threshold`, `--fail-on` severities, and a dev opt-out. Do not reimplement validation in the shim; always delegate to the binary.

Prefer to translate formats other than JSON/YAML (ARB, `.strings`, `.xml`, PO)? That is a locale-format adapter in the Rust core, not a shim. Open an issue first; the adapter layer is still being designed (see the roadmap).

## Getting listed

Open a PR that adds your integration to this page and the README's community section, with a one-line description, install command, and repo link. Requirements: open source, maintained (recent commits), and the shim delegates to the `intl-ai` binary rather than reimplementing fill or checks.
