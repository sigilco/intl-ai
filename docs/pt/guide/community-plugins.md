---
title: Integrações comunitárias
description: Tier-1 supported shims versus community-supported plugins, and how to build your own.
---

# Integrações comunitárias

intl-ai's contract is small: a stack runs `intl-ai fill` at the lifecycle point where translations should exist, and optionally `intl-ai check --fail-on` as a gate. Everything else (which build tool, which i18n library, which language) is interchangeable. That makes community-maintained integrations a natural fit.

## Níveis de suporte

**Tier 1, mantido neste repositório** e versionado em conjunto com o binário:

| Integração         | Stack                                        | Shape                                  |
| ------------------- | -------------------------------------------- | -------------------------------------- |
| `intl-ai`           | any                                          | o próprio binário                      |
| `@intl-ai/unplugin` | Vite, Webpack, Rollup, esbuild, Rspack, Farm | adaptador unplugin, executa no `buildStart` |
| `@intl-ai/next`     | Next.js (webpack e Turbopack)              | `withIntlAi()` wrapper do `next.config`   |
| `@intl-ai/expo`     | Expo / React Native                          | plugin de configuração, executa durante o prebuild    |

Os shims de Tier-1 recebem correções lançadas e a mesma superfície de opções (`fill`, `failOn`, `validate`, `judgeThreshold`, `dev`, `strict`, `config`, `cwd`, `bin`, `args`).

**Community-supported**: everything else, maintained by its author. A Flutter `intl_ai` package on pub.dev, a Gradle task, an Xcode build-phase script, a WordPress or Rails plugin, a Metro wrapper: all are welcome. We review and link them here and in the README, but they are owned by their authors for fixes and releases.

## Construir um shim

Um bom shim faz três coisas:

1. **Locate the binary.** For npm packages, depend on the `intl-ai` package (it installs the platform binary on postinstall) and resolve it like [`@intl-ai/unplugin` does](https://github.com/sigilco/intl-ai/blob/develop/packages/unplugin/src/run.ts): `bin` option, `INTL_AI_BIN` env, the npm package, then `PATH`. Outside npm, document the install channels (install script, brew, mise) and resolve the binary the same way.
2. **Spawn no ciclo de vida correto.** `intl-ai fill` onde as traduções devem existir (início da construção, pré-construção, pipeline de assets), mais `intl-ai check --fail-on <kinds>` se o shim atuar como uma barreira de construção.
3. **Exponha os controles de qualidade.** Mapeie a superfície de qualidade da CLI para as convenções da sua plataforma: a porta `[fill].validate`, limite de `judge` via `--judge-threshold`, severidades `--fail-on` e uma opção de desenvolvedor para desativar. Não reimplante a validação no shim; sempre delegue para o binário.

Prefere traduzir formatos outros que JSON/YAML (ARB, `.strings`, `.xml`, PO)? Registe um plugin de formato de execução via `[[formats]]` em `intl-ai.toml`, sem Rust necessário (veja [Format plugins](/pt/guide/format-plugins/)). Para distribuir um plugin reutilizável para a comunidade de um formato, abra um issue ou PR para o linkar aqui.

## Getting listed

Open a PR that adds your integration to this page and the README's community section, with a one-line description, install command, and repo link. Requirements: open source, maintained (recent commits), and the shim delegates to the `intl-ai` binary rather than reimplementing fill or checks.
