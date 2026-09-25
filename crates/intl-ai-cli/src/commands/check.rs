use anyhow::{Result, anyhow};
use intl_ai_core::check::{CheckOptions, check};
use intl_ai_core::diff::FindingKind;
use std::str::FromStr;

use crate::commands::{is_json, print_json, resolve_config};
use crate::{CheckArgs, Cli};

pub fn run(cli: &Cli, args: &CheckArgs) -> Result<u8> {
    let cfg = resolve_config(cli)?;
    let fail_on = if args.fail_on.is_empty() {
        None
    } else {
        Some(
            args.fail_on
                .iter()
                .map(|s| FindingKind::from_str(s).map_err(|e| anyhow!("{e}")))
                .collect::<Result<Vec<_>>>()?,
        )
    };
    let opts = CheckOptions {
        locales: (!args.locale.is_empty()).then(|| args.locale.clone()),
        origin_filter: args.origin.and_then(|o| o.to_origin()),
        fail_on,
    };
    let report = check(&cfg, &opts)?;

    if is_json(args.format) {
        print_json(&report)?;
    } else {
        for (locale, d) in &report.locales {
            println!(
                "{locale}: {} missing, {} stale, {} modified, {} extra, {} unreviewed",
                d.missing.len(),
                d.stale.len(),
                d.modified.len(),
                d.extra.len(),
                d.unreviewed,
            );
            for (kind, keys) in [
                ("missing", &d.missing),
                ("stale", &d.stale),
                ("modified", &d.modified),
                ("extra", &d.extra),
            ] {
                for key in keys.iter() {
                    eprintln!("{locale}: {kind}: {key}");
                }
            }
        }
    }

    Ok(if report.has_issues { 1 } else { 0 })
}
