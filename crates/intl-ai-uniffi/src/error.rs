use intl_ai_core::error::{Error as CoreError, ErrorType};

/// Error surface for the foreign bindings. Variants mirror the core
/// `Error` taxonomy so Swift/Kotlin catch clauses can branch on the same
/// buckets CLI consumers parse out of the JSON error envelope.
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum IntlAiError {
    #[error("config: {0}")]
    Config(String),
    #[error("lockfile: {0}")]
    Lockfile(String),
    #[error("io: {0}")]
    Io(String),
    #[error("format: {0}")]
    Format(String),
    /// Provider/transport failure; the string is prefixed with the core
    /// `ErrorType` (snake_case) and the provider message.
    #[error("{0}")]
    Transport(String),
    /// Caller-side contract violation (e.g. unscoped regenerate without
    /// `yes`, judge threshold outside 0..=1).
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    Other(String),
}

impl From<CoreError> for IntlAiError {
    fn from(e: CoreError) -> Self {
        match e {
            CoreError::Config(m) => Self::Config(m),
            CoreError::Lockfile(m) => Self::Lockfile(m),
            CoreError::Io { path, source } => Self::Io(format!("{}: {source}", path.display())),
            CoreError::Format(e) => Self::Format(e.to_string()),
            CoreError::Transport { kind, message, .. } => {
                Self::Transport(format!("{}: {message}", error_type_str(kind)))
            }
            CoreError::Message(m) => Self::Other(m),
        }
    }
}

impl From<serde_json::Error> for IntlAiError {
    fn from(e: serde_json::Error) -> Self {
        Self::Other(e.to_string())
    }
}

/// snake_case label matching the JSON the CLI prints for `ErrorType`.
pub(crate) fn error_type_str(kind: ErrorType) -> String {
    serde_json::to_value(kind)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_else(|| format!("{kind:?}"))
}
