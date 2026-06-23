# intl-ai

> AI-powered i18n translation plugin for all bundlers

[![GitHub stars](https://img.shields.io/github/stars/sigilco/intl-ai?style=social)](https://github.com/sigilco/intl-ai)

[Documentation](https://intl-ai.pages.dev) · [Report an issue](https://github.com/sigilco/intl-ai/issues)

Automatically translate your app with AI. intl-ai hooks into your build pipeline to generate and update translations on every build.

## Install

**Vite / Rollup / Webpack / esbuild / Rspack**

```bash
npm install -D @intl-ai/unplugin
```

**Next.js**

```bash
npm install -D @intl-ai/next
```

**CLI only**

```bash
npm install -D @intl-ai/cli
```

## Create a config file

intl-ai reads `intl-ai.config.ts` (or `.json`) at your project root.

### JSON config (recommended for non-Node runtimes)

```json
{
  "$schema": "https://www.schemastore.org/intl-ai.json",
  "defaultLocale": "en",
  "locales": ["en", "es", "fr"],
  "localeDir": "./locales",
  "model": "your-provider/your-model",
  "apiKey": "${OPENAI_API_KEY}"
}
```

### TypeScript config (when you need a live AI SDK model instance)

```ts
// intl-ai.config.ts
import { createOpenAICompatible } from "@ai-sdk/openai-compatible";

const openai = createOpenAICompatible({
  name: "openai",
  baseURL: "https://api.openai.com/v1",
  apiKey: process.env.OPENAI_API_KEY,
});

export default {
  model: openai("your-model-name"),
  defaultLocale: "en",
  locales: ["en", "es", "fr"],
  localeDir: "./locales",
};
```

See [Configuration](https://intl-ai.pages.dev/guide/configuration) for the full schema.

## Usage

### Vite

```ts
// vite.config.ts
import { defineConfig } from "vite";
import IntlAi from "@intl-ai/unplugin/vite";

export default defineConfig({
  plugins: [IntlAi()],
});
```

### Next.js

```ts
// next.config.ts
import withIntlAi from "@intl-ai/next";

export default withIntlAi({
  // your existing Next.js config
  reactStrictMode: true,
});
```

### CLI

```bash
npx intl-ai fill
```

## Packages

| Package                                                                | Description              | Registry |
| ---------------------------------------------------------------------- | ------------------------ | -------- |
| [`@intl-ai/api`](https://www.npmjs.com/package/@intl-ai/api)           | Runtime-agnostic core    | npm      |
| [`@intl-ai/unplugin`](https://www.npmjs.com/package/@intl-ai/unplugin) | Universal bundler plugin | npm      |
| [`@intl-ai/next`](https://www.npmjs.com/package/@intl-ai/next)         | Next.js integration      | npm      |
| [`@intl-ai/cli`](https://www.npmjs.com/package/@intl-ai/cli)           | `intl-ai fill` / `check` | npm      |

## License

[MIT](LICENSE)
