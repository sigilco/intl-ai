use anyhow::{Result, anyhow};
use intl_ai_core::lockfile::{check_shards, load_shard, save_shard};

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
        LockfileCommand::Merge => Err(anyhow!(
            "lockfile merge is not implemented yet (planned for W1); \
             resolve shard conflicts by hand or with lockfile fmt"
        )),
    }
}
