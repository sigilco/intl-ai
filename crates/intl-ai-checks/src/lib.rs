//! intl-ai-checks: builtin checks, declarative YAML specs, and the exec
//! check protocol (v1). Implements the `Check` trait defined in
//! intl-ai-core; `build` resolves `[[checks]]` config entries.

pub mod dialect;
pub mod exec;
pub mod icu;
pub mod judge;
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
    if let Some(id) = &entry.id {
        return builtin(id);
    }
    if let Some(spec_path) = &entry.spec {
        let path = if spec_path.is_absolute() {
            spec_path.clone()
        } else {
            cfg.config_dir.join(spec_path)
        };
        return Ok(Box::new(spec::SpecCheck::load(&path)?));
    }
    if let Some(cmd) = &entry.exec {
        return Ok(Box::new(exec::ExecCheck::new(
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
        )));
    }
    Err(Error::Config(format!(
        "check entry {}: exactly one of `id`, `spec`, `exec`",
        entry.label()
    )))
}

/// Resolve a builtin id (`icu`, `placeholder-parity`, `judge`,
/// `dialect:<locale>`).
pub fn builtin(id: &str) -> Result<Box<dyn Check>> {
    match id {
        "icu" => Ok(Box::new(icu::IcuCheck)),
        "placeholder-parity" => Ok(Box::new(icu::PlaceholderParity)),
        "judge" => Ok(Box::new(judge::JudgeCheck)),
        _ if id.starts_with("dialect:") => {
            let name = id.strip_prefix("dialect:").unwrap_or(id);
            Ok(Box::new(dialect::DialectCheck::new(name)?))
        }
        other => Err(Error::Config(format!(
            "unknown check '{other}'. Builtins: {}, dialect:<locale>",
            BUILTINS.join(", ")
        ))),
    }
}
