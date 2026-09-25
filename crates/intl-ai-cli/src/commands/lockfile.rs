use anyhow::Result;
use intl_ai_core::lockfile::{
    LOCKFILE_DIR, check_shards, load_shard, merge_shard_file, save_shard,
};
use serde_json::json;
use std::fs;

use crate::commands::{is_json, print_json, resolve_config};
use crate::{Cli, LockfileArgs, LockfileCommand};

pub fn run(cli: &Cli, args: &LockfileArgs) -> Result<u8> {
    let cfg = resolve_config(cli)?;
    let locale_dir = cfg.locale_dir();
    let json_out = is_json(args.format);
    match args.command {
        LockfileCommand::Check => {
            let shards = check_shards(&locale_dir)?;
            if json_out {
                let entries: serde_json::Map<_, _> =
                    shards.iter().map(|(l, n)| (l.clone(), json!(*n))).collect();
                print_json(&json!({"shards": entries}))?;
            } else {
                if shards.is_empty() {
                    println!("no shards in {}", locale_dir.display());
                }
                for (locale, count) in &shards {
                    println!("{locale}: {count} entries (ok)");
                }
            }
            Ok(0)
        }
        LockfileCommand::Fmt => {
            // Load+save is the canonical rewrite: save_shard skips identical
            // bytes, so fmt is a fixed point by construction.
            let shards = check_shards(&locale_dir)?;
            let mut rewritten = Vec::new();
            for (locale, _) in &shards {
                let shard = load_shard(&locale_dir, locale)?;
                if save_shard(&locale_dir, locale, &shard)? {
                    rewritten.push(locale.clone());
                }
            }
            if json_out {
                print_json(&json!({"rewritten": rewritten}))?;
            } else {
                for locale in &rewritten {
                    println!("{locale}: normalized");
                }
                if rewritten.is_empty() {
                    println!("all shards already canonical");
                }
            }
            Ok(0)
        }
        LockfileCommand::Merge => {
            let dir = locale_dir.join(LOCKFILE_DIR);
            let mut merged = Vec::new();
            if dir.is_dir() {
                for entry in fs::read_dir(&dir)? {
                    let path = entry?.path();
                    if path.extension().and_then(|e| e.to_str()) != Some("toml") {
                        continue;
                    }
                    let locale = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or_default()
                        .to_string();
                    if let Some(m) = merge_shard_file(&locale_dir, &locale)? {
                        merged.push(json!({
                            "locale": locale,
                            "entries": m.entries,
                            "overlaps": m.overlaps,
                        }));
                    }
                }
            }
            if json_out {
                print_json(&json!({"merged": merged}))?;
            } else {
                for m in &merged {
                    println!(
                        "{}: merged ({} entries, {} overlapping keys)",
                        m["locale"].as_str().unwrap_or_default(),
                        m["entries"],
                        m["overlaps"]
                    );
                }
                if merged.is_empty() {
                    println!("no conflicted shards");
                }
            }
            Ok(0)
        }
    }
}
