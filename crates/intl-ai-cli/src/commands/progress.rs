use intl_ai_core::progress::{KeyOutcome, Progress, ProgressEvent};

/// `--progress` observer: one compact stderr line per event so stdout
/// stays the report channel (parseable under `--format json`).
#[derive(Debug)]
pub struct CliProgress;

impl Progress for CliProgress {
    fn on_event(&self, event: ProgressEvent) {
        match event {
            ProgressEvent::RunStarted { pipeline, locales } => {
                eprintln!(
                    "{:?}: started ({} locale(s): {})",
                    pipeline,
                    locales.len(),
                    locales.join(", ")
                );
            }
            ProgressEvent::BatchStarted {
                locale,
                keys,
                attempt,
            } => {
                eprintln!("{locale}: batch attempt {attempt}: requesting {keys} key(s)");
            }
            ProgressEvent::BatchFinished {
                locale,
                attempt,
                answered,
                failed,
            } => {
                eprintln!(
                    "{locale}: batch attempt {attempt}: {answered} answered, {failed} failed"
                );
            }
            ProgressEvent::KeyDone {
                locale,
                key,
                outcome,
            } => match outcome {
                KeyOutcome::Written {
                    unresolved,
                    regenerated_human,
                    ..
                } => {
                    let mut extra = String::new();
                    if regenerated_human {
                        extra.push_str(" (regenerated human)");
                    }
                    if unresolved > 0 {
                        extra.push_str(&format!(" ({unresolved} unresolved)"));
                    }
                    eprintln!("{locale}: wrote {key}{extra}");
                }
                KeyOutcome::Failed { kind, message } => {
                    eprintln!("{locale}: failed {key}: {kind:?}: {message}");
                }
            },
            ProgressEvent::Finding {
                locale,
                kind,
                key,
                check,
                message,
                cached,
            } => {
                let detail = if check.is_empty() {
                    String::new()
                } else {
                    format!(" ({check}: {message})")
                };
                let replay = if cached { " [cached]" } else { "" };
                eprintln!("{locale}: {kind}: {key}{detail}{replay}");
            }
            ProgressEvent::LocaleFinished {
                pipeline: _,
                locale,
            } => {
                eprintln!("{locale}: done");
            }
            ProgressEvent::RunFinished {
                pipeline,
                locales,
                failures,
            } => {
                eprintln!("{pipeline:?}: finished ({locales} locale(s), {failures} failure(s))");
            }
        }
    }
}

/// Wrap the CLI renderer as a shared observer when `--progress` is set.
pub fn cli_observer(enabled: bool) -> Option<std::sync::Arc<dyn Progress>> {
    enabled.then(|| std::sync::Arc::new(CliProgress) as std::sync::Arc<dyn Progress>)
}
