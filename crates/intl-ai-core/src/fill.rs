use crate::check::{CheckCtx, CheckItem, Gate};
use crate::clock::now;
use crate::config::ResolvedConfig;
use crate::diff::{CheckFinding, effective_origin};
use crate::error::{Error, ErrorType, Result};
use crate::flatten::{FlatMap, flatten, set_nested};
use crate::lockfile::{Entry, Origin, Shard, ShardLock, load_shard, save_shard};
use crate::selector::KeySelector;
use crate::stat_cache::StatCache;
use crate::transport::{TranslateRequest, Translated, TranslationEntry, Transport};
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
    /// Skip the stat-cache read/write for this run.
    pub no_cache: bool,
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
    /// Provider answered for an in-chunk key (M4: honest count; `written`
    /// counts keys adopted into file+shard — they coincide until the
    /// validation gate can reject an answer).
    pub translated: usize,
    pub written: usize,
    /// Values regenerated that were human-owned (--include-human tier).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub regenerated_human: usize,
    /// Shard entries pruned: keys gone from both source and target.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub pruned: usize,
    pub adopted_human: usize,
    pub reconciled_human: usize,
    /// Scoped keys with an existing value fill left alone, AI-owned.
    pub skipped_existing: usize,
    /// Same set, human-owned subset (M4 split).
    pub skipped_human: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub omitted: Vec<String>,
    /// Existing values destroyed by a write (a scalar/array replaced by an
    /// intermediate object, or an object/array replaced by a leaf). These
    /// were human content the diff reported only as `extra`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clobbered: Vec<String>,
    /// Keys sent through a corrective round by the validation gate.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub refilled: usize,
    /// Findings still open after the gate's last corrective round; the
    /// values were adopted anyway and the findings live on the shard
    /// entry's `quality.unresolved` (provenance, not suppression).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unresolved: Vec<CheckFinding>,
}

fn is_zero(n: &usize) -> bool {
    *n == 0
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
    gate: Option<&Gate>,
) -> Result<FillReport> {
    let locale_dir = cfg.locale_dir();
    let source_path = cfg.locale_path(&cfg.config.source);
    // A missing source file is an empty corpus, not an error — a fresh
    // `init` scaffold has no strings yet (M6). A corrupt file still fails.
    let source_value = match read(&source_path)? {
        Some(v) => v,
        None => {
            eprintln!(
                "intl-ai: source locale file {} not found; treating as empty",
                source_path.display()
            );
            serde_json::Value::Object(serde_json::Map::new())
        }
    };
    let source = flatten(&source_value);
    let dropped = crate::flatten::dropped_leaf_count(&source_value);
    if dropped > 0 {
        eprintln!(
            "intl-ai: {dropped} source leaf(s) dropped by flattening \
             (empty objects or colliding dotted keys)"
        );
    }
    let mut cache = if opts.no_cache {
        StatCache::default()
    } else {
        StatCache::load(&cfg.cache_path())
    };
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
        let shadowed = cfg.shadowed_locale_files(&locale);
        if shadowed.len() > 1 {
            eprintln!(
                "intl-ai: {locale}: multiple locale files on disk ({}); using {}",
                shadowed
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
                shadowed[0].display()
            );
        }
        match fill_locale(
            cfg,
            &locale_dir,
            &source,
            &src_hashes,
            &locale,
            transport,
            opts,
            gate,
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
    if !opts.dry_run && !opts.no_cache {
        cache.save(&cfg.cache_path());
    }
    Ok(report)
}

#[allow(clippy::too_many_arguments)]
fn fill_locale(
    cfg: &ResolvedConfig,
    locale_dir: &Path,
    source: &FlatMap,
    src_hashes: &BTreeMap<String, String>,
    locale: &str,
    transport: &dyn Transport,
    opts: &FillOptions,
    gate: Option<&Gate>,
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

    // GC: entries for keys gone from both source and target are dead
    // weight (tombstones for deleted source keys included).
    let before = shard.entries.len();
    shard
        .entries
        .retain(|k, _| source.contains_key(k) || target.contains_key(k));
    res.pruned = before - shard.entries.len();

    let wanted: Vec<String> = source
        .keys()
        .filter(|k| opts.selector.matches(k))
        .filter(|k| needs_translation(k, source, src_hashes, &target, &shard, opts))
        .cloned()
        .collect();
    res.requested = wanted.len();
    // skipped_*: scoped keys with an existing value fill left alone,
    // split by effective (post-reconciliation) origin (M4).
    let wanted_set: HashSet<&str> = wanted.iter().map(String::as_str).collect();
    for k in source
        .keys()
        .filter(|k| opts.selector.matches(k) && target.contains_key(*k))
    {
        if wanted_set.contains(k.as_str()) {
            continue;
        }
        match shard.entries.get(k).map(|e| e.origin) {
            Some(Origin::Ai) => res.skipped_existing += 1,
            _ => res.skipped_human += 1,
        }
    }

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
            let chunk_keys: HashSet<&str> = chunk.iter().map(|k| k.as_str()).collect();
            let model = resp.model.clone();
            // Only keys we asked for: an over-eager provider must not
            // overwrite values for keys outside this chunk.
            let mut translations: Vec<Translated> = resp
                .translations
                .into_iter()
                .filter(|t| chunk_keys.contains(t.key.as_str()))
                .collect();
            let answered: HashSet<String> = translations.iter().map(|t| t.key.clone()).collect();
            let unresolved = gate_rounds(
                gate,
                locale,
                cfg,
                source,
                &instruction,
                &hint,
                transport,
                &mut translations,
                &mut res,
                &mut failures,
            );
            for f in unresolved.values().flatten() {
                res.unresolved.push(f.clone());
            }
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
            for t in translations {
                res.translated += 1;
                if shard
                    .entries
                    .get(&t.key)
                    .is_some_and(|e| e.origin == Origin::Human)
                {
                    res.regenerated_human += 1;
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
                        model: Some(model.clone()),
                        updated_at: Some(now()),
                        quality: unresolved.get(&t.key).map(|fs| unresolved_quality(fs)),
                        ..Default::default()
                    },
                );
                res.written += 1;
            }
        }
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

