//! `intl-ai-docs` — AI localization for document trees (experimental).
//! Thin CLI over the intl-ai-docs pipeline; the corpus `intl-ai` binary
//! is untouched.

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand, ValueEnum};
use intl_ai_core::check::{Check, CheckOptions};
use intl_ai_core::diff::FindingKind;
use intl_ai_core::fill::FillOptions;
use intl_ai_core::lockfile::Origin;
use intl_ai_core::progress::ProgressEvent;
use intl_ai_core::selector::KeySelector;
use intl_ai_docs::checks as doc_checks;
use intl_ai_docs::config;
use intl_ai_docs::pipeline::{
    DocsCheckReport, DocsFillArgs, DocsFillReport, check_docs, fill_docs,
};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Parser)]
#[command(
    name = "intl-ai-docs",
    version,
    about = "AI localization for document trees (experimental)"
)]
struct Cli {
    /// Path to intl-ai-docs.{toml,json,yaml}; discovered upward from cwd.
    #[arg(long, global = true)]
    config: Option<PathBuf>,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Fill target locales from the source tree.
    Fill(FillArgs),
    /// Report missing/stale/modified/invalid segments without writing.
    Check(CheckArgs),
}

#[derive(clap::Args)]
struct FillArgs {
    /// Target locale(s); defaults to config targets. Repeatable.
    #[arg(long)]
    locale: Vec<String>,
    /// Glob over document-relative paths (e.g. 'guide/*.md'). Repeatable.
    #[arg(long)]
    files: Vec<String>,
    /// Glob over segment ids. Repeatable.
    #[arg(long)]
    keys: Vec<String>,
    /// File with segment-id globs, one per line.
    #[arg(long = "keys-file")]
    keys_file: Option<PathBuf>,
    /// Only segments whose source changed since last fill.
    #[arg(long)]
    stale: bool,
    /// Retranslate AI-owned segments even when source is unchanged.
    #[arg(long)]
    regenerate: bool,
    /// Also fill segments whose current value is human-owned.
    #[arg(long)]
    include_human: bool,
    /// Compute and report without writing files or shards.
    #[arg(long)]
    dry_run: bool,
    /// Skip the incremental check cache.
    #[arg(long)]
    no_cache: bool,
    /// Assume yes: skip the translation-count confirmation.
    #[arg(long)]
    yes: bool,
    /// Output: human | json.
    #[arg(long, value_enum, default_value = "human")]
    format: OutFmt,
}

#[derive(clap::Args)]
struct CheckArgs {
    /// Target locale(s); defaults to config targets. Repeatable.
    #[arg(long)]
    locale: Vec<String>,
    /// Glob over document-relative paths. Repeatable.
    #[arg(long)]
    files: Vec<String>,
    /// Glob over segment ids. Repeatable.
    #[arg(long)]
    keys: Vec<String>,
    /// File with segment-id globs, one per line.
    #[arg(long = "keys-file")]
    keys_file: Option<PathBuf>,
    /// Comma-separated kinds that make check fail:
    /// missing,stale,modified,extra,unreviewed,invalid.
    #[arg(long)]
    fail_on: Option<String>,
    /// Limit findings to one origin: ai | human.
    #[arg(long, value_enum)]
    origin: Option<OriginArg>,
    /// Skip the incremental check cache.
    #[arg(long)]
    no_cache: bool,
    /// Output: human | json.
    #[arg(long, value_enum, default_value = "human")]
    format: OutFmt,
}

#[derive(Clone, Copy, ValueEnum)]
enum OutFmt {
    Human,
    Json,
}

#[derive(Clone, Copy, ValueEnum)]
enum OriginArg {
    Ai,
    Human,
}

/// Minimal stderr progress for docs runs.
#[derive(Debug)]
struct StderrProgress;
impl intl_ai_core::progress::Progress for StderrProgress {
    fn on_event(&self, ev: ProgressEvent) {
        if let ProgressEvent::BatchStarted {
            locale,
            keys,
            attempt,
        } = &ev
            && *attempt == 0
        {
            eprintln!("[{locale}] translating {keys} segments");
        }
    }
}

