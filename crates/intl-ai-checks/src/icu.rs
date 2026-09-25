//! ICU MessageFormat checks on the FormatJS parser (the same AST the JS
//! runtime in vue-i18n/next-intl/FormatJS validates against, plan 5.2).

use formatjs_icu_messageformat_parser::types::MessageFormatElement;
use formatjs_icu_messageformat_parser::{Parser, ParserOptions};
use intl_ai_core::check::{Check, CheckCtx, CheckItem};
use intl_ai_core::diff::CheckFinding;
use intl_ai_core::error::Result;
use std::collections::{BTreeMap, BTreeSet};

/// Parse a message as ICU MF1. Error text is the parser's own.
/// `requires_other_clause` matches FormatJS runtime behavior: a plural
/// or select without `other` is a runtime crash, so it is a parse error.
pub fn parse_message(message: &str) -> std::result::Result<Vec<MessageFormatElement>, String> {
    Parser::new(
        message,
        ParserOptions {
            requires_other_clause: true,
            ..Default::default()
        },
    )
    .parse()
    .map_err(|e| format!("{e:?}"))
}

/// Argument names a message binds. Port of the TS icu processor's
/// `extractTokens` (plan 5.1.1): element `value` for
/// Argument/Number/Date/Time/Select/Plural; Pound elements carry no value
/// in the AST, so they never contribute; Tag children and Select/Plural
/// option bodies are walked.
pub fn extract_tokens(message: &str) -> Vec<String> {
    let Ok(elements) = parse_message(message) else {
        return Vec::new();
    };
    let mut tokens = Vec::new();
    for el in &elements {
        extract_from(el, &mut tokens);
    }
    tokens
}

fn extract_from(el: &MessageFormatElement, out: &mut Vec<String>) {
    match el {
        MessageFormatElement::Literal(_) | MessageFormatElement::Pound(_) => {}
        MessageFormatElement::Argument(a) => out.push(a.value.clone()),
        MessageFormatElement::Number(e) => out.push(e.value.clone()),
        MessageFormatElement::Date(e) => out.push(e.value.clone()),
        MessageFormatElement::Time(e) => out.push(e.value.clone()),
        MessageFormatElement::Select(e) => {
            out.push(e.value.clone());
            for option in e.options.values() {
                for child in &option.value {
                    extract_from(child, out);
                }
            }
        }
        MessageFormatElement::Plural(e) => {
            out.push(e.value.clone());
            for option in e.options.values() {
                for child in &option.value {
                    extract_from(child, out);
                }
            }
        }
        MessageFormatElement::Tag(e) => {
            for child in &e.children {
                extract_from(child, out);
            }
        }
    }
}

/// `icu`: the target must parse as ICU MessageFormat.
pub struct IcuCheck;

impl Check for IcuCheck {
    fn id(&self) -> &str {
        "icu"
    }

    /// Syntax findings are mechanically actionable: the model can fix
    /// broken braces or a missing `other` from the error text.
    fn supports_feedback(&self) -> bool {
        true
    }

    fn cache_ctx(&self) -> BTreeMap<String, String> {
        BTreeMap::new()
    }

    fn run(&self, _ctx: &CheckCtx, items: &[CheckItem]) -> Result<Vec<CheckFinding>> {
        let mut out = Vec::new();
        for item in items {
            if let Err(e) = parse_message(&item.target) {
                out.push(CheckFinding {
                    key: item.key.clone(),
                    check: self.id().into(),
                    message: format!("invalid ICU MessageFormat: {e}"),
                    ..Default::default()
                });
            }
        }
        Ok(out)
    }
}

/// `placeholder-parity`: the target must bind exactly the arguments the
/// source binds (the TS processor's validation rule, verbatim messages).
pub struct PlaceholderParity;

impl Check for PlaceholderParity {
    fn id(&self) -> &str {
        "placeholder-parity"
    }

    fn cache_ctx(&self) -> BTreeMap<String, String> {
        BTreeMap::new()
    }

    /// Token-level findings name exactly which arguments to add or drop.
    fn supports_feedback(&self) -> bool {
        true
    }

    fn run(&self, _ctx: &CheckCtx, items: &[CheckItem]) -> Result<Vec<CheckFinding>> {
        let mut out = Vec::new();
        for item in items {
            let Some(source) = &item.source else { continue };
            // An unparseable source yields no tokens to compare against;
            // flagging the target's real placeholders as "extra" would
            // instruct a refill to strip them.
            if parse_message(source).is_err() {
                continue;
            }
            let src: BTreeSet<String> = extract_tokens(source).into_iter().collect();
            let tgt: BTreeSet<String> = extract_tokens(&item.target).into_iter().collect();
            let missing: Vec<String> = src.difference(&tgt).cloned().collect();
            let extra: Vec<String> = tgt.difference(&src).cloned().collect();
            if missing.is_empty() && extra.is_empty() {
                continue;
            }
            let mut parts = Vec::new();
            if !missing.is_empty() {
                parts.push(format!("Missing tokens: {}", missing.join(", ")));
            }
            if !extra.is_empty() {
                parts.push(format!("Extra tokens: {}", extra.join(", ")));
            }
            out.push(CheckFinding {
                key: item.key.clone(),
                check: self.id().into(),
                message: parts.join("; "),
                ..Default::default()
            });
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_plural_and_extracts_tokens() {
        let tokens: BTreeSet<String> =
            extract_tokens("{count, plural, one {# item for {name}} other {# items for {name}}}")
                .into_iter()
                .collect();
        assert_eq!(
            tokens,
            BTreeSet::from(["count".to_string(), "name".to_string()])
        );
    }

    #[test]
    fn unparseable_yields_no_tokens() {
        assert!(extract_tokens("{oops").is_empty());
    }

    #[test]
    fn icu_flags_broken_target() {
        let c = IcuCheck;
        let ctx = CheckCtx {
            source_locale: "en-US",
            target_locale: "de-DE",
            transport: None,
            locale_instruction: None,
        };
        let items = vec![
            CheckItem {
                key: "ok".into(),
                source: Some("Hi {name}".into()),
                target: "Hallo {name}".into(),
            },
            CheckItem {
                key: "bad".into(),
                source: None,
                target: "Hallo {name".into(),
            },
        ];
        let findings = c.run(&ctx, &items).unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].key, "bad");
    }

    #[test]
    fn parity_reports_missing_and_extra() {
        let c = PlaceholderParity;
        let ctx = CheckCtx {
            source_locale: "en-US",
            target_locale: "de-DE",
            transport: None,
            locale_instruction: None,
        };
        let items = vec![CheckItem {
            key: "k".into(),
            source: Some("{count, plural, one {#} other {#}} for {name}".into()),
            target: "{count, plural, one {#} other {#}} fuer {user}".into(),
        }];
        let findings = c.run(&ctx, &items).unwrap();
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("Missing tokens: name"));
        assert!(findings[0].message.contains("Extra tokens: user"));
    }
}
