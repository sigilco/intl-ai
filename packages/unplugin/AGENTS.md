# packages/unplugin — Agent Context

Universal bundler plugin via [unplugin 3](https://github.com/unjs/unplugin). Provides adapters for Vite, Rollup, Webpack, esbuild, Rspack, and others without code duplication.

---

## Architecture

| File/Dir                        | Purpose                                                                                           |
| ------------------------------- | ------------------------------------------------------------------------------------------------- |
| `src/index.ts`                  | `unpluginFactory()` — hooks `buildStart`, calls inlined `loadConfig()` + `runFill()`, thin facade |
| `src/config.ts`                 | Inlined `loadConfig()` — searches `intl-ai.config.ts` then `intl-ai.config.json`                  |
| `src/types.ts`                  | Re-exports of `IntlAiConfig` (from `@intl-ai/api`) and `Lockfile` (from `@intl-ai/api/internal`)  |
| `src/__mocks__/mock-bundler.ts` | Mock bundler context for tests                                                                    |

---

## How It Works

`unplugin` provides a single factory function that generates plugin code for every bundler:

```typescript
export default unpluginFactory((options) => {
  return {
    name: "intl-ai",
    async buildStart() {
      const config = await loadConfig();
      await runFill(config);
    },
  };
});

// Auto-generates: vite(), webpack(), esbuild(), rollup(), etc.
```

---

## Why It's Thin

All translation logic stays in `@intl-ai/api`. The unplugin layer only:

- Exposes bundler hooks (Webpack `emit`, Vite `resolveId`, etc.)
- Calls the inlined `loadConfig()` and `runFill()` from `@intl-ai/api`
- Emits the updated lockfile back to disk

This keeps bundler-specific code minimal and ensures translation behavior is consistent across all bundlers.

---

## Adding a New Bundler Adapter

1. **Check unplugin docs** for the adapter interface (e.g., `UnpluginOptions`)
2. **Add adapter export** in `src/index.ts`:
   ```typescript
   export const newBundler = unpluginFactory(...).newBundler;
   ```
3. **Test** with a reference app in `examples/`

No changes needed to the factory itself — unplugin handles the rest.

---

## Test Patterns

Use `mock-bundler.ts` to simulate bundler context:

```typescript
import { mockBundlerContext } from "./__mocks__/mock-bundler";

const ctx = mockBundlerContext();
// ctx has emit(), resolveId(), etc.
```

Never load real bundler configs in unit tests.
