use anyhow::Result;
use intl_ai_core::check::{CheckOptions, check};
use intl_ai_core::lockfile::{Origin, load_shard};
use intl_ai_core::selector::KeySelector;
use serde_json::json;

use crate::commands::{is_json, print_json, resolve_config};
use crate::{Cli, StatusArgs};

/// `intl-ai status`: per-locale inventory — shard entries by origin and
/// review state plus the structural diff counts. Read-only and cheap:
/// no `[[checks]]` run, no transport resolution.
pub fn run(cli: &Cli, args: &StatusArgs) -> Result<u8> {
    let cfg = resolve_config(cli)?;
    let opts = CheckOptions {
        locales: (!args.locale.is_empty()).then(|| args.locale.clone()),
        origin_filter: None,
        fail_on: Some(vec![]),
        selector: KeySelector::any(),
        no_cache: false,
    };
    let report = check(&cfg, &opts, &[], None)?;

    let mut locales = Vec::new();
    for (locale, d) in &report.locales {
        let shard = load_shard(&cfg.locale_dir(), locale)?;
        let mut ai = 0usize;
        let mut human = 0usize;
        let mut reviewed = 0usize;
        let mut absent = 0usize;
        for e in shard.entries.values() {
            match e.origin {
                Origin::Ai => ai += 1,
                Origin::Human => human += 1,
            }
            if e.reviewed {
                reviewed += 1;
            }
            if e.absent {
                absent += 1;
            }
        }
        locales.push(json!({
            "locale": locale,
            "entries": shard.entries.len(),
            "ai": ai,
            "human": human,
            "reviewed": reviewed,
            "absent": absent,
            "missing": d.missing.len(),
            "stale": d.stale.len(),
            "modified": d.modified.len(),
            "extra": d.extra.len(),
            "unreviewed": d.unreviewed.len(),
        }));
        if !is_json(args.format) {
            println!(
                "{locale}: {} entries ({ai} ai / {human} human, {reviewed} reviewed, {absent} absent) | {} missing, {} stale, {} modified, {} extra, {} unreviewed",
                shard.entries.len(),
                d.missing.len(),
                d.stale.len(),
                d.modified.len(),
                d.extra.len(),
                d.unreviewed.len(),
            );
        }
    }
    if is_json(args.format) {
        print_json(&json!({ "locales": locales }))?;
    }
    Ok(0)
}
