//! Fill-time validation gate (W2b part C): `[fill] validate` /
//! `--validate` run feedback-eligible checks between translate and
//! adoption; failures get one corrective round via `feedback` notes;
//! still-failing keys are adopted with `quality.unresolved` on the shard
//! entry and exit 1.

use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

fn write(dir: &Path, rel: &str, body: &str) -> PathBuf {
    let path = dir.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, body).unwrap();
    path
}

fn cmd(dir: &TempDir) -> Command {
    let mut c = Command::cargo_bin("intl-ai").unwrap();
    c.current_dir(dir.path());
    c
}

const REPLAY_CONFIG: &str = r#"locale_dir = "locales"
source = "en-US"
targets = ["fr"]

[provider]
kind = "replay"
file = "cassette.json"
"#;

fn seed(dir: &TempDir, config: &str, cassette: &str) {
    write(dir.path(), "intl-ai.toml", config);
    write(dir.path(), "cassette.json", cassette);
    write(dir.path(), "locales/en-US.json", r#"{"a":"Hello {name}"}"#);
}

fn fill_report(dir: &TempDir, args: &[&str]) -> (Value, bool) {
    let out = cmd(dir)
        .args(args)
        .args(["--format", "json"])
        .assert()
        .get_output()
        .clone();
    let ok = out.status.success();
    (serde_json::from_slice(&out.stdout).unwrap(), ok)
}

fn fr_file(dir: &TempDir) -> Value {
    serde_json::from_str(&fs::read_to_string(dir.path().join("locales/fr.json")).unwrap()).unwrap()
}

fn shard_entry(dir: &TempDir, key: &str) -> Value {
    let text = fs::read_to_string(dir.path().join("locales/intl-ai.lock.d/fr.toml")).unwrap();
    let v: toml::Value = toml::from_str(&text).unwrap();
    serde_json::to_value(&v["entries"][key]).unwrap()
}

#[test]
fn gate_refill_heals_bad_value() {
    let dir = TempDir::new().unwrap();
    seed(
        &dir,
        &format!("{REPLAY_CONFIG}\n[fill]\nvalidate = [\"icu\"]\n"),
        // Round 1 answer drops the placeholder brace (broken ICU); the
        // retry table answers the corrective request with a fixed value.
        r#"{"fr":{"Hello {name}":"Bonjour {name"},"_none_":"x"},
           "fr.retry":{"a":"Bonjour {name}!"}}"#,
    );
    // Make round 1 actually broken: cassette value has an unclosed brace.
    write(
        dir.path(),
        "cassette.json",
        r#"{"fr":{"Hello {name}":"Bonjour {name"},
           "fr.retry":{"a":"Bonjour {name}"}}"#,
    );
    let (report, ok) = fill_report(&dir, &["fill"]);
    assert!(ok, "{report}");
    let res = &report["locales"]["fr"];
    assert_eq!(res["refilled"], 1);
    assert_eq!(res["written"], 1);
    assert_eq!(fr_file(&dir)["a"], "Bonjour {name}");
}

#[test]
fn gate_unresolved_adopts_and_exits_1() {
    let dir = TempDir::new().unwrap();
    seed(
        &dir,
        &format!("{REPLAY_CONFIG}\n[fill]\nvalidate = [\"icu\"]\n"),
        r#"{"fr":{"Hello {name}":"Bonjour {name"}}"#,
    );
    let (report, ok) = fill_report(&dir, &["fill"]);
    assert!(!ok, "unresolved gate must exit 1: {report}");
    let res = &report["locales"]["fr"];
    assert_eq!(res["refilled"], 1);
    let unresolved = res["unresolved"].as_array().unwrap();
    assert_eq!(unresolved.len(), 1);
    assert_eq!(unresolved[0]["check"], "icu");
    // Never withhold: the last attempt is written and recorded.
    assert_eq!(fr_file(&dir)["a"], "Bonjour {name");
    let entry = shard_entry(&dir, "a");
    assert_eq!(entry["origin"], "ai");
    assert_eq!(entry["reviewed"], false);
    assert!(
        entry["quality"]["unresolved"][0]
            .as_str()
            .unwrap()
            .contains("icu")
    );
}

