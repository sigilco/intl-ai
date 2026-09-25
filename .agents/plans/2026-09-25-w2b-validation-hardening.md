# W2b: validation hardening, incremental check, shift-left gate

Status: proposed, 2026-09-25.
Consolidates four adversarial subagent reports plus two user directives.
Applies on top of W2 (PR #61, branch `feature/v1-w2-checks`).

## 1. Sources

- Shift-left designer: `Gate` in fill, `fill.validate` list, corrective rounds, `Entry.quality` unresolved records.
- Shift-left skeptic: opt-in only, never default-on, only feedback-eligible checks, found two latent check bugs.
- Incremental check design: per-`(locale, key, check_id)` content fingerprints in gitignored `.intl-ai/check-cache.json`.
- API surface review (ran on develop, pre-W2; `invalid` gating findings are already fixed by #61): 8 high-severity, 13 medium, ~15 low findings across fill/lockfile/config/commands.

## 2. Consensus verdicts

All four reports converge on three claims.

1. Validation must be incremental: `check` is O(all keys x all checks) per run, which compounds and burns judge tokens.
2. In-fill correction is opt-in and constrained: never default-on, never dialect/spec/exec in the feedback loop, never withhold the value.
3. The current data flow has real correctness bugs (write ordering, origin/reviewed drift, extends trust boundary) that are prerequisites for trusting any automation on top.

## 3. W2b part A, hardening fixes (ship first)

Everything here is a bug or a fail-open seam; small diffs, no design controversy.

### 3.1 Data-flow correctness

- H1 write ordering: `fill_locale` writes the locale file before the shard; a crash between them misattributes AI text as `origin: human` forever.
  Fix: shard-first ordering; an orphan shard entry degrades to `missing`, which the next fill heals.
- H2 `reviewed` never resets on the human arm: `effective_origin` only compares values for AI entries, and `entry.value` is a frozen snapshot.
  Fix: snapshot file value into `entry.value` at adoption/mark/review; on fill, a human entry whose file value differs from the snapshot gets `reviewed = false` and a refreshed snapshot.
- H3 `mark --origin ai` does not stick: it leaves `entry.value` stale, so the positional rule flips it back next run.
  Fix: `mark --origin ai` also sets `entry.value` to the current file value.
- H4 fail-open pipeline: provider-omitted keys exit 0 and `missing` is not in default `fail_on`.
  Fix: omitted keys become `failures` with `kind: output_truncated` (exit 1); add `missing` to default `fail_on`.
- H8 `set_nested` clobber: writing `nav.home` silently destroys a human scalar `nav`.
  Fix: detect non-object intermediates on the write path; surface as a `clobbered` report field (do not silently delete).
- M3 write discipline: fixed tmp name + unlocked read-modify-write loses concurrent updates.
  Fix: unique tmp names plus a per-shard flock.
- M10 merge markers: unterminated `<<<<<<<` resolves by silently dropping the theirs side.
  Fix: hard-error on marker imbalance.
- M11 severity contract: corrupt shard is soft in fill (exit 1) but hard in check (exit 10).
  Fix: corrupt shard is exit-10 everywhere (fail-closed per the lockfile doctrine); corrupt target file may stay soft.

### 3.2 Trust boundary and input validation

- H5 extends leaks: `${file:}`/`${env:}` interpolation runs over merged config, so a poisoned extends can exfiltrate secrets via `locale_instructions`/`glossary`/`base_url`.
  Fix: refuse `file:`/`env:` expressions in values sourced from non-root configs; gate `base_url` and `agent` presets behind the same non-root restriction as `provider.command`.
- H6 locale IDs used verbatim as filenames: `targets = ["../x"]` writes outside `locale_dir`.
  Fix: validate locale IDs at `config validate` and at `--locale`/`targets` use; reject `/`, `\`, `..`, NUL at minimum.
- H7 diamond extends false-positives as a cycle: `visiting` never pops.
  Fix: recursion stack for true cycles plus a dedup set; org+team layering (the stated use case) is a diamond.
- M12 config gaps: reject `provider.command` + `provider.agent` together; validate `version`; reject `targets = []` and `source in targets`; `--config -` must support `extends` or hard-error on it.

### 3.3 Report/output contract honesty (agent-first)

- M4: `skipped_human` counts AI-owned skips too; `translated == written` is dead; `--include-human` prints no count; `has_issues` means gate-hit not findings-exist; `unreviewed` is a count not a key list.
  Fix: split `skipped_existing`/`skipped_human`, drop or repurpose `translated`, add `regenerated_human` count, rename or supplement `has_issues`, return unreviewed keys.
- M7: `fill --regenerate` with no selector is a whole-locale footgun; require `--keys`/`--stale` scope or `--yes` acknowledgement; `--stale --regenerate` needs `conflicts_with`; add `--format json` to mark/review/unreview/lockfile/init and a JSON error envelope.
- M13: `check` needs `--keys`; unify key selection on the flag form; `--fail-on` needs an explicit empty value; hide `migrate` until implemented.
- M1: `modified` findings can never converge from `check` alone; emit guidance text (and evaluate `check --reconcile` as a follow-up).
- Missing-vs-deleted is inexpressible today (fill resurrects deleted keys); add `mark --absent` or equivalent.
- New commands: `intl-ai status` (inventory counts) is the natural agent-facing read command deferred from 5.6; now justified by the review.

### 3.4 Misc robustness

- M2: skip unchanged locale-file writes (byte-compare like `save_shard`); avoids mtime churn and comment loss on no-op runs (YAML comment loss on real writes is a documented format caveat).
- M5: `discover` must walk ancestors to `.git` boundary.
- M6: `init` scaffold must produce a config that works on first run (empty cassette or command arm), check all four config extensions, count yaml targets.
- M8: add `--no-cache`/`--refresh` for the stat cache; read-only-mount tolerance for cache writes stays best-effort.
- M9: retry honors `Retry-After` and exponential backoff for `rate_limit`; do not retry auth errors; give the stdout-cap error a real `ErrorType`.
- Prompt framing: JSON-escape quoted values in `user_prompt` to stop data-driven injection.
- Ghost keys (`"empty": {}`), flat-dot scalar drops, YAML scalar coercion mangling: document or warn.
- Fill internals: build the chunk key set once (O(batch) not O(batch^2)); cap `prompt_via = "argv"` size; warn on shadowed sibling locale files (`.json` + `.yaml`); `instruction_for` splits on `-` and `_`; populate `written_this_run`; GC shard entries for source-deleted keys; cap/rotate `.intl-ai/report-*.json`; document `INTL_AI_NOW` as test-only.

## 4. W2b part B, incremental check (design ratified)

Verdict: adopt the caching design essentially as proposed.

- `Check` trait gains `cacheable() -> Cacheable` (PerItem | WholeBatch | Never) and required `cache_ctx(&ctx) -> String` for ambient inputs; no defaults, so a new check cannot forget.
- Fingerprint: `sha1(key || source-hash-or-absent-sentinel || sha1(target) || check.id || cache_ctx)`, over in-memory flattened values, never mtimes.
- Storage: `.intl-ai/check-cache.json`, sibling of the stat cache, fail-open load, atomic save, versioned header; GC drops dead keys and unconfigured check ids on save.
- Findings are cached and replayed with `cached: true` on `CheckFinding` (additive JSON field, `(cached)` suffix in human output); errors are never cached.
- exec defaults to WholeBatch fingerprints (one spawn skipped only when the whole locale batch is unchanged); `per_item = true` opt-in on `[[checks]]` entries for item-independent checkers.
- Per-check `cache_ctx` contents: spec = spec text hash; dialect = wordlist `include_str!` hash; exec = protocol version + command + args + cwd + optional `version` field; judge = provider identity (kind + model + base_url + model_params, never api_key) + locale_instruction + `JUDGE_THRESHOLD` + `PROMPT_V`.
- `CHECK_V`/`PROMPT_V` constants gate cache-correctness on logic/prompt changes; AGENTS.md gets the bump rule; binary version is deliberately not hashed.
- Escape hatches: `check --no-cache` (skips read and write), `[check] cache = false`, file deletion always safe.
- Transport resolution becomes lazy: a fully-cached judge run must not demand an API key.
- Interactive detail to keep: `--origin`/`--fail-on`/`--format` never enter fingerprints; filtering stays post-merge.

## 5. W2b part C, shift-left gate in fill (synthesized)

Synthesis of designer + skeptic; mechanism from the designer, constraints from the skeptic.

- Mechanism: `fill()` gains a `gate: &Gate` param (`{ checks: Vec<Box<dyn Check>>, max_rounds: u32 }`); validation runs per-chunk inside `fill_locale` between `translate()` and adoption; `intl-ai-core` still does not depend on `intl-ai-checks`.
- Eligibility is honest: `Check` gains `supports_feedback() -> bool`; only `icu`, `placeholder-parity`, and `judge` return true; dialect/spec/exec are rejected from the gate at config validation with an explanatory error (their findings are not mechanically actionable or not trustworthy as reviewer notes).
- Opt-in surface: `[fill] validate = ["icu", "placeholder-parity"]` in config (names builtin ids or `[[checks]]` labels) plus `fill --validate [list]` and `--no-validate` per-invocation overrides; absent = off everywhere.
  Open call for the user: designer says config-persisted is right (teams want the gate consistently, especially in CI); skeptic says never config-persisted (per-invocation consent for cost). My lean is config-allowed: the Renovate lesson was about destructive overrides, and a gate adds safety rather than removing it.
- Corrective rounds: v1 caps at one corrective round (`refill_rounds` deferred; oscillation risk bounded by construction).
- Feedback framing: failed keys only, findings joined `; ` prefixed `{check}: `, plus `Previous attempt: {value}` (TS parity); frozen feedback block wording unchanged.
- Never withhold: a key that fails its last round is adopted anyway with `origin = ai`, `reviewed = false`, and `Entry.quality.unresolved` recording the findings; `check` keeps flagging it live (provenance, not suppression).
- Report: `LocaleFillResult` gains `refilled` and `unresolved: Vec<CheckFinding>`; exit 1 when a configured gate produces unresolved keys (the gate is opt-in, so no existing CI breaks) — flagged as a user decision below.
- Check-side fixes that are prerequisites: `placeholder-parity` skips keys whose source fails to parse (otherwise it instructs the model to strip real placeholders); `icu` enables `requires_other_clause` so plural-without-`other` (a FormatJS runtime crash) is caught; both fixes apply to `check` itself regardless of the gate.
- Deferred: `fill --invalid`/`check --fix` scoped refill verb (skeptic's Option A; a cheap follow-up once the gate exists); multi-round refills; remote cache for CI warming.

## 6. Sequencing

1. Land the prerequisite check fixes + part A hardening on `feature/v1-w2-checks` (or a stacked branch if #61 should stay clean for merge).
2. Part B incremental cache.
3. Part C fill gate.
   Suggested PR granularity: one `fix` PR for 3.1+3.2 (correctness/security), one for 3.3+3.4 (contract/robustness), one per B and C.

## 7. Decisions (resolved 2026-09-25)

- `fill.validate`: config-persisted opt-in, approved.
- Unresolved-under-gate exit code: exit 1, approved.
- Part A sequencing: lands after #61 merges, on a stacked `feature/v1-w2b-hardening` branch retargeted to develop.
- `fill --invalid`/`check --fix` scoped refill: deferred per the plan, not decided against.
