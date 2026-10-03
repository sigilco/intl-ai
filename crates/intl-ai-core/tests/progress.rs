//! Progress observer contract: event ordering through `fill` and `check`
//! with a deterministic (replay-style) transport and a recording sink.

use intl_ai_core::check::{CheckOptions, check};
use intl_ai_core::config::load;
use intl_ai_core::fill::{FillOptions, fill};
use intl_ai_core::progress::{KeyOutcome, Progress, ProgressEvent};
use intl_ai_core::selector::KeySelector;
use intl_ai_core::transport::{TranslateRequest, TranslateResponse, Translated, Transport};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tempfile::TempDir;

/// Deterministic transport: answers every requested key, except keys
/// listed in `drop` which it omits from the response.
struct StubTransport {
    drop: Vec<String>,
}

impl Transport for StubTransport {
    fn id(&self) -> &str {
        "stub"
    }

    fn translate(&self, req: &TranslateRequest) -> intl_ai_core::Result<TranslateResponse> {
        Ok(TranslateResponse {
            translations: req
                .entries
                .iter()
                .filter(|e| !self.drop.contains(&e.key))
                .map(|e| Translated {
                    key: e.key.clone(),
                    value: format!("{}:{}", req.target_locale, e.source),
                })
                .collect(),
            model: "stub".into(),
        })
    }
}

#[derive(Debug, Default)]
struct Recorder(Mutex<Vec<ProgressEvent>>);

impl Progress for Recorder {
    fn on_event(&self, event: ProgressEvent) {
        self.0.lock().unwrap().push(event);
    }
}

fn write(dir: &Path, rel: &str, body: &str) -> PathBuf {
    let path = dir.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, body).unwrap();
    path
}

fn seed(dir: &TempDir, batch_size: Option<usize>) -> intl_ai_core::config::ResolvedConfig {
    let batch = batch_size
        .map(|b| format!("batch_size = {b}\n"))
        .unwrap_or_default();
    write(
        dir.path(),
        "intl-ai.toml",
        &format!(
            "locale_dir = \"locales\"\nsource = \"en\"\ntargets = [\"fr\"]\n{batch}\n[provider]\nkind = \"replay\"\nfile = \"cassette.json\"\n"
        ),
    );
    write(
        dir.path(),
        "locales/en.json",
        r#"{"a":"Hello","b":"Bye","c":"Yes"}"#,
    );
    load(None, dir.path()).unwrap()
}

/// Compact per-event signature for ordering assertions.
fn signature(e: &ProgressEvent) -> String {
    match e {
        ProgressEvent::RunStarted { pipeline, .. } => format!("run_started:{pipeline:?}"),
        ProgressEvent::BatchStarted {
            locale,
            keys,
            attempt,
        } => format!("batch_started:{locale}:{attempt}:{keys}"),
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
        } => match outcome {
            KeyOutcome::Written { .. } => format!("key_written:{locale}:{key}"),
            KeyOutcome::Failed { kind, .. } => format!("key_failed:{locale}:{key}:{kind:?}"),
        },
        ProgressEvent::Finding {
            locale, kind, key, ..
        } => {
            format!("finding:{locale}:{kind}:{key}")
        }
        ProgressEvent::LocaleFinished { pipeline, locale } => {
            format!("locale_finished:{pipeline:?}:{locale}")
        }
        ProgressEvent::RunFinished {
            pipeline,
            locales,
            failures,
        } => format!("run_finished:{pipeline:?}:{locales}:{failures}"),
    }
}

fn signatures(rec: &Recorder) -> Vec<String> {
    rec.0.lock().unwrap().iter().map(signature).collect()
}

#[test]
fn fill_emits_ordered_events() {
    let dir = TempDir::new().unwrap();
    let cfg = seed(&dir, Some(1)); // 3 keys, batch_size 1 -> 3 batches
    let rec = Arc::new(Recorder::default());
    let opts = FillOptions {
        locales: None,
        selector: KeySelector::any(),
        stale_only: false,
        regenerate: false,
        include_human: false,
        dry_run: false,
        no_cache: true,
        observer: Some(rec.clone()),
    };
    let transport = StubTransport { drop: vec![] };
    let report = fill(&cfg, &transport, &opts, None).unwrap();
    assert_eq!(report.locales["fr"].written, 3);

    assert_eq!(
        signatures(&rec),
        vec![
            "run_started:Fill",
            "batch_started:fr:0:1",
            "batch_finished:fr:0:1:0",
            "key_written:fr:a",
            "batch_started:fr:0:1",
            "batch_finished:fr:0:1:0",
            "key_written:fr:b",
            "batch_started:fr:0:1",
            "batch_finished:fr:0:1:0",
            "key_written:fr:c",
            "locale_finished:Fill:fr",
            "run_finished:Fill:1:0",
        ]
    );
}

#[test]
fn fill_reports_failed_keys() {
    let dir = TempDir::new().unwrap();
    let cfg = seed(&dir, None);
    let rec = Arc::new(Recorder::default());
    let opts = FillOptions {
        locales: None,
        selector: KeySelector::any(),
        stale_only: false,
        regenerate: false,
        include_human: false,
        dry_run: false,
        no_cache: true,
        observer: Some(rec.clone()),
    };
    let transport = StubTransport {
        drop: vec!["b".into()],
    };
    let report = fill(&cfg, &transport, &opts, None).unwrap();
    assert_eq!(report.locales["fr"].written, 2);
    assert_eq!(report.locales["fr"].omitted, vec!["b"]);

    assert_eq!(
        signatures(&rec),
        vec![
            "run_started:Fill",
            "batch_started:fr:0:3",
            "batch_finished:fr:0:2:1",
            "key_failed:fr:b:OutputTruncated",
            "key_written:fr:a",
            "key_written:fr:c",
            "locale_finished:Fill:fr",
            "run_finished:Fill:1:1",
        ]
    );
}

#[test]
fn check_emits_finding_events() {
    let dir = TempDir::new().unwrap();
    let cfg = seed(&dir, None);
    // fr has "other" (extra vs source) and lacks a/b/c (missing).
    write(dir.path(), "locales/fr.json", r#"{"other":"x"}"#);
    let rec = Arc::new(Recorder::default());
    let opts = CheckOptions {
        locales: None,
        origin_filter: None,
        fail_on: Some(vec![]),
        selector: KeySelector::any(),
        no_cache: true,
        observer: Some(rec.clone()),
    };
    let report = check(&cfg, &opts, &[], None).unwrap();
    assert_eq!(report.locales["fr"].missing.len(), 3);
    assert_eq!(report.locales["fr"].extra, vec!["other"]);

    assert_eq!(
        signatures(&rec),
        vec![
            "run_started:Check",
            "finding:fr:missing:a",
            "finding:fr:missing:b",
            "finding:fr:missing:c",
            "finding:fr:extra:other",
            "locale_finished:Check:fr",
            "run_finished:Check:1:0",
        ]
    );
}

#[test]
fn no_observer_emits_nothing_and_passes() {
    let dir = TempDir::new().unwrap();
    let cfg = seed(&dir, None);
    let opts = FillOptions {
        locales: None,
        selector: KeySelector::any(),
        stale_only: false,
        regenerate: false,
        include_human: false,
        dry_run: false,
        no_cache: true,
        observer: None,
    };
    let transport = StubTransport { drop: vec![] };
    let report = fill(&cfg, &transport, &opts, None).unwrap();
    assert_eq!(report.locales["fr"].written, 3);
}
