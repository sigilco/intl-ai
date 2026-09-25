use crate::flatten::FlatMap;
use crate::lockfile::{Origin, Shard};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

/// One rule-level violation found by a `Check` (ICU syntax, placeholder
/// parity, dialect, spec rule, exec check, judge). Distinct from the
/// structural `FindingKind` buckets: an `invalid` finding always carries the
/// check that produced it.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CheckFinding {
    pub key: String,
    /// Check id that reported the violation (`icu`, `dialect:en-US`, spec
    /// id, exec check name).
    pub check: String,
    pub message: String,
    /// Replayed from the incremental check cache, not re-run this time
    /// (part B): the finding is identical because the inputs were.
    #[serde(default, skip_serializing_if = "is_false")]
    pub cached: bool,
}

fn is_false(b: &bool) -> bool {
    !*b
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FindingKind {
    Missing,
    Stale,
    Invalid,
    Modified,
    Extra,
    Unreviewed,
}

impl std::fmt::Display for FindingKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Missing => "missing",
            Self::Stale => "stale",
            Self::Invalid => "invalid",
            Self::Modified => "modified",
            Self::Extra => "extra",
            Self::Unreviewed => "unreviewed",
        };
        f.write_str(s)
    }
}

impl std::str::FromStr for FindingKind {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "missing" => Ok(Self::Missing),
            "stale" => Ok(Self::Stale),
            "invalid" => Ok(Self::Invalid),
            "modified" => Ok(Self::Modified),
            "extra" => Ok(Self::Extra),
            "unreviewed" => Ok(Self::Unreviewed),
            other => Err(format!("unknown finding kind '{other}'")),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct LocaleDiff {
    pub missing: Vec<String>,
    pub stale: Vec<String>,
    pub modified: Vec<String>,
    pub extra: Vec<String>,
    /// Keys needing a human review pass (M4: the list, not just a count).
    pub unreviewed: Vec<String>,
    /// Rule-level violations from configured checks (`[[checks]]`).
    pub invalid: Vec<CheckFinding>,
}

/// Positional human-ownership rule (plan 5.6): the file value differing from
/// the recorded lockfile `value` means a human wrote it. Entries fill wrote
/// this run are exempt so the tool never self-marks.
pub fn effective_origin(
    entry: &crate::lockfile::Entry,
    file_value: Option<&String>,
    written_this_run: &HashSet<String>,
    key: &str,
) -> Origin {
    match file_value {
        Some(v)
            if entry.origin == Origin::Ai
                && *v != entry.value
                && !written_this_run.contains(key) =>
        {
            Origin::Human
        }
        _ => entry.origin,
    }
}

/// Shared diff for `fill` and `check`. Pure function of committed shard +
/// current source/target flat maps. `src_hashes` is the per-key
/// `source_hash` map for `source` (stat-cache supplied by callers).
pub fn diff(
    source: &FlatMap,
    src_hashes: &BTreeMap<String, String>,
    target: &FlatMap,
    shard: &Shard,
    written_this_run: &HashSet<String>,
) -> LocaleDiff {
    let mut d = LocaleDiff::default();
    for key in source.keys() {
        // Absent tombstones are deliberately untranslated, not missing.
        if !target.contains_key(key) && !shard.entries.get(key).is_some_and(|e| e.absent) {
            d.missing.push(key.clone());
        }
    }
    for key in target.keys() {
        if !source.contains_key(key) {
            d.extra.push(key.clone());
        }
    }
    for (key, entry) in &shard.entries {
        // Tombstones carry no review state; every bucket below skips them.
        if entry.absent {
            continue;
        }
        let Some(src_hash) = source.get(key).and_then(|_| src_hashes.get(key)) else {
            continue;
        };
        if entry.source_hash != *src_hash {
            d.stale.push(key.clone());
        }
        if effective_origin(entry, target.get(key), written_this_run, key) == Origin::Human
            && entry.origin == Origin::Ai
        {
            d.modified.push(key.clone());
        }
        // Human-owned entries carry `value` as the last-approved snapshot:
        // a live value that drifted from it is unverified again, even if
        // `reviewed` was never flipped back in the file.
        let drifted_human =
            entry.origin == Origin::Human && target.get(key).is_some_and(|v| v != &entry.value);
        if target.contains_key(key) && (!entry.reviewed || drifted_human) {
            d.unreviewed.push(key.clone());
        }
    }
    d
}
