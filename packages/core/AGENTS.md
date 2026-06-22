# packages/core — Agent Context

Core translation engine for intl-ai. Exports config loading, missing translation detection, batch translation, and lockfile management.

---

## Source Layout

| File/Dir                         | Purpose                                                                                        |
| -------------------------------- | ---------------------------------------------------------------------------------------------- | --- | -------------------------------------- |
| `src/config.ts`                  | `loadConfig()` — reads `intl-ai.config.ts                                                      | .js | .intl-airc`via`jiti`, validates schema |
| `src/formats.ts`                 | Format detectors (JSON, YAML) and loaders                                                      |
| `src/processor.ts`               | `findMissingTranslations()` — walks locale dir, detects stale keys, returns batch              |
| `src/engine/fill.ts`             | `translateBatch()` — calls AI provider, formats results                                        |
| `src/engine/diff.ts`             | Diff utilities for human-editable text display                                                 |
| `src/lockfile.ts`                | `LockfileManager` — reads/writes `intl-ai.lock.json`, tracks `sourceHash`, `origin`, staleness |
| `src/types.ts`                   | `IntlAiConfig`, `LockfileEntry`, `TranslationResult`                                           |
| `src/index.ts`                   | Public exports                                                                                 |
| `src/__mocks__/mock-provider.ts` | `MockAiProvider` for tests — never use real APIs in unit tests                                 |

---

## Key Interfaces

### IntlAiConfig

```typescript
{
  provider: string; // "openai" | "anthropic" | ...
  model: string; // "gpt-4" | "claude-3-sonnet" | ...
  apiKey: string;
  localeDir: string; // path to locale files
  baseLocale: string; // "en"
  languages: string[]; // ["es", "fr", ...]
  formats?: "json" | "yaml" | "auto";
}
```

### LockfileEntry

```typescript
{
  sourceHash: string; // SHA-1 of source text
  translation: string;
  origin: "ai" | "human";
  timestamp: number;
}
```

---

## Core Data Flow

```
loadConfig()
  ↓ validates config, searches for .ts/.js/.intl-airc
findMissingTranslations()
  ↓ walks locale dir, compares sourceHash against lockfile
  ↓ returns batch of {key, source, missing_langs}
translateBatch()
  ↓ calls AI provider (Vercel AI SDK)
  ↓ returns {language: translation} map
LockfileManager.save()
  ↓ writes intl-ai.lock.json with sourceHash, translation, origin="ai"
```

---

## Staleness Detection

Each lockfile entry has a `sourceHash` (SHA-1 of source text). When `findMissingTranslations()` runs:

- If source text changed → hash mismatch → entry is stale → re-translate
- If `origin: "human"` → skip unless `--force` flag passed
- If `origin: "ai"` → always re-translate on staleness

---

## Test Patterns

Colocated tests: `*.test.ts` next to source files.

**Fixtures:**

```typescript
import { createTempDir } from "./__fixtures__/temp-dir";

const tmpDir = createTempDir();
// write config + locale files, then test against tmpDir
```

**Mocking AI:**

```typescript
import { MockAiProvider } from './__mocks__/mock-provider';

vi.mock("@anthropic-ai/sdk", () => ({
  Anthropic: vi.fn(() => ({ ... }))
}));
// or use mock-provider for simpler tests
```

Never call real APIs in unit tests. Use `MockAiProvider` or `vi.mock()`.

---

## Gotchas

1. **`readJsonFile` returns `{}` on missing file** — always check `Object.keys(data).length` before assuming file exists
2. **Dot-notation key paths** — keys like `common.greeting` are stored as `{"common": {"greeting": "..."}}` in JSON; processor flattens/unflattens automatically
3. **`jiti` caches modules** — if a config file is edited mid-session, it may not reload; this is by design (perf optimization)
4. **Lockfile is authoritative** — if lockfile is deleted, staleness detection is lost and all keys re-translate on next run
