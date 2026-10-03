//! Shared spawn discipline for every subprocess intl-ai drives (provider
//! agent CLIs, exec checks, exec formats): never a shell, stdin written
//! then closed, concurrent drains with caps, and a SIGTERM -> 5s -> SIGKILL
//! timeout escalation (SIGKILL only on non-unix). Callers map `ExecError`
//! onto their own error taxonomy.

use std::fmt;
use std::io::{Read, Write};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(300);
pub const DEFAULT_STDOUT_CAP: usize = 10 * 1024 * 1024;
/// Stderr is drained so the child never blocks, but only the head is kept
/// for the error preview (the W0 fix for the uncapped-buffer bug).
const STDERR_RETAIN: usize = 64 * 1024;
#[cfg(not(target_arch = "wasm32"))]
const GRACE_AFTER_SIGTERM: Duration = Duration::from_secs(5);
const POLL: Duration = Duration::from_millis(25);
/// Chars appended to error messages (plan 5.1.7: preview, not the whole log).
const STDERR_PREVIEW: usize = 2000;

/// Failure classification, one arm per way a child run can go wrong.
/// Mirrors `intl_ai_core::error::ErrorType`'s transport subset; kept
/// crate-local so this leaf never depends on core.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecErrorKind {
    SpawnFailure,
    Timeout,
    ProcessExit,
    OutputTruncated,
    ParseError,
    Unknown,
}

#[derive(Debug)]
pub struct ExecError {
    pub kind: ExecErrorKind,
    pub message: String,
}

impl ExecError {
    fn new(kind: ExecErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

impl fmt::Display for ExecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ExecError {}

pub struct RunSpec {
    pub command: String,
    pub args: Vec<String>,
    pub cwd: Option<std::path::PathBuf>,
    /// Written to stdin then the pipe is closed; `None` = close immediately.
    pub input: Option<String>,
    pub timeout: Duration,
    pub max_stdout: usize,
}

/// Runs the spec to completion or an error classified into the shared
/// taxonomy. Returns raw stdout on success.
pub fn run(spec: &RunSpec) -> Result<String, ExecError> {
    let mut cmd = Command::new(&spec.command);
    cmd.args(&spec.args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        // No shell ever: a stray metachar in config must not become code.
        .env("CI", "1")
        .env("NO_COLOR", "0"); // agents may still colorize; we strip later
    if let Some(cwd) = &spec.cwd {
        cmd.current_dir(cwd);
    }
    // Agent CLIs spawn grandchildren (shells, workers). The child gets its
    // own process group so the timeout escalation can signal the whole
    // tree — killing only the direct child leaves a grandchild holding
    // our pipes open and the drain threads waiting on EOF forever.
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }
    let mut child = cmd.spawn().map_err(|e| {
        ExecError::new(
            ExecErrorKind::SpawnFailure,
            format!("spawn {}: {e}", spec.command),
        )
    })?;

    let mut stdin = child.stdin.take();
    if let (Some(w), Some(input)) = (stdin.as_mut(), spec.input.as_ref()) {
        // A closed stdin on the child side surfaces as EPIPE — the child
        // just didn't read it; treat as spawn-phase failure.
        if let Err(e) = w.write_all(input.as_bytes()) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(ExecError::new(
                ExecErrorKind::SpawnFailure,
                format!("stdin write to {}: {e}", spec.command),
            ));
        }
    }
    drop(stdin); // close always, prompt-via-argv or not

    let stdout = child.stdout.take().expect("piped");
    let stderr = child.stderr.take().expect("piped");
    let capped = Arc::new(AtomicBool::new(false));

    let max_stdout = spec.max_stdout;
    let out_thread = {
        let capped = Arc::clone(&capped);
        thread::spawn(move || drain_capped(stdout, max_stdout, capped))
    };
    let err_thread = thread::spawn(move || {
        drain_capped(stderr, STDERR_RETAIN, Arc::new(AtomicBool::new(false)))
    });

    let started = Instant::now();
    let status = loop {
        if capped.load(Ordering::Relaxed) {
            escalate(&mut child);
            out_thread.join().ok();
            err_thread.join().ok();
            return Err(ExecError::new(
                ExecErrorKind::OutputTruncated,
                format!("{} stdout exceeded {} bytes", spec.command, spec.max_stdout),
            ));
        }
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {}
            Err(e) => {
                let _ = child.kill();
                return Err(ExecError::new(
                    ExecErrorKind::Unknown,
                    format!("poll {}: {e}", spec.command),
                ));
            }
        }
        if started.elapsed() > spec.timeout {
            escalate(&mut child);
            out_thread.join().ok();
            err_thread.join().ok();
            return Err(ExecError::new(
                ExecErrorKind::Timeout,
                format!(
                    "{} timed out after {}ms",
                    spec.command,
                    spec.timeout.as_millis()
                ),
            ));
        }
        thread::sleep(POLL);
    };

