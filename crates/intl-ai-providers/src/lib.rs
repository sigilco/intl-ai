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
