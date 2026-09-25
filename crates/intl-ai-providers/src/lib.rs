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
