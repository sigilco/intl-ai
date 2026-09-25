//! W2b-A2 e2e: report-contract and misc-robustness coverage.
//! Absent tombstones, key scoping, regenerate guards, --fail-on none,
//! no-cache, unchanged-write skipping, status, init scaffolding,
//! shadowed files, ancestor discovery, dropped leaves, GC, rotation.
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
    c.current_dir(dir.path())
        .env("INTL_AI_NOW", "2026-09-25T00:00:00Z");
    c
}

fn seed(dir: &TempDir, config: &str) {
    write(dir.path(), "intl-ai.toml", config);
    write(
        dir.path(),
        "locales/en.json",
        r#"{"nav":{"home":"Home","settings":"Settings"},"greeting":"Hello"}"#,
    );
    write(
        dir.path(),
        "cassette.json",
        r#"{"fr":{"Home":"Accueil","Settings":"Paramètres","Hello":"Bonjour"},
"de":{"Home":"Startseite","Settings":"Einstellungen","Hello":"Hallo"}}"#,
    );
}

const CONFIG: &str = r#"locale_dir = "locales"
source = "en"
targets = ["fr"]

[provider]
kind = "replay"
file = "cassette.json"
"#;

fn load_shard(dir: &TempDir, locale: &str) -> intl_ai_core::lockfile::Shard {
    intl_ai_core::lockfile::load_shard(&dir.path().join("locales"), locale).unwrap()
}

fn fill_ok(dir: &TempDir) {
    cmd(dir).arg("fill").assert().success();
}

#[test]
fn absent_tombstone_suppresses_missing_and_fill() {
    let dir = TempDir::new().unwrap();
    seed(&dir, CONFIG);
    fill_ok(&dir);

    // Delete greeting from the target: it is a missing key by default.
    write(
        dir.path(),
        "locales/fr.json",
        r#"{"nav":{"home":"Accueil","settings":"Paramètres"}}"#,
    );
    let out = cmd(&dir)
        .args(["check", "--format", "json"])
        .assert()
        .failure()
        .get_output()
        .clone();
    let report: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(
        report["locales"]["fr"]["missing"]
            .as_array()
            .unwrap()
            .contains(&Value::String("greeting".into())),
        "{report}"
    );

    // Tombstone it: check no longer reports it, fill will not refill it.
    cmd(&dir)
        .args(["mark", "greeting", "--locale", "fr", "--absent"])
        .assert()
        .success();
    cmd(&dir).arg("check").assert().success();
    fill_ok(&dir);
    let fr: Value =
        serde_json::from_str(&fs::read_to_string(dir.path().join("locales/fr.json")).unwrap())
            .unwrap();
    assert!(fr.get("greeting").is_none(), "{fr}");

    // --present re-arms it: missing again, and the next fill restores it.
    cmd(&dir)
        .args(["mark", "greeting", "--locale", "fr", "--present"])
        .assert()
        .success();
    let out = cmd(&dir)
        .args(["check", "--format", "json"])
        .assert()
        .failure()
        .get_output()
        .clone();
    let report: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(
        report["locales"]["fr"]["missing"]
            .as_array()
            .unwrap()
            .contains(&Value::String("greeting".into())),
        "{report}"
    );
    fill_ok(&dir);
    let fr: Value =
        serde_json::from_str(&fs::read_to_string(dir.path().join("locales/fr.json")).unwrap())
            .unwrap();
    assert_eq!(fr["greeting"], "Bonjour");
    assert!(!load_shard(&dir, "fr").entries["greeting"].absent);
}

#[test]
fn check_keys_scopes_findings() {
    let dir = TempDir::new().unwrap();
    seed(&dir, CONFIG);
    // No target file: every source key is missing.
    let out = cmd(&dir)
        .args(["check", "--keys", "greeting", "--format", "json"])
        .assert()
        .failure()
        .get_output()
        .clone();
    let report: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(
        report["locales"]["fr"]["missing"],
        Value::Array(vec![Value::String("greeting".into())]),
        "{report}"
    );
}

