---
title: Getting Started
description: Set up intl-ai in minutes. Install the binary, configure your provider, and translate.
---

# Getting Started

intl-ai is a single binary that translates the locale files your i18n library already reads. This guide gets it running in minutes.

## Install the binary

Pick the channel that matches your workflow.

::: tabs

== tab "install script"

```sh
curl -fsSL https://intl-ai.illo.fyi/install.sh | bash
```

== tab "Homebrew"

```sh
brew install sigilco/tap-intl-ai/intl-ai
```

== tab "mise"

```sh
mise use npm:intl-ai@latest
```

== tab "npm"

```sh
npm install -g intl-ai
```

:::

On Windows, use the PowerShell installer:

```powershell
irm https://github.com/sigilco/intl-ai/releases/latest/download/intl-ai-installer.ps1 | iex
```

Or run it without a global install via `npx intl-ai` / `bunx intl-ai`. Verify with `intl-ai --help`.

## 1. Initialize

Run `intl-ai init` at your project root. It writes `intl-ai.toml` and a `cassette.json` replay fixture so you can try `fill` with no provider configured.

Point `locale_dir` at the directory your i18n library reads, for example `locales/`, `messages/`, or `public/locales/`.

## 2. Configure a provider

`intl-ai.toml` needs one provider arm.

OpenAI-compatible HTTP endpoint (any hosted or local server that speaks chat completions):

```toml
[provider]
kind = "http"
model = "your-model"
api_key = "${env:OPENAI_API_KEY}"
# base_url = "https://api.openai.com/v1"  # optional override
```

Or go keyless with a local agent CLI:

```toml
[provider]
kind = "command"
agent = "claude-code" # claude-code | opencode | codex | crush | gemini
```

`api_key` accepts `${env:VAR}` and `${file:PATH}` interpolation, so secrets never sit in the file. Validate with `intl-ai config validate`.

## 3. Fill translations

```bash
intl-ai fill
```

Fill is additive by default: it writes only missing keys and never touches existing values, whether human-written or previously generated. Target locale files appear next to your source, and `intl-ai.lock.d/` shards record each key's origin (`ai` or `human`), source hash, and review state. Commit both.

::: warning Translations are ICU MessageFormat
intl-ai only emits and validates [ICU MessageFormat](https://unicode-org.github.io/icu/userguide/format_parse/messages/) syntax: `{name}` placeholders, `plural`/`select`/`selectordinal` with a required `other` arm, `#` pound. This is a hard constraint, not a setting: it is what lets `check` and the fill-time validation gate catch mangled placeholders. Non-ICU syntax (printf `%s`, mustache `{{x}}`) is out of scope.
:::

## 4. Check

```bash
intl-ai check
```

Check is read-only: it reports missing, stale, invalid, modified, extra, and unreviewed keys. Use `--fail-on missing,stale,invalid` in CI (exit code 10 on findings) and `--format json` for machine-readable output.

## Build tool and framework integrations

intl-ai pairs with the i18n library you already use rather than your bundler: point `locale_dir` at the directory the library reads, then translate in the lifecycle hook your stack provides. For JS/TS stacks the scoped shims do this for you: `@intl-ai/unplugin` (Vite, Webpack, Rollup, esbuild, Rspack, Farm), `@intl-ai/next` for Next.js, and `@intl-ai/expo` for React Native/Expo. On other stacks, run `intl-ai fill` directly (an npm `prebuild` script, a Gradle or Xcode build phase, `flutter gen-l10n` ordering, and so on).

See [Build systems](/guide/build-systems/) and the per-library guides under [i18n libraries](/guide/i18n-libraries/) for concrete recipes.
