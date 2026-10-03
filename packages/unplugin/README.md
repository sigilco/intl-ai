# @intl-ai/unplugin

[![npm](https://img.shields.io/npm/v/@intl-ai/unplugin?style=flat-square)](https://www.npmjs.com/package/@intl-ai/unplugin)

Bundler plugin for [intl-ai](https://github.com/sigilco/intl-ai): runs the `intl-ai` binary during your build so locale files are translated before your app ships. Works with Vite, Webpack, Rollup, esbuild, Rspack, Rolldown, and Farm via [unplugin](https://github.com/unjs/unplugin).

This package is a thin shim: all translation logic lives in the `intl-ai` binary (installed automatically as a dependency). Your project needs an `intl-ai.toml` config; see the [getting started guide](https://intl-ai.illo.fyi/guide/getting-started/).

## Install

```bash
npm install -D @intl-ai/unplugin
```

## Usage

```ts
// vite.config.ts
import { defineConfig } from "vite";
import intlAi from "@intl-ai/unplugin/vite";

export default defineConfig({
  plugins: [intlAi()],
});
```

Subpath exports exist for each bundler: `@intl-ai/unplugin/{vite,webpack,rollup,esbuild,rspack,rolldown,farm}`.

## Options

| Option           | Type                  | Default    | Description                                                                                                                                                          |
| ---------------- | --------------------- | ---------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `fill`           | `boolean`             | `true`     | Run `intl-ai fill` on build start.                                                                                                                                   |
| `failOn`         | `string \| string[]`  | `[]`       | Finding kinds that fail the build (`missing`, `stale`, `invalid`, `unreviewed`, ...). Maps to `intl-ai check --fail-on`.                                             |
| `validate`       | `boolean \| string[]` | `true`     | Fill-time validation gate. `true` uses your config's `[fill].validate`; a list overrides it (`icu`, `placeholder-parity`, `judge`); `false` maps to `--no-validate`. |
| `judgeThreshold` | `number`              | config     | Score threshold (0..1) for the `judge` check inside the fill gate.                                                                                                   |
| `dev`            | `boolean`             | `true`     | Set `false` to skip the plugin in dev/serve/watch mode.                                                                                                              |
| `config`         | `string`              | discovered | Path to `intl-ai.toml` (or `.json`/`.yaml`).                                                                                                                         |
| `cwd`            | `string`              | project    | Working directory for the `intl-ai` invocation.                                                                                                                      |
| `bin`            | `string`              | resolved   | Explicit path to the `intl-ai` binary. Falls back to the bundled dependency, then `PATH`.                                                                            |
| `args`           | `string[]`            | `[]`       | Extra arguments appended to `intl-ai fill`.                                                                                                                          |

## How it works

On `buildStart` the plugin spawns `intl-ai fill` (plus `intl-ai check` when `failOn` is set) in your project directory and streams its output into the build log. The lockfile (`intl-ai.lock.d/`) keeps AI output additive and review-aware, exactly as a CLI run would.

## License

[Apache-2.0](../../LICENSE)
