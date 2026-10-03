//! Incremental progress/events for the `fill` and `check` pipelines.
//!
//! The pipelines are synchronous and batch-oriented; an observer attached
//! via `FillOptions`/`CheckOptions` receives every event on the calling
//! thread. Events carry owned data only (no borrows) and serialize to a
//! tagged shape (`{type: "batch_started", ...}`) so FFI/WASM consumers —
//! UniFFI foreign futures, JS callbacks, a CLI progress UI — can forward
//! them across thread and language boundaries.

use crate::diff::FindingKind;
use crate::error::ErrorType;
use crate::lockfile::Origin;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fmt::Debug;

/// Sink for incremental pipeline events. `Debug` lets the options
/// structs keep their derived `Debug`; `Send + Sync` lets consumers move
/// or share the observer across threads.
pub trait Progress: Debug + Send + Sync {
    fn on_event(&self, event: ProgressEvent);
}

/// Observer that drops every event. Pipelines use it as the `None`
/// default so call sites emit unconditionally.
#[derive(Debug, Default)]
pub struct NoopProgress;

impl Progress for NoopProgress {
    fn on_event(&self, _event: ProgressEvent) {}
}

/// Which pipeline produced an event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Pipeline {
    Fill,
    Check,
}

/// One incremental pipeline event.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProgressEvent {
    /// The run started; `locales` is the resolved target list.
    RunStarted {
        pipeline: Pipeline,
        locales: Vec<String>,
    },
    /// Fill requested a batch from the provider. `attempt` is 0 for the
    /// initial pass, 1+ for gate corrective rounds.
    BatchStarted {
        locale: String,
        keys: usize,
        attempt: u32,
    },
    /// A provider batch resolved: `answered` keys came back, `failed`
    /// keys terminal-failed (transport error or provider omission).
    BatchFinished {
        locale: String,
        attempt: u32,
        answered: usize,
        failed: usize,
    },
    /// One key's outcome in a fill batch.
    KeyDone {
        locale: String,
        key: String,
        outcome: KeyOutcome,
    },
    /// A check run produced a finding. Structural buckets (missing,
    /// stale, modified, extra, unreviewed) carry empty `check`/`message`;
    /// `invalid` findings carry the check id and its message.
    Finding {
        locale: String,
        kind: FindingKind,
        key: String,
        check: String,
        message: String,
        /// Replayed from the incremental check cache, not re-run.
        cached: bool,
    },
    /// A locale's processing concluded (succeeded or soft-failed).
    LocaleFinished { pipeline: Pipeline, locale: String },
    /// The run finished: `locales` processed and `failures` run-level
    /// failures (fill key/batch failures, check check-level errors).
    RunFinished {
        pipeline: Pipeline,
        locales: usize,
        failures: usize,
    },
}

/// Per-key outcome of a fill batch (`ProgressEvent::KeyDone`).
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum KeyOutcome {
    /// Value adopted into the locale file and lockfile shard.
    Written {
        /// Lockfile origin of the adopted entry (always `ai` today;
        /// carried so consumers render provenance without internals).
        origin: Origin,
        /// The adopted value overwrote a human-owned one
        /// (--regenerate --include-human tier).
        regenerated_human: bool,
        /// Gate findings still open on the adopted value.
        unresolved: usize,
        /// Per-check quality scores the gate emitted (`judge`).
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        scores: BTreeMap<String, f64>,
    },
    /// Terminal failure: batch transport error or provider omission.
    Failed { kind: ErrorType, message: String },
}
