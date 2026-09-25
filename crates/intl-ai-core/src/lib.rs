//! intl-ai v1 core: flatten/hash, sharded lockfile, config, diff/fill/check
//! pipeline, transport contract. Serializable in/out; no subprocess types here
//! (command transport lives in intl-ai-providers, per plan section 3).

pub mod check;
pub mod clock;
pub mod config;
pub mod diff;
pub mod error;
pub mod fill;
pub mod flatten;
pub mod hash;
pub mod lockfile;
pub mod ops;
pub mod report;
pub mod selector;
pub mod stat_cache;
pub mod transport;

pub use error::{Error, ErrorType, Result};
