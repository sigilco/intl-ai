use crate::config::ResolvedConfig;
use crate::diff::{CheckFinding, FindingKind, LocaleDiff, diff, effective_origin};
use crate::error::{Error, Result};
use crate::flatten::flatten;
use crate::lockfile::{Origin, load_shard};
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
    pub has_issues: bool,
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
    let fail_on: &[FindingKind] = opts.fail_on.as_deref().unwrap_or(&cfg.config.check.fail_on);

    let mut report = CheckReport {
        locales: BTreeMap::new(),
        errors: Vec::new(),
        has_issues: false,
    };

    for locale in locales {
        let target_path = cfg.locale_path(&locale);
        let target = read(&target_path)?.map(|v| flatten(&v)).unwrap_or_default();
        let shard = load_shard(&locale_dir, &locale)?;
        let mut d = diff(&source, &src_hashes, &target, &shard, &HashSet::new());

        let items: Vec<CheckItem> = target
            .iter()
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
                    .count();
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
                    .count();
            }
        }

        for kind in fail_on {
            let hit = match kind {
                FindingKind::Missing => !d.missing.is_empty(),
                FindingKind::Stale => !d.stale.is_empty(),
                FindingKind::Invalid => !d.invalid.is_empty(),
                FindingKind::Modified => !d.modified.is_empty(),
                FindingKind::Extra => !d.extra.is_empty(),
                FindingKind::Unreviewed => d.unreviewed > 0,
            };
            if hit {
                report.has_issues = true;
            }
        }
        report.locales.insert(locale, d);
    }
    if !report.errors.is_empty() {
        report.has_issues = true;
    }
    cache.save(&cfg.cache_path());
    Ok(report)
}
