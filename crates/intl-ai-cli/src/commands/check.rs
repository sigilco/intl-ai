use anyhow::{Result, anyhow};
use intl_ai_checks::spec::SpecCheck;
use intl_ai_core::check::{Check, CheckOptions, check};
use intl_ai_core::diff::FindingKind;
use std::str::FromStr;

use crate::commands::{is_json, print_json, resolve_config, transport_for};
use crate::{CheckArgs, Cli};

/// `check --self-test`: run each configured spec check's self_test fixtures.
/// Reports one line per spec; exits 1 if any fixture violates the contract
/// or a spec lacks self_test data.
fn self_test(cli: &Cli) -> Result<u8> {
    let cfg = resolve_config(cli)?;
    let mut failures = 0usize;
    let mut specs = 0usize;
    for entry in &cfg.config.checks {
        let Some(spec_path) = &entry.spec else {
            continue;
        };
        let path = if spec_path.is_absolute() {
            spec_path.clone()
        } else {
            cfg.config_dir.join(spec_path)
        };
        specs += 1;
        let check = SpecCheck::load(&path)?;
        let errors = check.self_test();
        if errors.is_empty() {
            println!("{}: ok", check.id());
        } else {
            for e in &errors {
                eprintln!("{}: {e}", check.id());
            }
            failures += errors.len();
        }
    }
    if specs == 0 {
        println!("no spec checks configured");
    } else if failures == 0 {
        println!("{specs} spec(s) self-tested clean");
    }
    Ok(if failures > 0 { 1 } else { 0 })
}

pub fn run(cli: &Cli, args: &CheckArgs) -> Result<u8> {
    let cfg = resolve_config(cli)?;
    if args.self_test {
        return self_test(cli);
    }
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
    let checks = intl_ai_checks::build(&cfg)?;
    let transport = if checks.iter().any(|c| c.needs_transport()) {
        Some(transport_for(&cfg)?)
    } else {
        None
    };
    let opts = CheckOptions {
        locales: (!args.locale.is_empty()).then(|| args.locale.clone()),
        origin_filter: args.origin.and_then(|o| o.to_origin()),
        fail_on,
    };
    let report = check(&cfg, &opts, &checks, transport.as_deref())?;

    if is_json(args.format) {
        print_json(&report)?;
    } else {
        for (locale, d) in &report.locales {
            println!(
                "{locale}: {} missing, {} stale, {} modified, {} extra, {} unreviewed, {} invalid",
                d.missing.len(),
                d.stale.len(),
                d.modified.len(),
                d.extra.len(),
                d.unreviewed,
                d.invalid.len(),
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
            for f in &d.invalid {
                eprintln!("{locale}: invalid: {} ({}): {}", f.key, f.check, f.message);
            }
        }
        for e in &report.errors {
            eprintln!("check error: {e}");
        }
    }

    Ok(if report.has_issues { 1 } else { 0 })
}
