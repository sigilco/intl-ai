//! `judge`: adversarial quality scoring through the configured provider
//! transport (the TS `judgeBatch` port, plan 5.2). A score below the
//! entry's `threshold` becomes an `invalid` finding with the judge's
//! reason and errors; every judged key's score is emitted regardless so
//! the `[quality]` aggregator can weigh it. Keys without a source
//! string are skipped (nothing to compare against).

use intl_ai_core::check::{Check, CheckCtx, CheckItem, CheckOutput};
use intl_ai_core::diff::CheckFinding;
use intl_ai_core::error::{Error, Result};
use intl_ai_core::transport::{JudgeItem, JudgeRequest};
use std::collections::BTreeMap;

/// Publishable threshold, verbatim from the TS judge (0..1 scale).
/// `[[checks]] id = "judge"` overrides it via `threshold`.
pub const JUDGE_THRESHOLD: f64 = 0.8;

pub struct JudgeCheck {
    pub threshold: f64,
    pub weight: f64,
}

impl Default for JudgeCheck {
    fn default() -> Self {
        Self {
            threshold: JUDGE_THRESHOLD,
            weight: 1.0,
        }
    }
}

impl Check for JudgeCheck {
    fn id(&self) -> &str {
        "judge"
    }

    fn needs_transport(&self) -> bool {
        true
    }

    /// Judge scores carry reasons and error lists that read naturally as
    /// reviewer notes.
    fn supports_feedback(&self) -> bool {
        true
    }

    fn emits_scores(&self) -> bool {
        true
    }

    fn weight(&self) -> f64 {
        self.weight
    }

    fn cache_ctx(&self) -> BTreeMap<String, String> {
        // Provider identity is ambient but deliberately not fingerprinted
        // (same posture as the stat-cache) — `--no-cache` is the escape
        // hatch when switching models.
        BTreeMap::from([("threshold".into(), self.threshold.to_string())])
    }

    fn run(&self, ctx: &CheckCtx, items: &[CheckItem]) -> Result<CheckOutput> {
        let transport = ctx
            .transport
            .ok_or_else(|| Error::Message("judge check requires a provider transport".into()))?;
        let judge_items: Vec<JudgeItem> = items
            .iter()
            .filter_map(|i| {
                i.source.clone().map(|source| JudgeItem {
                    key: i.key.clone(),
                    locale: ctx.target_locale.to_string(),
                    source,
                    translation: i.target.clone(),
                })
            })
            .collect();
        if judge_items.is_empty() {
            return Ok(CheckOutput::default());
        }
        let judgements = transport.judge(&JudgeRequest {
            items: judge_items,
            locale_instruction: ctx.locale_instruction.map(str::to_string),
        })?;
        let mut out = CheckOutput::default();
        for j in judgements {
            out.scores.insert(j.key.clone(), j.score);
            if j.score < self.threshold {
                let mut message = format!("score {:.2} below {}", j.score, self.threshold);
                if let Some(reason) = &j.reason {
                    message.push_str(&format!(" ({reason})"));
                }
                if !j.errors.is_empty() {
                    message.push_str(&format!("; errors: {}", j.errors.join(", ")));
                }
                out.findings.push(CheckFinding {
                    key: j.key,
                    check: self.id().into(),
                    message,
                    ..Default::default()
                });
            }
        }
        Ok(out)
    }
}
