//! Provider transports: `replay` (deterministic cassettes), `http`
//! (OpenAI-compatible chat completions), and `command` (agent CLIs).
//! `chat`, `prompt`, `payload`, and `presets` are pure and always
//! compiled — the wasm build binds them directly while transports stay
//! behind features (`std::process` and `ureq` cannot target wasm).

pub mod chat;
#[cfg(feature = "command")]
pub mod command;
#[cfg(feature = "http")]
pub mod http;
pub mod payload;
pub mod presets;
#[cfg(feature = "command")]
pub mod process;
pub mod prompt;
#[cfg(feature = "replay")]
pub mod replay;
#[cfg(any(feature = "command", feature = "http"))]
pub mod retry;

#[cfg(feature = "command")]
use intl_ai_core::config::PromptVia;
use intl_ai_core::config::{ProviderConfig, ResolvedConfig};
use intl_ai_core::error::Result;
use intl_ai_core::transport::Transport;

/// Builds the transport for the configured provider kind. Shared by the
/// CLI and the UniFFI bindings so both resolve providers identically.
pub fn build_transport(cfg: &ResolvedConfig) -> Result<Box<dyn Transport>> {
    build_transport_parts(&cfg.config.provider, &cfg.config_dir, cfg.max_retries())
}

/// Provider-level transport builder for pipelines that own their own
/// config type (`intl-ai-docs`) and therefore have no `ResolvedConfig`.
/// `config_dir` resolves relative replay/command paths.
pub fn build_transport_parts(
    provider: &ProviderConfig,
    config_dir: &std::path::Path,
    max_retries: u32,
) -> Result<Box<dyn Transport>> {
    match provider {
        #[cfg(feature = "replay")]
        ProviderConfig::Replay(r) => {
            let path = intl_ai_core::config::resolve(config_dir, &r.file);
            Ok(Box::new(replay::ReplayTransport::load(&path)?))
        }
        #[cfg(not(feature = "replay"))]
        ProviderConfig::Replay(_) => Err(intl_ai_core::error::Error::Config(
            "replay transport is not compiled in this build".into(),
        )),
        #[cfg(feature = "http")]
        ProviderConfig::Http(h) => Ok(Box::new(
            http::HttpTransport::new(h).with_max_retries(max_retries),
        )),
        #[cfg(not(feature = "http"))]
        ProviderConfig::Http(_) => Err(intl_ai_core::error::Error::Config(
            "http transport is not compiled in this build".into(),
        )),
        #[cfg(feature = "command")]
        ProviderConfig::Command(c) => {
            // `command` beats `agent` preset (plan 5.1.6).
            let cwd = c
                .cwd
                .as_ref()
                .map(|p| intl_ai_core::config::resolve(config_dir, p));
            let spec = if let Some(cmd) = &c.command {
                command::CommandSpec {
                    id: cmd.clone(),
                    command: cmd.clone(),
                    args: c.args.clone().unwrap_or_default(),
                    prompt_via: c.prompt_via.unwrap_or(PromptVia::Stdin),
                    cwd: cwd.clone(),
                    timeout_ms: c.timeout_ms,
                    max_stdout_bytes: c.max_stdout_bytes,
                    max_retries,
                }
            } else {
                let preset = c.agent.expect("validated: agent or command present");
                let p = presets::resolve(preset);
                command::CommandSpec {
                    id: presets::preset_id(preset).to_string(),
                    command: p.command.to_string(),
                    args: p.args.iter().map(|s| s.to_string()).collect(),
                    prompt_via: c.prompt_via.unwrap_or(p.prompt_via),
                    cwd,
                    timeout_ms: c.timeout_ms,
                    max_stdout_bytes: c.max_stdout_bytes,
                    max_retries,
                }
            };
            Ok(Box::new(command::CommandTransport::new(spec)))
        }
        #[cfg(not(feature = "command"))]
        ProviderConfig::Command(_) => Err(intl_ai_core::error::Error::Config(
            "command transport is not compiled in this build".into(),
        )),
    }
}
