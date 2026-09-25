use anyhow::{Result, anyhow};
use std::fs;

use crate::InitArgs;

/// Writes a starter `intl-ai.toml`. Refuses to overwrite an existing config;
/// discovers a likely locale dir (a subdir holding >=2 *.json files) when
/// --locale-dir is not given.
pub fn run(args: &InitArgs) -> Result<u8> {
    let cwd = std::env::current_dir()?;
    let path = cwd.join("intl-ai.toml");
    if path.exists() {
        return Err(anyhow!("intl-ai.toml already exists"));
    }

    let locale_dir = args
        .locale_dir
        .clone()
        .or_else(detect_locale_dir)
        .unwrap_or_else(|| "locales".into());
    let source = args.source.clone().unwrap_or_else(|| "en".into());
    let targets = if args.targets.is_empty() {
        vec!["fr".to_string()]
    } else {
        args.targets.clone()
    };

    let targets_toml = targets
        .iter()
        .map(|t| format!("\"{t}\""))
        .collect::<Vec<_>>()
        .join(", ");
    let locale_dir = locale_dir.to_string_lossy();
    let body = format!(
        r#"version = 1
locale_dir = "{locale_dir}"
source = "{source}"
targets = [{targets_toml}]

# Provider: pick one arm. `replay` replays a cassette (deterministic,
# good for tests). `http` is the OpenAI-compatible surface:
#
#   [provider]
#   kind = "http"
#   model = "gpt-5-mini"
#   api_key = "${{env:OPENAI_API_KEY}}"
#
# `command` runs a local agent CLI (keyless): `agent = "claude-code"`
# presets claude-code | opencode | codex | crush | gemini, or a
# free-form `command`/`args` in this file.
[provider]
kind = "replay"
file = "cassette.json"

[check]
fail_on = ["stale", "invalid"]

# Checks run per target value. Builtins: icu | placeholder-parity |
# dialect:<locale> | judge. Or point `spec` at a YAML rules file, or
# `exec` at a command speaking the v1 JSONL protocol:
#
#   [[checks]]
#   id = "icu"
#
#   [[checks]]
#   spec = "checks/house-style.yaml"
#
#   [[checks]]
#   exec = "python3"
#   args = ["checks/my_check.py"]
"#
    );
    fs::write(&path, body)?;
    println!("wrote {}", path.display());
    gitignore_stat_cache(&cwd);
    Ok(0)
}

/// `.intl-ai/` holds the disposable stat-cache (plan 5.5); it must never
/// be committed. Idempotent: appends only when the line is absent.
fn gitignore_stat_cache(cwd: &std::path::Path) {
    let path = cwd.join(".gitignore");
    let existing = fs::read_to_string(&path).unwrap_or_default();
    if existing.lines().any(|l| l.trim() == ".intl-ai/") {
        return;
    }
    let mut body = existing;
    if !body.is_empty() && !body.ends_with('\n') {
        body.push('\n');
    }
    body.push_str(".intl-ai/\n");
    if fs::write(&path, body).is_ok() {
        println!("added .intl-ai/ to .gitignore");
    }
}

fn detect_locale_dir() -> Option<std::path::PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    for entry in fs::read_dir(cwd).ok()?.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let json_count = fs::read_dir(&path)
            .ok()
            .map(|rd| {
                rd.flatten()
                    .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("json"))
                    .count()
            })
            .unwrap_or(0);
        if json_count >= 2 {
            return path.file_name().and_then(|n| n.to_str()).map(|s| s.into());
        }
    }
    None
}
