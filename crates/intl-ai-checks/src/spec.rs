//! Declarative check specs: a YAML drop-in describing rules that run per
//! target value. This is the "colleague's en-US check" tier — a config
//! entry, no code (plan 5.2).
//!
//! Shape:
//!
//! ```yaml
//! id: acme-english
//! description: House style rules          # optional
//! rules:
//!   - forbidden_terms: ["colour", "centre"]   # flag when present
//!     message: "use US spelling"              # optional
//!   - required_terms: ["sign in", "log in"]   # flag when none present
//!   - regex: "\\bkinda\\b"                    # flag on match
//!   - length_ratio: {min: 0.5, max: 2.0}      # len(target)/len(source)
//! self_test:
//!   clean: ["Sign in to continue"]
//!   broken: ["Kinda log in at the centre"]
//! ```
//!
//! `self_test` fixtures run via `intl-ai check --self-test`: every `clean`
//! string must produce zero findings, every `broken` string at least one.
//! `length_ratio` rules need a source string, so they are skipped in
//! self-test fixtures.

use intl_ai_core::check::{Check, CheckCtx, CheckItem};
use intl_ai_core::diff::CheckFinding;
use intl_ai_core::error::{Error, Result};
use regex::Regex;
use std::path::Path;

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckSpec {
    pub id: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub rules: Vec<SpecRule>,
    #[serde(default)]
    pub self_test: Option<SelfTest>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelfTest {
    #[serde(default)]
    pub clean: Vec<String>,
    #[serde(default)]
    pub broken: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpecRule {
    /// Flag a target containing any of these terms (case-insensitive,
    /// word-boundary matched).
    #[serde(default)]
    pub forbidden_terms: Vec<String>,
    /// Flag a target containing none of these terms (case-insensitive
    /// substring).
    #[serde(default)]
    pub required_terms: Vec<String>,
    /// Flag a target matching this pattern.
    #[serde(default)]
    pub regex: Option<String>,
    /// Flag when len(target)/len(source) leaves [min, max]. Items with no
    /// source are skipped.
    #[serde(default)]
    pub length_ratio: Option<LengthRatio>,
    /// Optional message override for the finding.
    #[serde(default)]
    pub message: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LengthRatio {
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
}

pub struct SpecCheck {
    id: String,
    /// Rules with their compiled matchers, in file order.
    rules: Vec<CompiledRule>,
    self_test: Option<SelfTest>,
}

struct CompiledRule {
    message: Option<String>,
    /// One word-boundary regex per forbidden term.
    forbidden: Vec<(String, Regex)>,
    required: Vec<String>,
    regex: Option<Regex>,
    ratio: Option<LengthRatio>,
}

impl SpecCheck {
    pub fn load(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| Error::Config(format!("check spec {}: {e}", path.display())))?;
        let spec: CheckSpec = serde_yaml_ng::from_str(&text)
            .map_err(|e| Error::Config(format!("check spec {}: {e}", path.display())))?;
        Self::compile(spec)
            .map_err(|e| Error::Config(format!("check spec {}: {e}", path.display())))
    }

    /// Parse a spec from a string (tests, `config validate --spec`).
    pub fn from_str(text: &str, origin: &str) -> Result<Self> {
        let spec: CheckSpec = serde_yaml_ng::from_str(text)
            .map_err(|e| Error::Config(format!("check spec {origin}: {e}")))?;
        Self::compile(spec)
    }

    fn compile(spec: CheckSpec) -> Result<Self> {
        if spec.id.trim().is_empty() {
            return Err(Error::Config("check spec: `id` must not be empty".into()));
        }
        let mut rules = Vec::new();
        for (i, rule) in spec.rules.iter().enumerate() {
            let empty = rule.forbidden_terms.is_empty()
                && rule.required_terms.is_empty()
                && rule.regex.is_none()
                && rule.length_ratio.is_none();
            if empty {
                return Err(Error::Config(format!(
                    "rule {i}: needs forbidden_terms, required_terms, regex, or length_ratio"
                )));
            }
            let mut forbidden = Vec::new();
            for term in &rule.forbidden_terms {
                if term.trim().is_empty() {
                    return Err(Error::Config(format!(
                        "rule {i}: empty forbidden_terms entry"
                    )));
                }
                let re = Regex::new(&format!("(?i)\\b{}\\b", regex::escape(term)))
                    .map_err(|e| Error::Config(format!("rule {i} forbidden term '{term}': {e}")))?;
                forbidden.push((term.clone(), re));
            }
            for term in &rule.required_terms {
                if term.trim().is_empty() {
                    return Err(Error::Config(format!(
                        "rule {i}: empty required_terms entry"
                    )));
                }
            }
            let regex = match &rule.regex {
                Some(p) => Some(
                    Regex::new(p)
                        .map_err(|e| Error::Config(format!("rule {i} regex '{p}': {e}")))?,
                ),
                None => None,
            };
            if let Some(r) = &rule.length_ratio {
                let (min, max) = (r.min.unwrap_or(0.0), r.max.unwrap_or(f64::INFINITY));
                if min < 0.0 || max < min {
                    return Err(Error::Config(format!(
                        "rule {i} length_ratio: need 0 <= min <= max"
                    )));
                }
            }
            rules.push(CompiledRule {
                message: rule.message.clone(),
                forbidden,
                required: rule
                    .required_terms
                    .iter()
                    .map(|t| t.to_lowercase())
                    .collect(),
                regex,
                ratio: rule.length_ratio.clone(),
            });
        }
        Ok(Self {
            id: spec.id,
            rules,
            self_test: spec.self_test,
        })
    }

    /// Run the spec's own fixtures. Returns one error line per violation of
    /// the contract (empty = pass).
    pub fn self_test(&self) -> Vec<String> {
        let Some(st) = &self.self_test else {
            return vec![format!("{}: no self_test block", self.id)];
        };
        let ctx = CheckCtx {
            source_locale: "",
            target_locale: "",
            transport: None,
            locale_instruction: None,
        };
        let mut out = Vec::new();
        for (i, s) in st.clean.iter().enumerate() {
            let items = [CheckItem {
                key: format!("clean[{i}]"),
                source: None,
                target: s.clone(),
            }];
            match self.run(&ctx, &items) {
                Ok(f) if !f.is_empty() => out.push(format!(
                    "clean[{i}] produced {} finding(s) ({})",
                    f.len(),
                    f[0].message
                )),
                Err(e) => out.push(format!("clean[{i}] errored: {e}")),
                _ => {}
            }
        }
        for (i, s) in st.broken.iter().enumerate() {
            let items = [CheckItem {
                key: format!("broken[{i}]"),
                source: None,
                target: s.clone(),
            }];
            match self.run(&ctx, &items) {
                Ok(f) if f.is_empty() => out.push(format!("broken[{i}] produced no findings")),
                Err(e) => out.push(format!("broken[{i}] errored: {e}")),
                _ => {}
            }
        }
        out
    }
}

impl Check for SpecCheck {
    fn id(&self) -> &str {
        &self.id
    }

    fn run(&self, _ctx: &CheckCtx, items: &[CheckItem]) -> Result<Vec<CheckFinding>> {
        let mut out = Vec::new();
        for item in items {
            for rule in &self.rules {
                let lower = item.target.to_lowercase();
                if let Some(re) = &rule.regex {
                    if re.is_match(&item.target) {
                        out.push(CheckFinding {
                            key: item.key.clone(),
                            check: self.id.clone(),
                            message: rule
                                .message
                                .clone()
                                .unwrap_or_else(|| format!("matches /{}/", re.as_str())),
                        });
                    }
                }
                for (term, re) in &rule.forbidden {
                    if re.is_match(&item.target) {
                        out.push(CheckFinding {
                            key: item.key.clone(),
                            check: self.id.clone(),
                            message: rule
                                .message
                                .clone()
                                .unwrap_or_else(|| format!("forbidden term '{term}'")),
                        });
                    }
                }
                if !rule.required.is_empty()
                    && !rule.required.iter().any(|t| lower.contains(t.as_str()))
                {
                    out.push(CheckFinding {
                        key: item.key.clone(),
                        check: self.id.clone(),
                        message: rule.message.clone().unwrap_or_else(|| {
                            format!(
                                "missing required term (one of: {})",
                                rule.required.join(", ")
                            )
                        }),
                    });
                }
                if let Some(r) = &rule.ratio {
                    if let Some(source) = &item.source {
                        let src_len = source.chars().count() as f64;
                        if src_len > 0.0 {
                            let ratio = item.target.chars().count() as f64 / src_len;
                            let min = r.min.unwrap_or(0.0);
                            let max = r.max.unwrap_or(f64::INFINITY);
                            if ratio < min || ratio > max {
                                out.push(CheckFinding {
                                    key: item.key.clone(),
                                    check: self.id.clone(),
                                    message: rule.message.clone().unwrap_or_else(|| {
                                        format!(
                                            "length ratio {ratio:.2} outside [{min}, {}]",
                                            if max == f64::INFINITY {
                                                "inf".into()
                                            } else {
                                                max.to_string()
                                            }
                                        )
                                    }),
                                });
                            }
                        }
                    }
                }
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(yaml: &str) -> SpecCheck {
        SpecCheck::from_str(yaml, "test").unwrap()
    }

    #[test]
    fn forbidden_and_required_terms() {
        let c = spec(
            "id: en-house\nrules:\n  - forbidden_terms: [colour]\n  - required_terms: [\"sign in\"]\n",
        );
        let ctx = CheckCtx {
            source_locale: "",
            target_locale: "",
            transport: None,
            locale_instruction: None,
        };
        let items = vec![CheckItem {
            key: "a".into(),
            source: None,
            target: "the colour is nice".into(),
        }];
        let f = c.run(&ctx, &items).unwrap();
        assert_eq!(f.len(), 2);
        assert!(f[0].message.contains("colour"));
        assert!(f[1].message.contains("sign in"));
    }

    #[test]
    fn regex_and_length_ratio() {
        let c = spec("id: r\nrules:\n  - regex: \"kinda\"\n  - length_ratio: {max: 2.0}\n");
        let ctx = CheckCtx {
            source_locale: "",
            target_locale: "",
            transport: None,
            locale_instruction: None,
        };
        let items = vec![CheckItem {
            key: "a".into(),
            source: Some("hi".into()),
            target: "kinda long string here".into(),
        }];
        let f = c.run(&ctx, &items).unwrap();
        assert_eq!(f.len(), 2);
    }

    #[test]
    fn self_test_flags_broken_contract() {
        let c = spec(
            "id: st\nrules:\n  - forbidden_terms: [colour]\nself_test:\n  clean: [\"color\"]\n  broken: [\"colour\"]\n",
        );
        assert!(c.self_test().is_empty());
        let bad = spec(
            "id: st\nrules:\n  - forbidden_terms: [colour]\nself_test:\n  clean: [\"colour\"]\n",
        );
        assert!(!bad.self_test().is_empty());
    }
}
