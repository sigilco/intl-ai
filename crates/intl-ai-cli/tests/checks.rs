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

#[cfg(unix)]
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

#[cfg(unix)]
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

#[test]
fn judge_threshold_override() {
    let dir = TempDir::new().unwrap();
    seed(
        &dir,
        &format!("{REPLAY_CONFIG}\n[[checks]]\nid = \"judge\"\nthreshold = 0.95\n"),
    );
    write(
        dir.path(),
        "cassette.json",
        r#"{"fr":{},"fr.judge":{"a":"0.9"}}"#,
    );
    write(dir.path(), "locales/en-US.json", r#"{"a":"Hello"}"#);
    write(dir.path(), "locales/fr.json", r#"{"a":"Bonjour"}"#);
    let (report, ok) = report(&dir, &["check"]);
    assert!(!ok);
    let invalid = report["locales"]["fr"]["invalid"].as_array().unwrap();
    assert_eq!(invalid.len(), 1);
    assert_eq!(invalid[0]["check"], "judge");
    assert!(
        invalid[0]["message"]
            .as_str()
            .unwrap()
            .contains("0.90 below 0.95")
    );
}

#[test]
fn quality_bands_fail_review_pass() {
    let dir = TempDir::new().unwrap();
    // Judge's own floor is lowered so the aggregate band decides alone.
    seed(
        &dir,
        &format!(
            "{REPLAY_CONFIG}\n[[checks]]\nid = \"judge\"\nthreshold = 0.0\n\n[quality]\nfail_below = 0.5\nreview_below = 0.9\n"
        ),
    );
    write(
        dir.path(),
        "cassette.json",
        r#"{"fr":{},"fr.judge":{"a":"0.7","b":"0.3"}}"#,
    );
    write(
        dir.path(),
        "locales/en-US.json",
        r#"{"a":"Hello","b":"Bye","c":"Welcome"}"#,
    );
    write(
        dir.path(),
        "locales/fr.json",
        r#"{"a":"Bonjour","b":"Au revoir","c":"Bienvenue"}"#,
    );
    let (report, ok) = report(&dir, &["check"]);
    assert!(!ok);
    let quality = &report["locales"]["fr"]["quality"];
    assert_eq!(quality["a"]["band"], "review");
    assert_eq!(quality["b"]["band"], "fail");
    assert_eq!(quality["c"]["band"], "pass");
    // Fail band lands as an `invalid` finding under check id `quality`.
    let invalid = report["locales"]["fr"]["invalid"].as_array().unwrap();
    assert_eq!(invalid.len(), 1);
    assert_eq!(invalid[0]["check"], "quality");
    assert_eq!(invalid[0]["key"], "b");
    // Review band surfaces under unreviewed.
    let unreviewed = report["locales"]["fr"]["unreviewed"].as_array().unwrap();
    assert!(unreviewed.contains(&Value::String("a".into())));
    assert!(!unreviewed.contains(&Value::String("b".into())));
}

#[test]
fn quality_weighted_aggregation() {
    let dir = TempDir::new().unwrap();
    // judge 0.9 weighted 3 vs icu failing weighted 1: (0.9*3 + 0*1)/4
    // = 0.675 -> review band. Without the icu weight it would pass.
    seed(
        &dir,
        &format!(
            "{REPLAY_CONFIG}\n[[checks]]\nid = \"judge\"\nthreshold = 0.0\nweight = 3.0\n[[checks]]\nid = \"icu\"\nweight = 1.0\n\n[quality]\nfail_below = 0.5\nreview_below = 0.8\n"
        ),
    );
    write(
        dir.path(),
        "cassette.json",
        r#"{"fr":{},"fr.judge":{"a":"0.9","b":"0.9"}}"#,
    );
    write(
        dir.path(),
        "locales/en-US.json",
        r#"{"a":"Hello","b":"Bye"}"#,
    );
    write(
        dir.path(),
        "locales/fr.json",
        r#"{"a":"Bonjour {oops","b":"Salut"}"#,
    );
    let (report, ok) = report(&dir, &["check"]);
    // icu finding on "a" is invalid, so `check` still fails — but the
    // quality record shows the weighted aggregate, not a raw failure.
    assert!(!ok);
    let quality = &report["locales"]["fr"]["quality"];
    let a = quality["a"]["score"].as_f64().unwrap();
    assert!((a - 0.675).abs() < 0.001, "score {a}");
    assert_eq!(quality["a"]["band"], "review");
    assert_eq!(quality["b"]["band"], "pass");
}

#[test]
fn threshold_rejected_on_non_judge() {
    let dir = TempDir::new().unwrap();
    seed(
        &dir,
        &format!("{REPLAY_CONFIG}\n[[checks]]\nid = \"icu\"\nthreshold = 0.5\n"),
    );
    write(dir.path(), "locales/en-US.json", r#"{"a":"x"}"#);
    cmd(&dir)
        .arg("check")
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "`threshold` applies only to id = \"judge\"",
        ));
}

#[test]
fn quality_band_ordering_validated() {
    let dir = TempDir::new().unwrap();
    seed(
        &dir,
        &format!("{REPLAY_CONFIG}\n[quality]\nfail_below = 0.9\nreview_below = 0.5\n"),
    );
    write(dir.path(), "locales/en-US.json", r#"{"a":"x"}"#);
    cmd(&dir)
        .arg("check")
        .assert()
        .failure()
        .stderr(predicates::str::contains("fail_below must be <"));
}
