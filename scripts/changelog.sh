#!/usr/bin/env bash
# Generates a Keep a Changelog section for a release from conventional
# commits between two git refs, or validates that one exists.
#
#   scripts/changelog.sh generate v0.6.0 v0.7.0   # prints the section
#   scripts/changelog.sh update v0.6.0 v0.7.0     # inserts it into CHANGELOG.md
#   scripts/changelog.sh check v0.7.0             # fails if missing/incomplete
#   scripts/changelog.sh extract v0.7.0           # prints the section body (release notes)
#
# Commit subjects must follow Conventional Commits; everything else lands
# under "Other". Merge commits are skipped (their PR title is already the
# squash subject, which the log picks up directly).
set -euo pipefail

CHANGELOG="CHANGELOG.md"

die() { echo "changelog: $*" >&2; exit 1; }

# prev_tag (or empty) -> commits range
range_for() {
    local prev="$1" cur="$2"
    if [ -n "$prev" ]; then echo "${prev}..${cur}"; else echo "$cur"; fi
}

group_name() {
    case "$1" in
        feat) echo "Added" ;;
        fix) echo "Fixed" ;;
        docs) echo "Documentation" ;;
        perf) echo "Performance" ;;
        refactor) echo "Changed" ;;
        ci|build|chore|test|revert|style|merge|other) echo "Maintenance" ;;
        *) echo "Other" ;;
    esac
}

gen_section() {
    local prev="$1" cur="$2" version="${3:-${2#v}}"
    local date
    date="$(git log -1 --format=%cs "$cur" 2>/dev/null || date +%F)"
    echo "## [${version}] - ${date}"
    echo
    local range
    range="$(range_for "$prev" "$cur")"
    # subject lines only; -m on merges would double-list stacked commits
    local subjects
    subjects="$(mktemp)"
    trap 'rm -f "$subjects"' RETURN
    git log --format='%s' --no-merges "$range" > "$subjects"
    python3 - "$subjects" <<'PY'
import re, sys

groups = {}
order = ["Added", "Fixed", "Changed", "Documentation", "Performance", "Maintenance", "Other"]
for line in open(sys.argv[1]):
    s = line.strip()
    if not s:
        continue
    m = re.match(r"([a-z]+)(?:\([^)]*\))?!?:\s*(.*)", s)
    if m:
        kind, subject = m.group(1), m.group(2)
    else:
        kind, subject = "other", s
    name = {"feat": "Added", "fix": "Fixed", "docs": "Documentation",
            "perf": "Performance", "refactor": "Changed"}.get(kind, "Maintenance")
    groups.setdefault(name, []).append(subject)

for name in order:
    items = groups.get(name)
    if not items:
        continue
    print(f"### {name}")
    print()
    seen = set()
    for it in items:
        if it in seen:
            continue
        seen.add(it)
        print(f"- {it}")
    print()
PY
}

insert_into_changelog() {
    local section="$1" version="$2"
    [ -f "$CHANGELOG" ] || die "$CHANGELOG not found"
    grep -q "^## \[${version}\]" "$CHANGELOG" && die "section for ${version} already exists"
    # Insert before the first "## [" line, keeping the header preamble.
    python3 - "$CHANGELOG" "$section" "$version" <<'PY'
import sys, re

path, section, version = sys.argv[1], sys.argv[2], sys.argv[3]
body = open(path).read()
body = re.sub(r"^## \[" + re.escape(version) + r"\].*?(?=^## \[|\Z)", "",
              body, flags=re.M | re.S).rstrip() + "\n"
m = re.search(r"^## \[", body, flags=re.M)
if m:
    out = body[:m.start()] + section + body[m.start():]
else:
    out = body.rstrip() + "\n\n" + section
open(path, "w").write(out)
PY
}

extract_section() {
    local version="$1"
    [ -f "$CHANGELOG" ] || die "$CHANGELOG not found"
    python3 - "$CHANGELOG" "$version" <<'PY'
import sys, re

body = open(sys.argv[1]).read()
m = re.search(r"^## \[" + re.escape(sys.argv[2]) + r"\]\s*-\s*\S+\n(.*?)(?=^## \[|\Z)",
              body, flags=re.M | re.S)
if not m:
    sys.exit("changelog: no section for " + sys.argv[2])
print(m.group(1).strip())
PY
}

cmd="${1:-}"; a="${2:-}"; b="${3:-}"
case "$cmd" in
    generate)
        [ -n "$a" ] && [ -n "$b" ] || die "generate <prev_ref> <cur_ref> [version]"
        gen_section "$a" "$b" "${4:-${b#v}}" ;;
    update)
        [ -n "$a" ] && [ -n "$b" ] || die "update <prev_ref> <cur_ref> [version]"
        v="${4:-${b#v}}"
        gen_section "$a" "$b" "$v" | { section="$(cat)"; insert_into_changelog "$section"$'\n' "$v"; }
        echo "changelog: inserted section for ${v}" ;;
    check)
        [ -n "$a" ] || die "check <version_or_tag>"
        v="${a#v}"
        grep -q "^## \[${v}\]" "$CHANGELOG" || die "CHANGELOG.md has no section for ${v} — run scripts/changelog.sh update <prev_tag> <tag> and merge it before tagging"
        extract_section "$v" | grep -q . || die "CHANGELOG.md section for ${v} is empty"
        echo "changelog: section for ${v} present" ;;
    extract)
        [ -n "$a" ] || die "extract <version_or_tag>"
        extract_section "${a#v}" ;;
    *) die "usage: $0 {generate|update|check|extract} ..." ;;
esac