    let out = out_thread.join().unwrap_or_default();
    let err = err_thread.join().unwrap_or_default();
    if !status.success() {
        return Err(ExecError::new(
            ExecErrorKind::ProcessExit,
            format!(
                "{} exited with code {}: {}",
                spec.command,
                status.code().unwrap_or(-1),
                preview(&err)
            ),
        ));
    }
    String::from_utf8(out).map_err(|e| {
        ExecError::new(
            ExecErrorKind::ParseError,
            format!("{} non-utf8 stdout: {e}", spec.command),
        )
    })
}

fn drain_capped<R: Read>(mut reader: R, cap: usize, flagged: Arc<AtomicBool>) -> Vec<u8> {
    let mut buf = Vec::new();
    let mut chunk = [0u8; 8192];
    loop {
        match reader.read(&mut chunk) {
            Ok(0) | Err(_) => break,
            Ok(n) => {
                if buf.len() + n > cap {
                    flagged.store(true, Ordering::Relaxed);
                    // Keep draining so the child never blocks on a full pipe.
                } else {
                    buf.extend_from_slice(&chunk[..n]);
                }
            }
        }
    }
    buf
}

#[cfg(unix)]
fn escalate(child: &mut Child) {
    signal(child, libc::SIGTERM);
    let deadline = Instant::now() + GRACE_AFTER_SIGTERM;
    while Instant::now() < deadline {
        if matches!(child.try_wait(), Ok(Some(_))) {
            return;
        }
        thread::sleep(POLL);
    }
    signal(child, libc::SIGKILL);
    let _ = child.wait();
}

#[cfg(not(unix))]
fn escalate(child: &mut Child) {
    // No SIGTERM on non-unix: kill() (TerminateProcess) is all we have.
    let _ = child.kill();
    let _ = child.wait();
}

#[cfg(unix)]
fn signal(child: &mut Child, sig: libc::c_int) {
    // Negative pid targets the process group created by process_group(0),
    // so SIGTERM reaches grandchildren (shells/workers the agent spawned)
    // that would otherwise keep our pipes open.
    unsafe { libc::kill(-(child.id() as i32), sig) };
}

fn preview(stderr: &[u8]) -> String {
    let text = String::from_utf8_lossy(stderr);
    let text = text.trim();
    if text.is_empty() {
        return "(no stderr)".into();
    }
    text.chars().take(STDERR_PREVIEW).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sh_spec(script: &str) -> RunSpec {
        RunSpec {
            command: "sh".into(),
            args: vec!["-c".into(), script.into()],
            cwd: None,
            input: Some("hello".into()),
            timeout: Duration::from_secs(30),
            max_stdout: DEFAULT_STDOUT_CAP,
        }
    }

    #[test]
    fn captures_stdout() {
        let out = run(&sh_spec("cat >&2; echo '{\"ok\":1}'")).unwrap();
        assert_eq!(out, "{\"ok\":1}\n");
    }

    #[test]
    fn nonzero_exit_is_process_exit() {
        // `cat` consumes our stdin write first: without it the child can
        // exit before the write lands and the EPIPE classifies as a
        // spawn failure instead of the exit code (load-dependent flake).
        let err = run(&sh_spec("cat >/dev/null; echo errmsg >&2; exit 3")).unwrap_err();
        assert_eq!(err.kind, ExecErrorKind::ProcessExit);
        assert!(err.to_string().contains("exited with code 3"));
        assert!(err.to_string().contains("errmsg"));
    }

    #[test]
    fn timeout_escalates() {
        let err = run(&RunSpec {
            timeout: Duration::from_millis(200),
            ..sh_spec("sleep 60")
        })
        .unwrap_err();
        assert_eq!(err.kind, ExecErrorKind::Timeout);
    }

    #[test]
    fn stdout_cap_kills() {
        let err = run(&RunSpec {
            max_stdout: 8,
            ..sh_spec("yes x | head -c 65536")
        })
        .unwrap_err();
        assert_eq!(err.kind, ExecErrorKind::OutputTruncated);
        assert!(err.to_string().contains("stdout exceeded"));
    }

    #[test]
    fn missing_binary_is_spawn_failure() {
        let err = run(&RunSpec {
            command: "definitely-not-a-real-binary-xyz".into(),
            args: vec![],
            cwd: None,
            input: None,
            timeout: Duration::from_secs(5),
            max_stdout: DEFAULT_STDOUT_CAP,
        })
        .unwrap_err();
        assert_eq!(err.kind, ExecErrorKind::SpawnFailure);
    }
}
