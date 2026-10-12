//! Pipelines: `fill_docs` and `check_docs` walk the source tree,
//! extract each document into (skeleton + segment FlatMap), and drive
//! the shared FlatMap engine (`fill_flat` / `check_flat`) per file per
//! locale — reconciliation, adoption, GC, batching, gate rounds, origin
//! tracking, staleness and the incremental check cache all reuse the
//! corpus machinery unchanged. Document-specific work is exactly the
//! boundary pieces: shard-per-document, target re-identification
//! (`rekey`), merge splicing, and the write-gate/document lint.

use crate::config::ResolvedDocsConfig;
use crate::doc::DocFormat;
use crate::lint;
use crate::mdx::Mdx;
use crate::rekey::rekey;
use intl_ai_core::check::{Check, CheckCtx, CheckOptions, FlatCheckOutcome, Gate, check_flat};
use intl_ai_core::check_cache::CheckCache;
use intl_ai_core::diff::{CheckFinding, FindingKind, LocaleDiff};
use intl_ai_core::error::{Error, Result};
use intl_ai_core::fill::{FillFailure, FillOptions, FlatCtx, LocaleFillResult, fill_flat};
use intl_ai_core::flatten::FlatMap;
use intl_ai_core::hash::source_hash;
use intl_ai_core::lockfile::{ShardLock, load_shard_at, save_shard_at};
use intl_ai_core::progress::{Pipeline, Progress, ProgressEvent};
use intl_ai_core::selector::KeySelector;
use intl_ai_formats::json::write_atomic;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

/// A document's fill outcome: the engine result plus write-gate state.
#[derive(Debug, Serialize)]
pub struct FileFillOutcome {
    #[serde(flatten)]
    pub result: LocaleFillResult,
    /// Merged file written to disk (false on dry run, lint block, or
    /// no changes worth writing).
    pub wrote: bool,
    /// Segments spliced from source text because no target value
    /// existed (expected for omitted/failed segments).
    pub fell_back: usize,
    /// Write-gate lint findings; a nonempty list blocks the write.
    pub lint: Vec<String>,
    /// Segment extraction warnings for the source document.
    pub warnings: Vec<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct LocaleDocsFill {
    pub files: BTreeMap<String, FileFillOutcome>,
    /// Files skipped by `--files` scope or failures are absent; totals
    /// aggregate across files.
    pub files_written: usize,
    pub files_failed: usize,
    pub skipped_by_selector: usize,
}

#[derive(Debug, Default, Serialize)]
pub struct DocsFillReport {
    pub locales: BTreeMap<String, LocaleDocsFill>,
    /// Transport/batch-level failures (locale + file + optional key).
    pub failures: Vec<DocsFailure>,
}

#[derive(Debug, Serialize)]
pub struct DocsFailure {
    pub locale: String,
    pub file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    pub message: String,
}

pub struct DocsFillArgs<'a> {
    pub fill: &'a FillOptions,
    /// Glob scope on document-relative paths (`guide/*.md`).
    pub files: &'a KeySelector,
    pub gate: Option<&'a Gate>,
}

/// The `DocFormat` impl for the configured `format` ("mdx"/"markdown").
pub fn format_impl(cfg: &ResolvedDocsConfig) -> Result<Box<dyn DocFormat>> {
    match cfg.config.format.as_str() {
        "mdx" | "markdown" => Ok(Box::new(Mdx::new(cfg.config.frontmatter.keys.clone()))),
        other => Err(Error::Config(format!("unknown docs format {other:?}"))),
    }
}

/// Source documents under `source_dir`, document-relative paths —
/// excluding hidden dirs, `node_modules`, target locale dirs, and the
/// configured `exclude` globs.
pub fn discover_docs(cfg: &ResolvedDocsConfig) -> Result<Vec<String>> {
    let root = cfg.source_dir();
    let excl = globset::GlobSetBuilder::new();
    let mut excl = excl;
    for g in &cfg.config.exclude {
        excl.add(
            globset::Glob::new(g).map_err(|e| Error::Config(format!("exclude glob {g:?}: {e}")))?,
        );
    }
    let excl = excl
        .build()
        .map_err(|e| Error::Config(format!("exclude globs: {e}")))?;
    let skip_dirs: Vec<String> = std::iter::once("node_modules".to_string())
        .chain(cfg.config.targets.iter().cloned())
        .collect();
    let mut out = Vec::new();
    walk(&root, &root, &skip_dirs, &excl, &mut out)?;
    out.sort();
    Ok(out)
}

