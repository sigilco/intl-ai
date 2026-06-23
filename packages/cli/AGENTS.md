# packages/cli — Agent Context

Command-line interface for intl-ai. Two commands: `intl-ai fill` (translate missing keys) and `intl-ai check` (validate translations).

---

## Architecture

| File/Dir                | Purpose                                                                             |
| ----------------------- | ----------------------------------------------------------------------------------- |
| `src/index.ts`          | CLI entry, argument parsing, command routing                                        |
| `src/commands/fill.ts`  | `fill` command — loads config, runs translation, outputs progress                   |
| `src/commands/check.ts` | `check` command — validates lockfile, exits 1 if issues found                       |
| `src/config/loader.ts`  | `loadConfig()` — loads `intl-ai.config.ts` (via `jiti`) or `.json` (via `readFile`) |
| `src/utils/progress.ts` | `createProgressReporter()` — terminal output, respects `--silent`                   |

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

Mock the config loader and `@intl-ai/api` — don't import real implementations:

```typescript
vi.mock("../config/loader", () => ({
  loadConfig: vi.fn(async () => mockConfig),
}));

vi.mock("@intl-ai/api", () => ({
  runFill: vi.fn(async () => mockResult),
}));
```

Never run real translations in unit tests. Use mock provider.

---

## Example: Testing `fill` Command

```typescript
import { fillCommand } from "../commands/fill";

it("translates missing keys", async () => {
  await fillCommand.run({ args: { config: "x", silent: true, force: false, "dry-run": false } });
  expect(runFill).toHaveBeenCalled();
});
```
