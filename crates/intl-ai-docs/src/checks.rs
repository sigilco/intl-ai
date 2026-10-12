//! Document checks that run inside `check`/`fill --validate` on the
//! segment FlatMap — `doc:protected` (protected-span parity) and
//! `doc:corruption` (escape/marker leakage). Both are
//! `supports_feedback`: their findings are mechanical and the fill
//! gate can hand them to the provider as reviewer notes.
//! Document-level lint (opaque-line parity, link prefixes) is not a
//! `Check` impl — it needs the merged document, so `lint_document`
//! runs at the pipeline level and lands in `invalid` findings.

use crate::lint;
use intl_ai_core::check::{Check, CheckItem, CheckOutput};
use intl_ai_core::diff::CheckFinding;
use intl_ai_core::error::Result;
use std::collections::BTreeMap;

/// Builtin document check ids (always run by `intl-ai-docs check`;
/// `fill.validate` resolves them by name alongside corpus builtins).
pub const DOC_BUILTINS: &[&str] = &["doc:protected", "doc:corruption"];

pub fn builtin(id: &str) -> Option<Box<dyn Check>> {
    match id {
        "doc:protected" => Some(Box::new(ProtectedCheck)),
        "doc:corruption" => Some(Box::new(CorruptionCheck)),
        _ => None,
    }
}

/// All document builtins, for `check` runs where they always apply.
pub fn all() -> Vec<Box<dyn Check>> {
    DOC_BUILTINS.iter().filter_map(|id| builtin(id)).collect()
}

fn findings_for(id: &str, items: &[CheckItem]) -> Vec<CheckFinding> {
    let mut out = Vec::new();
    for item in items {
        let Some(src) = &item.source else { continue };
        let issues = match id {
            "doc:protected" => parity_issues(src, &item.target),
            _ => escape_issues(src, &item.target),
        };
        out.extend(issues.into_iter().map(|message| CheckFinding {
            key: item.key.clone(),
            check: id.to_string(),
            message,
            cached: false,
        }));
    }
    out
}

fn parity_issues(src: &str, tgt: &str) -> Vec<String> {
    let (a, b) = (lint::protected(src), lint::protected(tgt));
    let mut issues = Vec::new();
    for (k, n) in &a {
        let m = b.get(k).copied().unwrap_or(0);
        if m != *n {
            issues.push(format!(
                "protected span {k:?}: {n} in source, {m} in target"
            ));
        }
    }
    for k in b.keys() {
        if !a.contains_key(k) {
            issues.push(format!("added protected span {k:?}"));
        }
    }
    issues
}

fn escape_issues(src: &str, tgt: &str) -> Vec<String> {
    let mut issues = Vec::new();
    for pat in ["\\n", "\\\"", "\\{", "\\}", "\\[", "\\]"] {
        if tgt.contains(pat) && !src.contains(pat) {
            issues.push(format!("added literal escape {pat:?}"));
        }
    }
    if tgt.contains(":::") && !src.contains(":::") {
        issues.push("directive marker `:::` added to prose".to_string());
    }
    issues
}

#[derive(Debug)]
struct ProtectedCheck;
#[derive(Debug)]
struct CorruptionCheck;

impl Check for ProtectedCheck {
    fn id(&self) -> &str {
        "doc:protected"
    }
    fn supports_feedback(&self) -> bool {
        true
    }
    fn cache_ctx(&self) -> BTreeMap<String, String> {
        BTreeMap::from([("proto".to_string(), "1".to_string())])
    }
    fn run(
        &self,
        _ctx: &intl_ai_core::check::CheckCtx,
        items: &[CheckItem],
    ) -> Result<CheckOutput> {
        Ok(CheckOutput::from(findings_for("doc:protected", items)))
    }
}

impl Check for CorruptionCheck {
    fn id(&self) -> &str {
        "doc:corruption"
    }
    fn supports_feedback(&self) -> bool {
        true
    }
    fn cache_ctx(&self) -> BTreeMap<String, String> {
        BTreeMap::from([("proto".to_string(), "1".to_string())])
    }
    fn run(
        &self,
        _ctx: &intl_ai_core::check::CheckCtx,
        items: &[CheckItem],
    ) -> Result<CheckOutput> {
        Ok(CheckOutput::from(findings_for("doc:corruption", items)))
    }
}
