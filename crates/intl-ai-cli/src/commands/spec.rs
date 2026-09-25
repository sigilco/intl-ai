use anyhow::Result;
use intl_ai_core::ops::{mark as op_mark, set_reviewed};
use intl_ai_core::selector::KeySelector;

use crate::commands::resolve_config;
use crate::{Cli, MarkArgs, SpecArgs};

pub fn mark(cli: &Cli, args: &MarkArgs) -> Result<u8> {
    let cfg = resolve_config(cli)?;
    let sel = KeySelector::from_specs(&args.spec)?;
    let origin = args
        .origin
        .to_origin()
        .expect("--origin is required by clap");
    for locale in &args.locale {
        let res = op_mark(&cfg, locale, &sel, origin)?;
        println!(
            "{locale}: {} marked, {} skipped (no entry)",
            res.affected, res.skipped_no_entry
        );
    }
    Ok(0)
}

pub fn review(cli: &Cli, args: &SpecArgs, reviewed: bool) -> Result<u8> {
    let cfg = resolve_config(cli)?;
    let sel = KeySelector::from_specs(&args.spec)?;
    for locale in &args.locale {
        let res = set_reviewed(&cfg, locale, &sel, reviewed)?;
        println!(
            "{locale}: {} {}, {} skipped (no entry)",
            res.affected,
            if reviewed { "reviewed" } else { "unreviewed" },
            res.skipped_no_entry
        );
    }
    Ok(0)
}
