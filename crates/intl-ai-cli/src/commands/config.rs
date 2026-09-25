use anyhow::Result;

use crate::commands::{is_json, print_json, resolve_config};
use crate::{Cli, ConfigArgs, ConfigCommand};

pub fn run(cli: &Cli, args: &ConfigArgs) -> Result<u8> {
    match &args.command {
        ConfigCommand::Validate(v) => {
            let cfg = resolve_config(cli)?;
            if is_json(v.format) {
                // Serializes the resolved (post-extends, post-env-overlay,
                // post-interpolation) typed config.
                print_json(&serde_json::json!({
                    "valid": true,
                    "config": cfg.config,
                    "resolved": {
                        "locale_dir": cfg.locale_dir(),
                        "config_path": cfg.config_path,
                    }
                }))?;
            } else {
                println!("valid");
                println!("locale_dir = {}", cfg.locale_dir().display());
                if let Some(p) = &cfg.config_path {
                    println!("config = {}", p.display());
                }
            }
            Ok(0)
        }
    }
}
