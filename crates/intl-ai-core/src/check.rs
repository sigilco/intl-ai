use crate::config::ResolvedConfig;
use crate::diff::{FindingKind, LocaleDiff, diff, effective_origin};
use crate::error::{Error, Result};
use crate::flatten::flatten;
use crate::lockfile::{Origin, load_shard};
use intl_ai_formats::json::read;
use serde::Serialize;
use std::collections::{BTreeMap, HashSet};

#[derive(Debug)]
pub struct CheckOptions {
    pub locales: Option<Vec<String>>,
    /// None = all. Some(ai|human) scopes stale/modified/unreviewed to
    /// entries of that effective origin (missing/extra are origin-less).
    pub origin_filter: Option<Origin>,
    /// Kinds that make `check` report issues (config check.fail_on wins
    /// unless overridden per-invocation).
    pub fail_on: Option<Vec<FindingKind>>,
}

#[derive(Debug, Serialize)]
pub struct CheckReport {
    pub locales: BTreeMap<String, LocaleDiff>,
    pub has_issues: bool,
}

/// Report findings without writing anything (cargo check / tsc --noEmit
/// precedent). Same gates apply to human and AI values.
pub fn check(cfg: &ResolvedConfig, opts: &CheckOptions) -> Result<CheckReport> {
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
    let fail_on: &[FindingKind] = opts.fail_on.as_deref().unwrap_or(&cfg.config.check.fail_on);

    let mut report = CheckReport {
        locales: BTreeMap::new(),
        has_issues: false,
    };

    for locale in locales {
        let target_path = locale_dir.join(format!("{locale}.json"));
        let target = read(&target_path)?.map(|v| flatten(&v)).unwrap_or_default();
        let shard = load_shard(&locale_dir, &locale)?;
        let mut d = diff(&source, &target, &shard, &HashSet::new());

        if let Some(origin) = opts.origin_filter {
            let keep = |key: &String| {
                shard
                    .entries
                    .get(key)
                    .map(|e| effective_origin(e, target.get(key), &HashSet::new(), key) == origin)
                    .unwrap_or(false)
            };
            d.stale.retain(&keep);
            d.modified.retain(&keep);
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
                            && !e.reviewed
                            && effective_origin(e, target.get(*k), &HashSet::new(), k)
                                == Origin::Human
                    })
                    .count();
            }
        }

        for kind in fail_on {
            let hit = match kind {
                FindingKind::Missing => !d.missing.is_empty(),
                FindingKind::Stale => !d.stale.is_empty(),
                FindingKind::Invalid => false, // invalid lands with W2 checks
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
    Ok(report)
}
