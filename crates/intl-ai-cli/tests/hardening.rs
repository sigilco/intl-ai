//! W2b-A1 e2e: data-flow correctness and trust-boundary hardening.
//! Covers corrupt/conflicted shard handling, config cross-field rules,
//! the extends boundary, and human-edit unreviewed semantics.
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

#[test]
fn corrupt_shard_is_hard_error_exit_10() {
    let dir = TempDir::new().unwrap();
    seed(&dir, CONFIG);
    write(
        dir.path(),
        "locales/intl-ai.lock.d/fr.toml",
        "version = [oops\n",
    );
    // Both readers fail closed at 10, not a per-locale degrade.
    cmd(&dir).arg("check").assert().code(10);
    cmd(&dir).arg("fill").assert().code(10);
    cmd(&dir).args(["lockfile", "check"]).assert().code(10);
}

#[test]
fn unterminated_conflict_markers_are_hard_error() {
    let dir = TempDir::new().unwrap();
    seed(&dir, CONFIG);
    write(
        dir.path(),
        "locales/intl-ai.lock.d/fr.toml",
        "version = 1\n<<<<<<< ours\nversion = 1\n",
    );
    cmd(&dir).args(["lockfile", "merge"]).assert().code(10);
}

#[test]
fn config_rejects_bad_locale_ids() {
    for bad in ["../escape", "a/b", "..", ""] {
        let dir = TempDir::new().unwrap();
        seed(
            &dir,
            &CONFIG.replace("targets = [\"fr\"]", &format!("targets = [\"{bad}\"]")),
        );
        let out = cmd(&dir).arg("check").assert().code(10);
        let stderr = String::from_utf8_lossy(&out.get_output().stderr);
        assert!(stderr.contains("locale id"), "{bad:?}: {stderr}");
    }
}

#[test]
fn config_rejects_agent_and_command_together() {
    let dir = TempDir::new().unwrap();
    seed(
        &dir,
        r#"locale_dir = "locales"
source = "en"
targets = ["fr"]

[provider]
kind = "command"
agent = "claude-code"
command = "sh"
"#,
    );
    cmd(&dir).arg("check").assert().code(10);
}

#[test]
fn config_rejects_newer_version() {
    let dir = TempDir::new().unwrap();
    seed(&dir, &format!("version = 2\n{CONFIG}"));
    cmd(&dir).arg("check").assert().code(10);
}

#[test]
fn config_rejects_source_in_targets() {
    let dir = TempDir::new().unwrap();
    seed(&dir, &CONFIG.replace("[\"fr\"]", "[\"en\", \"fr\"]"));
    cmd(&dir).arg("check").assert().code(10);
}

#[test]
fn extends_diamond_is_a_dag_not_a_cycle() {
    let dir = TempDir::new().unwrap();
    write(
        dir.path(),
        "base.toml",
        "glossary = { \"PR\" = \"pull request\" }\n",
    );
    write(dir.path(), "a.toml", "extends = \"./base.toml\"\n");
    write(
        dir.path(),
        "b.toml",
        "extends = \"./base.toml\"\nbatch_size = 2\n",
    );
    seed(
        &dir,
        &format!("extends = [\"./a.toml\", \"./b.toml\"]\n{CONFIG}"),
    );
    cmd(&dir).args(["config", "validate"]).assert().success();
}

#[test]
fn extends_true_cycle_is_rejected() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "a.toml", "extends = \"./b.toml\"\n");
    write(dir.path(), "b.toml", "extends = \"./a.toml\"\n");
    seed(&dir, &format!("extends = \"./a.toml\"\n{CONFIG}"));
    let out = cmd(&dir).arg("check").assert().code(10);
    let stderr = String::from_utf8_lossy(&out.get_output().stderr);
    assert!(stderr.contains("cycle"), "{stderr}");
}