#[test]
fn fill_regenerate_requires_scope_or_yes() {
    let dir = TempDir::new().unwrap();
    seed(&dir, CONFIG);
    fill_ok(&dir);

    let out = cmd(&dir)
        .args(["fill", "--regenerate"])
        .assert()
        .failure()
        .get_output()
        .clone();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("--yes"), "{stderr}");

    // A key scope satisfies the guard without --yes.
    cmd(&dir)
        .args(["fill", "--regenerate", "--keys", "nav.*"])
        .assert()
        .success();
    // --yes satisfies it alone.
    cmd(&dir)
        .args(["fill", "--regenerate", "--yes"])
        .assert()
        .success();
}

#[test]
fn stale_and_regenerate_conflict() {
    let dir = TempDir::new().unwrap();
    seed(&dir, CONFIG);
    // clap conflicts_with -> usage error, exit code 2.
    cmd(&dir)
        .args(["fill", "--stale", "--regenerate"])
        .assert()
        .code(2);
}

#[test]
fn fail_on_none_never_gates() {
    let dir = TempDir::new().unwrap();
    seed(&dir, CONFIG);
    // Missing keys exist but the gate is explicitly cleared: exit 0.
    cmd(&dir)
        .args(["check", "--fail-on", "none"])
        .assert()
        .success();
}

#[test]
fn no_cache_flag_is_accepted() {
    let dir = TempDir::new().unwrap();
    seed(&dir, CONFIG);
    cmd(&dir).args(["fill", "--no-cache"]).assert().success();
    cmd(&dir).args(["check", "--no-cache"]).assert().success();
}

#[test]
fn unchanged_locale_file_keeps_its_mtime() {
    let dir = TempDir::new().unwrap();
    seed(&dir, CONFIG);
    fill_ok(&dir);

    // Stamp the file old, then refill: nothing to translate, nothing to
    // rewrite — mtime must survive.
    let fr = dir.path().join("locales/fr.json");
    std::process::Command::new("touch")
        .args(["-d", "@1000000000"])
        .arg(&fr)
        .status()
        .unwrap();
    let before = fs::metadata(&fr).unwrap().modified().unwrap();
    fill_ok(&dir);
    let after = fs::metadata(&fr).unwrap().modified().unwrap();
    assert_eq!(before, after);
}

#[test]
fn status_reports_inventory() {
    let dir = TempDir::new().unwrap();
    seed(&dir, CONFIG);
    fill_ok(&dir);

    let out = cmd(&dir)
        .arg("status")
        .assert()
        .success()
        .get_output()
        .clone();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("fr:"), "{stdout}");
    assert!(stdout.contains("3 entries"), "{stdout}");

    let out = cmd(&dir)
        .args(["status", "--format", "json"])
        .assert()
        .success()
        .get_output()
        .clone();
    let report: Value = serde_json::from_slice(&out.stdout).unwrap();
    let fr = &report["locales"][0];
    assert_eq!(fr["locale"], "fr");
    assert_eq!(fr["entries"], 3);
    assert_eq!(fr["ai"], 3);
    assert_eq!(fr["missing"], 0);
}

#[test]
fn shadowed_sibling_warns() {
    let dir = TempDir::new().unwrap();
    seed(&dir, CONFIG);
    // fr.json wins on read; a shadowed fr.yaml sibling must warn.
    write(dir.path(), "locales/fr.json", r#"{"greeting":"Bonjour"}"#);
    write(dir.path(), "locales/fr.yaml", "greeting: Salut\n");
    let out = cmd(&dir).arg("check").assert().get_output().clone();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("fr.yaml"), "{stderr}");
    assert!(stderr.contains("multiple locale files"), "{stderr}");
}

