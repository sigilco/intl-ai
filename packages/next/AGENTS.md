# packages/next — Agent Context

Next.js integration layer. Provides `withIntlAi()` wrapper that hooks Webpack (for webpack builds) and Turbopack (for Next.js 15+ dev/build). No changes to app code — just wrap the config.

---

## Architecture

| File/Dir                            | Purpose                                                                |
| ----------------------------------- | ---------------------------------------------------------------------- |
| `src/index.ts`                      | `withIntlAi(nextConfig)` — HOF + inline webpack plugin, registers Turbopack loader |
| `src/next-loader.ts`               | Turbopack loader — path resolution, dual CJS/ESM strategy              |
| `src/__mocks__/mock-next-config.ts` | Mock Next.js config for tests                                          |

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

1. Registers webpack plugin for `next build` + `next dev`
2. Registers Turbopack loader for Turbopack builds (Next.js 15+)
3. Translation happens at build time, zero runtime overhead

---

## Known Duplication: runFill() Implementation

⚠️ **Known Issue**: `runFill()` is re-implemented locally in `src/index.ts` instead of imported from `@intl-ai/core/engine/fill.ts`.

If you fix a translation bug, **check both files**:

- `packages/core/src/engine/fill.ts`
- `packages/next/src/index.ts`

Ideally, the Next.js layer would import `runFill` from core, but current Turbopack constraints require a local copy. This is on the backlog for refactoring.

---

## Loader Path Resolution

The Turbopack loader uses a dual CJS/ESM strategy:

- CJS context (Webpack): resolve via `require.resolve()`
- ESM context (Turbopack): resolve via `import.meta.url`

This is intentional — Turbopack and Webpack have different module resolution semantics. Do not simplify to a single strategy without testing both build modes.

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