fn walk(
    root: &PathBuf,
    dir: &PathBuf,
    skip_dirs: &[String],
    excl: &globset::GlobSet,
    out: &mut Vec<String>,
) -> Result<()> {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) if dir == root => {
            return Err(Error::Io {
                path: dir.clone(),
                source: e,
            });
        }
        Err(_) => return Ok(()),
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        if path.is_dir() {
            if !skip_dirs.contains(&name) {
                walk(root, &path, skip_dirs, excl, out)?;
            }
            continue;
        }
        let Ok(rel) = path.strip_prefix(root) else {
            continue;
        };
        let rel = rel.to_string_lossy().replace('\\', "/");
        if !(rel.ends_with(".md") || rel.ends_with(".mdx")) {
            continue;
        }
        if excl.is_match(&rel) {
            continue;
        }
        out.push(rel);
    }
    Ok(())
}

fn src_hashes(doc: &crate::doc::Document) -> BTreeMap<String, String> {
    doc.segments
        .iter()
        .map(|(k, v)| (k.clone(), source_hash(v)))
        .collect()
}

/// `FlatCtx` for the docs engine — markdown syntax hint instead of the
/// corpus's i18n placeholder contract.
fn flat_ctx(cfg: &ResolvedDocsConfig, locale: &str) -> FlatCtx {
    FlatCtx {
        source_locale: cfg.config.source.clone(),
        instruction: cfg.instruction_for(locale),
        glossary: cfg.config.glossary.clone(),
        batch_size: cfg.config.batch_size.unwrap_or(usize::MAX).max(1),
        syntax_hint: concat!(
            "Segments are raw markdown inline content from documentation. ",
            "Preserve markdown markup verbatim: code spans, links and their ",
            "destinations, images, inline HTML/JSX tags, footnote refs, ",
            "math, and emphasis/strikethrough delimiters must all survive."
        )
        .to_string(),
    }
}

/// Fill target locales from the source document tree. Per locale:
/// one lock, then per document — extract, rekey the existing target
/// (if any), run `fill_flat`, merge + lint, write.
pub fn fill_docs(
    cfg: &ResolvedDocsConfig,
    transport: &dyn intl_ai_core::transport::Transport,
    opts: &DocsFillArgs,
    obs: &dyn Progress,
) -> Result<DocsFillReport> {
    let fmt = format_impl(cfg)?;
    let files = discover_docs(cfg)?;
    let locales: Vec<String> = match &opts.fill.locales {
        Some(l) if !l.is_empty() => l.clone(),
        _ => cfg.config.targets.clone(),
    };
    obs.on_event(ProgressEvent::RunStarted {
        pipeline: Pipeline::Fill,
        locales: locales.clone(),
    });
    let mut report = DocsFillReport {
        locales: BTreeMap::new(),
        failures: Vec::new(),
    };

    for locale in &locales {
        let _lock = ShardLock::acquire(&cfg.config_dir, &format!("docs-{locale}"))?;
        let prefix = cfg.link_prefix(locale);
        let cx = flat_ctx(cfg, locale);
        let mut lr = LocaleDocsFill::default();

        for rel in &files {
            if !opts.files.matches(rel) {
                lr.skipped_by_selector += 1;
                continue;
            }
            let outcome = fill_file(
                cfg,
                fmt.as_ref(),
                rel,
                locale,
                &prefix,
                &cx,
                transport,
                opts,
                obs,
            );
            match outcome {
                Ok((o, file_failures)) => {
                    report.failures.extend(file_failures);
                    if o.wrote {
                        lr.files_written += 1;
                    }
                    if !o.lint.is_empty() || !o.result.omitted.is_empty() {
                        lr.files_failed += 1;
                    }
                    lr.files.insert(rel.clone(), o);
                }
                Err(e) => {
                    lr.files_failed += 1;
                    report.failures.push(DocsFailure {
                        locale: locale.clone(),
                        file: rel.clone(),
                        key: None,
                        message: e.to_string(),
                    });
                }
            }
        }
        obs.on_event(ProgressEvent::LocaleFinished {
            pipeline: Pipeline::Fill,
            locale: locale.clone(),
        });
        report.locales.insert(locale.clone(), lr);
    }
    obs.on_event(ProgressEvent::RunFinished {
        pipeline: Pipeline::Fill,
        locales: locales.len(),
        failures: report.failures.len(),
    });
    Ok(report)
}

