---
title: i18n libraries
---

# i18n libraries

intl-ai generates translation files at build time. You choose the runtime i18n library. This page lists the supported options and their ICU compatibility.

## ICU MessageFormat

Most modern i18n libraries speak ICU MessageFormat. The main exceptions are i18next (uses `&#123;&#123;var&#125;&#125;` and plural suffixes) and vue-i18n (uses its own syntax by default). Pick a library that fits your app's needs.

## Library compatibility

| Library | Native ICU | Setup | Best for |
|---|---|---|---|
| react-intl | Yes | None | React apps that need ICU |
| @formatjs/intl | Yes | None | Framework-agnostic ICU runtime |
| lingui | Yes | Build macro | Compile-time safety, any framework |
| i18next-icu | Yes (via plugin) | Install `i18next-icu` | Existing i18next apps migrating to ICU |
| vue-i18n + intl-messageformat | Yes (via custom compiler) | Custom `messageCompiler` | Vue apps that need ICU |
| svelte-i18n | Partial | Manual formatter calls | Svelte apps |
| @cookbook/solid-intl | Yes | None | Solid apps that need ICU |
| @lit/localize | Partial | Community workaround | Web components |
| typesafe-i18n | No | N/A | Type-safe i18n without ICU |
| rosetta | No | N/A | Tiny footprint (298 bytes) |
| @solid-primitives/i18n | No | N/A | Minimal Solid i18n |

## Configuration

Set `processor: "icu"` in your `intl-ai.config.ts` if your target library uses ICU MessageFormat. Without it, intl-ai uses the default processor which preserves literal `&#123;&#123;var&#125;&#125;` placeholders for i18next.

```ts
// intl-ai.config.ts
export default {
  // ... your model setup
  defaultLocale: "en",
  locales: ["en", "es", "fr"],
  localeDir: "./locales",
  processor: "icu", // omit for i18next's {{var}} style
};
```

## Per-library guides

- [Next.js](/guide/next-js)
- [Vue (vue-i18n)](/guide/vue-i18n)
- [i18next](/guide/i18next)

## Choosing a library

| Need | Use |
|---|---|
| React + ICU | `react-intl` |
| Vue + ICU | `vue-i18n` with `intl-messageformat` compiler |
| i18next syntax (no ICU) | `i18next` + `react-i18next` |
| Compile-time type safety | `lingui` or `typesafe-i18n` |
| Smallest bundle | `rosetta` |
