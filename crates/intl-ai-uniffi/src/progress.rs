//! Progress/streaming surface for the foreign bindings.
//!
//! `IntlAiProgress` is the callback interface foreign consumers
//! implement to receive `core::progress::ProgressEvent`s. The events
//! are mirrored here as `uniffi::Enum`s (usize -> u64, BTreeMap ->
//! HashMap) so they lower cleanly to Swift/Kotlin.
//!
//! Threading: the pipelines are synchronous. `fill_async`/`check_async`
//! run them on a dedicated worker thread and the observer is invoked on
//! that thread, in pipeline order, until the run finishes. The returned
//! future never blocks the foreign executor; it completes once the
//! worker has sent the report (or the worker died, surfaced as an
//! `IntlAiError::Other`).

use crate::error::IntlAiError;
use crate::types::{ErrorKind, FindingKind, OriginFilter};
use intl_ai_core::diff as core_diff;
use intl_ai_core::lockfile::Origin;
use intl_ai_core::progress as core_progress;
use std::collections::HashMap;
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

/// Foreign-implemented sink for pipeline progress events.
///
/// `on_event` is called once per event, in emission order, on the
/// pipeline's worker thread — never concurrently for a single run, and
/// never after `RunFinished`. Implementations must be thread-safe
/// (`Send + Sync`) and should return quickly: the pipeline waits on
/// each callback.
#[uniffi::export(with_foreign)]
pub trait IntlAiProgress: Send + Sync {
    fn on_event(&self, event: ProgressEvent);
}

/// Which pipeline produced an event (mirrors `core::progress::Pipeline`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum Pipeline {
    Fill,
    Check,
}

impl From<core_progress::Pipeline> for Pipeline {
    fn from(p: core_progress::Pipeline) -> Self {
        match p {
            core_progress::Pipeline::Fill => Self::Fill,
            core_progress::Pipeline::Check => Self::Check,
        }
    }
}

/// Per-key outcome of a fill batch (`ProgressEvent::KeyDone`; mirrors
/// `core::progress::KeyOutcome`).
#[derive(Debug, Clone, uniffi::Enum)]
pub enum KeyOutcome {
    /// Value adopted into the locale file and lockfile shard.
    Written {
        /// Lockfile origin of the adopted entry.
        origin: OriginFilter,
        /// The adopted value overwrote a human-owned one.
        regenerated_human: bool,
        /// Gate findings still open on the adopted value.
        unresolved: u64,
        /// Per-check quality scores the gate emitted.
        scores: HashMap<String, f64>,
    },
    /// Terminal failure: batch transport error or provider omission.
    Failed { kind: ErrorKind, message: String },
}

impl From<core_progress::KeyOutcome> for KeyOutcome {
    fn from(o: core_progress::KeyOutcome) -> Self {
        match o {
            core_progress::KeyOutcome::Written {
                origin,
                regenerated_human,
                unresolved,
                scores,
            } => Self::Written {
                origin: origin.into(),
                regenerated_human,
                unresolved: unresolved as u64,
                scores: scores.into_iter().collect(),
            },
            core_progress::KeyOutcome::Failed { kind, message } => Self::Failed {
                kind: kind.into(),
                message,
            },
        }
    }
}

impl From<Origin> for OriginFilter {
    fn from(o: Origin) -> Self {
        match o {
            Origin::Ai => Self::Ai,
            Origin::Human => Self::Human,
        }
    }
}

impl From<core_diff::FindingKind> for FindingKind {
    fn from(k: core_diff::FindingKind) -> Self {
        match k {
            core_diff::FindingKind::Missing => Self::Missing,
            core_diff::FindingKind::Stale => Self::Stale,
            core_diff::FindingKind::Invalid => Self::Invalid,
            core_diff::FindingKind::Modified => Self::Modified,
            core_diff::FindingKind::Extra => Self::Extra,
            core_diff::FindingKind::Unreviewed => Self::Unreviewed,
        }
    }
}

/// One incremental pipeline event (mirrors
/// `core::progress::ProgressEvent`; owned data only).
#[derive(Debug, Clone, uniffi::Enum)]
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
        keys: u64,
        attempt: u32,
    },
    /// A provider batch resolved: `answered` keys came back, `failed`
    /// keys terminal-failed.
    BatchFinished {
        locale: String,
        attempt: u32,
        answered: u64,
        failed: u64,
    },
    /// One key's outcome in a fill batch.
    KeyDone {
        locale: String,
        key: String,
        outcome: KeyOutcome,
    },
    /// A check run produced a finding.
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
    /// failures.
    RunFinished {
        pipeline: Pipeline,
        locales: u64,
        failures: u64,
    },
}

