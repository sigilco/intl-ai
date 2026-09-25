use crate::clock::now;
use crate::config::ResolvedConfig;
use crate::error::Result;
use crate::flatten::flatten;
use crate::hash::source_hash;
use crate::lockfile::{Entry, Origin, load_shard, save_shard};
use crate::selector::KeySelector;
use intl_ai_formats::json::read;
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
    let (source, target) = load_flats(cfg, locale)?;
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
                if entry.origin != origin {
                    entry.origin = origin;
                    entry.updated_at = Some(now_s.clone());
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
                                source_hash: source
                                    .get(&key)
                                    .map(|s| source_hash(s))
                                    .unwrap_or_default(),
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

/// `intl-ai review` / `unreview`: toggles `reviewed` on existing entries.
/// Reviewing a key with no entry is a no-op (counted as skipped).
pub fn set_reviewed(
    cfg: &ResolvedConfig,
    locale: &str,
    selector: &KeySelector,
    reviewed: bool,
) -> Result<OpResult> {
    let locale_dir = cfg.locale_dir();
    let (source, target) = load_flats(cfg, locale)?;
    let mut shard = load_shard(&locale_dir, locale)?;
    let mut res = OpResult::default();
    let now_s = now();

    for (key, entry) in shard.entries.iter_mut() {
        if !selector.matches(key) {
            continue;
        }
        if entry.reviewed != reviewed {
            entry.reviewed = reviewed;
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
) -> Result<(crate::flatten::FlatMap, crate::flatten::FlatMap)> {
    let locale_dir = cfg.locale_dir();
    let source = read(&locale_dir.join(format!("{}.json", cfg.config.source)))?
        .map(|v| flatten(&v))
        .unwrap_or_default();
    let target = read(&locale_dir.join(format!("{locale}.json")))?
        .map(|v| flatten(&v))
        .unwrap_or_default();
    Ok((source, target))
}
