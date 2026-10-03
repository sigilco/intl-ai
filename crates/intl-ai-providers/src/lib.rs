//! Provider transports: `replay` (deterministic cassettes), `http`
//! (OpenAI-compatible chat completions), and `command` (agent CLIs).

pub mod command;
pub mod http;
pub mod payload;
pub mod presets;
pub mod process;
pub mod prompt;
pub mod replay;
pub mod retry;

use intl_ai_core::config::{PromptVia, ProviderConfig, ResolvedConfig};
use intl_ai_core::error::{Error, Result};
use intl_ai_core::transport::Transport;

/// Builds the transport for the configured provider kind. Shared by the
/// CLI and the UniFFI bindings so both resolve providers identically.
pub fn build_transport(cfg: &ResolvedConfig) -> Result<Box<dyn Transport>> {
    match &cfg.config.provider {
        ProviderConfig::Replay(_) => {
            let path = cfg
                .replay_file()
                .ok_or_else(|| Error::Config("replay provider missing file".into()))?;
            Ok(Box::new(replay::ReplayTransport::load(&path)?))
        }
        ProviderConfig::Http(h) => Ok(Box::new(
            http::HttpTransport::new(h).with_max_retries(cfg.max_retries()),
        )),
        ProviderConfig::Command(c) => {
            // `command` beats `agent` preset (plan 5.1.6).
            let spec = if let Some(cmd) = &c.command {
                command::CommandSpec {
                    id: cmd.clone(),
                    command: cmd.clone(),
                    args: c.args.clone().unwrap_or_default(),
                    prompt_via: c.prompt_via.unwrap_or(PromptVia::Stdin),
                    cwd: cfg.command_cwd(),
                    timeout_ms: c.timeout_ms,
                    max_stdout_bytes: c.max_stdout_bytes,
                    max_retries: cfg.max_retries(),
                }
            } else {
                let preset = c.agent.expect("validated: agent or command present");
                let p = presets::resolve(preset);
                command::CommandSpec {
                    id: presets::preset_id(preset).to_string(),
                    command: p.command.to_string(),
                    args: p.args.iter().map(|s| s.to_string()).collect(),
                    prompt_via: c.prompt_via.unwrap_or(p.prompt_via),
                    cwd: cfg.command_cwd(),
                    timeout_ms: c.timeout_ms,
                    max_stdout_bytes: c.max_stdout_bytes,
                    max_retries: cfg.max_retries(),
                }
            };
            Ok(Box::new(command::CommandTransport::new(spec)))
        }
    }
}
