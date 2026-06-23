# @intl-ai/cli

[![npm](https://img.shields.io/npm/v/@intl-ai/cli?style=flat-square)](https://www.npmjs.com/package/@intl-ai/cli)

CLI for AI-powered i18n translation.

## Install

```bash
npm install -D @intl-ai/cli
```

Requires `intl-ai.config.{ts,js,intl-airc}` at your project root.

## Usage

### Fill missing translations

```bash
# Fill all missing translations
npx intl-ai fill

# Fill a specific language only
npx intl-ai fill --locale es

# Preview changes without writing
npx intl-ai fill --dry-run

# Re-translate even human-edited entries
npx intl-ai fill --force
```

### Check translation state

```bash
npx intl-ai check
```

Exits with code 1 if there are stale or missing translations.

## Commands

| Command | Description                                     |
| ------- | ----------------------------------------------- |
| `fill`  | Translate missing keys using AI                 |
| `check` | Validate translation state and exit with status |

[Documentation](https://intl-ai.pages.dev) · [Report an issue](https://github.com/espetro/intl-ai/issues)
