# intl-ai Vite Example

A minimal example demonstrating the use of `@intl-ai/unplugin` with Vite 8.

## Setup

```bash
npm install
```

## Development

```bash
npm run dev
```

## Build

```bash
npm run build
```

## Configuration

- **vite.config.ts**: Vite configuration with intl-ai plugin
- **intl-ai.config.ts**: intl-ai configuration using LM Studio with Qwen 3.5 4B Instruct
- **locales/en.json**: English locale strings

## Usage

The plugin provides `getLocale()` and `t()` functions for accessing translations at runtime.
