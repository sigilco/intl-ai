//! intl-ai-checks: builtin checks, declarative YAML specs, and the exec
//! check protocol (v1). Implements the `Check` trait defined in
//! intl-ai-core; `build` resolves `[[checks]]` config entries.

pub mod dialect;
#[cfg(feature = "exec")]
pub mod exec;
pub mod icu;
pub mod judge;
#[cfg(feature = "spec")]
pub mod spec;

use intl_ai_core::check::Check;
use intl_ai_core::config::{CheckEntry, ResolvedConfig};
use intl_ai_core::error::{Error, Result};

/// Non-parameterized builtin ids (`dialect:<locale>` is parameterized).
pub const BUILTINS: &[&str] = &["icu", "placeholder-parity", "judge"];

/// Build every configured `[[checks]]` entry.
pub fn build(cfg: &ResolvedConfig) -> Result<Vec<Box<dyn Check>>> {
    cfg.config
        .checks
        .iter()
        .map(|entry| build_entry(cfg, entry))
        .collect()
}

pub fn build_entry(cfg: &ResolvedConfig, entry: &CheckEntry) -> Result<Box<dyn Check>> {
    // `cfg` only feeds the spec/exec filesystem branches below.
    #[cfg(not(any(feature = "spec", feature = "exec")))]
    let _ = cfg;
    let weight = entry.weight.unwrap_or(1.0);
    if let Some(id) = &entry.id {
        return builtin_tuned(id, entry.threshold, weight);
    }
    if let Some(spec_path) = &entry.spec {
        #[cfg(feature = "spec")]
        {
            let path = if spec_path.is_absolute() {
                spec_path.clone()
            } else {
                cfg.config_dir.join(spec_path)
            };
            let mut c = spec::SpecCheck::load(&path)?;
            c.weight = weight;
            return Ok(Box::new(c));
        }
        #[cfg(not(feature = "spec"))]
        return Err(Error::Config(format!(
            "check entry {}: spec checks not available in this build ({})",
            entry.label(),
            spec_path.display()
        )));
    }
    if let Some(cmd) = &entry.exec {
        #[cfg(feature = "exec")]
        {
            let mut c = exec::ExecCheck::new(
                cmd.clone(),
                entry.args.clone().unwrap_or_default(),
                entry.cwd.as_ref().map(|p| {
                    if p.is_absolute() {
                        p.clone()
                    } else {
                        cfg.config_dir.join(p)
                    }
                }),
                entry.timeout_ms,
                entry.max_stdout_bytes,
            );
            c.weight = weight;
            return Ok(Box::new(c));
        }
        #[cfg(not(feature = "exec"))]
        return Err(Error::Config(format!(
            "check entry {}: exec checks not available in this build ({cmd})",
            entry.label()
        )));
    }
    Err(Error::Config(format!(
        "check entry {}: exactly one of `id`, `spec`, `exec`",
        entry.label()
    )))
}

/// Resolve a `fill.validate` name to a gate-eligible check: a configured
/// `[[checks]]` entry whose id matches wins, else a builtin id. Entries
/// that cannot serve as reviewer notes (no `supports_feedback`) are
/// rejected here so `config validate` and `fill` reject them alike.
pub fn gate_check(cfg: &ResolvedConfig, name: &str) -> Result<Box<dyn Check>> {
    let mut found = None;
    for entry in &cfg.config.checks {
        let c = build_entry(cfg, entry)?;
        if c.id() == name {
            found = Some(c);
            break;
        }
    }
    let c = match found {
        Some(c) => c,
        None => builtin(name)?,
    };
    if !c.supports_feedback() {
        return Err(Error::Config(format!(
            "fill.validate: check '{name}' cannot gate fill — its findings \
             are not mechanically actionable (eligible: icu, \
             placeholder-parity, judge)"
        )));
    }
    Ok(c)
}

/// Resolve a builtin id (`icu`, `placeholder-parity`, `judge`,
/// `dialect:<locale>`) with default tuning — used for `fill.validate`
/// names that are not backed by a `[[checks]]` entry.
pub fn builtin(id: &str) -> Result<Box<dyn Check>> {
    builtin_tuned(id, None, 1.0)
}

/// Builtin resolution with `[[checks]]` tuning: `threshold` applies to
/// `judge` only (validated in config), `weight` feeds `[quality]`.
fn builtin_tuned(id: &str, threshold: Option<f64>, weight: f64) -> Result<Box<dyn Check>> {
    match id {
        "icu" => Ok(Box::new(icu::IcuCheck { weight })),
        "placeholder-parity" => Ok(Box::new(icu::PlaceholderParity { weight })),
        "judge" => Ok(Box::new(judge::JudgeCheck {
            threshold: threshold.unwrap_or(judge::JUDGE_THRESHOLD),
            weight,
        })),
        _ if id.starts_with("dialect:") => {
            let name = id.strip_prefix("dialect:").unwrap_or(id);
            let mut c = dialect::DialectCheck::new(name)?;
            c.weight = weight;
            Ok(Box::new(c))
        }
        other => Err(Error::Config(format!(
            "unknown check '{other}'. Builtins: {}, dialect:<locale>",
            BUILTINS.join(", ")
        ))),
    }
}
