# Expo

The Expo integration is provided as a self-contained config plugin in [`examples/expo/plugin`](https://github.com/sigilco/intl-ai/tree/main/examples/expo/plugin). It runs `intl-ai fill` during `expo prebuild` and has zero runtime overhead because all translations are written to disk before Metro bundles your app.

## Copy the plugin

Copy `examples/expo/plugin/` into your Expo project (for example, to `./plugins/intl-ai/`). The plugin is not published as an npm package, so you own and can customize the code.

## Configure `app.json`

Reference the local plugin in your `app.json`:

```json
{
  "expo": {
    "plugins": [["./plugins/intl-ai", { "configPath": "intl-ai.config.json" }]]
  }
}
```

## Create `intl-ai.config.json`

```json
{
  "$schema": "https://sigilco.github.io/intl-ai/schema/v1.json",
  "defaultLocale": "en",
  "locales": ["en", "es"],
  "localeDir": "locales",
  "model": "openai/gpt-4o-mini",
  "apiKey": "${OPENAI_API_KEY}",
  "baseURL": "https://api.openai.com/v1",
  "maxRetries": 3
}
```

## Run prebuild

```bash
expo prebuild
```

The plugin invokes:

```bash
npx intl-ai fill --config intl-ai.config.json
```

## Plugin options

| Option            | Type      | Default               | Description                                             |
| ----------------- | --------- | --------------------- | ------------------------------------------------------- |
| `configPath`      | `string`  | `intl-ai.config.json` | Path to your JSON config, relative to the project root. |
| `verbose`         | `boolean` | `false`               | Forward CLI output to the parent process.               |
| `continueOnError` | `boolean` | `false`               | Allow prebuild to continue if translation fails.        |

## Runtime usage

Load the generated JSON files directly with your preferred i18n library (`i18next`, `react-intl`, etc.). The plugin only writes translations; it does not impose a runtime API.

## Example

See [`examples/expo`](https://github.com/sigilco/intl-ai/tree/main/examples/expo) for a complete working app.
