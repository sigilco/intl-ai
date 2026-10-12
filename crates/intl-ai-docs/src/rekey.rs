//! Target-document re-identification: a translated document's extracted
//! segments must map back onto source seg ids for diff/fill to compare
//! like with like. Three passes:
//!
//! 1. **value-hash match**: the shard records the values the tool wrote;
//!    an unchanged tool-written document pairs instantly and exactly.
//! 2. **positional pairing**: unpaired target segments (human-written or
//!    human-edited docs) zip with unpaired source seg ids by ordinal.
//!    Heuristic by design — `check` still reports the pair's true state.
//! 3. **`x:` keys**: leftovers get content-hash extra keys, which surface
//!    as `extra` findings and translate as fresh segments on `fill`.

use crate::doc::Document;
use crate::links;
use intl_ai_core::flatten::FlatMap;
use intl_ai_core::hash::source_hash;
use intl_ai_core::lockfile::Shard;
use std::collections::BTreeMap;

/// Map `target_doc`'s spans to seg ids using `shard` provenance and
/// `src_order` (the source document's seg ids in order). Extracted
/// segment text is normalized to its *logical* value before hashing and
/// storage — on-disk link destinations carry the locale prefix that
/// `merge` adds at write time, and the shard stores the unprefixed
/// value. Without the normalization every tool-written document reads
/// as human-modified (`effective_origin` flips AI to Human).
pub fn rekey(
    target_doc: &Document,
    shard: &Shard,
    src_order: &[String],
    link_prefix: &str,
) -> FlatMap {
    // adopted-value hash -> candidate seg ids, in shard key order.
    // Distinct source segments can translate to identical text (a
    // "Setup" cell and a "Configuration" heading both become
    // "Configuración"): each on-disk span consumes the next candidate,
    // and surplus duplicates of one segment fall back to the last id —
    // dropping them would report the second occurrence as missing.
    let mut by_val: BTreeMap<String, Vec<&str>> = BTreeMap::new();
    for (k, e) in &shard.entries {
        if !e.value.is_empty() {
            by_val
                .entry(source_hash(&e.value))
                .or_default()
                .push(k.as_str());
        }
    }

    let mut out = FlatMap::new();
    let mut unpaired: Vec<(String, String)> = Vec::new();
    let mut taken: BTreeMap<String, usize> = BTreeMap::new();
    for sp in &target_doc.spans {
        let text = links::strip(&target_doc.source[sp.range.clone()], link_prefix);
        match by_val.get(&source_hash(&text)) {
            Some(cands) => {
                let i = taken.entry(source_hash(&text)).or_insert(0);
                let k = cands.get(*i).copied().unwrap_or(cands[cands.len() - 1]);
                *i += 1;
                out.entry(k.to_string()).or_insert(text);
            }
            None => unpaired.push((sp.seg.clone(), text)),
        }
    }
    let open: Vec<&String> = src_order.iter().filter(|s| !out.contains_key(*s)).collect();
    // Positional pairing runs on DISTINCT segments: several spans share
    // one seg id when a document repeats identical text (table cells
    // like `list`, `bool`). Letting every span consume an open id
    // steals slots and scrambles every downstream pair.
    let mut seen: std::collections::HashSet<String> = Default::default();
    let mut i = 0;
    for (seg, text) in unpaired {
        if !seen.insert(seg) {
            continue;
        }
        match open.get(i) {
            Some(k) => {
                out.insert((*k).clone(), text);
            }
            None => {
                out.insert(format!("x:{}", &source_hash(&text)[..8]), text);
            }
        }
        i += 1;
    }
    out
}
