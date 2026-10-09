//! Deterministic end-to-end test for `intl-ai-docs` against the replay
//! transport: fill `es` + `pt` trees from a source doc tree, assert
//! skeleton preservation, shard provenance, and check behavior.

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

fn fixture() -> TempDir {
    let dir = TempDir::new().unwrap();
    write(
        dir.path(),
        "intl-ai-docs.toml",
        r#"experimental = true
source_dir = "docs"
source = "en"
targets = ["es", "pt"]

[provider]
kind = "replay"
file = "cassette.json"
"#,
    );
    write(
        dir.path(),
        "docs/index.md",
        r#"---
title: Home
description: The intl-ai home page.
---

# Welcome to intl-ai

intl-ai translates your **locale files** at build time.

::: warning
Read the docs first.
:::

```sh
intl-ai fill --locale es
```

See the [guide](/guide/getting-started/) for details.
"#,
    );
    write(
        dir.path(),
        "docs/guide/setup.md",
        r#"---
title: Setup
---

# Setup

Install the binary with your package manager.

- first step
- second step
"#,
    );
    write(
        dir.path(),
        "cassette.json",
        r#"{
  "es": {
    "Home": "Inicio",
    "The intl-ai home page.": "La página de inicio de intl-ai.",
    "Welcome to intl-ai": "Bienvenido a intl-ai",
    "intl-ai translates your **locale files** at build time.": "intl-ai traduce tus **archivos de idioma** en tiempo de compilación.",
    "Read the docs first.": "Lee la documentación primero.",
    "See the [guide](/guide/getting-started/) for details.": "Consulta la [guía](/guide/getting-started/) para más detalles.",
    "Setup": "Configuración",
    "Install the binary with your package manager.": "Instala el binario con tu gestor de paquetes.",
    "first step": "primer paso",
    "second step": "segundo paso"
  },
  "pt": {
    "Home": "Início",
    "The intl-ai home page.": "A página inicial do intl-ai.",
    "Welcome to intl-ai": "Bem-vindo ao intl-ai",
    "intl-ai translates your **locale files** at build time.": "O intl-ai traduz os seus **ficheiros de idioma** em tempo de compilação.",
    "Read the docs first.": "Leia a documentação primeiro.",
    "See the [guide](/guide/getting-started/) for details.": "Veja o [guia](/guide/getting-started/) para mais detalhes.",
    "Setup": "Configuração",
    "Install the binary with your package manager.": "Instale o binário com o seu gestor de pacotes.",
    "first step": "primeiro passo",
    "second step": "segundo passo"
  }
}
"#,
    );
    dir
}

fn cmd(dir: &TempDir) -> Command {
    let mut c = Command::cargo_bin("intl-ai-docs").unwrap();
    c.current_dir(dir.path())
        .env("INTL_AI_NOW", "2026-10-09T00:00:00Z");
    c
}

#[test]
fn experimental_gate_refuses_without_flag() {
    let dir = TempDir::new().unwrap();
    write(
        dir.path(),
        "intl-ai-docs.toml",
        "source_dir = \"docs\"\ntargets = [\"es\"]\n\n[provider]\nkind = \"replay\"\nfile = \"c.json\"\n",
    );
    cmd(&dir)
        .arg("fill")
        .assert()
        .failure()
        .stderr(predicates::str::contains("experimental"));
}

#[test]
fn fill_writes_translated_trees_and_shards() {
    let dir = fixture();
    cmd(&dir).arg("fill").arg("--yes").assert().success();

    let es = fs::read_to_string(dir.path().join("docs/es/index.md")).unwrap();
    assert!(es.contains("title: Inicio"));
    assert!(es.contains("# Bienvenido a intl-ai"));
    // skeleton preserved byte-for-byte
    assert!(es.contains("```sh\nintl-ai fill --locale es\n```"));
    assert!(es.contains("::: warning"));
    // links carry the locale prefix
    assert!(es.contains("](/es/guide/getting-started/)"));

    let pt = fs::read_to_string(dir.path().join("docs/pt/guide/setup.md")).unwrap();
    assert!(pt.contains("# Configuração"));
    assert!(pt.contains("- primeiro passo"));

    // shards exist per document
    let shard =
        fs::read_to_string(dir.path().join("intl-ai-docs.lock.d/es/index.md.toml")).unwrap();
    assert!(shard.contains("h:"), "shard keys are content-hash ids");
    assert!(shard.contains("Bienvenido a intl-ai"));
}

#[test]
fn check_clean_after_fill_and_detects_edits() {
    let dir = fixture();
    cmd(&dir).arg("fill").arg("--yes").assert().success();
    cmd(&dir).arg("check").assert().success();

    // stale detection: change the source, check reports stale + fill
    // retranslates only the changed segment.
    fs::write(
        dir.path().join("docs/guide/setup.md"),
        "---\ntitle: Setup\n---\n\n# Setup\n\nInstall the binary with your package manager.\n\n- first step revised\n- second step\n",
    )
    .unwrap();
    write(
        dir.path(),
        "cassette.json",
        r#"{
  "es": { "first step revised": "primer paso revisado" },
  "pt": { "first step revised": "primeiro passo revisto" }
}"#,
    );
    // check exits nonzero: `missing` sits in default fail_on.
    let out = cmd(&dir)
        .arg("check")
        .arg("--format")
        .arg("json")
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    let es_setup = &v["locales"]["es/guide/setup.md"];
    // Content-hash identity: an edited segment is a NEW missing id plus
    // an orphaned extra id, not a stale flag.
    assert_eq!(es_setup["missing"].as_array().unwrap().len(), 1);
    assert_eq!(es_setup["extra"].as_array().unwrap().len(), 1);

    cmd(&dir).arg("fill").arg("--yes").assert().success();
    let es = fs::read_to_string(dir.path().join("docs/es/guide/setup.md")).unwrap();
    assert!(es.contains("primer paso revisado"));
    assert!(es.contains("segundo paso")); // untranslated keys untouched
}
