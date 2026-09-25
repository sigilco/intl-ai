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

fn seed(dir: &TempDir, config: &str) {
    write(dir.path(), "intl-ai.toml", config);
    write(dir.path(), "cassette.json", r#"{"fr":{}}"#);
}

fn report(dir: &TempDir, args: &[&str]) -> (Value, bool) {
    let out = cmd(dir)
        .args(args)
        .args(["--format", "json"])
        .assert()
        .get_output()
        .clone();
    let ok = out.status.success();
    (serde_json::from_slice(&out.stdout).unwrap(), ok)
}

#[test]
fn check_reports_icu_and_parity_findings() {
    let dir = TempDir::new().unwrap();
    seed(
        &dir,
        &format!(
            "{REPLAY_CONFIG}\n[[checks]]\nid = \"icu\"\n[[checks]]\nid = \"placeholder-parity\"\n"
        ),
    );
    write(
        dir.path(),
        "locales/en-US.json",
        r#"{"greeting":"Hello {name}","ok":"fine"}"#,
    );
    write(
        dir.path(),
        "locales/fr.json",
        r#"{"greeting":"Bonjour {nom}","ok":"unclosed {brace"}"#,
    );
    let (report, ok) = report(&dir, &["check"]);
    assert!(!ok);
    let invalid = report["locales"]["fr"]["invalid"].as_array().unwrap();
    let pairs: Vec<(&str, &str)> = invalid
        .iter()
        .map(|f| (f["key"].as_str().unwrap(), f["check"].as_str().unwrap()))
        .collect();
    assert!(pairs.contains(&("greeting", "placeholder-parity")));
    assert!(pairs.contains(&("ok", "icu")));
    // parity message keeps the TS wording.
    let parity = invalid
        .iter()
        .find(|f| f["check"] == "placeholder-parity")
        .unwrap();
    assert!(
        parity["message"]
            .as_str()
            .unwrap()
            .contains("Missing tokens: name")
    );
    assert!(
        parity["message"]
            .as_str()
            .unwrap()
            .contains("Extra tokens: nom")
    );
}

#[test]
fn check_dialect_flags_opposite_spellings() {
    let dir = TempDir::new().unwrap();
    seed(
        &dir,
        &format!("{REPLAY_CONFIG}\n[[checks]]\nid = \"dialect:en-GB\"\n"),
    );
    write(dir.path(), "locales/en-US.json", r#"{"a":"the color"}"#);
    write(dir.path(), "locales/fr.json", r#"{"a":"the color"}"#);
    let (report, ok) = report(&dir, &["check"]);
    assert!(!ok);
    let invalid = report["locales"]["fr"]["invalid"].as_array().unwrap();
    assert_eq!(invalid.len(), 1);
    assert_eq!(invalid[0]["check"], "dialect:en-GB");
    assert!(invalid[0]["message"].as_str().unwrap().contains("colour"));
}

#[test]
fn dialect_check_ignores_placeholder_tokens() {
    let dir = TempDir::new().unwrap();
    seed(
        &dir,
        &format!("{REPLAY_CONFIG}\n[[checks]]\nid = \"dialect:en-GB\"\n"),
    );
    write(
        dir.path(),
        "locales/en-US.json",
        r#"{"a":"Pick your {color}"}"#,
    );
    write(
        dir.path(),
        "locales/fr.json",
        r#"{"a":"Pick your {color}"}"#,
    );
    // {color} binds a source argument: not a spelling violation.
    let (report, ok) = report(&dir, &["check"]);
    assert!(ok, "{}", report["locales"]["fr"]["invalid"]);
}

#[test]
fn check_spec_rules_and_self_test() {
    let dir = TempDir::new().unwrap();
    seed(
        &dir,
        &format!("{REPLAY_CONFIG}\n[[checks]]\nspec = \"checks/house.yaml\"\n"),
    );
    write(
        dir.path(),
        "checks/house.yaml",
        r#"id: house-rules
rules:
  - forbidden_terms: ["color"]
    message: "en-GB copy only"
self_test:
  clean: ["the colour"]
  broken: ["the color"]
"#,
    );
    write(dir.path(), "locales/en-US.json", r#"{"a":"the color"}"#);
    write(dir.path(), "locales/fr.json", r#"{"a":"the color"}"#);

    cmd(&dir).args(["check", "--self-test"]).assert().success();

    let (report, ok) = report(&dir, &["check"]);
    assert!(!ok);
    let invalid = report["locales"]["fr"]["invalid"].as_array().unwrap();
    assert_eq!(invalid[0]["check"], "house-rules");
    assert_eq!(invalid[0]["message"], "en-GB copy only");
}

#[test]
fn self_test_fails_when_fixture_contradicts_rules() {
    let dir = TempDir::new().unwrap();
    seed(
        &dir,
        &format!("{REPLAY_CONFIG}\n[[checks]]\nspec = \"checks/bad.yaml\"\n"),
    );
    write(dir.path(), "locales/en-US.json", r#"{"a":"x"}"#);
    write(
        dir.path(),
        "checks/bad.yaml",
        r#"id: bad
rules:
  - forbidden_terms: ["color"]
self_test:
  clean: ["the color"]
"#,
    );
    cmd(&dir)
        .args(["check", "--self-test"])
        .assert()
        .failure()
        .code(1);
}

#[test]
fn exec_check_protocol_v1() {
    let dir = TempDir::new().unwrap();
    seed(
        &dir,
        &format!("{REPLAY_CONFIG}\n[[checks]]\nexec = \"./check.sh\"\n",),
    );
    write(
        dir.path(),
        "locales/en-US.json",
        r#"{"a":"Hello","b":"Bye"}"#,
    );
    write(
        dir.path(),
        "locales/fr.json",
        r#"{"a":"Bonjour","b":"Au revoir"}"#,
    );
    let script = write(
        dir.path(),
        "check.sh",
        r#"#!/bin/sh
# v1 protocol: one request line, one findings line.
read req
echo "$req" | grep -q '"v":1' || { echo '{"v":1,"error":"parse:bad request"}'; exit 0; }
printf '{"v":1,"findings":[{"key":"a","message":"exec flagged it"}]}\n'
"#,
    );
    // make executable
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();

    let (report, ok) = report(&dir, &["check"]);
    assert!(!ok);
    let invalid = report["locales"]["fr"]["invalid"].as_array().unwrap();
    assert_eq!(invalid.len(), 1);
    assert_eq!(invalid[0]["key"], "a");
    assert!(
        invalid[0]["message"]
            .as_str()
            .unwrap()
            .contains("exec flagged")
    );
}

#[test]
fn exec_check_error_fails_closed() {
    let dir = TempDir::new().unwrap();
    seed(
        &dir,
        &format!("{REPLAY_CONFIG}\n[[checks]]\nexec = \"./check.sh\"\n"),
    );
    write(dir.path(), "locales/en-US.json", r#"{"a":"Hello"}"#);
    write(dir.path(), "locales/fr.json", r#"{"a":"Bonjour"}"#);
    let script = write(
        dir.path(),
        "check.sh",
        "#!/bin/sh\nread req\necho '{\"v\":1,\"error\":\"rate_limited:budget\"}'\n",
    );
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();

    let (report, ok) = report(&dir, &["check"]);
    assert!(!ok);
    assert!(
        report["errors"][0]
            .as_str()
            .unwrap()
            .contains("rate_limited")
    );
}

#[test]
fn judge_findings_below_threshold() {
    let dir = TempDir::new().unwrap();
    seed(
        &dir,
        &format!("{REPLAY_CONFIG}\n[[checks]]\nid = \"judge\"\n"),
    );
    write(
        dir.path(),
        "cassette.json",
        r#"{"fr":{},"fr.judge":{"bad":"0.4"}}"#,
    );
    write(
        dir.path(),
        "locales/en-US.json",
        r#"{"ok":"Hi","bad":"Hello"}"#,
    );
    write(
        dir.path(),
        "locales/fr.json",
        r#"{"ok":"Salut","bad":"Bonjour"}"#,
    );
    let (report, ok) = report(&dir, &["check"]);
    assert!(!ok);
    let invalid = report["locales"]["fr"]["invalid"].as_array().unwrap();
    assert_eq!(invalid.len(), 1);
    assert_eq!(invalid[0]["key"], "bad");
    assert_eq!(invalid[0]["check"], "judge");
    assert!(invalid[0]["message"].as_str().unwrap().contains("0.40"));
}

#[test]
fn unknown_check_id_fails_fast() {
    let dir = TempDir::new().unwrap();
    seed(
        &dir,
        &format!("{REPLAY_CONFIG}\n[[checks]]\nid = \"bogus\"\n"),
    );
    write(dir.path(), "locales/en-US.json", r#"{"a":"x"}"#);
    cmd(&dir)
        .arg("check")
        .assert()
        .failure()
        .stderr(predicates::str::contains("unknown check"));
}
