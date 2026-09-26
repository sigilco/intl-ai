# packages/next — Agent Context

Next.js integration layer. Provides `withIntlAi()` wrapper that delegates webpack integration to `@intl-ai/unplugin/webpack` and registers a Turbopack loader for Next.js 15+. No changes to app code; just wrap the config.

---

## Architecture

| File/Dir                            | Purpose                                                                                                      |
| ----------------------------------- | ------------------------------------------------------------------------------------------------------------ |
| `src/index.ts`                      | `withIntlAi()` HOF — registers Turbopack loader + delegates webpack integration to @intl-ai/unplugin/webpack |
| `src/next-loader.ts`                | Turbopack loader — path resolution, compiled-JS/TS fallback strategy                                         |
| `src/__mocks__/mock-next-config.ts` | Mock Next.js config for tests                                                                                |

---

## withIntlAi() Usage

```typescript
// next.config.js
import { withIntlAi } from "@intl-ai/next";

export default withIntlAi({
  // ... your Next.js config
  reactStrictMode: true,
});
```

No changes to app code. The wrapper:

1. Registers the `@intl-ai/unplugin/webpack` plugin for `next build` + `next dev`
2. Registers Turbopack loader for Turbopack builds (Next.js 15+)
3. Translation happens at build time, zero runtime overhead

For Next.js 14 with webpack, users should use `@intl-ai/unplugin/webpack` directly with the `webpack` config callback (not `withIntlAi`).

---

## Loader Path Resolution

The Turbopack loader uses a compiled-JS/TS fallback strategy:

- Compiled-JS context: resolve via `require.resolve()`
- TS context (development): resolve via `import.meta.url`

This handles both the published package (compiled to JS) and local development (source .ts files).

---

## Test Patterns

Use `mock-next-config.ts` to simulate Next.js config:

```typescript
import { createMockNextConfig } from "./__mocks__/mock-next-config";

const config = createMockNextConfig({
  /* ... */
});
// config has webpack and turbopack hooks
```

Never test against a real Next.js project in unit tests.
