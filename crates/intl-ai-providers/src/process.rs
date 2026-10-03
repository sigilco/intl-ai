//! Spawn discipline for the command transport (plan 5.1.7), now delegated
//! to `intl-ai-exec`: never a shell, stdin written then closed, concurrent
//! drains with caps, and a SIGTERM -> 5s -> SIGKILL timeout escalation.
//! Kept as a shim so callers keep the `Error::Transport` taxonomy and the
//! `command transport:` message prefix.

use intl_ai_core::error::{Error, ErrorType, Result};

pub use intl_ai_exec::RunSpec;
pub use intl_ai_exec::{DEFAULT_STDOUT_CAP, DEFAULT_TIMEOUT};

/// Runs the spec to completion or an error classified into the shared
/// taxonomy. Returns raw stdout on success.
pub fn run(spec: &RunSpec) -> Result<String> {
    intl_ai_exec::run(spec).map_err(|e| {
        let kind = match e.kind {
            intl_ai_exec::ExecErrorKind::SpawnFailure => ErrorType::SpawnFailure,
            intl_ai_exec::ExecErrorKind::Timeout => ErrorType::Timeout,
            intl_ai_exec::ExecErrorKind::ProcessExit => ErrorType::ProcessExit,
            intl_ai_exec::ExecErrorKind::OutputTruncated => ErrorType::OutputTruncated,
            intl_ai_exec::ExecErrorKind::ParseError => ErrorType::ParseError,
            intl_ai_exec::ExecErrorKind::Unknown => ErrorType::Unknown,
        };
        // The original messages carried this prefix on every kind but the
        // spawn-phase ones; keep it so agent-facing output does not drift.
        let message = match e.kind {
            intl_ai_exec::ExecErrorKind::SpawnFailure | intl_ai_exec::ExecErrorKind::Unknown => {
                e.message
            }
            _ => format!("command transport: {}", e.message),
        };
        Error::transport(kind, message)
    })
}
