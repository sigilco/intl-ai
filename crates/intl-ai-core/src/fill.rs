use crate::clock::now;
use crate::config::ResolvedConfig;
use crate::diff::effective_origin;
use crate::error::{Error, ErrorType, Result};
use crate::flatten::{FlatMap, flatten, set_nested};
use crate::lockfile::{Entry, Origin, Shard, ShardLock, load_shard, save_shard};
use crate::selector::KeySelector;
use crate::stat_cache::StatCache;
use crate::transport::{TranslateRequest, TranslationEntry, Transport};
use intl_ai_formats::{read, write};
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
    /// Per-key/per-locale failures; a failed batch marks every key in it
    /// (plan 5.1.5: per-key failures are terminal, never retried).
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
    /// Existing values destroyed by a write (a scalar/array replaced by an
    /// intermediate object, or an object/array replaced by a leaf). These
    /// were human content the diff reported only as `extra`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clobbered: Vec<String>,
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
    let source_path = cfg.locale_path(&cfg.config.source);
    let source_value = read(&source_path)?.ok_or_else(|| {
        Error::Message(format!(
            "source locale file {} not found",
            source_path.display()
        ))
    })?;
    let source = flatten(&source_value);
    let mut cache = StatCache::load(&cfg.cache_path());
    let src_hashes = cache.source_hashes(&source_path, &source);

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
        match fill_locale(
            cfg,
            &locale_dir,
            &source,
            &src_hashes,
            &locale,
            transport,
            opts,
        ) {
            Ok((res, failures)) => {
                report.locales.insert(locale, res);
                report.failures.extend(failures);
            }
            Err(e) => {
                // Corrupt/locked shard, bad locale file, IO failure:
                // unrecoverable state, fail the whole run rather than
                // degrade to a per-locale report entry.
                if e.is_hard() {
                    return Err(e);
                }
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
    if !opts.dry_run {
        cache.save(&cfg.cache_path());
    }
    Ok(report)
}

fn fill_locale(
    cfg: &ResolvedConfig,
    locale_dir: &Path,
    source: &FlatMap,
    src_hashes: &BTreeMap<String, String>,
    locale: &str,
    transport: &dyn Transport,
    opts: &FillOptions,
) -> Result<(LocaleFillResult, Vec<FillFailure>)> {
    let mut res = LocaleFillResult::empty();
    let target_path = cfg.locale_path(locale);
    let mut target_value = read(&target_path)?.unwrap_or_else(|| Value::Object(Map::new()));
    let target = flatten(&target_value);
    // Hold the per-locale lock across load -> modify -> save so a second
    // process cannot interleave a write into this read-modify-write cycle.
    let _lock = ShardLock::acquire(&cfg.config_dir, locale)?;
    let mut shard = load_shard(locale_dir, locale)?;
    let mut written_this_run = HashSet::new();

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
        // Human-arm drift: `entry.value` is the last-approved snapshot for
        // human-owned keys too. A file value that diverged since then is
        // unverified again (modified => unverified applies to both arms).
        if entry.origin == Origin::Human
            && let Some(v) = target.get(key)
            && *v != entry.value
        {
            entry.value = v.clone();
            entry.reviewed = false;
        }
    }

    // Adoption: file keys with no entry are human-authored.
    for (key, val) in &target {
        if source.contains_key(key) && !shard.entries.contains_key(key) {
            shard.entries.insert(
                key.clone(),
                Entry {
                    value: val.clone(),
                    source_hash: src_hashes[key].clone(),
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
        .filter(|k| needs_translation(k, source, src_hashes, &target, &shard, opts))
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

    let mut failures = Vec::new();
    if !wanted.is_empty() {
        let instruction = cfg.instruction_for(locale);
        let hint = cfg.syntax_hint().to_string();
        // batchSize applies to the initial pass too (plan 5.3.3).
        let batch_size = cfg.config.batch_size.unwrap_or(usize::MAX).max(1);
        for chunk in wanted.chunks(batch_size) {
            let req = TranslateRequest {
                source_locale: cfg.config.source.clone(),
                target_locale: locale.to_string(),
                entries: chunk
                    .iter()
                    .map(|k| TranslationEntry {
                        key: k.clone(),
                        source: source[k].clone(),
                    })
                    .collect(),
                glossary: cfg.config.glossary.clone(),
                locale_instruction: instruction.clone(),
                feedback: Default::default(),
                syntax_hint: Some(hint.clone()),
            };
            let resp = match transport.translate(&req) {
                Ok(r) => r,
                Err(e) => {
                    // Batch failure: every key in the chunk is terminal-failed
                    // but other batches and locales still run.
                    let kind = match &e {
                        Error::Transport { kind, .. } => *kind,
                        _ => ErrorType::Unknown,
                    };
                    for key in chunk {
                        failures.push(FillFailure {
                            locale: locale.to_string(),
                            key: Some(key.clone()),
                            kind,
                            message: e.to_string(),
                        });
                    }
                    continue;
                }
            };
            let answered: HashSet<&str> =
                resp.translations.iter().map(|t| t.key.as_str()).collect();
            let chunk_keys: HashSet<&str> = chunk.iter().map(|k| k.as_str()).collect();
            for key in chunk {
                if !answered.contains(key.as_str()) {
                    // Omitted key is terminal per plan 5.1.5, not retried —
                    // and it is a failure (output_truncated), not a silent
                    // skip: a provider that drops keys fails the run.
                    res.omitted.push(key.clone());
                    failures.push(FillFailure {
                        locale: locale.to_string(),
                        key: Some(key.clone()),
                        kind: ErrorType::OutputTruncated,
                        message: "provider returned no translation for this key".into(),
                    });
                }
            }
            for t in resp.translations {
                // Only write keys we asked for: an over-eager provider must
                // not overwrite values for keys outside this chunk.
                if !chunk_keys.contains(t.key.as_str()) {
                    continue;
                }
                if !opts.dry_run {
                    if let Some(path) =
                        set_nested(&mut target_value, &t.key, Value::String(t.value.clone()))
                    {
                        res.clobbered.push(path);
                    }
                }
                written_this_run.insert(t.key.clone());
                shard.entries.insert(
                    t.key.clone(),
                    Entry {
                        value: t.value,
                        source_hash: src_hashes[&t.key].clone(),
                        origin: Origin::Ai,
                        reviewed: false,
                        model: Some(resp.model.clone()),
                        updated_at: Some(now()),
                        ..Default::default()
                    },
                );
                res.written += 1;
            }
        }
        res.translated = res.written;
    }

    if !opts.dry_run {
        // Shard first, file second: a shard without the file degrades to
        // `missing` (the next fill heals it), while the reverse order
        // misattributes AI text as human-authored on the next run.
        save_shard(locale_dir, locale, &shard)?;
        write(&target_path, &target_value)?;
    }
    Ok((res, failures))
}

fn needs_translation(
    key: &str,
    source: &FlatMap,
    src_hashes: &BTreeMap<String, String>,
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
            Some(e) if e.origin == Origin::Ai => e.source_hash != src_hashes[key],
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
