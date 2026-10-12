---
title: Internos
description: Como o intl-ai está estruturado. Crates principais em Rust, o binário e os shims npm.
---

# Internos

## Layout do espaço de trabalho

O motor de tradução é um workspace Rust; os pacotes npm são interfaces simples que iniciam o binário.

| Pacote                                    | Propósito                                                                                                                                                     |
| ----------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `crates/intl-ai-core`                     | Carregamento de configuração (`intl-ai.toml`/`.json`/`.yaml`), interpolação, fragmentos de ficheiro de bloqueio (`intl-ai.lock.d/`), proveniência.            |
| `crates/intl-ai-formats`                  | Formatos de ficheiro de locale: JSON e YAML, achatamento/desachatamento, cache de estatísticas.                                                               |
| `crates/intl-ai-providers`                | Transportes: `http` (compatível com OpenAI), `command` (agentes locais), `replay` (cassettes). Contrato de prompt + novas tentativas.                         |
| `crates/intl-ai-checks`                   | Framework de validação: `icu`, `placeholder-parity`, `dialect`, `judge`, especificações declarativas, verificações `exec`, cache de descobertas incrementais. |
| `crates/intl-ai-cli`                      | O binário `intl-ai`: `fill`, `check`, `mark`, `review`, `status`, `lockfile`, `config`.                                                                       |
| `packages/unplugin` (`@intl-ai/unplugin`) | Shim do bundler (vite, webpack, rollup, esbuild, rspack, rolldown, farm, bun). Executa o pipeline em `buildStart`.                                            |
| `packages/next` (`@intl-ai/next`)         | Wrapper de configuração `withIntlAi()`. Executa o pipeline durante a avaliação de `next.config`, antes do webpack ou Turbopack.                               |
| `packages/expo` (`@intl-ai/expo`)         | Plugin de configuração do Expo. Executa o pipeline durante `expo prebuild`/`eas build`.                                                                       |
| `intl-ai` (npm)                           | Entrega binária: o postinstall faz o download do tarball de plataforma a partir das Releases do GitHub. Os shims resolvem-no através de `createRequire`.      |

## Pipeline de lançamento

`cargo dist` gera o fluxo de trabalho de lançamento: a tag `vX.Y.Z` produz tarballs de plataforma, `intl-ai-installer.sh`/`.ps1`, uma fórmula homebrew, publicação de wrapper npm e attestations do GitHub. Consulte [Versioning in AGENTS.md](https://github.com/sigilco/intl-ai/blob/develop/AGENTS.md#versioning).

## Esquema de configuração

O contrato de configuração é gerado a partir dos tipos Rust e commitado em `docs/public/schema/intl-ai.schema.json`; `docs/public/schema/v1.json` é a cópia publicada. O CI falha em caso de divergência, por isso edite os tipos e gere-os novamente com `intl-ai config schema`.