fn selector(keys: &[String], keys_file: &Option<PathBuf>) -> Result<KeySelector> {
    let mut sel = KeySelector::from_specs(keys)?;
    if let Some(f) = keys_file {
        sel = sel.merge(KeySelector::from_file(f)?)?;
    }
    Ok(sel)
}

fn fail_on_kinds(s: &Option<String>) -> Result<Option<Vec<FindingKind>>> {
    let Some(s) = s else { return Ok(None) };
    s.split(',')
        .map(|k| match k.trim() {
            "missing" => Ok(FindingKind::Missing),
            "stale" => Ok(FindingKind::Stale),
            "modified" => Ok(FindingKind::Modified),
            "extra" => Ok(FindingKind::Extra),
            "unreviewed" => Ok(FindingKind::Unreviewed),
            "invalid" => Ok(FindingKind::Invalid),
            other => bail!("unknown fail-on kind {other:?}"),
        })
        .collect::<Result<Vec<_>>>()
        .map(Some)
}

/// `fill.validate` names resolve against doc builtins first, then corpus
/// builtins (icu, placeholder-parity, judge, dialect:<locale>).
fn gate_check(name: &str) -> Result<Box<dyn Check>> {
    if let Some(c) = doc_checks::builtin(name) {
        return Ok(c);
    }
    let c = intl_ai_checks::builtin(name).map_err(|e| anyhow::anyhow!("{e}"))?;
    if !c.supports_feedback() {
        bail!("fill.validate: check '{name}' cannot gate fill (needs supports_feedback)");
    }
    Ok(c)
}

fn cmd_fill(cfg: config::ResolvedDocsConfig, args: &FillArgs) -> Result<i32> {
    let checks: Vec<Box<dyn Check>> = args_config_checks(&cfg)?;
    let mut gate_checks: Vec<Box<dyn Check>> = Vec::new();
    for name in &cfg.config.fill.validate {
        gate_checks.push(gate_check(name)?);
    }
    let gate = if gate_checks.is_empty() {
        None
    } else {
        Some(intl_ai_core::check::Gate {
            checks: std::mem::take(&mut gate_checks),
            max_rounds: intl_ai_core::check::Gate::DEFAULT_MAX_ROUNDS,
        })
    };
    let _ = checks; // fill gate uses validate list only; checks run in `check`
    let transport = intl_ai_providers::build_transport_parts(
        &cfg.config.provider,
        &cfg.config_dir,
        cfg.max_retries(),
    )?;
    let files_sel = KeySelector::from_specs(&args.files)?;
    let fill_opts = FillOptions {
        locales: if args.locale.is_empty() {
            None
        } else {
            Some(args.locale.clone())
        },
        selector: selector(&args.keys, &args.keys_file)?,
        stale_only: args.stale,
        regenerate: args.regenerate,
        include_human: args.include_human,
        dry_run: args.dry_run,
        no_cache: args.no_cache,
        observer: Some(Arc::new(StderrProgress)),
    };
    let dargs = DocsFillArgs {
        fill: &fill_opts,
        files: &files_sel,
        gate: gate.as_ref(),
    };
    let report = fill_docs(&cfg, transport.as_ref(), &dargs, &StderrProgress)?;
    match args.format {
        OutFmt::Json => println!("{}", serde_json::to_string_pretty(&report)?),
        OutFmt::Human => print_fill_report(&report),
    }
    let failed = !report.failures.is_empty()
        || report.locales.values().any(|l| {
            l.files
                .values()
                .any(|f| !f.lint.is_empty() || !f.result.omitted.is_empty())
        });
    Ok(if failed { 1 } else { 0 })
}

