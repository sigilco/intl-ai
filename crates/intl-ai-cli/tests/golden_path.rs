use assert_cmd::Command;
use intl_ai_core::lockfile::Origin;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Golden-path e2e against the replay transport (plan section 8): no real
/// API is ever touched. INTL_AI_NOW pins lockfile timestamps so outputs are
/// byte-deterministic.
fn write(dir: &Path, rel: &str, body: &str) -> PathBuf {
    let path = dir.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, body).unwrap();
    path
}

fn fixture() -> TempDir {
    let dir = TempDir::new().unwrap();
    write(
        dir.path(),
        "intl-ai.toml",
        r#"locale_dir = "locales"
source = "en"
targets = ["fr", "de"]

[provider]
kind = "replay"
file = "cassette.json"

[check]
fail_on = ["missing", "stale"]
"#,
    );
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
    write(
        dir.path(),
        "cassette.json",
        r#"{
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
"#,
    );
    dir
}

fn cmd(dir: &TempDir) -> Command {
    let mut c = Command::cargo_bin("intl-ai").unwrap();
    c.current_dir(dir.path())
        .env("INTL_AI_NOW", "2026-09-25T00:00:00Z");
    c
}

fn read_json(dir: &TempDir, rel: &str) -> Value {
    let text = fs::read_to_string(dir.path().join(rel)).unwrap();
    serde_json::from_str(&text).unwrap()
}

#[test]
fn golden_path_fill_adopts_and_never_overwrites_human() {
    let dir = fixture();

    cmd(&dir).arg("fill").assert().success();

    // fr.json: nav.home was human-written -> adopted untouched; the two
    // missing keys got AI values, nested shape preserved.
    let fr = read_json(&dir, "locales/fr.json");
    assert_eq!(fr["nav"]["home"], "Accueil humain");
    assert_eq!(fr["nav"]["settings"], "Paramètres");
    assert_eq!(fr["greeting"], "Bonjour");

    // de.json did not exist -> created, all keys AI-filled.
    let de = read_json(&dir, "locales/de.json");
    assert_eq!(de["nav"]["home"], "Startseite");
    assert_eq!(de["greeting"], "Hallo");

    // Lockfile shards: human adoption for nav.home, ai origin elsewhere.
    let fr_lock = fs::read_to_string(dir.path().join("locales/intl-ai.lock.d/fr.toml")).unwrap();
    assert!(fr_lock.contains("[entries.\"nav.home\"]"));
    assert!(fr_lock.contains("origin = \"human\""));
    assert!(fr_lock.contains("origin = \"ai\""));
    assert!(fr_lock.contains("updated_at = \"2026-09-25T00:00:00Z\""));

    // Idempotent: second fill writes nothing new.
    let before = fs::read_to_string(dir.path().join("locales/fr.json")).unwrap();
    cmd(&dir).arg("fill").assert().success();
    let after = fs::read_to_string(dir.path().join("locales/fr.json")).unwrap();
    assert_eq!(before, after);
}

#[test]
fn check_reports_missing_and_exits_1_when_gated() {
    let dir = fixture();

    // Before any fill, everything is missing for de and partially for fr.
    let out = cmd(&dir)
        .args(["check", "--format", "json"])
        .assert()
        .failure()
        .get_output()
        .clone();
    let report: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(report["has_issues"], true);
    assert!(report["locales"]["de"]["missing"].as_array().unwrap().len() == 3);
    assert_eq!(
        report["locales"]["fr"]["missing"].as_array().unwrap().len(),
        2
    );

    // After fill, check is clean.
    cmd(&dir).arg("fill").assert().success();
    cmd(&dir)
        .args(["check", "--format", "json"])
        .assert()
        .success();
}

