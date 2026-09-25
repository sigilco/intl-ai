//! Incremental check cache (plan 5.7 / part B): findings are replayed
//! when the fingerprint of a check's inputs is unchanged — growing the
//! project doesn't mean re-validating every key on every `check`.
//! Gitignored, like the stat-cache; never the source of truth.

use crate::diff::CheckFinding;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

const CACHE_VERSION: u32 = 1;
/// Whole-batch checks (exec protocol) fingerprint the item list as one
/// request; their hit is stored under this synthetic key.
pub const BATCH_KEY: &str = "*";

/// locale -> key (or BATCH_KEY) -> check_id -> hit
#[derive(Debug, Serialize, Deserialize)]
pub struct CheckCache {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub entries: BTreeMap<String, BTreeMap<String, BTreeMap<String, CacheHit>>>,
}

impl Default for CheckCache {
    // A fresh cache writes the current version — derive would write 0,
    // which `load` then rejects as stale on every run.
    fn default() -> Self {
        Self {
            version: CACHE_VERSION,
            entries: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheHit {
    pub fingerprint: String,
    #[serde(default)]
    pub findings: Vec<CheckFinding>,
}

fn default_version() -> u32 {
    CACHE_VERSION
}

impl CheckCache {
    /// A corrupt or older-version cache degrades to empty — the next run
    /// rebuilds it (same posture as the stat-cache).
    pub fn load(path: &Path) -> Self {
        let text = match fs::read_to_string(path) {
            Ok(t) => t,
            Err(_) => return Self::default(),
        };
        let cache: Self = serde_json::from_str(&text).unwrap_or_default();
        if cache.version != CACHE_VERSION {
            Self::default()
        } else {
            cache
        }
    }

    pub fn save(&self, path: &Path) {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(body) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, body);
        }
    }

    pub fn get(&self, locale: &str, key: &str, check_id: &str) -> Option<&CacheHit> {
        self.entries.get(locale)?.get(key)?.get(check_id)
    }

    /// Remember the finding set a run produced (empty included — absence
    /// of findings is a cacheable result).
    pub fn put(
        &mut self,
        locale: &str,
        key: &str,
        check_id: &str,
        fingerprint: String,
        findings: Vec<CheckFinding>,
    ) {
        self.entries
            .entry(locale.to_string())
            .or_default()
            .entry(key.to_string())
            .or_default()
            .insert(
                check_id.to_string(),
                CacheHit {
                    fingerprint,
                    findings,
                },
            );
    }

    /// Drop entries for keys no longer in the locale's item set (only
    /// called on unscoped runs — a `--keys` run must not evict the rest).
    pub fn prune_locale(&mut self, locale: &str, live_keys: &std::collections::BTreeSet<String>) {
        if let Some(keys) = self.entries.get_mut(locale) {
            keys.retain(|k, _| k == BATCH_KEY || live_keys.contains(k));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn roundtrip_and_corrupt_fallback() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("check-cache.json");
        let mut c = CheckCache::default();
        c.put("fr", "k", "icu", "fp1".into(), vec![]);
        c.save(&path);
        let loaded = CheckCache::load(&path);
        assert_eq!(loaded.get("fr", "k", "icu").unwrap().fingerprint, "fp1");

        fs::write(&path, "{oops").unwrap();
        assert!(CheckCache::load(&path).entries.is_empty());
    }

    #[test]
    fn prune_keeps_batch_and_live_keys() {
        let mut c = CheckCache::default();
        c.put("fr", "a", "icu", "f".into(), vec![]);
        c.put("fr", "dead", "icu", "f".into(), vec![]);
        c.put("fr", BATCH_KEY, "exec:x", "f".into(), vec![]);
        let live = BTreeSet::from(["a".to_string()]);
        c.prune_locale("fr", &live);
        let keys: Vec<&String> = c.entries["fr"].keys().collect();
        assert_eq!(keys, [BATCH_KEY, "a"]);
    }
}
