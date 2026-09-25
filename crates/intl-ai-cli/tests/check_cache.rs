//! Incremental check cache (W2b part B): unchanged (key, source, target)
//! inputs replay findings marked `cached` without re-running the check;
//! any change re-validates. The exec counter file is the ground truth for
//! "did the check actually run" — per-key checks assert the `cached`
//! flag on replayed findings instead.

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

fn report(dir: &TempDir, args: &[&str]) -> Value {
    let out = cmd(dir)
        .args(args)
        .args(["--format", "json"])
        .assert()
        .get_output()
        .clone();
    serde_json::from_slice(&out.stdout).unwrap()
}

fn cache_file(dir: &TempDir) -> Value {
    serde_json::from_str(&fs::read_to_string(dir.path().join(".intl-ai/check-cache.json")).unwrap())
        .unwrap()
}

#[test]
fn second_run_replays_findings_marked_cached() {
    let dir = TempDir::new().unwrap();
    seed(
        &dir,
        &format!("{REPLAY_CONFIG}\n[[checks]]\nid = \"dialect:en-GB\"\n"),
    );
    write(dir.path(), "locales/en-US.json", r#"{"a":"the color"}"#);
    write(dir.path(), "locales/fr.json", r#"{"a":"the color"}"#);

    let first = report(&dir, &["check"]);
    let invalid = first["locales"]["fr"]["invalid"].as_array().unwrap();
    assert_eq!(invalid.len(), 1);
    assert!(invalid[0].get("cached").is_none());

    let second = report(&dir, &["check"]);
    let invalid = second["locales"]["fr"]["invalid"].as_array().unwrap();
    assert_eq!(invalid.len(), 1);
    assert_eq!(invalid[0]["check"], "dialect:en-GB");
    assert_eq!(invalid[0]["cached"], true);
}

#[test]
fn changed_key_reruns_while_unchanged_replays() {
    let dir = TempDir::new().unwrap();
    seed(
        &dir,
        &format!("{REPLAY_CONFIG}\n[[checks]]\nid = \"dialect:en-GB\"\n"),
    );
    write(
        dir.path(),
        "locales/en-US.json",
        r#"{"a":"the color","b":"the flavor"}"#,
    );
    write(
        dir.path(),
        "locales/fr.json",
        r#"{"a":"the color","b":"the flavor"}"#,
    );
    report(&dir, &["check"]);

    write(
        dir.path(),
        "locales/fr.json",
        r#"{"a":"the color","b":"the taste"}"#,
    );
    let second = report(&dir, &["check"]);
    let invalid = second["locales"]["fr"]["invalid"].as_array().unwrap();
    let a = invalid.iter().find(|f| f["key"] == "a").unwrap();
    assert_eq!(a["cached"], true);
    // "the taste" is clean under en-GB rules; its re-run produces no
    // finding, so only the cache file proves the re-validation.
    let cache = cache_file(&dir);
    let fr = &cache["entries"]["fr"];
    assert!(fr["b"]["dialect:en-GB"].is_object());
    assert!(fr["a"]["dialect:en-GB"].is_object());
}

#[test]
fn exec_whole_batch_invalidates_on_any_key_change() {
    let dir = TempDir::new().unwrap();
    seed(
        &dir,
        &format!("{REPLAY_CONFIG}\n[[checks]]\nexec = \"./check.sh\"\n"),
    );
    write(dir.path(), "locales/en-US.json", r#"{"a":"Hi","b":"Bye"}"#);
    write(
        dir.path(),
        "locales/fr.json",
        r#"{"a":"Bonjour","b":"Au revoir"}"#,
    );
    let script = write(
        dir.path(),
        "check.sh",
        r#"#!/bin/sh
read req
n=$(cat runs.txt 2>/dev/null || echo 0)
echo $((n + 1)) > runs.txt
printf '{"v":1,"findings":[{"key":"a","message":"exec flagged"}]}\n'
"#,
    );
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();

    let first = report(&dir, &["check"]);
    assert_eq!(
        fs::read_to_string(dir.path().join("runs.txt"))
            .unwrap()
            .trim(),
        "1"
    );
    assert!(first["locales"]["fr"]["invalid"][0].get("cached").is_none());

    let second = report(&dir, &["check"]);
    assert_eq!(
        fs::read_to_string(dir.path().join("runs.txt"))
            .unwrap()
            .trim(),
        "1",
        "identical input must replay without re-running the child"
    );
    assert_eq!(second["locales"]["fr"]["invalid"][0]["cached"], true);

    write(
        dir.path(),
        "locales/fr.json",
        r#"{"a":"Bonjour","b":"Salut"}"#,
    );
    let third = report(&dir, &["check"]);
    assert_eq!(
        fs::read_to_string(dir.path().join("runs.txt"))
            .unwrap()
            .trim(),
        "2",
        "one changed key re-runs the whole batch"
    );
    assert!(third["locales"]["fr"]["invalid"][0].get("cached").is_none());
}

#[test]
fn config_cache_false_and_no_cache_flag_bypass() {
    for extra in ["[check]\ncache = false\n", ""] {
        let dir = TempDir::new().unwrap();
        let config = format!("{REPLAY_CONFIG}\n[[checks]]\nexec = \"./check.sh\"\n{extra}");
        seed(&dir, &config);
        write(dir.path(), "locales/en-US.json", r#"{"a":"Hi"}"#);
        write(dir.path(), "locales/fr.json", r#"{"a":"Bonjour"}"#);
        let script = write(
            dir.path(),
            "check.sh",
            "#!/bin/sh\nread req\nn=$(cat runs.txt 2>/dev/null || echo 0)\necho $((n + 1)) > runs.txt\nprintf '{\"v\":1,\"findings\":[]}\\n'\n",
        );
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();

        let mut args = vec!["check"];
        if extra.is_empty() {
            args.push("--no-cache");
        }
        report(&dir, &args);
        report(&dir, &args);
        assert_eq!(
            fs::read_to_string(dir.path().join("runs.txt"))
                .unwrap()
                .trim(),
            "2",
            "disabled cache must re-run every time (case: {extra:?})"
        );
        assert!(!dir.path().join(".intl-ai/check-cache.json").exists());
    }
}

#[test]
fn cache_ctx_change_reruns_check() {
    let dir = TempDir::new().unwrap();
    seed(
        &dir,
        &format!("{REPLAY_CONFIG}\n[[checks]]\nid = \"dialect:en-GB\"\n"),
    );
    write(dir.path(), "locales/en-US.json", r#"{"a":"the color"}"#);
    write(dir.path(), "locales/fr.json", r#"{"a":"the color"}"#);
    let first = report(&dir, &["check"]);
    assert_eq!(
        first["locales"]["fr"]["invalid"].as_array().unwrap().len(),
        1
    );

    // Same corpus, different check identity: en-US flags "colour", so
    // "the color" is now clean — but only if the check re-ran.
    write(
        dir.path(),
        "intl-ai.toml",
        &format!("{REPLAY_CONFIG}\n[[checks]]\nid = \"dialect:en-US\"\n"),
    );
    let second = report(&dir, &["check"]);
    assert!(
        second["locales"]["fr"]["invalid"]
            .as_array()
            .unwrap()
            .is_empty(),
        "stale en-GB finding must not replay under the en-US id"
    );
    // The re-run also wrote a clean (empty) entry for the new id.
    let cache = cache_file(&dir);
    assert!(cache["entries"]["fr"]["a"]["dialect:en-US"].is_object());
}

#[test]
fn prune_evicts_deleted_keys_only_on_unscoped_runs() {
    let dir = TempDir::new().unwrap();
    seed(
        &dir,
        &format!("{REPLAY_CONFIG}\n[[checks]]\nid = \"dialect:en-GB\"\n"),
    );
    write(
        dir.path(),
        "locales/en-US.json",
        r#"{"a":"the color","b":"the flavor"}"#,
    );
    write(
        dir.path(),
        "locales/fr.json",
        r#"{"a":"the color","b":"the flavor"}"#,
    );
    report(&dir, &["check"]);
    assert!(cache_file(&dir)["entries"]["fr"]["b"].is_object());

    // A scoped --keys run must not evict out-of-scope cache.
    write(dir.path(), "locales/fr.json", r#"{"a":"the color"}"#);
    report(&dir, &["check", "--keys", "a"]);
    assert!(cache_file(&dir)["entries"]["fr"]["b"].is_object());

    // The next unscoped run prunes it.
    report(&dir, &["check"]);
    assert!(cache_file(&dir)["entries"]["fr"]["b"].is_null());
}
