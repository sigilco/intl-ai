use anyhow::{Result, anyhow};
use intl_ai_core::ops::{mark as op_mark, mark_absent, set_reviewed};
use intl_ai_core::selector::KeySelector;

use crate::commands::{is_json, print_json, resolve_config};
use crate::{Cli, MarkArgs, SpecArgs};

pub fn mark(cli: &Cli, args: &MarkArgs) -> Result<u8> {
    let cfg = resolve_config(cli)?;
    let sel = KeySelector::from_specs(&args.spec)?;
    for locale in &args.locale {
        let res = if args.absent || args.present {
            mark_absent(&cfg, locale, &sel, args.absent)?
        } else {
            let origin = args
                .origin
                .and_then(|o| o.to_origin())
                .ok_or_else(|| anyhow!("pass --origin ai|human, or --absent/--present"))?;
            op_mark(&cfg, locale, &sel, origin)?
        };
        if is_json(args.format) {
            print_json(&res)?;
        } else {
            println!(
                "{locale}: {} marked, {} skipped (no entry)",
                res.affected, res.skipped_no_entry
            );
        }
    }
    Ok(0)
}

pub fn review(cli: &Cli, args: &SpecArgs, reviewed: bool) -> Result<u8> {
    let cfg = resolve_config(cli)?;
    let sel = KeySelector::from_specs(&args.spec)?;
    for locale in &args.locale {
        let res = set_reviewed(&cfg, locale, &sel, reviewed)?;
        if is_json(args.format) {
            print_json(&res)?;
        } else {
            println!(
                "{locale}: {} {}, {} skipped (no entry)",
                res.affected,
                if reviewed { "reviewed" } else { "unreviewed" },
                res.skipped_no_entry
            );
        }
    }
    Ok(0)
}