fn cmd_check(cfg: config::ResolvedDocsConfig, args: &CheckArgs) -> Result<i32> {
    let files_sel = KeySelector::from_specs(&args.files)?;
    let checks_opts = CheckOptions {
        locales: if args.locale.is_empty() {
            None
        } else {
            Some(args.locale.clone())
        },
        origin_filter: args.origin.map(|o| match o {
            OriginArg::Ai => Origin::Ai,
            OriginArg::Human => Origin::Human,
        }),
        fail_on: fail_on_kinds(&args.fail_on)?,
        selector: selector(&args.keys, &args.keys_file)?,
        no_cache: args.no_cache,
        observer: Some(Arc::new(StderrProgress)),
    };
    // Document builtins always run; configured [[checks]] join them.
    let mut checks = doc_checks::all();
    checks.extend(intl_ai_checks::build_from(
        &cfg.config.checks,
        &cfg.config_dir,
    )?);
    let transport = if checks.iter().any(|c| c.needs_transport()) {
        Some(intl_ai_providers::build_transport_parts(
            &cfg.config.provider,
            &cfg.config_dir,
            cfg.max_retries(),
        )?)
    } else {
        None
    };
    let report = check_docs(
        &cfg,
        transport.as_deref(),
        &checks_opts,
        &files_sel,
        &checks,
    )?;
    match args.format {
        OutFmt::Json => println!("{}", serde_json::to_string_pretty(&report)?),
        OutFmt::Human => print_check_report(&report),
    }
    Ok(if report.has_issues { 1 } else { 0 })
}

fn args_config_checks(_cfg: &config::ResolvedDocsConfig) -> Result<Vec<Box<dyn Check>>> {
    Ok(Vec::new())
}

fn print_fill_report(r: &DocsFillReport) {
    for (locale, lr) in &r.locales {
        let (mut wrote, mut translated, mut failed, mut omitted) = (0, 0, 0, 0);
        let mut lint_n = 0;
        for f in lr.files.values() {
            wrote += usize::from(f.wrote);
            translated += f.result.written;
            omitted += f.result.omitted.len();
            lint_n += f.lint.len();
        }
        failed += lr.files_failed;
        println!(
            "{locale}: {wrote} file(s) written, {translated} segments translated, \
             {omitted} omitted, {lint_n} lint finding(s){}",
            if lr.files_failed > 0 {
                format!(", {failed} file(s) failed")
            } else {
                String::new()
            }
        );
        for (rel, f) in &lr.files {
            for m in &f.lint {
                println!("  {rel}: lint: {m}");
            }
        }
    }
    for f in &r.failures {
        match &f.key {
            Some(k) => eprintln!("  {}/{}: {k}: {}", f.locale, f.file, f.message),
            None => eprintln!("  {}/{}: {}", f.locale, f.file, f.message),
        }
    }
}

fn print_check_report(r: &DocsCheckReport) {
    for (scope, d) in &r.locales {
        let mut bits: Vec<String> = Vec::new();
        if !d.missing.is_empty() {
            bits.push(format!("{} missing", d.missing.len()));
        }
        if !d.stale.is_empty() {
            bits.push(format!("{} stale", d.stale.len()));
        }
        if !d.modified.is_empty() {
            bits.push(format!("{} modified", d.modified.len()));
        }
        if !d.extra.is_empty() {
            bits.push(format!("{} extra", d.extra.len()));
        }
        if !d.unreviewed.is_empty() {
            bits.push(format!("{} unreviewed", d.unreviewed.len()));
        }
        if !d.invalid.is_empty() {
            bits.push(format!("{} invalid", d.invalid.len()));
        }
        if bits.is_empty() {
            println!("{scope}: ok");
        } else {
            println!("{scope}: {}", bits.join(", "));
            for f in &d.invalid {
                println!("  {}: [{}] {}", f.key, f.check, f.message);
            }
        }
    }
    for e in &r.errors {
        eprintln!("  {e}");
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let cwd = std::env::current_dir()?;
    let cfg = config::load(cli.config.as_deref(), &cwd).context("loading intl-ai-docs config")?;
    let code = match &cli.cmd {
        Cmd::Fill(a) => cmd_fill(cfg, a)?,
        Cmd::Check(a) => cmd_check(cfg, a)?,
    };
    std::process::exit(code);
}
