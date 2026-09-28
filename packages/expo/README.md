# @intl-ai/expo

Expo config plugin that runs the `intl-ai` binary during `expo prebuild`, `eas build`, and `expo run:*` — a thin shim; all translation logic lives in the binary.

## Install

```bash
pnpm add -D @intl-ai/expo
```

The `intl-ai` npm package comes along as a dependency and downloads the platform binary on install. To use a binary you installed yourself (brew, mise, install script), set `INTL_AI_BIN` or pass `bin`.

## Usage

```jsonc
// app.json
{
  "expo": {
    "plugins": ["@intl-ai/expo"],
  },
}
```

With options:

```jsonc
{
  "expo": {
    "plugins": [
      [
        "@intl-ai/expo",
        {
          "failOn": ["missing", "invalid"],
          "judgeThreshold": 0.9,
        },
      ],
    ],
  },
}
```

The plugin registers on both platforms and runs the pipeline once during prebuild, with your app's project root as the working directory.

## Options

| Option           | Type                  | Default  | Description                                      |
| ---------------- | --------------------- | -------- | ------------------------------------------------ |
| `fill`           | `boolean`             | `true`   | Run `intl-ai fill`                               |
| `failOn`         | `string \| string[]`  | —        | Run `intl-ai check --fail-on <kinds>` after fill |
| `validate`       | `boolean \| string[]` | `true`   | Fill-time validation gate (`[fill].validate`)    |
| `judgeThreshold` | `number`              | —        | Judge score threshold inside the fill gate       |
| `dev`            | `boolean`             | `true`   | `false` skips when `NODE_ENV !== "production"`   |
| `strict`         | `boolean`             | `true`   | `false` warns instead of failing the prebuild    |
| `config`         | `string`              | —        | Path to `intl-ai.toml`/`.json`/`.yaml`           |
| `cwd`            | `string`              | app root | Working directory for the binary                 |
| `bin`            | `string`              | —        | Explicit path to the `intl-ai` binary            |
| `args`           | `string[]`            | —        | Extra args appended to `intl-ai fill`            |

Binary resolution order: `bin` option → `INTL_AI_BIN` env → the `intl-ai` npm package → `PATH`.

## License

Apache-2.0
