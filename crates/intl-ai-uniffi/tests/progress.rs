//! Exercises the `IntlAiProgress` bridge: an observer attached to
//! `fill_async`/`check_async` receives every event in order on the
//! worker thread. Same replay-transport fixture as the smoke test.

use intl_ai_uniffi::{
    CheckOptions, FillOptions, IntlAi, IntlAiProgress, KeyOutcome, ProgressEvent,
};
use std::fs;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Wake, Waker};
use std::thread::ThreadId;
use tempfile::TempDir;

fn write(dir: &Path, rel: &str, body: &str) -> PathBuf {
    let path = dir.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, body).unwrap();
    path
}

const CONFIG_TOML: &str = r#"locale_dir = "locales"
source = "en"
targets = ["fr", "de"]

[provider]
kind = "replay"
file = "cassette.json"

[check]
fail_on = ["missing", "stale"]
"#;

const CASSETTE: &str = r#"{
  "fr": {
    "Settings": "Paramètres",
    "Hello": "Bonjour"
  },
  "de": {
    "Settings": "Einstellungen",
    "Hello": "Hallo",
    "Home": "Startseite"
  }
}
"#;

fn fixture() -> TempDir {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "intl-ai.toml", CONFIG_TOML);
    write(
        dir.path(),
        "locales/en.json",
        r#"{
  "nav": {
    "home": "Home",
    "settings": "Settings"
  },
  "greeting": "Hello"
}
"#,
    );
    // fr keeps one human-owned key so the fill only requests the two
    // keys the cassette answers ("nav.home" has no fr entry on purpose).
    write(
        dir.path(),
        "locales/fr.json",
        r#"{ "nav": { "home": "Accueil humain" } }"#,
    );
    write(dir.path(), "cassette.json", CASSETTE);
    dir
}

fn fill_defaults() -> FillOptions {
    FillOptions {
        locales: vec![],
        keys: vec![],
        keys_file: None,
        stale: false,
        regenerate: false,
        include_human: false,
        dry_run: false,
        no_cache: true,
        validate: vec![],
        no_validate: false,
        judge_threshold: None,
        yes: false,
    }
}

fn check_defaults() -> CheckOptions {
    CheckOptions {
        locales: vec![],
        keys: vec![],
        keys_file: None,
        origin: None,
        fail_on: None,
        no_cache: true,
    }
}

/// Drive a future on the test thread: park/unpark waker, repoll on
/// every wake (spurious unparks just cost one extra poll).
fn block_on<F: Future>(mut fut: F) -> F::Output {
    struct Parker(std::thread::Thread);
    impl Wake for Parker {
        fn wake(self: Arc<Self>) {
            self.0.unpark();
        }
        fn wake_by_ref(self: &Arc<Self>) {
            self.0.unpark();
        }
    }
    let waker = Waker::from(Arc::new(Parker(std::thread::current())));
    let mut cx = Context::from_waker(&waker);
    // Safe: `fut` is pinned for the rest of this scope and never moved.
    let mut fut = unsafe { Pin::new_unchecked(&mut fut) };
    loop {
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(v) => return v,
            Poll::Pending => std::thread::park(),
        }
    }
}

/// One compact label per event so ordering is easy to assert, plus the
/// thread the callback ran on.
#[derive(Debug)]
struct Seen {
    label: String,
    thread: ThreadId,
}

#[derive(Default)]
struct Collector(Mutex<Vec<Seen>>);

impl IntlAiProgress for Collector {
    fn on_event(&self, event: ProgressEvent) {
        let label = match &event {
            ProgressEvent::RunStarted { pipeline, locales } => {
                format!("run_started:{pipeline:?}:{}", locales.join(","))
            }
            ProgressEvent::BatchStarted {
                locale,
                keys,
                attempt,
            } => format!("batch_started:{locale}:{keys}:{attempt}"),
            ProgressEvent::BatchFinished {
                locale,
                attempt,
                answered,
                failed,
            } => format!("batch_finished:{locale}:{attempt}:{answered}:{failed}"),
            ProgressEvent::KeyDone {
                locale,
                key,
                outcome,
            } => {
                let status = match outcome {
                    KeyOutcome::Written { .. } => "written",
                    KeyOutcome::Failed { .. } => "failed",
                };
                format!("key_done:{locale}:{key}:{status}")
            }
            ProgressEvent::Finding {
                locale, kind, key, ..
            } => {
                format!("finding:{locale}:{kind:?}:{key}")
            }
            ProgressEvent::LocaleFinished { pipeline, locale } => {
                format!("locale_finished:{pipeline:?}:{locale}")
            }
            ProgressEvent::RunFinished {
                pipeline,
                locales,
                failures,
            } => format!("run_finished:{pipeline:?}:{locales}:{failures}"),
        };
        self.0.lock().unwrap().push(Seen {
            label,
            thread: std::thread::current().id(),
        });
    }
}

