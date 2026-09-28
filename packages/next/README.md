# @intl-ai/next

[![npm](https://img.shields.io/npm/v/@intl-ai/next?style=flat-square)](https://www.npmjs.com/package/@intl-ai/next)

Next.js integration for [intl-ai](https://github.com/sigilco/intl-ai): wraps your `next.config` so the `intl-ai` binary translates locale files before the build, on webpack and Turbopack alike.

This package is a thin shim: all translation logic lives in the `intl-ai` binary (installed automatically as a dependency). Your project needs an `intl-ai.toml` config; see the [getting started guide](https://intl-ai.pages.dev/guide/getting-started/).

## Install

```bash
npm install -D @intl-ai/next
```

## Usage

```ts
// next.config.ts
import { withIntlAi } from "@intl-ai/next";

export default withIntlAi({
  // your usual Next.js config
});
```

Options for the translation step go in the second argument:

```ts
export default withIntlAi(nextConfig, {
  failOn: ["missing", "invalid"],
  dev: false, // skip during `next dev`
});
```

## Options

Same surface as [`@intl-ai/unplugin`](https://www.npmjs.com/package/@intl-ai/unplugin#options): `fill`, `failOn`, `validate`, `judgeThreshold`, `dev`, `config`, `cwd`, `bin`, `args`.

## License

[Apache-2.0](../../LICENSE)
