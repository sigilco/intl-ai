use serde::Serialize;
use std::io;
use std::path::PathBuf;

/// Error taxonomy carried over from the TS surface (plan 5.1.10): message
/// classification maps to the same kinds agents already see.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorType {
    RateLimit,
    Http,
    SpawnFailure,
    Timeout,
    ProcessExit,
    ParseError,
    OutputTruncated,
    Validation,
    Empty,
    Config,
    Lockfile,
    Io,
    /// 401/403: credential problem, never worth retrying.
    Auth,
    Unknown,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("config: {0}")]
    Config(String),
    #[error("lockfile: {0}")]
    Lockfile(String),
    #[error("io {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("{kind:?}: {message}")]
    Transport {
        kind: ErrorType,
        message: String,
        /// Server-supplied retry hint (Retry-After), honored by the retry
        /// loop instead of its own backoff when present.
        retry_after_ms: Option<u64>,
    },
    #[error("format: {0}")]
    Format(#[from] intl_ai_formats::json::FormatError),
    #[error("{0}")]
    Message(String),
}

impl Error {
    pub fn transport(kind: ErrorType, message: impl Into<String>) -> Self {
        Self::Transport {
            kind,
            message: message.into(),
            retry_after_ms: None,
        }
    }

    pub fn with_retry_after(mut self, retry_after: std::time::Duration) -> Self {
        if let Self::Transport { retry_after_ms, .. } = &mut self {
            *retry_after_ms = Some(retry_after.as_millis() as u64);
        }
        self
    }

    pub fn retry_after(&self) -> Option<std::time::Duration> {
        match self {
            Self::Transport {
                retry_after_ms: Some(ms),
                ..
            } => Some(std::time::Duration::from_millis(*ms)),
            _ => None,
        }
    }

    /// Auth failures are terminal (a bad key never succeeds on retry);
    /// everything else consumes retry budget as before.
    pub fn retryable(&self) -> bool {
        !matches!(
            self,
            Self::Transport {
                kind: ErrorType::Auth,
                ..
            }
        )
    }

    /// Exit-code bucket (plan 5.1.8): 10 hard error, findings are reported
    /// separately via exit 1 by the caller.
    pub fn is_hard(&self) -> bool {
        matches!(
            self,
            Self::Config(_) | Self::Lockfile(_) | Self::Io { .. } | Self::Format(_)
        )
    }
}

pub type Result<T> = std::result::Result<T, Error>;
