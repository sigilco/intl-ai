use crate::config::ResolvedConfig;
use crate::diff::{CheckFinding, FindingKind, LocaleDiff, diff, effective_origin};
use crate::error::Result;
use crate::flatten::flatten;
use crate::lockfile::{Origin, load_shard};
use crate::selector::KeySelector;
use crate::stat_cache::StatCache;
use crate::transport::Transport;
use intl_ai_formats::read;
use serde::Serialize;
use std::collections::{BTreeMap, HashSet};

#[derive(Debug)]
pub struct CheckOptions {
    pub locales: Option<Vec<String>>,
    /// None = all. Some(ai|human) scopes stale/modified/unreviewed/invalid to
    /// entries of that effective origin (missing/extra are origin-less).
    pub origin_filter: Option<Origin>,
    /// Kinds that make `check` report issues (config check.fail_on wins
    /// unless overridden per-invocation).
    pub fail_on: Option<Vec<FindingKind>>,
    /// `check --keys`: scopes every finding bucket and the check-item batch.
    pub selector: KeySelector,
    /// Skip the stat-cache read/write for this run.
    pub no_cache: bool,
}

/// One flattened target value under review.
#[derive(Debug, Clone)]
pub struct CheckItem {
    pub key: String,
    /// Source string for the key when the source locale has one.
    pub source: Option<String>,
    pub target: String,
}

/// Per-locale context handed to a check run.
pub struct CheckCtx<'a> {
    pub source_locale: &'a str,
    pub target_locale: &'a str,
    /// Resolved provider transport; provider-backed checks (`judge`) use it,
    /// rule checks ignore it.
    pub transport: Option<&'a dyn Transport>,
    /// Freeform locale instruction resolved for the target locale.
    pub locale_instruction: Option<&'a str>,
}

/// A check rule or external checker. Implementations live in
/// `intl-ai-checks`; the trait sits in core so `check()` can run them without
/// knowing the impls. `run` gets the locale's full item batch in one call
/// (provider-backed and exec checks batch; the trait never makes
/// per-item calls).
pub trait Check {
    /// Stable id used in findings (`icu`, `placeholder-parity`,
    /// `dialect:en-US`, spec id, exec check name).
    fn id(&self) -> &str;
    /// True when the check needs the provider transport (`judge`). Lets the
    /// caller skip transport setup for pure rule runs.
    fn needs_transport(&self) -> bool {
        false
    }
    fn run(&self, ctx: &CheckCtx, items: &[CheckItem]) -> Result<Vec<CheckFinding>>;
}

#[derive(Debug, Serialize)]
pub struct CheckReport {
    pub locales: BTreeMap<String, LocaleDiff>,
    /// Check-level failures (spec load, exec spawn, provider error).
    /// `{locale}/{check}: {error}` lines; fail-closed: any entry sets
    /// `has_issues` regardless of `fail_on`.
    pub errors: Vec<String>,
    /// Gate outcome: a `fail_on` kind or a check-level error fired (M4:
    /// kept for callers that mean the gate; `has_findings` means "the
    /// report lists anything").
    pub has_issues: bool,
    /// Any finding at all — missing/stale/modified/extra/unreviewed/
    /// invalid or a check-level error, independent of `fail_on`.
    pub has_findings: bool,
}