impl From<core_progress::ProgressEvent> for ProgressEvent {
    fn from(e: core_progress::ProgressEvent) -> Self {
        match e {
            core_progress::ProgressEvent::RunStarted { pipeline, locales } => Self::RunStarted {
                pipeline: pipeline.into(),
                locales,
            },
            core_progress::ProgressEvent::BatchStarted {
                locale,
                keys,
                attempt,
            } => Self::BatchStarted {
                locale,
                keys: keys as u64,
                attempt,
            },
            core_progress::ProgressEvent::BatchFinished {
                locale,
                attempt,
                answered,
                failed,
            } => Self::BatchFinished {
                locale,
                attempt,
                answered: answered as u64,
                failed: failed as u64,
            },
            core_progress::ProgressEvent::KeyDone {
                locale,
                key,
                outcome,
            } => Self::KeyDone {
                locale,
                key,
                outcome: outcome.into(),
            },
            core_progress::ProgressEvent::Finding {
                locale,
                kind,
                key,
                check,
                message,
                cached,
            } => Self::Finding {
                locale,
                kind: kind.into(),
                key,
                check,
                message,
                cached,
            },
            core_progress::ProgressEvent::LocaleFinished { pipeline, locale } => {
                Self::LocaleFinished {
                    pipeline: pipeline.into(),
                    locale,
                }
            }
            core_progress::ProgressEvent::RunFinished {
                pipeline,
                locales,
                failures,
            } => Self::RunFinished {
                pipeline: pipeline.into(),
                locales: locales as u64,
                failures: failures as u64,
            },
        }
    }
}

/// `core::Progress` adapter around a foreign `IntlAiProgress`
/// implementation. Attached to `FillOptions`/`CheckOptions.observer`.
struct ForeignObserver(Arc<dyn IntlAiProgress>);

impl Debug for ForeignObserver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ForeignObserver")
    }
}

impl core_progress::Progress for ForeignObserver {
    fn on_event(&self, event: core_progress::ProgressEvent) {
        self.0.on_event(event.into());
    }
}

/// Wrap a foreign observer for `FillOptions`/`CheckOptions.observer`.
pub(crate) fn core_observer(observer: Arc<dyn IntlAiProgress>) -> Arc<dyn core_progress::Progress> {
    Arc::new(ForeignObserver(observer))
}

/// Run `f` on a dedicated OS thread and await its result.
///
/// The crate has no async runtime, so this is `std::thread` + a oneshot
/// rather than `tokio::task::spawn_blocking`. The returned future is
/// `Send` (UniFFI polls it from the foreign executor's threads) and
/// completes once: either the worker delivered `f()`'s output, or it
/// panicked and the sender was dropped.
pub(crate) async fn run_blocking<T, F>(f: F) -> Result<T, IntlAiError>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    let (tx, rx) = oneshot::channel();
    std::thread::Builder::new()
        .name("intl-ai-worker".into())
        .spawn(move || {
            let _ = tx.send(f());
        })
        .map_err(|e| IntlAiError::Io(format!("spawn worker thread: {e}")))?;
    rx.await
        .map_err(|_| IntlAiError::Other("worker thread exited without a result".into()))
}

/// Minimal oneshot channel: one value, sender wakes the awaiting
/// receiver. Stands in for `futures::channel::oneshot`/`tokio::sync`
/// since no async runtime (or futures crate) is a dependency.
mod oneshot {
    use super::*;

    struct Shared<T> {
        value: Option<T>,
        waker: Option<Waker>,
    }

    pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
        let shared = Arc::new(Mutex::new(Shared {
            value: None,
            waker: None,
        }));
        (Sender(Arc::clone(&shared)), Receiver(shared))
    }

    pub struct Sender<T>(Arc<Mutex<Shared<T>>>);

    impl<T> Sender<T> {
        /// Fails only if the receiver hung up first; the value is lost
        /// either way, so the error carries nothing.
        pub fn send(self, value: T) -> Result<(), ()> {
            let waker = {
                let mut guard = lock(&self.0);
                guard.value = Some(value);
                guard.waker.take()
            };
            if let Some(w) = waker {
                w.wake();
            }
            Ok(())
        }
    }

    pub struct Receiver<T>(Arc<Mutex<Shared<T>>>);

    impl<T> Future for Receiver<T> {
        /// `Err(())` = the sender was dropped without sending (worker panic).
        type Output = Result<T, ()>;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let mut guard = lock(&self.0);
            if let Some(value) = guard.value.take() {
                Poll::Ready(Ok(value))
            } else if Arc::strong_count(&self.0) == 1 {
                Poll::Ready(Err(()))
            } else {
                guard.waker = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }

    /// Poison-tolerant lock: a panicked worker may still have delivered
    /// its value, and a poisoned receiver can still read it.
    fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
        m.lock().unwrap_or_else(|e| e.into_inner())
    }
}