#[allow(clippy::too_many_arguments)]
fn fill_file(
    cfg: &ResolvedDocsConfig,
    fmt: &dyn DocFormat,
    rel: &str,
    locale: &str,
    link_prefix: &str,
    cx: &FlatCtx,
    transport: &dyn intl_ai_core::transport::Transport,
    opts: &DocsFillArgs,
    obs: &dyn Progress,
) -> Result<(FileFillOutcome, Vec<DocsFailure>)> {
    let src_path = cfg.source_dir().join(rel);
    let src_text = fs::read_to_string(&src_path).map_err(|source| Error::Io {
        path: src_path.clone(),
        source,
    })?;
    let doc = fmt.extract(&src_text, &src_path)?;
    let hashes = src_hashes(&doc);
    let shard_path = cfg.shard_path(locale, rel);
    let mut shard = load_shard_at(&shard_path)?;

    // Existing target document: rekey its spans onto source seg ids.
    let target_path = cfg.target_dir(locale).join(rel);
    let mut tgt_flat = FlatMap::new();
    if let Ok(tgt_text) = fs::read_to_string(&target_path) {
        let tgt_doc = fmt.extract(&tgt_text, &target_path)?;
        tgt_flat = rekey(&tgt_doc, &shard, &doc.order, link_prefix);
    }

    let mut out_flat = tgt_flat.clone();
    let mut adopt = |k: &str, v: &str| -> Option<String> {
        out_flat.insert(k.to_string(), v.to_string());
        None
    };
    let (result, raw_failures) = fill_flat(
        &doc.segments,
        &hashes,
        &tgt_flat,
        &mut adopt,
        &mut shard,
        opts.fill,
        cx,
        locale,
        transport,
        opts.gate,
        obs,
    )?;

    // Write-gate: merge the adopted flat map into the skeleton, then
    // lint segments + document. Any lint finding blocks the write —
    // a corrupted document is worse than a missing one.
    let merged = fmt.merge(&doc, &out_flat, link_prefix)?;
    let merged_text = String::from_utf8_lossy(&merged.bytes).to_string();
    let mut lint_findings = Vec::new();
    for (seg, val) in &out_flat {
        if let Some(src_seg) = doc.segments.get(seg) {
            lint_findings.extend(
                lint::lint_segment(src_seg, val)
                    .into_iter()
                    .map(|m| format!("{seg}: {m}")),
            );
        }
    }
    // Re-extract the merged document: its own spans exempt
    // partially-translated lines (frontmatter scalars) from opaque-line
    // parity, and an unparseable merge is itself a finding.
    // The gate must see what `check` will see: rekey the merged spans
    // back onto source ids and lint THAT flat, not just the pre-splice
    // values — a value carrying an extra paragraph break re-parses into
    // a new segment and misaligns every span that follows it.
    let mut out_spans: Vec<std::ops::Range<usize>> = Vec::new();
    match fmt.extract(&merged_text, &target_path) {
        Ok(md) => {
            out_spans = md.spans.iter().map(|s| s.range.clone()).collect();
            let merged_flat = rekey(&md, &shard, &doc.order, link_prefix);
            for (seg, src_seg) in &doc.segments {
                match merged_flat.get(seg) {
                    Some(v) => lint_findings.extend(
                        lint::lint_segment(src_seg, v)
                            .into_iter()
                            .map(|m| format!("{seg}: post-merge: {m}")),
                    ),
                    None if out_flat.contains_key(seg) => lint_findings.push(format!(
                        "{seg}: post-merge: translated segment lost in splice"
                    )),
                    None => {}
                }
            }
            for seg in merged_flat.keys() {
                if !doc.segments.contains_key(seg) {
                    lint_findings.push(format!("{seg}: post-merge: extra segment in output"));
                }
            }
        }
        Err(e) => {
            lint_findings.push(format!("merged document fails re-extraction: {e}"));
        }
    }
    let src_span_ranges: Vec<std::ops::Range<usize>> =
        doc.spans.iter().map(|s| s.range.clone()).collect();
    lint_findings.extend(lint::lint_document(
        &doc.source,
        &src_span_ranges,
        &merged_text,
        &out_spans,
        link_prefix,
    ));

    let shard_dirty = result.written > 0
        || result.adopted_human > 0
        || result.reconciled_human > 0
        || result.pruned > 0;
    let mut wrote = false;
    if !opts.fill.dry_run && lint_findings.is_empty() {
        if shard_dirty {
            save_shard_at(&shard_path, &shard)?;
        }
        if target_outdated(&target_path, &merged.bytes) {
            write_atomic(&target_path, &merged.bytes).map_err(Error::Format)?;
            wrote = true;
        }
    }
    let failures = raw_failures
        .into_iter()
        .map(|f: FillFailure| DocsFailure {
            locale: locale.to_string(),
            file: rel.to_string(),
            key: f.key,
            message: f.message,
        })
        .collect();
    Ok((
        FileFillOutcome {
            result,
            wrote,
            fell_back: merged.fell_back.len(),
            lint: lint_findings,
            warnings: doc.warnings,
        },
        failures,
    ))
}

fn target_outdated(path: &PathBuf, bytes: &[u8]) -> bool {
    fs::read(path).map(|b| b != bytes).unwrap_or(true)
}

