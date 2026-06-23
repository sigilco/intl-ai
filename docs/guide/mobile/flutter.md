# Flutter

The Flutter integration is provided as a self-contained [`build_runner`](https://pub.dev/packages/build_runner) builder in [`examples/flutter/plugin`](https://github.com/sigilco/intl-ai/tree/main/examples/flutter/plugin). It runs `intl-ai fill` during `flutter pub run build_runner build` and has zero runtime overhead because all translations are written to disk before the Flutter app is bundled.

## Copy the builder

Copy `examples/flutter/plugin/` into your Flutter project (for example, to `./tools/intl_ai_builder/`). The builder is not published as a package, so you own and can customize the code.

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

Update the `import` path in `build.yaml` to match where you copied the builder, for example:

```yaml
builders:
  intl_ai:
    import: "package:my_app/tools/intl_ai_builder/lib/builder.dart"
```

## Create `intl-ai.config.json`

```json
{
  "$schema": "https://www.schemastore.org/intl-ai.json",
  "defaultLocale": "en",
  "locales": ["en", "es"],
  "localeDir": "assets/locales",
  "model": "your-provider/your-model",
  "apiKey": "${OPENAI_API_KEY}",
  "baseURL": "https://api.openai.com/v1",
  "maxRetries": 3
}
```

## Run build_runner

```bash
flutter pub get
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

The builder is registered on the `$package$` asset and writes an `intl-ai.done` marker file. When `build_runner` runs, it invokes the `intl-ai` CLI once per package. Add `intl-ai.done` to your `.gitignore`; it is only used to track build completion.

## Runtime usage

Load the generated JSON files with your preferred i18n library (`easy_localization`, `i18n_extension`, `flutter_gen`, etc.). The builder only writes translations; it does not impose a runtime API.

## Requirements

- Dart SDK 3.0+
- `intl-ai` CLI on `PATH`
- An `intl-ai.config.json` file and locale directory

## Example

See [`examples/flutter`](https://github.com/sigilco/intl-ai/tree/main/examples/flutter) for a complete working app.
