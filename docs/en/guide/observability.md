---
title: Observability
description: Monitor the intl-ai pipeline. JSON reports, findings kinds, quality scores, and exec checks as custom hooks.
---

# Observability

The `intl-ai` binary emits machine-readable output on every command. There is no callback API to configure: pipe `--format json` to whatever you use for telemetry, or add an `exec` check for custom instrumentation.

## JSON reports

Every command takes `--format json`:

```bash
intl-ai fill --format json | jq '{translated, written, unresolved: (.unresolved | length)}'
intl-ai check --format json | jq '.locales.es.invalid'
intl-ai status --format json
```

Errors on stderr carry the same shape: `{"error": {"message": ..., "code": ...}}`.

## Findings

`intl-ai check` reports per-key findings. Kinds:

| Kind         | Meaning                                                                                    |
| ------------ | ------------------------------------------------------------------------------------------ |
| `missing`    | Key absent from a target locale file.                                                      |
| `stale`      | AI-owned value whose source changed since fill.                                            |
| `modified`   | Human-owned value whose source changed since review.                                       |
| `unreviewed` | Key never marked `reviewed` (or re-flagged by `[quality]`).                                |
| `extra`      | Key present in a target file but not in the source.                                        |
| `invalid`    | A `[[checks]]` entry rejected the value (icu, parity, judge, spec, exec, or `fail_below`). |

Check-level failures (a spec that won't load, an exec that won't spawn, a provider error) land in the report's `errors` array and always fail the run. Replaying a finding from the incremental cache marks it `"cached": true` in JSON output.

## Quality scores in the lockfile

`fill` records what it produced in `intl-ai.lock.d/` shards. Per key:

- `origin`: `ai` or `human`.
- `reviewed`: flipped back to `false` automatically when a human edits the file.
- `quality.scores`: normalized per-check scores from the last fill run.
- `quality.unresolved`: keys that failed the `[fill].validate` gate and were adopted anyway.

Track review coverage in CI with `intl-ai status --format json` and gate merges on `intl-ai check --fail-on`.

## Exec checks as custom telemetry

Need per-run instrumentation inside the pipeline (post to Sentry, emit a metric, append to a log)? Write an `exec` check: the binary sends it one JSONL request per batch of keys and expects one JSONL response of findings. It is a natural seam for "call this on every check run" without forking the pipeline.

```toml
[[checks]]
exec = "./scripts/i18n-telemetry"
timeout_ms = 60000
```

See `examples/checks/` for the protocol.

## Exit codes

- `0`: success (or findings only below `--fail-on` kinds).
- `1`: a `--fail-on` kind was found, a configured gate failed, or a hard error occurred. In CI, non-zero always means act.

See [Configuration](/guide/configuration/) for `[[checks]]`, `[check]`, `[fill]`, and `[quality]`.