/// Report findings without writing anything (cargo check / tsc --noEmit
/// precedent). Same gates apply to human and AI values. `checks` are the
/// built `[[checks]]` entries (intl-ai-checks); `transport` is resolved only
/// when some check needs it (judge), callers may pass None otherwise.
pub fn check(
    cfg: &ResolvedConfig,
    opts: &CheckOptions,
    checks: &[Box<dyn Check>],
    transport: Option<&dyn Transport>,
) -> Result<CheckReport> {
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
    let fail_on: &[FindingKind] = opts.fail_on.as_deref().unwrap_or(&cfg.config.check.fail_on);

    let mut report = CheckReport {
        locales: BTreeMap::new(),
        errors: Vec::new(),
        has_issues: false,
        has_findings: false,
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
        let target_path = cfg.locale_path(&locale);
        let target = read(&target_path)?.map(|v| flatten(&v)).unwrap_or_default();
        let shard = load_shard(&locale_dir, &locale)?;
        let mut d = diff(&source, &src_hashes, &target, &shard, &HashSet::new());

        let items: Vec<CheckItem> = target
            .iter()
            .filter(|(key, _)| opts.selector.matches(key))
            .map(|(key, value)| CheckItem {
                key: key.clone(),
                source: source.get(key).cloned(),
                target: value.clone(),
            })
            .collect();
        let instruction = cfg.instruction_for(&locale);
        let ctx = CheckCtx {
            source_locale: &cfg.config.source,
            target_locale: &locale,
            transport,
            locale_instruction: instruction.as_deref(),
        };
        for c in checks {
            match c.run(&ctx, &items) {
                Ok(findings) => d.invalid.extend(findings),
                Err(e) => report.errors.push(format!("{locale}/{}: {e}", c.id())),
            }
        }

        if let Some(origin) = opts.origin_filter {
            // Keys with no lockfile entry are file-resident values the tool
            // never wrote: effectively human.
            let keep = |key: &String| {
                shard
                    .entries
                    .get(key)
                    .map(|e| effective_origin(e, target.get(key), &HashSet::new(), key))
                    .unwrap_or(Origin::Human)
                    == origin
            };
            d.stale.retain(&keep);
            d.modified.retain(&keep);
            d.invalid.retain(|f| keep(&f.key));
            if origin == Origin::Ai {
                d.unreviewed = shard
                    .entries
                    .iter()
                    .filter(|(k, e)| {
                        target.contains_key(*k)
                            && !e.reviewed
                            && effective_origin(e, target.get(*k), &HashSet::new(), k) == Origin::Ai
                    })
                    .map(|(k, _)| k.clone())
                    .collect();
            } else {
                d.unreviewed = shard
                    .entries
                    .iter()
                    .filter(|(k, e)| {
                        target.contains_key(*k)
                            && effective_origin(e, target.get(*k), &HashSet::new(), k)
                                == Origin::Human
                            && (!e.reviewed
                                || (e.origin == Origin::Human && target.get(*k) != Some(&e.value)))
                    })
                    .map(|(k, _)| k.clone())
                    .collect();
            }
        }

        // `check --keys`: scope every finding bucket, not just the item
        // batch the checks ran on.
        d.missing.retain(|k| opts.selector.matches(k));
        d.stale.retain(|k| opts.selector.matches(k));
        d.modified.retain(|k| opts.selector.matches(k));
        d.extra.retain(|k| opts.selector.matches(k));
        d.unreviewed.retain(|k| opts.selector.matches(k));
        d.invalid.retain(|f| opts.selector.matches(&f.key));

        if !d.missing.is_empty()
            || !d.stale.is_empty()
            || !d.modified.is_empty()
            || !d.extra.is_empty()
            || !d.unreviewed.is_empty()
            || !d.invalid.is_empty()
        {
            report.has_findings = true;
        }

        for kind in fail_on {
            let hit = match kind {
                FindingKind::Missing => !d.missing.is_empty(),
                FindingKind::Stale => !d.stale.is_empty(),
                FindingKind::Invalid => !d.invalid.is_empty(),
                FindingKind::Modified => !d.modified.is_empty(),
                FindingKind::Extra => !d.extra.is_empty(),
                FindingKind::Unreviewed => !d.unreviewed.is_empty(),
            };
            if hit {
                report.has_issues = true;
            }
        }
        report.locales.insert(locale, d);
    }
    if !report.errors.is_empty() {
        report.has_issues = true;
        report.has_findings = true;
    }
    if !opts.no_cache {
        cache.save(&cfg.cache_path());
    }
    Ok(report)
}
