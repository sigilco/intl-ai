use crate::clock::now;
use crate::config::ResolvedConfig;
use crate::error::Result;
use crate::flatten::flatten;
use crate::lockfile::{Entry, Origin, ShardLock, load_shard, save_shard};
use crate::selector::KeySelector;
use crate::stat_cache::StatCache;
use intl_ai_formats::read;
use serde::Serialize;

/// Per-locale outcome of mark/review/unreview.
#[derive(Debug, Default, Serialize)]
pub struct OpResult {
    pub affected: usize,
    /// Spec-matched keys with no lockfile entry where one was required.
    pub skipped_no_entry: usize,
}

/// `intl-ai mark --origin human` is the escape hatch for the positional rule
/// (a human wrote or restored text byte-identical to the recorded AI value).
/// It upserts an adopted entry when the key exists on disk but has none.
/// `--origin ai` returns a human-owned key to AI stewardship and requires an
/// existing entry (the tool cannot claim authorship of text it never wrote).
pub fn mark(
    cfg: &ResolvedConfig,
    locale: &str,
    selector: &KeySelector,
    origin: Origin,
) -> Result<OpResult> {
    let locale_dir = cfg.locale_dir();
    let (source, target, src_hashes) = load_flats(cfg, locale)?;
    let _lock = ShardLock::acquire(&cfg.config_dir, locale)?;
    let mut shard = load_shard(&locale_dir, locale)?;
    let mut res = OpResult::default();
    let now_s = now();

    let keys: Vec<String> = source
        .keys()
        .chain(target.keys())
        .filter(|k| selector.matches(k))
        .cloned()
        .collect();
    for key in keys {
        match shard.entries.get_mut(&key) {
            Some(entry) => {
                let mut changed = false;
                // Explicit re-mark re-arms a tombstoned key.
                if entry.absent {
                    entry.absent = false;
                    changed = true;
                }
                if entry.origin != origin {
                    entry.origin = origin;
                    // Rebaseline the snapshot to the current file text: for
                    // `--origin ai` the tool adopts whatever is on disk as
                    // the AI baseline (otherwise the positional rule flips
                    // it straight back); for `--origin human` the snapshot
                    // records exactly what the human approved.
                    if let Some(v) = target.get(&key) {
                        entry.value = v.clone();
                    }
                    entry.updated_at = Some(now_s.clone());
                    changed = true;
                }
                if changed {
                    res.affected += 1;
                }
            }
            None => {
                if origin == Origin::Human {
                    if let Some(val) = target.get(&key) {
                        shard.entries.insert(
                            key.clone(),
                            Entry {
                                value: val.clone(),
                                source_hash: src_hashes.get(&key).cloned().unwrap_or_default(),
                                origin: Origin::Human,
                                reviewed: false,
                                model: None,
                                updated_at: Some(now_s.clone()),
                                ..Default::default()
                            },
                        );
                        res.affected += 1;
                    } else {
                        res.skipped_no_entry += 1;
                    }
                } else {
                    res.skipped_no_entry += 1;
                }
            }
        }
    }
    save_shard(&locale_dir, locale, &shard)?;
    Ok(res)
}

/// `intl-ai mark --absent` / `--present`: the missing-vs-deleted tombstone
/// (plan 5.6 M-item). A source key marked absent stays untranslated by
/// choice — `fill` never refills it, `check` doesn't flag it missing.
/// Tombstones attach to source keys (a key with no source needs none).
pub fn mark_absent(
    cfg: &ResolvedConfig,
    locale: &str,
    selector: &KeySelector,
    absent: bool,
) -> Result<OpResult> {
    let locale_dir = cfg.locale_dir();
    let (source, _target, src_hashes) = load_flats(cfg, locale)?;
    let _lock = ShardLock::acquire(&cfg.config_dir, locale)?;
    let mut shard = load_shard(&locale_dir, locale)?;
    let mut res = OpResult::default();
    let now_s = now();

    for key in source.keys().filter(|k| selector.matches(k)) {
        match shard.entries.get_mut(key) {
            Some(entry) => {
                if entry.absent != absent {
                    entry.absent = absent;
                    entry.updated_at = Some(now_s.clone());
                    res.affected += 1;
                }
            }
            None if absent => {
                shard.entries.insert(
                    key.clone(),
                    Entry {
                        // A tombstone a human set: deliberately untranslated,
                        // reviewed by definition (it IS the human's call).
                        value: String::new(),
                        source_hash: src_hashes.get(key).cloned().unwrap_or_default(),
                        origin: Origin::Human,
                        reviewed: true,
                        model: None,
                        updated_at: Some(now_s.clone()),
                        absent: true,
                        ..Default::default()
                    },
                );
                res.affected += 1;
            }
            None => res.skipped_no_entry += 1,
        }
    }
    save_shard(&locale_dir, locale, &shard)?;
    Ok(res)
}

/// `intl-ai review` / `unreview`: toggles `reviewed` on existing entries.
/// Reviewing a key with no entry is a no-op (counted as skipped).
pub fn set_reviewed(
    cfg: &ResolvedConfig,
    locale: &str,
    selector: &KeySelector,
    reviewed: bool,
) -> Result<OpResult> {
    let locale_dir = cfg.locale_dir();
    let (source, target, _src_hashes) = load_flats(cfg, locale)?;
    let _lock = ShardLock::acquire(&cfg.config_dir, locale)?;
    let mut shard = load_shard(&locale_dir, locale)?;
    let mut res = OpResult::default();
    let now_s = now();

    for (key, entry) in shard.entries.iter_mut() {
        if !selector.matches(key) {
            continue;
        }
        if entry.reviewed != reviewed {
            entry.reviewed = reviewed;
            // Approving snapshots the text being approved; a later file
            // edit diverging from it makes the key unverified again.
            if reviewed && let Some(v) = target.get(key) {
                entry.value = v.clone();
            }
            entry.updated_at = Some(now_s.clone());
            res.affected += 1;
        }
    }
    let spec_keys: Vec<String> = source
        .keys()
        .chain(target.keys())
        .filter(|k| selector.matches(k) && !shard.entries.contains_key(*k))
        .cloned()
        .collect();
    res.skipped_no_entry = spec_keys.len();
    save_shard(&locale_dir, locale, &shard)?;
    Ok(res)
}

fn load_flats(
    cfg: &ResolvedConfig,
    locale: &str,
) -> Result<(
    crate::flatten::FlatMap,
    crate::flatten::FlatMap,
    std::collections::BTreeMap<String, String>,
)> {
    let source_path = cfg.locale_path(&cfg.config.source);
    let source = read(&source_path)?.map(|v| flatten(&v)).unwrap_or_default();
    let mut cache = StatCache::load(&cfg.cache_path());
    let src_hashes = cache.source_hashes(&source_path, &source);
    let target = read(&cfg.locale_path(locale))?
        .map(|v| flatten(&v))
        .unwrap_or_default();
    Ok((source, target, src_hashes))
}