/// The shift-left gate (plan W2b-C): validate the provider's answers
/// before adoption, then give failed keys one corrective round with the
/// findings as reviewer notes. Keys still failing are adopted anyway —
/// their findings return as `unresolved` so the entry can record them —
/// and a gate check that errors is a run failure, never a silent pass.
/// Returns failed-key findings keyed by key (empty when clean or no gate).
#[allow(clippy::too_many_arguments)]
fn gate_rounds(
    gate: Option<&Gate>,
    locale: &str,
    cfg: &ResolvedConfig,
    source: &FlatMap,
    instruction: &Option<String>,
    hint: &str,
    transport: &dyn Transport,
    translations: &mut [Translated],
    res: &mut LocaleFillResult,
    failures: &mut Vec<FillFailure>,
) -> BTreeMap<String, Vec<CheckFinding>> {
    let Some(g) = gate.filter(|g| !g.checks.is_empty()) else {
        return BTreeMap::new();
    };
    if translations.is_empty() {
        return BTreeMap::new();
    }
    let ctx = CheckCtx {
        source_locale: &cfg.config.source,
        target_locale: locale,
        transport: Some(transport),
        locale_instruction: instruction.as_deref(),
    };
    for round in 0..=g.max_rounds {
        let items: Vec<CheckItem> = translations
            .iter()
            .map(|t| CheckItem {
                key: t.key.clone(),
                source: source.get(&t.key).cloned(),
                target: t.value.clone(),
            })
            .collect();
        let mut findings = Vec::new();
        for c in &g.checks {
            match c.run(&ctx, &items) {
                Ok(f) => findings.extend(f),
                Err(e) => failures.push(FillFailure {
                    locale: locale.to_string(),
                    key: None,
                    kind: ErrorType::Unknown,
                    message: format!("gate check '{}': {e}", c.id()),
                }),
            }
        }
        if findings.is_empty() {
            return BTreeMap::new();
        }
        let mut by_key: BTreeMap<String, Vec<CheckFinding>> = BTreeMap::new();
        for f in findings {
            by_key.entry(f.key.clone()).or_default().push(f);
        }
        // Findings on keys the gate did not translate (a check misbehaving)
        // belong nowhere: keep only keys actually in this batch.
        by_key.retain(|k, _| translations.iter().any(|t| &t.key == k));
        if by_key.is_empty() {
            return BTreeMap::new();
        }
        if round == g.max_rounds {
            return by_key;
        }
        // Corrective round: failed keys only, findings become reviewer
        // notes. "Previous attempt" wording is TS parity (plan W2b-C).
        let mut feedback = BTreeMap::new();
        for (key, fs) in &by_key {
            let prev = translations
                .iter()
                .find(|t| &t.key == key)
                .map(|t| t.value.as_str())
                .unwrap_or_default();
            let notes = fs
                .iter()
                .map(|f| format!("{}: {}", f.check, f.message))
                .collect::<Vec<_>>()
                .join("; ");
            feedback.insert(key.clone(), format!("Previous attempt: {prev}\n{notes}"));
        }
        let req = TranslateRequest {
            source_locale: cfg.config.source.clone(),
            target_locale: locale.to_string(),
            entries: by_key
                .keys()
                .map(|k| TranslationEntry {
                    key: k.clone(),
                    source: source[k].clone(),
                })
                .collect(),
            glossary: cfg.config.glossary.clone(),
            locale_instruction: instruction.clone(),
            feedback,
            syntax_hint: Some(hint.to_string()),
        };
        match transport.translate(&req) {
            Ok(resp) => {
                // Adopt corrections only for keys that actually failed —
                // an over-eager provider must not touch the rest.
                for t in resp.translations {
                    if let Some(existing) = translations.iter_mut().find(|x| x.key == t.key) {
                        existing.value = t.value;
                        res.refilled += 1;
                    }
                }
            }
            Err(e) => {
                // The refill itself failed: adopt the rejected values and
                // report the transport error; findings still record.
                let kind = match &e {
                    Error::Transport { kind, .. } => *kind,
                    _ => ErrorType::Unknown,
                };
                failures.push(FillFailure {
                    locale: locale.to_string(),
                    key: None,
                    kind,
                    message: format!("gate refill: {e}"),
                });
                return by_key;
            }
        }
    }
    BTreeMap::new()
}

/// `quality.unresolved` shard payload: one `{check}: {message}` line per
/// finding, so the lockfile records *why* the gate let a value through.
fn unresolved_quality(findings: &[CheckFinding]) -> toml::Value {
    let mut m = toml::Table::new();
    m.insert(
        "unresolved".into(),
        toml::Value::Array(
            findings
                .iter()
                .map(|f| toml::Value::String(format!("{}: {}", f.check, f.message)))
                .collect(),
        ),
    );
    toml::Value::Table(m)
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
    // Absent tombstones are deliberately untranslated: no mode refills
    // them; `mark` re-arms a key explicitly.
    if shard.entries.get(key).is_some_and(|e| e.absent) {
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