#[test]
fn positional_rule_flips_origin_on_human_edit() {
    let dir = fixture();
    cmd(&dir).arg("fill").assert().success();

    // Human edits the AI-written greeting.
    let fr_path = dir.path().join("locales/fr.json");
    let mut fr: Value = serde_json::from_str(&fs::read_to_string(&fr_path).unwrap()).unwrap();
    fr["greeting"] = Value::String("Salut édité à la main".into());
    fs::write(&fr_path, serde_json::to_string_pretty(&fr).unwrap()).unwrap();

    // fill reconciles: entry flips to human and is left alone forever after.
    cmd(&dir).arg("fill").assert().success();
    let shard = intl_ai_core::lockfile::load_shard(&dir.path().join("locales"), "fr").unwrap();
    assert_eq!(
        shard.entries["greeting"].origin,
        intl_ai_core::lockfile::Origin::Human
    );

    // check reports it as modified->human already reconciled? The entry is
    // human now, so `modified` is empty and fill never touches it again.
    let out = cmd(&dir)
        .args(["check", "--format", "json"])
        .assert()
        .get_output()
        .clone();
    let report: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(
        report["locales"]["fr"]["modified"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn stale_source_is_a_finding_then_fixed_by_fill_stale() {
    let dir = fixture();
    cmd(&dir).arg("fill").assert().success();

    // Source greeting changes -> de/fr greeting entries become stale.
    write(
        dir.path(),
        "locales/en.json",
        r#"{
  "nav": { "home": "Home", "settings": "Settings" },
  "greeting": "Hello there"
}
"#,
    );
    let cassette = read_json(&dir, "cassette.json");
    let mut cassette = cassette;
    cassette["fr"]["Hello there"] = "Bonjour là".into();
    cassette["de"]["Hello there"] = "Hallo dort".into();
    write(
        dir.path(),
        "cassette.json",
        &serde_json::to_string_pretty(&cassette).unwrap(),
    );

    let out = cmd(&dir)
        .args(["check", "--format", "json"])
        .assert()
        .failure()
        .get_output()
        .clone();
    let report: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(
        report["locales"]["de"]["stale"]
            .as_array()
            .unwrap()
            .contains(&Value::String("greeting".into()))
    );
    // nav.home in fr is human: stale is still reported for it? nav.home's
    // source didn't change, only greeting did.
    assert!(
        report["locales"]["fr"]["stale"]
            .as_array()
            .unwrap()
            .contains(&Value::String("greeting".into()))
    );

    // fr greeting was AI-owned, de greeting too: fill --stale re-fills both.
    cmd(&dir).args(["fill", "--stale"]).assert().success();
    let de = read_json(&dir, "locales/de.json");
    assert_eq!(de["greeting"], "Hallo dort");
    // Check clean again (missing is gated too but nothing is missing).
    cmd(&dir)
        .args(["check", "--format", "json"])
        .assert()
        .success();
}

#[test]
fn config_validate_json() {
    let dir = fixture();
    let out = cmd(&dir)
        .args(["config", "validate", "--format", "json"])
        .assert()
        .success()
        .get_output()
        .clone();
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["valid"], true);
    assert_eq!(v["config"]["source"], "en");
}

#[test]
fn mark_and_review_roundtrip() {
    let dir = fixture();
    cmd(&dir).arg("fill").assert().success();

    // greeting is ai+unreviewed. review it.
    cmd(&dir)
        .args(["review", "greeting", "--locale", "fr"])
        .assert()
        .success();
    let shard = load_shard(&dir, "fr");
    assert!(shard.entries["greeting"].reviewed);

    // Return it to AI stewardship explicitly.
    cmd(&dir)
        .args(["mark", "greeting", "--locale", "fr", "--origin", "ai"])
        .assert()
        .success();
    let shard = load_shard(&dir, "fr");
    assert_eq!(shard.entries["greeting"].origin, Origin::Ai);
}

fn load_shard(dir: &TempDir, locale: &str) -> intl_ai_core::lockfile::Shard {
    intl_ai_core::lockfile::load_shard(&dir.path().join("locales"), locale).unwrap()
}
