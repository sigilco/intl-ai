use anyhow::{Result, anyhow};
use intl_ai_core::fill::{FillOptions, fill};

use crate::commands::{is_json, print_json, resolve_config, selector};
use crate::{Cli, FillArgs};

pub fn run(cli: &Cli, args: &FillArgs) -> Result<u8> {
    let cfg = resolve_config(cli)?;
    let selector = selector(&args.keys, args.keys_file.as_ref())?;
    // Destructive-tier guard (M7): rewriting every AI-owned value in a
    // locale needs an explicit scope or an explicit yes.
    if args.regenerate && selector.is_any() && !args.yes {
        return Err(anyhow!(
            "--regenerate overwrites every AI-owned value in the selected locales; \
             scope it with --keys/--keys-file, or pass --yes to confirm"
        ));
    }
    let gate_names = if !args.validate.is_empty() {
        args.validate.clone()
    } else if args.no_validate {
        Vec::new()
    } else {
        cfg.config.fill.validate.clone()
    };
    if let Some(t) = args.judge_threshold {
        if !(0.0..=1.0).contains(&t) {
            return Err(anyhow!("--judge-threshold must be in 0..=1 (got {t})"));
        }
    }
    let gate = intl_ai_checks::build_gate(&cfg, &gate_names, args.judge_threshold)?;
    let transport = intl_ai_providers::build_transport(&cfg)?;
    let opts = FillOptions {
        locales: (!args.locale.is_empty()).then(|| args.locale.clone()),
        selector,
        stale_only: args.stale,
        regenerate: args.regenerate,
        include_human: args.include_human,
        dry_run: args.dry_run,
        no_cache: args.no_cache,
    };
    let report = fill(&cfg, transport.as_ref(), &opts, gate.as_ref())?;

    if is_json(args.format) {
        print_json(&report)?;
    } else {
        for (locale, res) in &report.locales {
            println!(
                "{locale}: {} translated ({} requested), {} written, \
                 {} adopted-human, {} reconciled-human, \
                 {} skipped-existing ({} human), {} regenerated-human, {} pruned",
                res.translated,
                res.requested,
                res.written,
                res.adopted_human,
                res.reconciled_human,
                res.skipped_existing,
                res.skipped_human,
                res.regenerated_human,
                res.pruned,
            );
            if res.refilled > 0 {
                eprintln!(
                    "{locale}: {} key(s) refilled via gate feedback",
                    res.refilled
                );
            }
            for f in &res.unresolved {
                eprintln!("{locale}: unresolved {}: {}: {}", f.key, f.check, f.message);
            }
            if !res.omitted.is_empty() {
                eprintln!("{locale}: {} keys omitted by provider", res.omitted.len());
            }
            for path in &res.clobbered {
                eprintln!("{locale}: clobbered existing value at '{path}'");
            }
        }
        for f in &report.failures {
            eprintln!("{}: {:?}: {}", f.locale, f.kind, f.message);
        }
    }

    if let Some(path) = intl_ai_core::report::write_if_failures(&cfg.config_dir, &report)? {
        eprintln!("report written to {path}");
    }

    // Unresolved gate findings fail the run (approved: a configured gate
    // that cannot produce compliant output is a red signal, not a note).
    let unresolved = report.locales.values().any(|r| !r.unresolved.is_empty());
    Ok(if report.failures.is_empty() && !unresolved {
        0
    } else {
        1
    })
}