fn labels(collector: &Collector) -> Vec<String> {
    collector
        .0
        .lock()
        .unwrap()
        .iter()
        .map(|s| s.label.clone())
        .collect()
}

#[test]
fn fill_async_streams_ordered_events_on_worker_thread() {
    let dir = fixture();
    let api = IntlAi::new(None, Some(dir.path().display().to_string())).unwrap();
    let collector = Arc::new(Collector::default());
    let observer: Arc<dyn IntlAiProgress> = collector.clone();
    let caller = std::thread::current().id();

    let report = block_on(api.fill_async(fill_defaults(), Some(observer))).unwrap();
    assert_eq!(report.locales["de"].written, 3);

    let events = labels(&collector);
    // Frame: RunStarted first, RunFinished last, both on Fill.
    assert_eq!(events.first().unwrap(), "run_started:Fill:fr,de");
    assert_eq!(events.last().unwrap(), "run_finished:Fill:2:0");

    // Per-locale order: batch_started -> batch_finished -> key_done* ->
    // locale_finished, with no interleaving between the two locales
    // (the pipeline is sequential).
    let de: Vec<&String> = events.iter().filter(|e| e.contains(":de")).collect();
    assert_eq!(de[0], "batch_started:de:3:0");
    assert_eq!(de[1], "batch_finished:de:0:3:0");
    assert!(
        de[2..de.len() - 1]
            .iter()
            .all(|e| e.starts_with("key_done:"))
    );
    assert_eq!(de[2..de.len() - 1].len(), 3);
    assert_eq!(de[de.len() - 1], "locale_finished:Fill:de");

    let fr_last = events
        .iter()
        .position(|e| e == "locale_finished:Fill:fr")
        .unwrap();
    let de_first = events.iter().position(|e| e.contains(":de")).unwrap();
    assert!(
        fr_last < de_first,
        "locales processed sequentially: {events:?}"
    );

    // The callback ran on the worker thread, not the caller's.
    let threads: Vec<ThreadId> = collector
        .0
        .lock()
        .unwrap()
        .iter()
        .map(|s| s.thread)
        .collect();
    assert!(threads.iter().all(|t| *t != caller));
    assert!(threads.iter().all(|t| *t == threads[0]));
}

#[test]
fn check_async_emits_findings_and_frames() {
    let dir = fixture();
    let api = IntlAi::new(None, Some(dir.path().display().to_string())).unwrap();
    let collector = Arc::new(Collector::default());
    let observer: Arc<dyn IntlAiProgress> = collector.clone();

    let report = block_on(api.check_async(check_defaults(), Some(observer))).unwrap();
    assert!(report.has_findings);

    let events = labels(&collector);
    assert_eq!(events.first().unwrap(), "run_started:Check:fr,de");
    assert!(events.last().unwrap().starts_with("run_finished:Check:2:"));
    assert!(
        events
            .iter()
            .any(|e| e == &"finding:fr:Missing:nav.settings".to_string())
    );
    assert!(
        events
            .iter()
            .any(|e| e == &"locale_finished:Check:de".to_string())
    );
    // Every locale_finished is preceded by its findings.
    let fr_done = events
        .iter()
        .position(|e| e == "locale_finished:Check:fr")
        .unwrap();
    assert!(
        events[..fr_done]
            .iter()
            .any(|e| e.starts_with("finding:fr:"))
    );
}

#[test]
fn async_without_observer_still_completes() {
    let dir = fixture();
    let api = IntlAi::new(None, Some(dir.path().display().to_string())).unwrap();
    let report = block_on(api.fill_async(fill_defaults(), None)).unwrap();
    assert_eq!(report.locales["fr"].written, 2);
    assert!(report.failures.is_empty());
}
