# packages/cli — Agent Context

Command-line interface for intl-ai. Two commands: `intl-ai fill` (translate missing keys) and `intl-ai check` (validate translations).

---

## Architecture

| File/Dir                     | Purpose                                                             |
| ---------------------------- | ------------------------------------------------------------------- |
| `src/index.ts`               | CLI entry, argument parsing, command routing                        |
| `src/commands/fill.ts`       | `fill` command — loads config, runs translation, outputs progress   |
| `src/commands/check.ts`      | `check` command — validates lockfile, exits 1 if issues found       |
| `src/utils/progress.ts`      | `createProgressReporter()` — terminal output, respects `--silent`   |
| `src/__mocks__/mock-core.ts` | Mocks for `loadConfig`, `findMissingTranslations`, `translateBatch` |

---

## Commands

### `intl-ai fill`

Translates missing keys.

**Flags:**

- `--locale <lang>` — only fill specified language (default: all)
- `--force` — re-translate human-edited entries (default: skip)
- `--dry-run` — preview changes without writing lockfile
- `--silent` — suppress progress output

**Exit codes:**

- `0` — success
- `1` — config or translation error

### `intl-ai check`

Validates translation state.

**Exit codes:**

- `0` — all translations present and valid
- `1` — stale entries, missing translations, or config error

---

## Progress Reporting

`createProgressReporter()` outputs to stderr (so stdout stays clean for piping):

```typescript
const reporter = createProgressReporter({ silent: args["--silent"] });

reporter.start("Translating...");
reporter.progress(50);
reporter.success("Done");
// or
reporter.error("Something failed");
```

Respects `--silent` flag — no output if set.

---

## Test Patterns

Mock core functions directly — don't import real implementations:

```typescript
vi.mock("@intl-ai/core", () => ({
  loadConfig: vi.fn(async () => mockConfig),
  findMissingTranslations: vi.fn(async () => mockBatch),
  translateBatch: vi.fn(async () => mockResults),
}));
```

Never run real translations in unit tests. Use mock provider.

---

## Example: Testing `fill` Command

```typescript
import { fill } from "../commands/fill";
import * as core from "@intl-ai/core";

vi.mock("@intl-ai/core");

it("translates missing keys", async () => {
  const result = await fill({
    locale: "es",
    dryRun: false,
    force: false,
    silent: true,
  });

  expect(core.translateBatch).toHaveBeenCalled();
  expect(result.success).toBe(true);
});
```
