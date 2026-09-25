//! Exec checks (`[[checks]] exec = "..."`): the batch contract. One child
//! process per check run; stdin gets one JSONL request line, stdout must
//! produce one JSONL response line. Spawning uses the same discipline as
//! the command transport (process group kill, buffered stderr, stdout cap).
//!
//! Protocol v1:
//!
//! request:
//! `{"v":1,"check":"<id>","source_locale":"en-US","target_locale":"de-DE","items":[{"key":"a","source":"Hi","target":"Hallo"}]}`
//! (`source` is null for keys the source locale lacks)
//!
//! response:
//! `{"v":1,"findings":[{"key":"a","message":"...","check":"optional id"}]}`
//! or `{"v":1,"error":"code[:detail]"}`
//!
//! Version is strict on both sides in v1: the parent sends `v: 1` and
//! requires `v: 1` back. (The cauce exec protocol — optimistic version
//! negotiation, warm-process rules — is the model; it applies when a v2
//! ever exists.)

use intl_ai_core::check::{Check, CheckCtx, CheckGranularity, CheckItem};
use intl_ai_core::diff::CheckFinding;
use intl_ai_core::error::{Error, Result};
use intl_ai_providers::process::{self, RunSpec};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;

/// Wire version implemented here.
pub const PROTOCOL_VERSION: u8 = 1;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);
const DEFAULT_MAX_STDOUT: usize = 1024 * 1024;

#[derive(Debug, Serialize)]
struct ExecItem {
    key: String,
    source: Option<String>,
    target: String,
}

#[derive(Debug, Serialize)]
struct ExecRequest {
    v: u8,
    check: String,
    source_locale: String,
    target_locale: String,
    items: Vec<ExecItem>,
}

#[derive(Debug, Deserialize)]
struct ExecFindingRow {
    key: String,
    message: String,
    /// Optional: which sub-check inside the child produced it; findings are
    /// attributed to the entry id when absent.
    #[serde(default)]
    check: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ExecResponse {
    v: u8,
    #[serde(default)]
    findings: Vec<ExecFindingRow>,
    #[serde(default)]
    error: Option<String>,
}

/// A `[[checks]] exec = ...` entry.
pub struct ExecCheck {
    id: String,
    command: String,
    args: Vec<String>,
    cwd: Option<PathBuf>,
    timeout: Duration,
    max_stdout: usize,
}

impl ExecCheck {
    pub fn new(
        command: String,
        args: Vec<String>,
        cwd: Option<PathBuf>,
        timeout_ms: Option<u64>,
        max_stdout_bytes: Option<u64>,
    ) -> Self {
        let timeout = timeout_ms
            .map(Duration::from_millis)
            .unwrap_or(DEFAULT_TIMEOUT);
        let max_stdout = max_stdout_bytes
            .map(|b| b as usize)
            .unwrap_or(DEFAULT_MAX_STDOUT);
        Self {
            id: command.clone(),
            command,
            args,
            cwd,
            timeout,
            max_stdout,
        }
    }
}

impl Check for ExecCheck {
    fn id(&self) -> &str {
        &self.id
    }

    /// Exec sees the item list as one request, so its cache fingerprint
    /// covers all of it — a single changed key re-runs the batch.
    fn granularity(&self) -> CheckGranularity {
        CheckGranularity::WholeBatch
    }

    fn cache_ctx(&self) -> BTreeMap<String, String> {
        BTreeMap::from([
            ("v".into(), PROTOCOL_VERSION.to_string()),
            ("command".into(), self.command.clone()),
            ("args".into(), self.args.join("\u{1f}")),
            (
                "cwd".into(),
                self.cwd
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default(),
            ),
        ])
    }

    fn run(&self, ctx: &CheckCtx, items: &[CheckItem]) -> Result<Vec<CheckFinding>> {
        let req = ExecRequest {
            v: PROTOCOL_VERSION,
            check: self.id.clone(),
            source_locale: ctx.source_locale.to_string(),
            target_locale: ctx.target_locale.to_string(),
            items: items
                .iter()
                .map(|i| ExecItem {
                    key: i.key.clone(),
                    source: i.source.clone(),
                    target: i.target.clone(),
                })
                .collect(),
        };
        let mut line = serde_json::to_string(&req)
            .map_err(|e| Error::Message(format!("exec request: {e}")))?;
        line.push('\n');
        let out = process::run(&RunSpec {
            command: self.command.clone(),
            args: self.args.clone(),
            cwd: self.cwd.clone(),
            input: Some(line),
            timeout: self.timeout,
            max_stdout: self.max_stdout,
        })?;

        let text = out.trim();
        let response_line = text.lines().next().unwrap_or_default();
        if response_line.is_empty() {
            return Err(Error::Message(format!(
                "exec check '{}': empty response (expected one JSON line)",
                self.id
            )));
        }
        let resp: ExecResponse = serde_json::from_str(response_line).map_err(|e| {
            Error::Message(format!(
                "exec check '{}': invalid response line: {e}",
                self.id
            ))
        })?;
        if resp.v != PROTOCOL_VERSION {
            return Err(Error::Message(format!(
                "exec check '{}': protocol v{} not supported (this intl-ai speaks v{})",
                self.id, resp.v, PROTOCOL_VERSION
            )));
        }
        if let Some(err) = resp.error {
            return Err(Error::Message(format!("exec check '{}': {err}", self.id)));
        }
        Ok(resp
            .findings
            .into_iter()
            .map(|f| CheckFinding {
                key: f.key,
                check: f.check.unwrap_or_else(|| self.id.clone()),
                message: f.message,
                ..Default::default()
            })
            .collect())
    }
}
