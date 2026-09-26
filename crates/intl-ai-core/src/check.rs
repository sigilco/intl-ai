use crate::check_cache::{BATCH_KEY, CheckCache};
use crate::config::ResolvedConfig;
use crate::diff::{CheckFinding, FindingKind, LocaleDiff, diff, effective_origin};
use crate::error::Result;
use crate::flatten::flatten;
use crate::hash::fingerprint;
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

/// How a check consumes its inputs — drives the incremental cache
/// (part B). PerKey fingerprints each item separately; WholeBatch
/// fingerprints the whole item list once (exec sees it as a single
/// request, so one changed key re-runs all of them).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckGranularity {
    PerKey,
    WholeBatch,
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
    /// Whether this check's findings are mechanically actionable reviewer
    /// notes — the fill-time gate (plan W2b-C) only accepts these.
    /// `icu`/`placeholder-parity`/`judge` qualify; dialect/spec findings
    /// describe style judgments the model cannot reliably re-derive, and
    /// exec findings are arbitrary text.
    fn supports_feedback(&self) -> bool {
        false
    }
    fn granularity(&self) -> CheckGranularity {
        CheckGranularity::PerKey
    }
    /// Ambient inputs beyond (key, source, target) that change a check's
    /// output — wordlist identities, thresholds, the exec command line.
    /// Folded into the cache fingerprint; required so every impl makes a
    /// deliberate decision about what its results depend on.
    fn cache_ctx(&self) -> BTreeMap<String, String>;
    fn run(&self, ctx: &CheckCtx, items: &[CheckItem]) -> Result<Vec<CheckFinding>>;
}

/// The fill-time validation gate (plan W2b-C): a configured subset of
/// checks runs inside `fill` between `translate()` and adoption. Members
/// must all report `supports_feedback()`. v1 allows one corrective round
/// (`max_rounds = 1`); oscillation is bounded by construction.
pub struct Gate {
    pub checks: Vec<Box<dyn Check>>,
    pub max_rounds: u32,
}

impl Gate {
    /// Fixed corrective-round cap for v1 (`refill_rounds` config deferred).
    pub const DEFAULT_MAX_ROUNDS: u32 = 1;
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
    // `[check] cache = false` or --no-cache disables the findings cache.
    let check_cache_enabled = !opts.no_cache && cfg.config.check.cache;
    let mut check_cache = if check_cache_enabled {
        CheckCache::load(&cfg.check_cache_path())
    } else {
        CheckCache::default()
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
        run_checks(
            &locale,
            checks,
            &ctx,
            &items,
            &mut check_cache,
            &mut d,
            &mut report,
        );
        if opts.selector.is_any() {
            let live: std::collections::BTreeSet<String> =
                items.iter().map(|i| i.key.clone()).collect();
            check_cache.prune_locale(&locale, &live);
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
    if check_cache_enabled {
        check_cache.save(&cfg.check_cache_path());
    }
    Ok(report)
}

/// Run `checks` over `items`, replaying from `cache` where the
/// fingerprint of a check's inputs still matches. Cache misses run and
/// store; errors are never cached (a flaky check retries next run).
fn run_checks(
    locale: &str,
    checks: &[Box<dyn Check>],
    ctx: &CheckCtx,
    items: &[CheckItem],
    cache: &mut CheckCache,
    d: &mut LocaleDiff,
    report: &mut CheckReport,
) {
    for c in checks {
        // Ambient inputs the check itself declares (wordlist, command
        // line, threshold) plus the locale pair it ran under.
        let mut ambient: Vec<String> = vec![
            ctx.source_locale.to_string(),
            ctx.target_locale.to_string(),
            ctx.locale_instruction.unwrap_or_default().to_string(),
        ];
        for (k, v) in c.cache_ctx() {
            ambient.push(k);
            ambient.push(v);
        }
        let ambient_refs: Vec<&str> = ambient.iter().map(|s| s.as_str()).collect();

        match c.granularity() {
            CheckGranularity::WholeBatch => {
                let mut parts = ambient_refs.clone();
                for i in items {
                    parts.extend([i.key.as_str(), i.source.as_deref().unwrap_or(""), &i.target]);
                }
                let fp = fingerprint(&parts);
                if let Some(hit) = cache.get(locale, BATCH_KEY, c.id())
                    && hit.fingerprint == fp
                {
                    d.invalid
                        .extend(hit.findings.iter().cloned().map(mark_cached));
                    continue;
                }
                match c.run(ctx, items) {
                    Ok(findings) => {
                        cache.put(locale, BATCH_KEY, c.id(), fp, findings.clone());
                        d.invalid.extend(findings);
                    }
                    Err(e) => report.errors.push(format!("{locale}/{}: {e}", c.id())),
                }
            }
            CheckGranularity::PerKey => {
                let mut run_items: Vec<CheckItem> = Vec::new();
                let mut run_fps: Vec<(String, String)> = Vec::new();
                for item in items {
                    let mut parts = ambient_refs.clone();
                    parts.extend([
                        item.key.as_str(),
                        item.source.as_deref().unwrap_or(""),
                        &item.target,
                    ]);
                    let fp = fingerprint(&parts);
                    if let Some(hit) = cache.get(locale, &item.key, c.id())
                        && hit.fingerprint == fp
                    {
                        d.invalid
                            .extend(hit.findings.iter().cloned().map(mark_cached));
                    } else {
                        run_fps.push((item.key.clone(), fp));
                        run_items.push(item.clone());
                    }
                }
                if run_items.is_empty() {
                    continue;
                }
                match c.run(ctx, &run_items) {
                    Ok(findings) => {
                        let mut per_key: BTreeMap<String, Vec<CheckFinding>> = BTreeMap::new();
                        for f in findings {
                            per_key.entry(f.key.clone()).or_default().push(f);
                        }
                        for f in per_key.values().flatten() {
                            d.invalid.push(f.clone());
                        }
                        for (key, fp) in run_fps {
                            let fs = per_key.get(&key).cloned().unwrap_or_default();
                            cache.put(locale, &key, c.id(), fp, fs);
                        }
                    }
                    Err(e) => report.errors.push(format!("{locale}/{}: {e}", c.id())),
                }
            }
        }
    }
}

fn mark_cached(mut f: CheckFinding) -> CheckFinding {
    f.cached = true;
    f
}
