---
title: Internos
description: Cómo está estructurado intl-ai. Los crates núcleo de Rust, el binario y los shims de npm.
---

# Internos

## Distribución del espacio de trabajo

El motor de traducción es un workspace de Rust; los paquetes de npm son shims delgados que generan el binario.

| Paquete                                   | Propósito                                                                                                                                  |
| ----------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| `crates/intl-ai-core`                     | Carga de configuración (`intl-ai.toml`/`.json`/`.yaml`), interpolación, fragmentos de lockfile (`intl-ai.lock.d/`), procedencia.                         |
| `crates/intl-ai-formats`                  | Formatos de archivo de configuración regional: JSON y YAML, aplanar/desaplanar, caché de estadísticas.                                                                       |
| `crates/intl-ai-providers`                | Transportes: `http` (compatible con OpenAI), `command` (agentes locales), `replay` (cintas). Contrato de prompt + reintentos.                       |
| `crates/intl-ai-checks`                   | Marco de validación: `icu`, `placeholder-parity`, `dialect`, `judge`, especificaciones declarativas, `exec` comprobaciones, la caché de hallazgos incrementales. |
| `crates/intl-ai-cli`                      | El binario `intl-ai`: `fill`, `check`, `mark`, `review`, `status`, `lockfile`, `config`.                                                 |
| `packages/unplugin` (`@intl-ai/unplugin`) | Shim de empaquetador (vite, webpack, rollup, esbuild, rspack, rolldown, farm, bun). Ejecuta la canalización en `buildStart`.                           |
| `packages/next` (`@intl-ai/next`)         | `withIntlAi()` envoltorio de configuración. Ejecuta la canalización durante la evaluación de `next.config`, antes de webpack o Turbopack.                           |
| `packages/expo` (`@intl-ai/expo`)         | Plugin de configuración de Expo. Ejecuta el pipeline durante `expo prebuild`/`eas build`.                                                                |
| `intl-ai` (npm)                           | Entrega del binario: la instalación posterior descarga el tarball de la plataforma desde GitHub Releases. Los shims lo resuelven mediante `createRequire`.              |

## Canalización de lanzamiento

`cargo dist` genera el flujo de trabajo de lanzamiento: la etiqueta `vX.Y.Z` produce los tarballs de plataforma, `intl-ai-installer.sh`/`.ps1`, una fórmula de Homebrew, la publicación del wrapper de npm, y las attestaciones de GitHub. Consulte [Versioning in AGENTS.md](https://github.com/sigilco/intl-ai/blob/develop/AGENTS.md#versioning).

## Esquema de configuración

El contrato de configuración se genera a partir de los tipos de Rust y se confirma en `docs/public/schema/intl-ai.schema.json`; `docs/public/schema/v1.json` es la copia publicada. CI falla ante la deriva, así que edita los tipos y vuelve a generar con `intl-ai config schema`.
