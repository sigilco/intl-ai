use anyhow::Result;
use intl_ai_core::lockfile::{
    LOCKFILE_DIR, check_shards, load_shard, merge_shard_file, save_shard,
};
use std::fs;

use crate::commands::resolve_config;
use crate::{Cli, LockfileArgs, LockfileCommand};

pub fn run(cli: &Cli, args: &LockfileArgs) -> Result<u8> {
    let cfg = resolve_config(cli)?;
    let locale_dir = cfg.locale_dir();
    match args.command {
        LockfileCommand::Check => {
            let shards = check_shards(&locale_dir)?;
            if shards.is_empty() {
                println!("no shards in {}", locale_dir.display());
            }
            for (locale, count) in &shards {
                println!("{locale}: {count} entries (ok)");
            }
            Ok(0)
        }
        LockfileCommand::Fmt => {
            // Load+save is the canonical rewrite: save_shard skips identical
            // bytes, so fmt is a fixed point by construction.
            let shards = check_shards(&locale_dir)?;
            let mut rewritten = 0usize;
            for (locale, _) in &shards {
                let shard = load_shard(&locale_dir, locale)?;
                if save_shard(&locale_dir, locale, &shard)? {
                    rewritten += 1;
                    println!("{locale}: normalized");
                }
            }
            if rewritten == 0 {
                println!("all shards already canonical");
            }
            Ok(0)
        }
        LockfileCommand::Merge => {
            let dir = locale_dir.join(LOCKFILE_DIR);
            let mut merged = 0usize;
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
                        merged += 1;
                        println!(
                            "{locale}: merged ({} entries, {} overlapping keys)",
                            m.entries, m.overlaps
                        );
                    }
                }
            }
            if merged == 0 {
                println!("no conflicted shards");
            }
            Ok(0)
        }
    }
}
