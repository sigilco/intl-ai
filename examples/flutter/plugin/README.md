# intl_ai_flutter

A [`build_runner`](https://pub.dev/packages/build_runner) builder for `intl-ai`. Runs `intl-ai fill` during `dart run build_runner build` so your locale files are translated before the Flutter app is bundled.

## Installation

```bash
flutter pub add --dev intl_ai_flutter
```

Make sure the `intl-ai` CLI is on your `PATH`:

```bash
curl -fsSL https://intl-ai.pages.dev/install.sh | sh
```

## Configure `build.yaml`

Add the builder to your app's `build.yaml`:

```yaml
targets:
  $default:
    builders:
      intl_ai_flutter|intl_ai:
        enabled: true
        options:
          configPath: "intl-ai.config.json"
          executable: "intl-ai"
          verbose: false
```

## Create `intl-ai.config.json`

```json
{
  "$schema": "https://intl-ai.pages.dev/schema/v1.json",
  "defaultLocale": "en",
  "locales": ["en", "es"],
  "localeDir": "assets/locales",
  "model": "openai/gpt-4o-mini",
  "apiKey": "${OPENAI_API_KEY}",
  "baseURL": "https://api.openai.com/v1",
  "maxRetries": 3
}
```

## Run build_runner

```bash
flutter pub run build_runner build --delete-conflicting-outputs
```

The builder invokes:

```bash
intl-ai fill --config intl-ai.config.json
```

## Builder options

| Option       | Type     | Default               | Description                                                                                              |
| ------------ | -------- | --------------------- | -------------------------------------------------------------------------------------------------------- |
| `configPath` | `String` | `intl-ai.config.json` | Path to your JSON config, relative to the package root.                                                  |
| `executable` | `String` | `intl-ai`             | Command used to invoke the CLI. Use `npx intl-ai` or `bunx intl-ai` if you do not want a global install. |
| `verbose`    | `bool`   | `false`               | Forward CLI output to the build runner log.                                                              |

## How it works

The builder is registered on the `$package$` asset and writes an `intl-ai.done` marker file. When `build_runner` runs, it invokes the `intl-ai` CLI once per package. Add `intl-ai.done` to your `.gitignore` — it is only used to track build completion.

## Requirements

- Dart SDK 3.0+
- `intl-ai` CLI on `PATH`
- An `intl-ai.config.json` file and locale directory

## Runtime usage

Load the generated JSON files with `flutter_gen`, `easy_localization`, `i18n_extension`, or any other i18n library. The builder only writes translations; it does not impose a runtime API.
