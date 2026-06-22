---
title: Migrating from @intl-ai/core
---

# Migrating from `@intl-ai/core`

`@intl-ai/core` is deprecated and will be removed in v1.0. Import from `@intl-ai/api` instead.

## Package changes

| Before          | After          |
| --------------- | -------------- |
| `@intl-ai/core` | `@intl-ai/api` |

## Import changes

```typescript
// Before
import { runFill } from "@intl-ai/core";

// After
import { runFill } from "@intl-ai/api";
```

## Config loading

`@intl-ai/core` exported `loadConfig()` for TypeScript/JavaScript config files. In `@intl-ai/api`, config loading is split by use case.

### For JSON configs (CLI, CI, mobile build scripts)

Use the JSON config schema and converter from `@intl-ai/api/internal`:

```typescript
import { readFile } from "node:fs/promises";
import { IntlAiJsonConfigSchema, jsonConfigToIntlAiConfig } from "@intl-ai/api/internal";

const raw = JSON.parse(await readFile("intl-ai.config.json", "utf-8"));
const config = jsonConfigToIntlAiConfig(IntlAiJsonConfigSchema.parse(raw));
```

### For TypeScript configs with a live AI SDK model

Keep using your bundler plugin or import the config directly:

```typescript
import { runFill } from "@intl-ai/api";
import config from "./intl-ai.config";

await runFill(config);
```

## Runtime-agnostic core

`@intl-ai/api` has no Node.js-only dependencies. You can run `runFill` in any JavaScript runtime, or shell out to the CLI from any platform.
