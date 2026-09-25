use crate::clock::now;
use crate::config::ResolvedConfig;
use crate::diff::effective_origin;
use crate::error::{Error, ErrorType, Result};
use crate::flatten::{FlatMap, flatten, set_nested};
use crate::hash::source_hash;
use crate::lockfile::{Entry, Origin, Shard, load_shard, save_shard};
use crate::selector::KeySelector;
use crate::transport::{TranslateRequest, TranslationEntry, Transport};
use intl_ai_formats::json::{read, write};
use serde::Serialize;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, HashSet};
use std::path::Path;

#[derive(Debug)]
pub struct FillOptions {
    /// None or empty = all config targets.
    pub locales: Option<Vec<String>>,
    pub selector: KeySelector,
    pub stale_only: bool,
    pub regenerate: bool,
    pub include_human: bool,
    pub dry_run: bool,
}

#[derive(Debug, Serialize)]
pub struct FillReport {
    pub locales: BTreeMap<String, LocaleFillResult>,
    pub failures: Vec<FillFailure>,
    pub dry_run: bool,
}

#[derive(Debug, Default, Serialize)]
pub struct LocaleFillResult {
    pub requested: usize,
    pub translated: usize,
    pub written: usize,
    pub adopted_human: usize,
    pub reconciled_human: usize,
    pub skipped_human: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub omitted: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct FillFailure {
    pub locale: String,
    pub key: Option<String>,
    pub kind: ErrorType,
    pub message: String,
}

impl LocaleFillResult {
    fn empty() -> Self {
        Self::default()
    }
}

/// Additive-only fill (plan 5.6): writes only keys with no value by default.
/// `--stale` re-fills AI-owned keys whose source changed; `--regenerate`
/// overwrites AI-owned values; `--include-human` extends that to human-owned.
/// Overrides are per-invocation, never config.
pub fn fill(
    cfg: &ResolvedConfig,
    transport: &dyn Transport,
    opts: &FillOptions,
) -> Result<FillReport> {
    let locale_dir = cfg.locale_dir();
    let source_path = locale_dir.join(format!("{}.json", cfg.config.source));
    let source_value = read(&source_path)?.ok_or_else(|| {
        Error::Message(format!(
            "source locale file {} not found",
            source_path.display()
        ))
    })?;
    let source = flatten(&source_value);

    let locales: Vec<String> = match &opts.locales {
        Some(l) if !l.is_empty() => l.clone(),
        _ => cfg.config.targets.clone(),
    };

    let mut report = FillReport {
        locales: BTreeMap::new(),
        failures: Vec::new(),
        dry_run: opts.dry_run,
    };

    for locale in locales {
        match fill_locale(cfg, &locale_dir, &source, &locale, transport, opts) {
            Ok(res) => {
                report.locales.insert(locale, res);
            }
            Err(e) => {
                report.failures.push(FillFailure {
                    locale,
                    key: None,
                    kind: match &e {
                        Error::Transport { kind, .. } => *kind,
                        _ => ErrorType::Unknown,
                    },
                    message: e.to_string(),
                });
            }
        }
    }
    Ok(report)
}

fn fill_locale(
    cfg: &ResolvedConfig,
    locale_dir: &Path,
    source: &FlatMap,
    locale: &str,
    transport: &dyn Transport,
    opts: &FillOptions,
) -> Result<LocaleFillResult> {
    let mut res = LocaleFillResult::empty();
    let target_path = locale_dir.join(format!("{locale}.json"));
    let mut target_value = read(&target_path)?.unwrap_or_else(|| Value::Object(Map::new()));
    let target = flatten(&target_value);
    let mut shard = load_shard(locale_dir, locale)?;
    let written_this_run = HashSet::new();

    // Positional reconciliation (plan 5.6): a recorded-AI value that no
    // longer matches the file was human-edited; flip origin, keep the
    // recorded value as provenance.
    for (key, entry) in shard.entries.iter_mut() {
        if effective_origin(entry, target.get(key), &written_this_run, key) == Origin::Human
            && entry.origin == Origin::Ai
        {
            entry.origin = Origin::Human;
            entry.reviewed = false;
            res.reconciled_human += 1;
        }
    }

    // Adoption: file keys with no entry are human-authored.
    for (key, val) in &target {
        if source.contains_key(key) && !shard.entries.contains_key(key) {
            shard.entries.insert(
                key.clone(),
                Entry {
                    value: val.clone(),
                    source_hash: source_hash(&source[key]),
                    origin: Origin::Human,
                    reviewed: false,
                    model: None,
                    updated_at: Some(now()),
                    ..Default::default()
                },
            );
            res.adopted_human += 1;
        }
    }

    let wanted: Vec<String> = source
        .keys()
        .filter(|k| opts.selector.matches(k))
        .filter(|k| needs_translation(k, source, &target, &shard, opts))
        .cloned()
        .collect();
    res.requested = wanted.len();
    // skipped_human: scoped keys with an existing value that fill left
    // alone (human-owned or AI-owned under the additive default).
    let scoped_with_value = source
        .keys()
        .filter(|k| opts.selector.matches(k) && target.contains_key(*k))
        .count();
    let translated_existing = wanted.iter().filter(|k| target.contains_key(*k)).count();
    res.skipped_human = scoped_with_value.saturating_sub(translated_existing);

    if !wanted.is_empty() {
        let req = TranslateRequest {
            source_locale: cfg.config.source.clone(),
            target_locale: locale.to_string(),
            entries: wanted
                .iter()
                .map(|k| TranslationEntry {
                    key: k.clone(),
                    source: source[k].clone(),
                })
                .collect(),
        };
        let resp = transport.translate(&req)?;
        let answered: HashSet<&str> = resp.translations.iter().map(|t| t.key.as_str()).collect();
        for key in &wanted {
            if !answered.contains(key.as_str()) {
                // Omitted key is terminal per plan 5.1.5, not retried.
                res.omitted.push(key.clone());
            }
        }
        for t in resp.translations {
            if !opts.dry_run {
                set_nested(&mut target_value, &t.key, Value::String(t.value.clone()));
            }
            shard.entries.insert(
                t.key.clone(),
                Entry {
                    value: t.value,
                    source_hash: source_hash(&source[&t.key]),
                    origin: Origin::Ai,
                    reviewed: false,
                    model: Some(resp.model.clone()),
                    updated_at: Some(now()),
                    ..Default::default()
                },
            );
            res.written += 1;
        }
        res.translated = res.written;
    }

    if !opts.dry_run {
        write(&target_path, &target_value)?;
        save_shard(locale_dir, locale, &shard)?;
    }
    Ok(res)
}

fn needs_translation(
    key: &str,
    source: &FlatMap,
    target: &FlatMap,
    shard: &Shard,
    opts: &FillOptions,
) -> bool {
    if !source.contains_key(key) {
        return false;
    }
    if opts.stale_only {
        // AI-owned entries whose recorded source hash drifted.
        return match shard.entries.get(key) {
            Some(e) if e.origin == Origin::Ai => e.source_hash != source_hash(&source[key]),
            _ => false,
        };
    }
    if opts.regenerate {
        if !target.contains_key(key) {
            return true; // regenerating a missing key is just filling it
        }
        return match shard.entries.get(key) {
            Some(e) => e.origin == Origin::Ai || opts.include_human,
            None => opts.include_human, // adopted-not-yet has an entry by now
        };
    }
    !target.contains_key(key)
}
