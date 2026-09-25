pub mod check;
pub mod config;
pub mod fill;
pub mod init;
pub mod lockfile;
pub mod migrate;
pub mod spec;

use ::config::FileFormat;
use anyhow::{Result, anyhow};
use intl_ai_core::config::{ResolvedConfig, load, load_from_str};
use intl_ai_core::selector::KeySelector;
use intl_ai_core::transport::Transport;
use intl_ai_providers::replay::ReplayTransport;
use std::io::Read;
use std::path::PathBuf;

use crate::{Cli, OutFormat};

/// Loads config from --config (path, '-' for stdin TOML) or cwd discovery.
pub fn resolve_config(cli: &Cli) -> Result<ResolvedConfig> {
    let cwd = std::env::current_dir()?;
    match cli.config.as_deref() {
        Some(p) if p.as_os_str() == "-" => {
            let mut text = String::new();
            std::io::stdin().read_to_string(&mut text)?;
            Ok(load_from_str(&text, FileFormat::Toml, &cwd)?)
        }
        other => Ok(load(other, &cwd)?),
    }
}

/// W0 has exactly one provider kind; W1 will dispatch on http/command here.
pub fn transport_for(cfg: &ResolvedConfig) -> Result<Box<dyn Transport>> {
    match &cfg.config.provider {
        intl_ai_core::config::ProviderConfig::Replay(_) => {
            let path = cfg
                .replay_file()
                .ok_or_else(|| anyhow!("replay provider missing file"))?;
            Ok(Box::new(ReplayTransport::load(&path)?))
        }
    }
}

pub fn print_json(value: &impl serde::Serialize) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

pub fn is_json(format: OutFormat) -> bool {
    matches!(format, OutFormat::Json)
}

/// Merges `--keys` specs with a `--keys-file` into one selector.
pub fn selector(keys: &[String], keys_file: Option<&PathBuf>) -> Result<KeySelector> {
    let mut sel = KeySelector::from_specs(keys)?;
    if let Some(path) = keys_file {
        sel = sel.merge(KeySelector::from_file(path)?)?;
    }
    Ok(sel)
}
