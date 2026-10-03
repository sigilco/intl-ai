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
source = "en"
targets = ["fr"]

[provider]
kind = "replay"
file = "cassette.json"
"#;

fn yaml_config() -> String {
    REPLAY_CONFIG.replacen(
        "targets = [\"fr\"]",
        "targets = [\"fr\"]\nformat = \"yaml\"",
        1,
    )
}

fn seed(dir: &TempDir, config: &str) {
    write(dir.path(), "intl-ai.toml", config);
    write(
        dir.path(),
        "cassette.json",
        r#"{"fr":{"Hello":"Bonjour","Home":"Accueil"}}"#,
    );
    write(
        dir.path(),
        "locales/en.json",
        r#"{"greeting":"Hello","nav":{"home":"Home"}}"#,
    );
}

#[test]
fn fill_writes_yaml_when_configured() {
    let dir = TempDir::new().unwrap();
    seed(&dir, &yaml_config());
    cmd(&dir).arg("fill").assert().success();

    // New files are minted with the configured extension.
    assert!(!dir.path().join("locales/fr.json").exists());
    let yaml = fs::read_to_string(dir.path().join("locales/fr.yaml")).unwrap();
    assert!(yaml.contains("greeting: Bonjour"));
    assert!(yaml.contains("home: Accueil"));
    // And the lockfile still records the translations.
    let lock = fs::read_to_string(dir.path().join("locales/intl-ai.lock.d/fr.toml")).unwrap();
    assert!(lock.contains("Bonjour"));
}

#[test]
fn fill_preserves_existing_yaml_extension() {
    let dir = TempDir::new().unwrap();
    // Default format (json), but the target file already exists as YAML:
    // its own extension wins over the preference.
    seed(&dir, REPLAY_CONFIG);
    write(dir.path(), "locales/fr.yaml", "greeting: Salut!\n");
    cmd(&dir).arg("fill").assert().success();

    assert!(!dir.path().join("locales/fr.json").exists());
    let fr: Value =
        serde_yaml_ng::from_str(&fs::read_to_string(dir.path().join("locales/fr.yaml")).unwrap())
            .unwrap();
    // Existing (human) value kept, missing key filled in the same file.
    assert_eq!(fr["greeting"], "Salut!");
    assert_eq!(fr["nav"]["home"], "Accueil");
}

#[test]
fn stat_cache_written_after_fill() {
    let dir = TempDir::new().unwrap();
    seed(&dir, REPLAY_CONFIG);
    cmd(&dir).arg("fill").assert().success();
    let cache: Value =
        serde_json::from_str(&fs::read_to_string(dir.path().join(".intl-ai/cache.json")).unwrap())
            .unwrap();
    // Source file hashed and cached under its absolute path.
    assert!(
        cache["files"]
            .as_object()
            .unwrap()
            .keys()
            .any(|k| k.ends_with("en.json"))
    );
}

#[test]
fn lockfile_merge_resolves_conflicts() {
    let dir = TempDir::new().unwrap();
    seed(&dir, REPLAY_CONFIG);
    write(
        dir.path(),
        "locales/intl-ai.lock.d/fr.toml",
        r#"version = 1

[entries."solo"]
value = "x"
source_hash = "h"
origin = "ai"
<<<<<<< ours
[entries."shared"]
value = "ours"
source_hash = "h"
origin = "ai"
updated_at = "2026-09-20T00:00:00Z"
=======
[entries."shared"]
value = "theirs"
source_hash = "h"
origin = "human"
updated_at = "2026-09-25T00:00:00Z"
>>>>>>> theirs
"#,
    );
    cmd(&dir)
        .args(["lockfile", "merge"])
        .assert()
        .success()
        .stdout(predicates::str::contains("fr: merged"));

    let merged = fs::read_to_string(dir.path().join("locales/intl-ai.lock.d/fr.toml")).unwrap();
    assert!(!merged.contains("<<<<<<<"));
    let shard: toml::Value = toml::from_str(&merged).unwrap();
    // Both sides' keys survive; the human-owned entry wins the overlap.
    assert_eq!(shard["entries"]["solo"]["value"].as_str(), Some("x"));
    assert_eq!(shard["entries"]["shared"]["value"].as_str(), Some("theirs"));
    assert_eq!(shard["entries"]["shared"]["origin"].as_str(), Some("human"));
    // Idempotent: second run reports nothing left to merge.
    cmd(&dir)
        .args(["lockfile", "merge"])
        .assert()
        .success()
        .stdout(predicates::str::contains("no conflicted shards"));
}

