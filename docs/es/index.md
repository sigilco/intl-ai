---
title: intl-ai
description: Traducción i18n impulsada por IA en tiempo de compilación. Un binario, cualquier empaquetador, cualquier modelo. Cero dependencias de runtime.
---

# intl-ai

Traducción i18n de IA en tiempo de compilación. Cero runtime, cualquier empaquetador, cualquier modelo.

[Comenzar](/es/guide/getting-started/) · [GitHub](https://github.com/sigilco/intl-ai) · [Patrocinar](https://buy.polar.sh/polar_cl_Mv1gdlG7bw3I70EC9IHtfeSHJj4PEKvA7JAUz23CFhj)

::: info
La <a href="/demo/" data-spa-ignore>demostración en vivo</a> ejecuta el pipeline de llenado en WebAssembly: arrastra un archivo de locale, observa cómo las claves fluyen, no se necesita instalación.
:::

## Por qué intl-ai

- **Cualquier empaquetador**: Vite, Webpack, Rollup, esbuild, Rspack, Rolldown, Farm y Next.js. Un plugin, todas las herramientas de compilación.
- **Cualquier modelo**: OpenAI, Anthropic, Ollama o cualquier API compatible con OpenAI. Configura una vez, traduce en todas partes.
- **Cero runtime**: Toda la traducción ocurre en tiempo de compilación. Sin SDK de IA en producción, sin latencia, sin bloqueo de proveedor.
- **Multiplataforma**: Integraciones nativas de Expo, Flutter, SwiftUI y .NET. Envía traducciones a cualquier compilación móvil o de escritorio.

## Instalación rápida

::: tabs

== tab "Homebrew"

```sh
brew install sigilco/tap-intl-ai/intl-ai
```

== tab "mise"

```sh
mise use npm:intl-ai@latest
```

== tab "install script"

```sh
curl -fsSL https://intl-ai.illo.fyi/install.sh | sh
```

:::
