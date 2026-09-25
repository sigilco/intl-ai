//! Frozen prompt contract (plan 5.1.5 — ported verbatim from the TS
//! translator; wording is part of the external contract with models).

use intl_ai_core::transport::TranslateRequest;
use serde::Deserialize;

pub const SYSTEM_PROMPT: &str = "You are a professional translation engine. You respond only with valid JSON matching the requested schema.";

pub const DEFAULT_SYNTAX_HINT: &str =
    "Preserve any placeholders like {variable} exactly as they appear.";

/// Response contract shared by both transports: providers answer
/// `{ "translations": [{ "key": ..., "translated": ... }] }`.
#[derive(Debug, Deserialize)]
pub struct TranslationsPayload {
    pub translations: Vec<TranslationsRow>,
}

#[derive(Debug, Deserialize)]
pub struct TranslationsRow {
    pub key: String,
    pub translated: String,
}

pub fn system_prompt(locale_instruction: Option<&str>) -> String {
    match locale_instruction {
        Some(instruction) => format!("{SYSTEM_PROMPT}\n\nLocale style instruction: {instruction}"),
        None => SYSTEM_PROMPT.to_string(),
    }
}

pub fn user_prompt(req: &TranslateRequest) -> String {
    let syntax_hint = req.syntax_hint.as_deref().unwrap_or(DEFAULT_SYNTAX_HINT);

    let glossary_text = if req.glossary.is_empty() {
        String::new()
    } else {
        let mut lines = String::from("\n\nGlossary (use these exact translations):");
        for (term, translation) in &req.glossary {
            lines.push_str(&format!("\n- {term} → {translation}"));
        }
        lines
    };

    let feedback_block = if req.feedback.is_empty() {
        String::new()
    } else {
        let mut lines = String::from(
            "\n\nThe following entries were rejected by a quality reviewer. \
             Address each note in your new translation.",
        );
        for (key, note) in &req.feedback {
            if !note.trim().is_empty() {
                lines.push_str(&format!("\n- {key}: {note}"));
            }
        }
        lines
    };

    let entries_text = req
        .entries
        .iter()
        .enumerate()
        .map(|(i, e)| format!("{}. \"{}\" (key: {})", i + 1, e.source, e.key))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "Translate the following {src} strings to {tgt}.\n\
         {hint}{glossary}{feedback}\n\n\
         Input strings:\n{entries}\n\n\
         Respond with JSON: {{ \"translations\": [{{ \"key\": \"...\", \"translated\": \"...\" }}] }}",
        src = req.source_locale,
        tgt = req.target_locale,
        hint = syntax_hint,
        glossary = glossary_text,
        feedback = feedback_block,
        entries = entries_text,
    )
}

/// Parses `{"translations":[{key,translated}]}` — lenient on extra fields
/// (models add `confidence`, `notes`, etc.) but strict on the contract keys.
pub fn parse_translations(content: &str) -> serde_json::Result<TranslationsPayload> {
    serde_json::from_str(content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use intl_ai_core::transport::TranslationEntry;
    use std::collections::BTreeMap;

    fn req() -> TranslateRequest {
        TranslateRequest {
            source_locale: "en".into(),
            target_locale: "fr".into(),
            entries: vec![
                TranslationEntry {
                    key: "a".into(),
                    source: "Hello".into(),
                },
                TranslationEntry {
                    key: "b".into(),
                    source: "Bye".into(),
                },
            ],
            ..Default::default()
        }
    }

    #[test]
    fn system_prompt_bare() {
        assert_eq!(system_prompt(None), SYSTEM_PROMPT);
    }

    #[test]
    fn system_prompt_with_instruction() {
        assert_eq!(
            system_prompt(Some("use informal vous→tu")),
            format!("{SYSTEM_PROMPT}\n\nLocale style instruction: use informal vous→tu")
        );
    }

    #[test]
    fn user_prompt_contract() {
        let prompt = user_prompt(&req());
        let expected = "Translate the following en strings to fr.\n\
            Preserve any placeholders like {variable} exactly as they appear.\n\n\
            Input strings:\n\
            1. \"Hello\" (key: a)\n\
            2. \"Bye\" (key: b)\n\n\
            Respond with JSON: { \"translations\": [{ \"key\": \"...\", \"translated\": \"...\" }] }";
        assert_eq!(prompt, expected);
    }

    #[test]
    fn user_prompt_glossary_and_feedback() {
        let mut r = req();
        r.glossary = BTreeMap::from([("sigil".to_string(), "sceau".to_string())]);
        r.feedback = BTreeMap::from([("a".to_string(), "too literal".to_string())]);
        let p = user_prompt(&r);
        assert!(p.contains("Glossary (use these exact translations):\n- sigil → sceau"));
        assert!(p.contains("The following entries were rejected by a quality reviewer."));
        assert!(p.contains("- a: too literal"));
    }
}