#[cfg(unix)]
const STRINGS_XML_PLUGIN: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/formats/strings-xml/strings_xml.py"
);

#[cfg(unix)]
fn xml_config() -> String {
    REPLAY_CONFIG.replacen(
        "targets = [\"fr\"]",
        &format!(
            "targets = [\"fr\"]\nformat = \"xml\"\n\n[[formats]]\nname = \"xml\"\nexec = \"python3\"\nargs = [\"{STRINGS_XML_PLUGIN}\"]\nextension = \"xml\""
        ),
        1,
    )
}

#[cfg(unix)]
fn python3_available() -> bool {
    std::process::Command::new("python3")
        .arg("--version")
        .output()
        .is_ok()
}

#[cfg(unix)]
#[test]
fn fill_mints_and_reads_xml_via_exec_format() {
    if !python3_available() {
        return;
    }
    let dir = TempDir::new().unwrap();
    seed(&dir, &xml_config());
    // An existing xml file round-trips through the plugin's read op.
    write(
        dir.path(),
        "locales/fr.xml",
        r#"<?xml version="1.0" encoding="utf-8"?>
<resources>
    <string name="greeting">Salut!</string>
</resources>
"#,
    );
    cmd(&dir).arg("fill").assert().success();

    let xml = fs::read_to_string(dir.path().join("locales/fr.xml")).unwrap();
    // Existing value kept; the missing key was minted in the same file
    // (nested keys flatten to dotted names per the plugin's contract).
    assert!(xml.contains(r#"<string name="greeting">Salut!</string>"#));
    assert!(xml.contains(r#"<string name="nav.home">Accueil</string>"#));
    // `check` reads the xml target through the plugin too.
    cmd(&dir).arg("check").assert().success();
}

#[cfg(unix)]
#[test]
fn fill_writes_xml_when_configured() {
    if !python3_available() {
        return;
    }
    let dir = TempDir::new().unwrap();
    seed(&dir, &xml_config());
    cmd(&dir).arg("fill").assert().success();

    // New files are minted with the exec format's canonical extension.
    assert!(!dir.path().join("locales/fr.json").exists());
    let xml = fs::read_to_string(dir.path().join("locales/fr.xml")).unwrap();
    assert!(xml.contains(r#"<string name="greeting">Bonjour</string>"#));
    assert!(xml.contains(r#"<string name="nav.home">Accueil</string>"#));
}

#[test]
fn unknown_format_name_fails_closed() {
    let dir = TempDir::new().unwrap();
    seed(
        &dir,
        &REPLAY_CONFIG.replacen(
            "targets = [\"fr\"]",
            "targets = [\"fr\"]\nformat = \"nope\"",
            1,
        ),
    );
    cmd(&dir)
        .arg("check")
        .assert()
        .code(10)
        .stderr(predicates::str::contains("unknown format 'nope'"));
}

#[test]
fn format_name_colliding_with_builtin_fails() {
    let dir = TempDir::new().unwrap();
    seed(
        &dir,
        &format!(
            "{REPLAY_CONFIG}\n[[formats]]\nname = \"json\"\nexec = \"cat\"\nextension = \"jsonx\"\n"
        ),
    );
    cmd(&dir)
        .arg("check")
        .assert()
        .code(10)
        .stderr(predicates::str::contains("conflicts with a builtin"));
}

#[test]
fn config_schema_subcommand_prints_schema() {
    let dir = TempDir::new().unwrap();
    // No config file needed: the schema describes the contract itself.
    let out = cmd(&dir)
        .args(["config", "schema"])
        .assert()
        .success()
        .get_output()
        .clone();
    let schema: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(schema["type"], "object");
    assert!(schema["properties"]["provider"].is_object());
    assert!(schema["$defs"]["ProviderConfig"].is_object());
}
