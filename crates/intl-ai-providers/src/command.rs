//! The command transport: frames `system\n\n---\n\nuser`, feeds it via
//! stdin (or argv when `prompt_via = "argv"`), and parses JSON out of
//! however the agent CLIs dress their output.

use intl_ai_core::config::PromptVia;
use intl_ai_core::error::{Error, ErrorType, Result};
use intl_ai_core::transport::{TranslateRequest, TranslateResponse, Translated, Transport};
use std::path::PathBuf;

use crate::payload::{payload_or_self, strip_vt};
use crate::process::{self, DEFAULT_STDOUT_CAP, DEFAULT_TIMEOUT, RunSpec};
use crate::prompt::{parse_translations, system_prompt, user_prompt};
use crate::retry::attempt_with_retries;

#[derive(Debug, Clone)]
pub struct CommandSpec {
    /// Display id used as `model` on lockfile entries ("claude-code", ...).
    pub id: String,
    pub command: String,
    pub args: Vec<String>,
    pub prompt_via: PromptVia,
    pub cwd: Option<PathBuf>,
    pub timeout_ms: Option<u64>,
    pub max_stdout_bytes: Option<u64>,
    pub max_retries: u32,
}

pub struct CommandTransport {
    spec: CommandSpec,
}

impl CommandTransport {
    pub fn new(spec: CommandSpec) -> Self {
        Self { spec }
    }
}

impl Transport for CommandTransport {
    fn id(&self) -> &str {
        &self.spec.id
    }

    fn translate(&self, req: &TranslateRequest) -> Result<TranslateResponse> {
        let system = system_prompt(req.locale_instruction.as_deref());
        let user = user_prompt(req);
        let framed = format!("{system}\n\n---\n\n{user}");

        // Fetch + parse are inside the attempt so malformed output (plain
        // text, truncated JSON) consumes retry budget like transport errors.
        attempt_with_retries(self.spec.max_retries, || {
            let content = self.attempt(&framed)?;
            let cleaned = strip_vt(&content);
            let payload = payload_or_self(&cleaned);
            let parsed = parse_translations(&payload).map_err(|e| {
                Error::transport(
                    ErrorType::ParseError,
                    format!("command transport: {}: {e}", self.spec.id),
                )
            })?;
            Ok(TranslateResponse {
                translations: parsed
                    .translations
                    .into_iter()
                    .map(|r| Translated {
                        key: r.key,
                        value: r.translated,
                    })
                    .collect(),
                model: self.spec.id.clone(),
            })
        })
    }
}

impl CommandTransport {
    fn attempt(&self, framed: &str) -> Result<String> {
        let (args, input) = match self.spec.prompt_via {
            PromptVia::Argv => {
                let mut args = self.spec.args.clone();
                args.push(framed.to_string());
                (args, None)
            }
            PromptVia::Stdin => (self.spec.args.clone(), Some(framed.to_string())),
        };
        process::run(&RunSpec {
            command: self.spec.command.clone(),
            args,
            cwd: self.spec.cwd.clone(),
            input,
            timeout: self
                .spec
                .timeout_ms
                .map(Duration::from_millis)
                .unwrap_or(DEFAULT_TIMEOUT),
            max_stdout: self
                .spec
                .max_stdout_bytes
                .map(|b| b as usize)
                .unwrap_or(DEFAULT_STDOUT_CAP),
        })
    }
}

use std::time::Duration;
