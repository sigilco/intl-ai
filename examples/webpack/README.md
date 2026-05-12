# Webpack Example

This example demonstrates how to use the `@intl-ai/unplugin` with Webpack 5 (webpack-cli 7).

## Setup

Install dependencies:

```bash
pnpm install
```

## Build

Build the project:

```bash
pnpm build
```

## Development

Start the development server:

```bash
pnpm dev
```

## Configuration

The example uses:

- **Webpack 5 (webpack-cli 7)** as the bundler
- **TypeScript** for type safety
- **intl-ai** plugin for internationalization
- **LM Studio** as the AI backend (running on `http://127.0.0.1:1234/v1`)

The `intl-ai.config.ts` file configures the plugin with:

- Model: `qwen3.5-4b-instruct`
- Default locale: `en`
- Supported locales: `en`, `de`, `es`, `fr`
- Locale files location: `./locales`
