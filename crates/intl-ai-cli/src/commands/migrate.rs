use anyhow::{Result, anyhow};

pub fn run() -> Result<u8> {
    Err(anyhow!(
        "intl-ai migrate is not implemented yet; it will import a 0.4.x \
         intl-ai.lock.json into intl-ai.lock.d shards (nice-to-have, not a \
         compat contract — see plan 5.5)"
    ))
}
