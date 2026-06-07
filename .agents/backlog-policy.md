# Backlog Policy — intl-ai

## Task Refinement Checklist

Every item added to GitHub Project #10 must have all five fields before moving out of "Backlog":

| Field | Values | Notes |
|---|---|---|
| **Iteration / Quarter** | `Q2 2026`, `Q3 2026`, `Q4 2026` | Quarter granularity |
| **Effort** | `S` / `M` / `L` / `XL` | S=hours, M=1-2d, L=3-5d, XL=1+w |
| **Start date** | `YYYY-MM-DD` | Approximate is fine |
| **Target date** | `YYYY-MM-DD` | Default: end of iteration quarter |
| **Classification label** | `feature` / `bug` / `cosmetic` / `infra` / `docs` | One per issue |

Additional labels: `release-blocker`, `priority/high`, `priority/low`.

---

## Using `ghx` for Project Operations

`ghx` is available at `~/.local/bin/ghx`. It wraps `gh project` commands with project #10 defaults.

```bash
ghx add <issue-number>
ghx list
ghx update <item-id> --field Effort --value M
ghx update <item-id> --field "Start date" --value 2026-07-01
ghx update <item-id> --field "Target date" --value 2026-09-30
ghx update <item-id> --field "Iteration" --value "Q3 2026"
```

Fallback (raw `gh`):
```bash
gh project item-add 10 --owner espetro --url <issue-url>
gh project item-list 10 --owner espetro --format json
```

---

## Creating Issues

```bash
gh issue create \
  --title "feat(core): incremental delta translation" \
  --body "..." \
  --label "feature" \
  --milestone "0.2.0"
```

Then: `ghx add <number>` and set all five required fields.

---

## Plan Files

Plans go to `.agents/plans/<YYYY-MM-DD>-<kebab-purpose>.md`. Each plan header must reference the GitHub issue it delivers:

```markdown
## TL;DR
> **GitHub Issue**: #42
> **Project Item**: https://github.com/users/espetro/projects/10/items/<N>
```

---

## Effort Calibration

| Effort | Typical scope |
|---|---|
| `S` | Single-file, ≤50 LOC, no new APIs |
| `M` | 1-3 files, new function or small feature, tests included |
| `L` | New module or command, cross-package change, integration test |
| `XL` | New architectural layer, multiple packages, full doc update |

---

## Task Lifecycle

```
Backlog → Refinement → In Progress → In Review → Done
```

Move to "In Progress" when a plan is created and linked. Move to "In Review" when PR is open. Move to "Done" after merge to `develop`.
