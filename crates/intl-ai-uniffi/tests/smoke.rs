//! Golden-path smoke test of the FFI surface against the replay
//! transport: no real API is touched. Same fixture shape as the CLI's
//! golden_path e2e.

use intl_ai_uniffi::{CheckOptions, ConfigFormat, FillOptions, IntlAi};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
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
    write(
        dir.path(),
        "locales/fr.json",
        r#"{
  "nav": {
    "home": "Accueil humain"
  }
}
"#,
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

#[test]
fn path_constructor_fills_and_checks() {
    let dir = fixture();
    let api = IntlAi::new(
        Some(dir.path().join("intl-ai.toml").display().to_string()),
        Some(dir.path().display().to_string()),
    )
    .unwrap();

    assert_eq!(api.source_locale(), "en");
    assert_eq!(
        api.target_locales(),
        vec!["fr".to_string(), "de".to_string()]
    );

    // Before fill: fr is missing two keys, de is missing the file itself.
    let report = api.check(check_defaults()).unwrap();
    assert!(report.has_issues);
    assert_eq!(report.locales["fr"].missing.len(), 2);

    let report = api.fill(fill_defaults()).unwrap();
    assert!(report.failures.is_empty());
    // fr keeps the human-owned "nav.home" (adopted into the shard) and
    // fills the two missing keys; de gets all three.
    assert_eq!(report.locales["fr"].written, 2);
    assert_eq!(report.locales["fr"].adopted_human, 1);
    assert_eq!(report.locales["de"].written, 3);

    let report = api.check(check_defaults()).unwrap();
    assert!(report.locales["fr"].missing.is_empty());

    let status = api.status(vec![]).unwrap();
    let fr = status.iter().find(|s| s.locale == "fr").unwrap();
    assert_eq!(fr.entries, 3);
    assert_eq!(fr.missing, 0);
    assert_eq!(fr.human, 1);
}

#[test]
fn inline_string_constructor() {
    let dir = fixture();
    let api = IntlAi::from_config_string(
        CONFIG_TOML.to_string(),
        ConfigFormat::Toml,
        Some(dir.path().display().to_string()),
    )
    .unwrap();
    let status = api.status(vec!["de".to_string()]).unwrap();
    assert_eq!(status[0].missing, 3);
}

#[test]
fn json_variants_emit_cli_shaped_json() {
    let dir = fixture();
    let api = IntlAi::new(None, Some(dir.path().display().to_string())).unwrap();

    let json: serde_json::Value =
        serde_json::from_str(&api.check_json(check_defaults()).unwrap()).unwrap();
    assert_eq!(json["has_issues"], true);
    assert!(json["locales"]["fr"]["missing"].is_array());

    let json: serde_json::Value = serde_json::from_str(&api.status_json(vec![]).unwrap()).unwrap();
    assert_eq!(json["locales"].as_array().unwrap().len(), 2);
}

#[test]
fn regenerate_without_scope_or_yes_is_rejected() {
    let dir = fixture();
    let api: Arc<IntlAi> = IntlAi::new(None, Some(dir.path().display().to_string())).unwrap();
    let mut opts = fill_defaults();
    opts.regenerate = true;
    let err = api.fill(opts).unwrap_err();
    assert!(err.to_string().contains("regenerate"));
}