#[test]
fn no_validate_flag_and_flag_validate_list() {
    // --no-validate bypasses a configured gate entirely.
    let dir = TempDir::new().unwrap();
    seed(
        &dir,
        &format!("{REPLAY_CONFIG}\n[fill]\nvalidate = [\"icu\"]\n"),
        r#"{"fr":{"Hello {name}":"Bonjour {name"}}"#,
    );
    let (report, ok) = fill_report(&dir, &["fill", "--no-validate"]);
    assert!(ok, "{report}");
    assert_eq!(fr_file(&dir)["a"], "Bonjour {name");

    // --validate works without any config entry.
    let dir2 = TempDir::new().unwrap();
    seed(&dir2, REPLAY_CONFIG, r#"{"fr":{"Hello {name}":"Bonjour"}}"#);
    let (report, ok) = fill_report(&dir2, &["fill", "--validate", "placeholder-parity"]);
    assert!(!ok);
    assert_eq!(
        report["locales"]["fr"]["unresolved"][0]["check"],
        "placeholder-parity"
    );
}

#[test]
fn gate_rejects_non_feedback_checks() {
    let dir = TempDir::new().unwrap();
    seed(
        &dir,
        &format!("{REPLAY_CONFIG}\n[fill]\nvalidate = [\"dialect:en-US\"]\n"),
        r#"{"fr":{}}"#,
    );
    cmd(&dir)
        .args(["fill"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("cannot gate fill"));

    // `config validate` rejects it too, before any fill runs.
    cmd(&dir)
        .args(["config", "validate"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("cannot gate fill"));

    // Unknown names fail fast too.
    cmd(&dir)
        .args(["fill", "--validate", "bogus"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("unknown check"));
}

#[test]
fn gate_records_judge_scores() {
    let dir = TempDir::new().unwrap();
    seed(
        &dir,
        &format!(
            "{REPLAY_CONFIG}\n[fill]\nvalidate = [\"judge\"]\n\n[[checks]]\nid = \"judge\"\nthreshold = 0.8\n"
        ),
        // a scores clean (0.9): adopted, quality.scores recorded.
        // b scores 0.5 and stays broken: unresolved + scores recorded.
        r#"{"fr":{"Hello {name}":"Bonjour {name}","Bye":"Au revoir"},
            "fr.judge":{"a":"0.9","b":"0.5"},
            "fr.retry":{"b":"encore"}}"#,
    );
    write(
        dir.path(),
        "locales/en-US.json",
        r#"{"a":"Hello {name}","b":"Bye"}"#,
    );
    let (report, ok) = fill_report(&dir, &["fill"]);
    assert!(!ok, "unresolved key b must fail the run: {report}");
    let a = shard_entry(&dir, "a");
    assert_eq!(a["quality"]["scores"]["judge"], 0.9);
    assert!(a["quality"]["unresolved"].is_null());
    let b = shard_entry(&dir, "b");
    assert_eq!(b["quality"]["scores"]["judge"], 0.5);
    assert_eq!(
        b["quality"]["unresolved"][0],
        Value::String("judge: score 0.50 below 0.8".into())
    );
}

#[test]
fn judge_threshold_flag_overrides_config() {
    let dir = TempDir::new().unwrap();
    seed(
        &dir,
        &format!(
            "{REPLAY_CONFIG}\n[fill]\nvalidate = [\"judge\"]\n\n[[checks]]\nid = \"judge\"\nthreshold = 0.8\n"
        ),
        r#"{"fr":{"Hello {name}":"Bonjour {name}"},
            "fr.judge":{"a":"0.9"}}"#,
    );
    // 0.9 passes the configured 0.8; --judge-threshold 0.95 re-gates it.
    let (report, ok) = fill_report(&dir, &["fill", "--judge-threshold", "0.95"]);
    assert!(!ok, "{report}");
    let a = shard_entry(&dir, "a");
    assert_eq!(
        a["quality"]["unresolved"][0],
        Value::String("judge: score 0.90 below 0.95".into())
    );

    cmd(&dir)
        .args(["fill", "--judge-threshold", "1.5"])
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "--judge-threshold must be in 0..=1",
        ));
}
