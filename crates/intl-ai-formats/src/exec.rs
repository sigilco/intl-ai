//! Exec formats (`[[formats]] exec = "..."`): the subprocess contract. One
//! child process per read or write op; stdin gets one JSONL request line,
//! stdout must produce one JSONL response line. The plugin is a pure
//! transform: the parent owns all file IO (missing files, atomic writes,
//! unchanged-byte skips) so a format plugin never needs filesystem access.
//!
//! Protocol v1:
//!
//! read request:
//! `{"v":1,"op":"read","format":"<name>","path":"<abs path>","content":"<file text>"}`
//!
//! read response:
//! `{"v":1,"data":{<corpus tree>}}` or `{"v":1,"error":"code[:detail]"}`
//!
//! write request:
//! `{"v":1,"op":"write","format":"<name>","path":"<abs path>","data":{<corpus tree>}}`
//!
//! write response:
//! `{"v":1,"content":"<file text>"}` or `{"v":1,"error":"code[:detail]"}`
//!
//! Version is strict on both sides in v1: the parent sends `v: 1` and
//! requires `v: 1` back (same rule as the exec check protocol). `data`
//! must be a JSON object — the corpus model is a nested key -> string
//! map. File text is UTF-8: binary formats are out of v1's scope.
//! `path` is informational (plugins may use it in error messages);
//! plugins must not read or write it — the parent does.

use crate::{Format, FormatError};
use serde::Deserialize;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Wire version implemented here.
pub const PROTOCOL_VERSION: u8 = 1;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);
/// Responses carry whole files; keep the command transport's 10 MiB cap
/// rather than the exec check's 1 MiB (findings are small, corpora are not).
const DEFAULT_MAX_STDOUT: usize = intl_ai_exec::DEFAULT_STDOUT_CAP;

#[derive(Debug, Deserialize)]
struct ExecResponse {
    v: u8,
    #[serde(default)]
    data: Option<Value>,
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    error: Option<String>,
}

/// A `[[formats]] exec = ...` entry: an external program translating
/// between file text and the corpus tree.
#[derive(Debug)]
pub struct ExecFormat {
    name: String,
    command: String,
    args: Vec<String>,
    /// Child working directory; config resolves it against the config
    /// file's directory (default: the config directory itself).
    cwd: Option<PathBuf>,
    /// Claimed extensions, canonical (minted) first.
    extensions: Vec<String>,
    timeout: Duration,
    max_stdout: usize,
}

impl ExecFormat {
    pub fn new(
        name: String,
        command: String,
        args: Vec<String>,
        cwd: Option<PathBuf>,
        extensions: Vec<String>,
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
            name,
            command,
            args,
            cwd,
            extensions,
            timeout,
            max_stdout,
        }
    }

    /// One request line out, one response line back, fail-closed on any
    /// deviation (same discipline as exec checks).
    fn exchange(
        &self,
        op: &str,
        path: &Path,
        extra: serde_json::Map<String, Value>,
    ) -> Result<ExecResponse, FormatError> {
        let mut body = serde_json::json!({
            "v": PROTOCOL_VERSION,
            "op": op,
            "format": self.name,
            "path": path,
        });
        body.as_object_mut()
            .expect("request is an object")
            .extend(extra);
        let mut line = serde_json::to_string(&body).map_err(|e| self.err(e.to_string()))?;
        line.push('\n');
        let out = intl_ai_exec::run(&intl_ai_exec::RunSpec {
            command: self.command.clone(),
            args: self.args.clone(),
            cwd: self.cwd.clone(),
            input: Some(line),
            timeout: self.timeout,
            max_stdout: self.max_stdout,
        })
        .map_err(|e| self.err(e.message))?;

        let text = out.trim();
        let response_line = text.lines().next().unwrap_or_default();
        if response_line.is_empty() {
            return Err(self.err("empty response (expected one JSON line)".into()));
        }
        let resp: ExecResponse = serde_json::from_str(response_line)
            .map_err(|e| self.err(format!("invalid response line: {e}")))?;
        if resp.v != PROTOCOL_VERSION {
            return Err(self.err(format!(
                "protocol v{} not supported (this intl-ai speaks v{})",
                resp.v, PROTOCOL_VERSION
            )));
        }
        if let Some(err) = resp.error {
            return Err(self.err(err));
        }
        Ok(resp)
    }

    fn err(&self, message: String) -> FormatError {
        FormatError::Exec {
            format: self.name.clone(),
            message,
        }
    }
}

impl Format for ExecFormat {
    fn name(&self) -> &str {
        &self.name
    }

    fn extensions(&self) -> Vec<&str> {
        self.extensions.iter().map(String::as_str).collect()
    }

    fn parse(&self, text: &str, path: &Path) -> Result<Value, FormatError> {
        let resp = self.exchange(
            "read",
            path,
            serde_json::Map::from_iter([("content".into(), Value::String(text.into()))]),
        )?;
        match resp.data {
            Some(data) if data.is_object() => Ok(data),
            Some(_) => Err(self.err("read response `data` must be an object".into())),
            None => Err(self.err("read response missing `data`".into())),
        }
    }

    fn serialize(&self, value: &Value, path: &Path) -> Result<Vec<u8>, FormatError> {
        let resp = self.exchange(
            "write",
            path,
            serde_json::Map::from_iter([("data".into(), value.clone())]),
        )?;
        resp.content
            .map(String::into_bytes)
            .ok_or_else(|| self.err("write response missing `content`".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `sh -c <script>` format: canned responses exercise the wire
    /// contract without a real parser.
    fn sh_format(script: &str) -> ExecFormat {
        ExecFormat::new(
            "demo".into(),
            "sh".into(),
            vec!["-c".into(), script.into()],
            None,
            vec!["demo".into()],
            Some(10_000),
            None,
        )
    }

    #[test]
    fn read_returns_data_object() {
        let f = sh_format("read req; echo '{\"v\":1,\"data\":{\"a\":\"b\"}}'");
        let v = f.parse("ignored", Path::new("/tmp/x.demo")).unwrap();
        assert_eq!(v["a"], "b");
    }

    #[test]
    fn read_rejects_non_object_data() {
        let f = sh_format("read req; echo '{\"v\":1,\"data\":[1,2]}'");
        let err = f.parse("x", Path::new("/tmp/x.demo")).unwrap_err();
        assert!(err.to_string().contains("must be an object"));
    }

    #[test]
    fn write_returns_content() {
        let f = sh_format("read req; printf '%s\\n' '{\"v\":1,\"content\":\"<x/>\\n\"}'");
        let bytes = f
            .serialize(&serde_json::json!({"a": "b"}), Path::new("/tmp/x.demo"))
            .unwrap();
        assert_eq!(bytes, b"<x/>\n");
    }

    #[test]
    fn error_response_fails_closed() {
        let f = sh_format("read req; echo '{\"v\":1,\"error\":\"parse:tag soup\"}'");
        let err = f.parse("x", Path::new("/tmp/x.demo")).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("demo"));
        assert!(msg.contains("parse:tag soup"));
    }

    #[test]
    fn version_mismatch_fails_closed() {
        let f = sh_format("read req; echo '{\"v\":2,\"data\":{}}'");
        let err = f.parse("x", Path::new("/tmp/x.demo")).unwrap_err();
        assert!(err.to_string().contains("protocol v2 not supported"));
    }

    #[test]
    fn nonzero_exit_fails_closed() {
        let f = sh_format("read req; echo bad >&2; exit 3");
        let err = f.parse("x", Path::new("/tmp/x.demo")).unwrap_err();
        assert!(err.to_string().contains("exited with code 3"));
    }
}