#[test]
fn extends_boundary_refuses_code_and_secrets() {
    // ${env:} interpolation in a base file: refused.
    let dir = TempDir::new().unwrap();
    write(
        dir.path(),
        "base.toml",
        "locale_dir = \"${env:I18N_DIR:-locales}\"\n",
    );
    seed(&dir, &format!("extends = \"./base.toml\"\n{CONFIG}"));
    cmd(&dir).arg("check").assert().code(10);

    // provider.command in a base file: refused.
    let dir = TempDir::new().unwrap();
    write(
        dir.path(),
        "base.toml",
        "[provider]\nkind = \"command\"\ncommand = \"curl\"\n",
    );
    seed(&dir, &format!("extends = \"./base.toml\"\n{CONFIG}"));
    cmd(&dir).arg("check").assert().code(10);

    // provider.base_url redirect in a base file: refused (api_key exfil).
    let dir = TempDir::new().unwrap();
    write(
        dir.path(),
        "base.toml",
        "[provider]\nkind = \"http\"\nbase_url = \"https://evil.example\"\n",
    );
    seed(&dir, &format!("extends = \"./base.toml\"\n{CONFIG}"));
    cmd(&dir).arg("check").assert().code(10);

    // exec check in a base file: refused.
    let dir = TempDir::new().unwrap();
    write(
        dir.path(),
        "base.toml",
        "[[checks]]\nexec = \"sh\"\nargs = [\"-c\", \"id\"]\n",
    );
    seed(&dir, &format!("extends = \"./base.toml\"\n{CONFIG}"));
    cmd(&dir).arg("check").assert().code(10);
}

#[test]
fn human_edit_marks_entry_unreviewed_again() {
    let dir = TempDir::new().unwrap();
    seed(&dir, CONFIG);
    write(
        dir.path(),
        "locales/fr.json",
        r#"{"nav":{"home":"Accueil humain"},"greeting":"Hello"}"#,
    );
    cmd(&dir).arg("fill").assert().success();

    // nav.home was adopted human + unreviewed; approve it.
    cmd(&dir)
        .args(["review", "nav.home", "--locale", "fr"])
        .assert()
        .success();
    assert!(load_shard(&dir, "fr").entries["nav.home"].reviewed);

    // The human edits it again -> drift resets the approval.
    write(
        dir.path(),
        "locales/fr.json",
        r#"{"nav":{"home":"Retour à l'accueil","settings":"Paramètres"},"greeting":"Bonjour"}"#,
    );
    let out = cmd(&dir)
        .args([
            "check",
            "--origin",
            "human",
            "--fail-on",
            "unreviewed",
            "--format",
            "json",
        ])
        .assert()
        .failure()
        .get_output()
        .clone();
    let report: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(
        !report["locales"]["fr"]["unreviewed"]
            .as_array()
            .unwrap()
            .is_empty(),
        "{report}"
    );
}

#[test]
fn mark_origin_ai_rebaselines_to_current_text() {
    let dir = TempDir::new().unwrap();
    seed(&dir, CONFIG);
    cmd(&dir).arg("fill").assert().success();

    // Human adopts greeting (positional drift) -> next fill flips it human.
    write(
        dir.path(),
        "locales/fr.json",
        r#"{"nav":{"home":"Accueil","settings":"Paramètres"},"greeting":"Salut à la main"}"#,
    );
    cmd(&dir).arg("fill").assert().success();
    let shard = load_shard(&dir, "fr");
    assert_eq!(
        shard.entries["greeting"].origin,
        intl_ai_core::lockfile::Origin::Human
    );

    // Explicitly return it to AI stewardship: the snapshot must become the
    // on-disk text, otherwise check reports modified forever.
    cmd(&dir)
        .args(["mark", "greeting", "--locale", "fr", "--origin", "ai"])
        .assert()
        .success();
    let shard = load_shard(&dir, "fr");
    let entry = &shard.entries["greeting"];
    assert_eq!(entry.origin, intl_ai_core::lockfile::Origin::Ai);
    assert_eq!(entry.value, "Salut à la main");

    let out = cmd(&dir)
        .args(["check", "--format", "json"])
        .assert()
        .get_output()
        .clone();
    let report: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(
        !report["locales"]["fr"]["modified"]
            .as_array()
            .unwrap()
            .contains(&Value::String("greeting".into())),
        "{report}"
    );
}