#[test]
fn discover_walks_ancestors_to_config() {
    let dir = TempDir::new().unwrap();
    seed(&dir, CONFIG);
    fs::create_dir_all(dir.path().join("nested/deep")).unwrap();
    fill_ok(&dir);
    let mut c = Command::cargo_bin("intl-ai").unwrap();
    c.current_dir(dir.path().join("nested/deep"))
        .env("INTL_AI_NOW", "2026-09-25T00:00:00Z");
    // Finds intl-ai.toml at the project root, two directories up.
    c.arg("check").assert().success();
}

#[test]
fn init_scaffolds_a_working_project() {
    let dir = TempDir::new().unwrap();
    cmd(&dir).arg("init").assert().success();
    assert!(dir.path().join("intl-ai.toml").exists());
    assert!(dir.path().join("cassette.json").exists());
    // First run works: empty corpus, exit 0, not a hard error.
    cmd(&dir).arg("check").assert().success();
    cmd(&dir).arg("fill").assert().success();
    // Re-init refuses because a config already exists.
    cmd(&dir).arg("init").assert().failure();
}

#[test]
fn dropped_leaves_warn() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "intl-ai.toml", CONFIG);
    write(
        dir.path(),
        "locales/en.json",
        // "empty" is a leafless object; "a.b" collides with object a.
        r#"{"empty":{},"a":{"b":"x"},"a.b":"y","greeting":"Hello"}"#,
    );
    write(dir.path(), "cassette.json", r#"{"fr":{"Hello":"Bonjour"}}"#);
    let out = cmd(&dir).arg("fill").assert().get_output().clone();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("dropped"), "{stderr}");
}

#[test]
fn gc_prunes_shard_entries_for_deleted_source_keys() {
    let dir = TempDir::new().unwrap();
    seed(&dir, CONFIG);
    fill_ok(&dir);
    assert!(load_shard(&dir, "fr").entries.contains_key("greeting"));

    // Remove greeting from both source and target: the dead shard entry
    // is pruned on the next fill.
    write(
        dir.path(),
        "locales/en.json",
        r#"{"nav":{"home":"Home","settings":"Settings"}}"#,
    );
    write(
        dir.path(),
        "locales/fr.json",
        r#"{"nav":{"home":"Accueil","settings":"Paramètres"}}"#,
    );
    fill_ok(&dir);
    assert!(!load_shard(&dir, "fr").entries.contains_key("greeting"));
}

#[test]
fn report_files_rotate_at_ten() {
    let dir = TempDir::new().unwrap();
    seed(&dir, CONFIG);
    // Pre-seed 12 old report files; a failing fill adds one and GCs.
    for i in 0..12 {
        write(
            dir.path(),
            &format!(".intl-ai/report-2026-01-{i:02}T00-00-00.json"),
            "{}",
        );
    }
    write(dir.path(), "cassette.json", "{}");
    cmd(&dir).arg("fill").assert().failure();
    let reports = fs::read_dir(dir.path().join(".intl-ai"))
        .unwrap()
        .filter(|e| {
            e.as_ref()
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with("report-")
        })
        .count();
    assert!(reports <= 10, "{reports} report files remain");
}

#[test]
fn json_errors_use_the_envelope() {
    let dir = TempDir::new().unwrap();
    // A broken config on a --format json call: structured error on stdout.
    write(
        dir.path(),
        "intl-ai.toml",
        "version = 2\nlocale_dir = \"locales\"\n",
    );
    let out = cmd(&dir)
        .args(["check", "--format", "json"])
        .assert()
        .code(10)
        .get_output()
        .clone();
    let err: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(err["error"]["message"].is_string(), "{err}");
    assert_eq!(err["error"]["code"], 10, "{err}");
}

#[test]
fn unreviewed_lists_key_names_in_json() {
    let dir = TempDir::new().unwrap();
    seed(&dir, CONFIG);
    fill_ok(&dir);
    let out = cmd(&dir)
        .args(["check", "--format", "json"])
        .assert()
        .get_output()
        .clone();
    let report: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(
        report["locales"]["fr"]["unreviewed"]
            .as_array()
            .unwrap()
            .contains(&Value::String("greeting".into())),
        "{report}"
    );
}
