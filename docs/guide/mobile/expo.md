---
title: Expo
description: Build-time AI translation for Expo i18n. Translations happen during prebuild, zero runtime.
---

# Expo

`@intl-ai/expo` is a config plugin that runs the `intl-ai` binary while Expo evaluates config plugins during `expo prebuild`, `eas build`, and `expo run:*`. Translations are written to disk before Metro bundles your app, so there is zero runtime overhead.

## Install

```bash
pnpm add -D @intl-ai/expo
```

The `intl-ai` npm package comes along as a dependency and downloads the platform binary on install. To use a binary you installed another way (brew, mise, install script), set `INTL_AI_BIN` or pass `bin`.

## Configure `app.json`

```json
{
  "expo": {
    "plugins": ["@intl-ai/expo"]
  }
}
```

With options:

```json
{
  "expo": {
    "plugins": [
      [
        "@intl-ai/expo",
        {
          "failOn": ["missing", "invalid"],
          "judgeThreshold": 0.9
        }
      ]
    ]
  }
}
```

## Create `intl-ai.toml`

```toml
locale_dir = "locales"
source = "en"
targets = ["es", "fr"]

[provider]
kind = "http"
base_url = "https://api.openai.com/v1"
model = "your-model-name"
api_key = "${OPENAI_API_KEY}"
```

See [Configuration](/guide/configuration/) for the full schema.

## Run prebuild

```bash
expo prebuild
```

The plugin registers on both the ios and android platforms and runs the pipeline once, with your app root as the working directory.

## Options

| Option           | Type                 | Default  | Description                                      |
| ---------------- | -------------------- | -------- | ------------------------------------------------ |
| `fill`           | `boolean`            | `true`   | Run `intl-ai fill`                               |
| `failOn`         | `string \| string[]` | —        | Run `intl-ai check --fail-on <kinds>` after fill |
| `validate`       | `boolean \| string[]`| `true`   | Fill-time validation gate                        |
| `judgeThreshold` | `number`             | —        | Judge score threshold inside the fill gate       |
| `dev`            | `boolean`            | `true`   | `false` skips when `NODE_ENV !== "production"`   |
| `strict`         | `boolean`            | `true`   | `false` warns instead of failing the prebuild    |
| `config`         | `string`             | —        | Path to `intl-ai.toml`/`.json`/`.yaml`           |
| `cwd`            | `string`             | app root | Working directory for the binary                 |
| `bin`            | `string`             | —        | Explicit path to the `intl-ai` binary            |
| `args`           | `string[]`           | —        | Extra args appended to `intl-ai fill`            |

Binary resolution order: `bin` option → `INTL_AI_BIN` env → the `intl-ai` npm package → `PATH`.

## Runtime usage

Load the generated JSON files directly with your preferred i18n library (`i18next`, `react-intl`, etc.). The plugin only writes translations; it does not impose a runtime API.

## Example

See [`examples/expo`](https://github.com/sigilco/intl-ai/tree/main/examples/expo) for a complete working app.
