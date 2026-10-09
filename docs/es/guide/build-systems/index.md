---
title: Sistemas de compilación
description: intl-ai admite Vite, Webpack, Rollup, esbuild, Rspack, Rolldown y Farm mediante unplugin. Elija su empaquetador.
---

# Sistemas de compilación

intl-ai se ejecuta en tiempo de compilación mediante `@intl-ai/unplugin`, un delgado shim que genera el binario `intl-ai` (instalado automáticamente como una dependencia; consulte [Getting started](/es/guide/getting-started/) para otros canales de instalación y la configuración de `intl-ai.toml`). Admite todos los empaquetadores principales a través de [unjs/unplugin](https://github.com/unjs/unplugin). Elija su empaquetador a continuación.

Las opciones del plugin reflejan los controles de calidad de la CLI: `validate` (puerta de tiempo de llenado), `judgeThreshold`, `failOn` (`check --fail-on` severidades), `dev`, y `strict`. Consulte el [package README](https://www.npmjs.com/package/@intl-ai/unplugin) para la tabla completa.

- [Vite](/es/guide/build-systems/vite) - Herramienta de compilación moderna y rápida
- [Webpack](/es/guide/build-systems/webpack) - Empaquetador estándar de la industria
- [Rollup](/es/guide/build-systems/rollup) - Empaquetador de módulos flexible
- [esbuild](/es/guide/build-systems/esbuild) - Empaquetador de JavaScript extremadamente rápido
- [Rspack](/es/guide/build-systems/rspack) - Empaquetador compatible con webpack basado en Rust
- [Rolldown](/es/guide/build-systems/rolldown) - Empaquetador compatible con Rollup impulsado por Rust
- [Farm](/es/guide/build-systems/farm) - Herramienta de compilación web basada en Rust
- [Next.js](/es/guide/build-systems/next-js) - Marco de React con puente Turbopack

Si usa un framework, conecte el empaquetador aquí y combínelo con una biblioteca i18n de [i18n libraries](/es/guide/i18n-libraries/).
