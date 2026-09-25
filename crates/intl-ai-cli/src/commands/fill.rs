use anyhow::Result;
use intl_ai_core::fill::{FillOptions, fill};

use crate::commands::{is_json, print_json, resolve_config, selector, transport_for};
use crate::{Cli, FillArgs};

pub fn run(cli: &Cli, args: &FillArgs) -> Result<u8> {
    let cfg = resolve_config(cli)?;
    let transport = transport_for(&cfg)?;
    let opts = FillOptions {
        locales: (!args.locale.is_empty()).then(|| args.locale.clone()),
        selector: selector(&args.keys, args.keys_file.as_ref())?,
        stale_only: args.stale,
        regenerate: args.regenerate,
        include_human: args.include_human,
        dry_run: args.dry_run,
    };
    let report = fill(&cfg, transport.as_ref(), &opts)?;

    if is_json(args.format) {
        print_json(&report)?;
    } else {
        for (locale, res) in &report.locales {
            println!(
                "{locale}: {} translated ({} requested), {} written, \
                 {} adopted-human, {} reconciled-human, {} skipped-existing",
                res.translated,
                res.requested,
                res.written,
                res.adopted_human,
                res.reconciled_human,
                res.skipped_human,
            );
            if !res.omitted.is_empty() {
                eprintln!("{locale}: {} keys omitted by provider", res.omitted.len());
            }
        }
        for f in &report.failures {
            eprintln!("{}: {:?}: {}", f.locale, f.kind, f.message);
        }
    }

    if let Some(path) = intl_ai_core::report::write_if_failures(&cfg.config_dir, &report)? {
        eprintln!("report written to {path}");
    }

    Ok(if report.failures.is_empty() { 0 } else { 1 })
}
