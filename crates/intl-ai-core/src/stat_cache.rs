//! `.intl-ai/cache.json` — per-file (mtime, size) -> per-key hash map.
//! Skips the re-hash pass over locale files unchanged since the last run
//! (plan 5.5 "lightweight staleness", tsbuildinfo's file-version-as-
//! signature precedent). Gitignored and disposable: a missing or corrupt
//! cache just means a cold run, it is never an error.

use crate::flatten::FlatMap;
use crate::hash::source_hash;
use intl_ai_formats::json::write_atomic;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct StatCache {
    #[serde(default)]
    files: BTreeMap<String, FileStat>,
    #[serde(skip)]
    dirty: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct FileStat {
    mtime_ms: u64,
    size: u64,
    hashes: BTreeMap<String, String>,
}

impl StatCache {
    /// Loads the cache; a missing or corrupt file yields an empty one.
    pub fn load(path: &Path) -> Self {
        fs::read_to_string(path)
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default()
    }

    /// Best-effort write; failures are ignored (the next run is simply cold).
    pub fn save(&self, path: &Path) {
        if !self.dirty {
            return;
        }
        if let Ok(body) = serde_json::to_vec(self) {
            let _ = write_atomic(path, &body);
        }
    }

    /// Per-key `source_hash` map for `path`'s flattened content. Cached
    /// values are reused when the file's mtime+size match; otherwise every
    /// key is re-hashed and the entry replaced.
    pub fn source_hashes(&mut self, path: &Path, flat: &FlatMap) -> BTreeMap<String, String> {
        let stat = fs::metadata(path).ok().and_then(|m| {
            let mtime_ms = m
                .modified()
                .ok()?
                .duration_since(UNIX_EPOCH)
                .ok()?
                .as_millis() as u64;
            Some((mtime_ms, m.len()))
        });
        let key = path.to_string_lossy().to_string();
        if let Some((mtime_ms, size)) = stat {
            if let Some(f) = self.files.get(&key) {
                if f.mtime_ms == mtime_ms && f.size == size && f.hashes.len() == flat.len() {
                    return f.hashes.clone();
                }
            }
            let hashes: BTreeMap<String, String> = flat
                .iter()
                .map(|(k, v)| (k.clone(), source_hash(v)))
                .collect();
            self.files.insert(
                key,
                FileStat {
                    mtime_ms,
                    size,
                    hashes: hashes.clone(),
                },
            );
            self.dirty = true;
            hashes
        } else {
            // Unstattable file: hash without caching.
            flat.iter()
                .map(|(k, v)| (k.clone(), source_hash(v)))
                .collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reuses_hashes_when_file_unchanged() {
        let dir = tempfile::TempDir::new().unwrap();
        let file = dir.path().join("en.json");
        std::fs::write(&file, "{}").unwrap();
        let flat: FlatMap = [("a".to_string(), "Hello".to_string())]
            .into_iter()
            .collect();

        let mut cache = StatCache::default();
        let h1 = cache.source_hashes(&file, &flat);
        assert_eq!(h1["a"], source_hash("Hello"));
        // A different flat map with the same file still hits the cache
        // (callers only ask about files they just read, so this models
        // the real flow: same file -> same flat).
        let h2 = cache.source_hashes(&file, &flat);
        assert_eq!(h1, h2);
    }

    #[test]
    fn roundtrip_load_save() {
        let dir = tempfile::TempDir::new().unwrap();
        let cache_path = dir.path().join(".intl-ai/cache.json");
        let file = dir.path().join("en.json");
        std::fs::write(&file, "{}").unwrap();
        let flat: FlatMap = [("a".to_string(), "Hello".to_string())]
            .into_iter()
            .collect();

        let mut cache = StatCache::default();
        let _ = cache.source_hashes(&file, &flat);
        cache.save(&cache_path);
        assert!(cache_path.exists());

        let mut loaded = StatCache::load(&cache_path);
        let h = loaded.source_hashes(&file, &flat);
        assert_eq!(h["a"], source_hash("Hello"));
    }
}
