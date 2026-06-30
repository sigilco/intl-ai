---
title: Observability with hooks
description: Monitor the intl-ai translation pipeline with hooks. Track batch progress, retries, and failures.
---

# Observability with hooks

TranslationHook is an optional callback interface on the config object. It gives you visibility into every step of the AI translation pipeline: when a batch is sent, when it succeeds, and when it fails.

Hooks are available only in TypeScript config files (`intl-ai.config.ts`). JSON config does not support function values.

## The three callbacks

Each callback receives a single info object. All three are optional; implement only the ones you need.

### `onRequest`

Fires before each batch is sent to the AI provider.

```typescript
onRequest?: (info: {
  provider: string;   // e.g. "openai", "anthropic"
  model: string;      // e.g. "gpt-4o-mini"
  locale: string;     // target locale code
  entryCount: number; // number of keys in this batch
}) => void;
```

Use this to log which locales and batch sizes are being processed.

### `onSuccess`

Fires after a batch completes successfully.

```typescript
onSuccess?: (info: {
  provider: string;
  model: string;
  locale: string;
  results: TranslationResult[]; // per-key results
  durationMs: number;           // wall-clock time for this request
}) => void;
```

Each `TranslationResult` has:

| Field        | Type      | Description                                    |
| ------------ | --------- | ---------------------------------------------- |
| `key`        | `string`  | The locale key being translated.               |
| `translated` | `string?` | The translated text, if translation succeeded. |
| `success`    | `boolean` | Whether this individual key was translated.    |
| `error`      | `string?` | Error message if this key failed validation.   |

### `onError`

Fires when a batch exhausts all retry attempts without a successful response.

```typescript
onError?: (info: {
  provider: string;
  model: string;
  locale: string;
  error: string;   // human-readable error from the last attempt
  attempt: number; // the retry attempt that failed (equals maxRetries)
}) => void;
```

Note that `onError` fires only after all retries are exhausted. Individual retry attempts within a batch are handled internally and do not trigger `onError`.

## Setting up a hook

Pass the `hook` property on your config object:

```typescript
import type { IntlAiConfig } from "@intl-ai/api";
import type { TranslationResult } from "@intl-ai/api/internal";

const hook = {
  onRequest(info) {
    console.log(
      `[${info.locale}] sending ${info.entryCount} keys to ${info.provider}/${info.model}`,
    );
  },

  onSuccess(info) {
    const succeeded = info.results.filter((r) => r.success).length;
    console.log(
      `[${info.locale}] translated ${succeeded}/${info.results.length} keys in ${info.durationMs.toFixed(0)}ms`,
    );
  },

  onError(info) {
    console.error(`[${info.locale}] failed after ${info.attempt} attempts: ${info.error}`);
  },
};

const config: IntlAiConfig = {
  provider: "openai",
  model: "gpt-4o-mini",
  apiKey: "${OPENAI_API_KEY}",
  defaultLocale: "en",
  locales: ["en", "es", "fr"],
  localeDir: "./locales",
  hook,
};

export default config;
```

This produces output like:

```
[es] sending 24 keys to openai/gpt-4o-mini
[es] translated 24/24 keys in 1823ms
[fr] sending 24 keys to openai/gpt-4o-mini
[fr] translated 23/24 keys in 2104ms
```

## Use cases

### Progress logging

The simplest use case is logging translation progress to stdout during builds. This helps you confirm which locales are being processed and how long each takes:

```typescript
onRequest(info) {
  console.log(`Translating ${info.locale} (${info.entryCount} keys)...`);
},
onSuccess(info) {
  console.log(`Done: ${info.locale} (${info.durationMs.toFixed(0)}ms)`);
},
```

### Error tracking

Route translation failures to an error monitoring service. Because `onError` receives the locale and error message, you can tag and group issues:

```typescript
import * as Sentry from "@sentry/node";

onError(info) {
  Sentry.captureMessage("Translation batch failed", {
    level: "warning",
    tags: { locale: info.locale, provider: info.provider, model: info.model },
    extra: { error: info.error, attempt: info.attempt },
  });
},
```

### Duration tracking

Use `onSuccess` to track how long each locale takes. This helps you identify slow locales or provider issues:

```typescript
const timings: Record<string, number> = {};

onSuccess(info) {
  timings[info.locale] = (timings[info.locale] ?? 0) + info.durationMs;
},
```

### Per-key validation reporting

The `results` array in `onSuccess` includes per-key status. Use this to surface validation failures that do not bubble up to `onError`:

```typescript
onSuccess(info) {
  const failed = info.results.filter((r) => !r.success);
  for (const r of failed) {
    console.warn(`[${info.locale}] key "${r.key}": ${r.error}`);
  }
},
```

## Sync-only callbacks

All three callbacks are synchronous. They must return `void` and cannot be async. The translation engine calls them inline during the batch loop, so blocking or async work would stall the pipeline.

If you need to perform async work like sending telemetry or writing to a remote service, queue the data in the callback and process it outside the translation pipeline:

```typescript
const telemetryQueue: Array<Record<string, unknown>> = [];

const hook = {
  onSuccess(info) {
    telemetryQueue.push({
      locale: info.locale,
      provider: info.provider,
      durationMs: info.durationMs,
      keyCount: info.results.length,
    });
  },
};

// After translation completes
await runFill(config);
await flushTelemetry(telemetryQueue);
```
