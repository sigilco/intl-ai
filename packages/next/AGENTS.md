# packages/next — Agent Context

Next.js integration for the `intl-ai` binary. `withIntlAi(nextConfig, options)` wraps `next.config` so `intl-ai fill` (+ the `check --fail-on` gate) runs while Next evaluates the config, before either webpack or Turbopack starts. A thin shim: all translation logic lives in the binary.

---

## Architecture

| File/Dir            | Purpose                                                                      |
| ------------------- | ---------------------------------------------------------------------------- |
| `src/index.ts`      | `withIntlAi()`: returns an async phase-aware config function                 |
| `src/index.test.ts` | Shim tests against a stub `intl-ai` executable (records argv, controls exit) |

The spawn/binary-resolution logic is shared with `@intl-ai/unplugin` via its exported `runIntlAiPipeline` and `IntlAiPluginOptions`; do not duplicate it here.

---

## How It Works

```ts
// next.config.ts
import { withIntlAi } from "@intl-ai/next";
export default withIntlAi(nextConfig, { failOn: ["missing"] });
```

- `withIntlAi` returns `(phase, ctx) => Promise<NextConfig>`, the supported async-function form of `next.config`, so it runs on both webpack and Turbopack without bundler hooks.
- Phase gating (`next/constants`): runs on `PHASE_PRODUCTION_BUILD` and `PHASE_EXPORT`, on `PHASE_DEVELOPMENT_SERVER` unless `dev: false`, never on `PHASE_PRODUCTION_SERVER`.
- Option surface is `IntlAiPluginOptions` from `@intl-ai/unplugin` (`fill`, `failOn`, `validate`, `judgeThreshold`, `dev`, `strict`, `config`, `cwd`, `bin`, `args`). Nonzero exits throw (`strict: false` warns instead).

---

## Test Patterns

- Tests write a stub `intl-ai-stub.cjs` (shebang + chmod 755) to a tmpdir and pass it via the `bin` option; it appends argv to `INTL_AI_STUB_LOG` and exits with `INTL_AI_STUB_EXIT`.
- Drive the returned function with real phase constants from `next/constants`; never boot a real Next.js app in unit tests.
