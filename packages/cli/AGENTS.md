# packages/cli — Agent Context

Command-line interface for intl-ai. Two commands: `intl-ai fill` (translate missing keys) and `intl-ai check` (validate translations).

---

## Architecture

| File/Dir                | Purpose                                                                             |
| ----------------------- | ----------------------------------------------------------------------------------- |
| `src/index.ts`          | CLI entry — sets up `cleye` with name, version, and both commands                   |
| `src/logger.ts`         | `configureLogger(silent)` bootstrap + shared `logger` instance (logtape)            |
| `src/commands/fill.ts`  | `fillCommand` (cleye) + exported `runFillCommand` for testing                       |
| `src/commands/check.ts` | `checkCommand` (cleye) + exported `runCheckCommand` for testing                     |
| `src/config/loader.ts`  | `loadConfig()` — loads `intl-ai.config.ts` (via `jiti`) or `.json` (via `readFile`) |

---

## Commands

### `intl-ai fill`

Translates missing keys.

**Flags (camelCase, cleye convention):**

- `--locale <lang>` — only fill specified language (default: all)
- `--force` — re-translate human-edited entries (default: false)
- `--dry-run` → `dryRun` — preview changes without writing lockfile (default: false)
- `--silent` — suppress progress output (default: false)

**Exit codes:**

- `0` — success
- `1` — config or translation error

### `intl-ai check`

Validates translation state.

**Exit codes:**

- `0` — all translations present and valid
- `10` — stale entries or missing translations
- `1` — config error

---

## Logging

All user-facing output goes through the logtape logger (`["intl-ai", "cli"]`).
`configureLogger(silent)` in `src/logger.ts` sets up the console sink with ANSI color formatting.
Raw progress lines (`\r`, per-locale counts) are still written directly to `process.stdout`/`stderr`
because they are deliberate inline terminal writes, not structured log events.

---

## Test Patterns

Call the exported standalone functions directly — do not invoke the cleye command object:

```typescript
import { runCheckCommand } from "../commands/check";
import { runFillCommand } from "../commands/fill";

await runCheckCommand({ config: "intl-ai.config.json" });
await runFillCommand({ config: "x", silent: true, force: false, dryRun: false });
```

Check command shape via `command.options.flags`, not `command.args`:

```typescript
expect(fillCommand.options.name).toBe("fill");
expect(fillCommand.options.flags?.dryRun?.type).toBe(Boolean);
```
