use crate::flatten::FlatMap;
use crate::lockfile::{Origin, Shard};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

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
    pub unreviewed: usize,
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
        if !target.contains_key(key) {
            d.missing.push(key.clone());
        }
    }
    for key in target.keys() {
        if !source.contains_key(key) {
            d.extra.push(key.clone());
        }
    }
    for (key, entry) in &shard.entries {
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
        if target.contains_key(key) && !entry.reviewed {
            d.unreviewed += 1;
        }
    }
    d
}
