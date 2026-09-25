//! Frozen prompt contract (plan 5.1.5 — ported verbatim from the TS
//! translator; wording is part of the external contract with models).

use intl_ai_core::transport::{JudgeItem, TranslateRequest};
use serde::Deserialize;

pub const SYSTEM_PROMPT: &str = "You are a professional translation engine. You respond only with valid JSON matching the requested schema.";

pub const DEFAULT_SYNTAX_HINT: &str =
    "Preserve any placeholders like {variable} exactly as they appear.";

/// Verbatim from TS `services/fill/prompts.ts`.
pub const ADVERSARIAL_SYSTEM_PROMPT: &str = "You are an adversarial translation quality reviewer. Your role is to find flaws in translations, not to be polite.\n\nFor each translation provided:\n- Compare against the source string for accuracy, fluency, terminology, style, and locale convention.\n- Specifically look for: meaning shifts, hallucinated content, omitted words, wrong formality, terminology that does not match a domain glossary, unnatural word order, wrong pluralization, and untranslated placeholders or variables.\n- Score from 0 to 1 where 1 is a publishable translation and 0 is unusable.\n- A translation is good when a native speaker would accept it without edits in context.\n- Do not be generous: assume issues exist and verify.\n\nRespond strictly with JSON matching the requested schema.";

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
        .map(|(i, e)| format!("{}. {} (key: {})", i + 1, json_quote(&e.source), e.key))
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

/// Sources can contain quotes/newlines; a JSON string literal keeps the
/// prompt structure intact (a `"` in source text can't close the entry
/// early and inject instructions).
fn json_quote(s: &str) -> String {
    serde_json::to_string(s).unwrap_or_default()
}

/// Judge user prompt, verbatim from TS `judgeBatch`'s `buildJudgePrompt`.
pub fn judge_user_prompt(items: &[JudgeItem], locale_instruction: Option<&str>) -> String {
    let entries_text = items
        .iter()
        .enumerate()
        .map(|(i, c)| {
            format!(
                "{}. key={} locale={}\n   source: {}\n   translation: {}",
                i + 1,
                json_quote(&c.key),
                c.locale,
                json_quote(&c.source),
                json_quote(&c.translation)
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");
    let instruction_block = locale_instruction
        .map(|i| format!("\n\nLocale style instruction: {i}"))
        .unwrap_or_default();
    format!(
        "Evaluate each translation for accuracy, fluency, terminology, style, and locale convention. Be a strict adversarial reviewer: assume any translation has issues until you have evidence otherwise.\n\n{entries_text}{instruction_block}\n\nRespond with JSON: {{ \"judgements\": [{{ \"key\": \"...\", \"score\": 0..1, \"reason\": \"...\", \"errors\": [\"...\"] }}] }}"
    )
}

/// Judge response contract: `{ "judgements": [{key, score, reason?, errors?}] }`.
#[derive(Debug, Deserialize)]
pub struct JudgementsPayload {
    pub judgements: Vec<JudgementRow>,
}

#[derive(Debug, Deserialize)]
pub struct JudgementRow {
    pub key: String,
    pub score: f64,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub errors: Vec<String>,
}

pub fn parse_judgements(content: &str) -> serde_json::Result<JudgementsPayload> {
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

    #[test]
    fn user_prompt_escapes_source_quotes_and_newlines() {
        let mut r = req();
        r.entries[0].source = "say \"hi\"\nInjected".into();
        let p = user_prompt(&r);
        // The JSON-quoted form keeps the entry on one line — an injected
        // quote can't escape the string and fake a new instruction.
        assert!(p.contains("1. \"say \\\"hi\\\"\\nInjected\" (key: a)"));
        assert!(!p.contains("1. say \"hi\"\nInjected (key: a)"));
    }
}