/// `check` over documents: diff/staleness/provenance per file plus the
/// always-on document builtins and any configured `[[checks]]`.
/// Lint findings on the on-disk target land in `invalid`.
pub fn check_docs(
    cfg: &ResolvedDocsConfig,
    transport: Option<&dyn intl_ai_core::transport::Transport>,
    opts: &CheckOptions,
    files_sel: &KeySelector,
    checks: &[Box<dyn Check>],
) -> Result<DocsCheckReport> {
    let fmt = format_impl(cfg)?;
    let files = discover_docs(cfg)?;
    let locales: Vec<String> = match &opts.locales {
        Some(l) if !l.is_empty() => l.clone(),
        _ => cfg.config.targets.clone(),
    };
    let fail_on: &[FindingKind] = opts.fail_on.as_deref().unwrap_or(&cfg.config.check.fail_on);
    static NOOP: intl_ai_core::progress::NoopProgress = intl_ai_core::progress::NoopProgress;
    let obs: &dyn Progress = opts.observer.as_deref().unwrap_or(&NOOP);
    obs.on_event(ProgressEvent::RunStarted {
        pipeline: Pipeline::Check,
        locales: locales.clone(),
    });
    let mut check_cache = if !opts.no_cache && cfg.config.check.cache {
        CheckCache::load(&cfg.check_cache_path())
    } else {
        CheckCache::default()
    };
    let mut report = DocsCheckReport {
        locales: BTreeMap::new(),
        errors: Vec::new(),
        has_issues: false,
        has_findings: false,
    };

    for locale in &locales {
        let prefix = cfg.link_prefix(locale);
        let instruction = cfg.instruction_for(locale);
        let ctx = CheckCtx {
            source_locale: &cfg.config.source,
            target_locale: locale,
            transport,
            locale_instruction: instruction.as_deref(),
        };
        for rel in &files {
            if !files_sel.matches(rel) {
                continue;
            }
            let scope = format!("{locale}/{rel}");
            let src_path = cfg.source_dir().join(rel);
            let Ok(src_text) = fs::read_to_string(&src_path) else {
                continue;
            };
            let doc = fmt.extract(&src_text, &src_path)?;
            let hashes = src_hashes(&doc);
            let shard = load_shard_at(&cfg.shard_path(locale, rel))?;
            let target_path = cfg.target_dir(locale).join(rel);
            let tgt_text = fs::read_to_string(&target_path).ok();
            let mut tgt_doc = None;
            let tgt_flat = match &tgt_text {
                Some(t) => {
                    let d = fmt.extract(t, &target_path)?;
                    let flat = rekey(&d, &shard, &doc.order, &prefix);
                    tgt_doc = Some(d);
                    flat
                }
                None => FlatMap::new(),
            };
            let out: FlatCheckOutcome = check_flat(
                locale,
                &scope,
                &doc.segments,
                &hashes,
                &tgt_flat,
                &shard,
                &opts.selector,
                opts.origin_filter,
                fail_on,
                checks,
                &ctx,
                cfg.config.quality.as_ref(),
                &mut check_cache,
                obs,
            );
            let mut d = out.diff;
            // Document-level lint on the on-disk target: opaque-line
            // parity vs the source skeleton and link-prefix coverage.
            if let (Some(t), Some(td)) = (&tgt_text, &tgt_doc) {
                let src_spans: Vec<std::ops::Range<usize>> =
                    doc.spans.iter().map(|s| s.range.clone()).collect();
                let tgt_spans: Vec<std::ops::Range<usize>> =
                    td.spans.iter().map(|s| s.range.clone()).collect();
                for msg in lint::lint_document(&doc.source, &src_spans, t, &tgt_spans, &prefix) {
                    d.invalid.push(CheckFinding {
                        key: rel.to_string(),
                        check: "doc:lint".to_string(),
                        message: msg,
                        cached: false,
                    });
                }
            }
            report.errors.extend(out.errors);
            if out.has_findings || !d.invalid.is_empty() {
                report.has_findings = true;
            }
            if out.has_issues || (fail_on.contains(&FindingKind::Invalid) && !d.invalid.is_empty())
            {
                report.has_issues = true;
            }
            report.locales.insert(scope, d);
        }
        obs.on_event(ProgressEvent::LocaleFinished {
            pipeline: Pipeline::Check,
            locale: locale.clone(),
        });
    }
    if !report.errors.is_empty() {
        report.has_issues = true;
        report.has_findings = true;
    }
    if !opts.no_cache && cfg.config.check.cache {
        check_cache.save(&cfg.check_cache_path());
    }
    obs.on_event(ProgressEvent::RunFinished {
        pipeline: Pipeline::Check,
        locales: locales.len(),
        failures: report.errors.len(),
    });
    Ok(report)
}

#[derive(Debug, Default, Serialize)]
pub struct DocsCheckReport {
    /// Keyed `"{locale}/{relpath}"`.
    pub locales: BTreeMap<String, LocaleDiff>,
    pub errors: Vec<String>,
    pub has_issues: bool,
    pub has_findings: bool,
}
