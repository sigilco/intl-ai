# Expo Example

Reference Expo app showing how to wire the local intl-ai config plugin into `app.json`.

## Plugin code

The plugin lives in `examples/expo/plugin/`. It is a self-contained Expo config plugin that invokes `npx intl-ai fill` during `expo prebuild`. Copy it into your own project and adapt as needed — it is not published as a package.

## Usage

1. Install dependencies:

   ```bash
   pnpm install
   cd examples/expo
   pnpm ios     # or pnpm android
   ```

2. Set `OPENAI_API_KEY` in your environment.

3. Run prebuild to trigger the translation step:
   ```bash
   pnpm prebuild
   ```

The plugin will invoke `npx intl-ai fill --config intl-ai.config.json` before the native build.

## Tests

```bash
pnpm test:plugin
```
