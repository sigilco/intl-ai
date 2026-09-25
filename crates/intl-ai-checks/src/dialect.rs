//! `dialect:<variant-or-locale>` builtin: flags words belonging to the
//! opposite English dialect. Wordlist is the `american-british-english-
//! translator` map (MIT, specs/dialect/en.json): american -> british, with
//! the inverse derived at startup.
//!
//! Ported verbatim from the TS dialect adapter (plan 5.1.1): same WORD_RE,
//! same case-matched suggestions, same exclusion of matches that overlap
//! ICU/placeholder token spans.

use intl_ai_core::check::{Check, CheckCtx, CheckItem};
use intl_ai_core::diff::CheckFinding;
use intl_ai_core::error::{Error, Result};
use regex::Regex;
use std::collections::BTreeMap;
use std::sync::LazyLock;

static WORD_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[A-Za-z]+(?:'[A-Za-z]+)?").unwrap());

static AMERICAN_TO_BRITISH: LazyLock<BTreeMap<String, String>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../../specs/dialect/en.json"))
        .expect("specs/dialect/en.json is a checked-in object map")
});

static BRITISH_TO_AMERICAN: LazyLock<BTreeMap<String, String>> = LazyLock::new(|| {
    let mut out = BTreeMap::new();
    for (american, british) in AMERICAN_TO_BRITISH.iter() {
        out.entry(british.clone())
            .or_insert_with(|| american.clone());
    }
    out
});

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialectVariant {
    American,
    British,
}

/// Maps a locale's region subtag to the dialect variant it implies.
/// `dialect:en-US` flags British spellings; `dialect:en-GB` flags American
/// ones. Bare `american`/`british` spellings also work.
pub fn variant_for(name: &str) -> Option<DialectVariant> {
    match name.to_ascii_lowercase().as_str() {
        "american" | "en-us" => return Some(DialectVariant::American),
        "british" | "en-gb" | "en-uk" => return Some(DialectVariant::British),
        _ => {}
    }
    match name.split('-').nth(1)?.to_ascii_uppercase().as_str() {
        "US" => Some(DialectVariant::American),
        "GB" | "UK" => Some(DialectVariant::British),
        _ => None,
    }
}

fn lookup_table(variant: DialectVariant) -> &'static BTreeMap<String, String> {
    match variant {
        DialectVariant::American => &BRITISH_TO_AMERICAN,
        DialectVariant::British => &AMERICAN_TO_BRITISH,
    }
}

fn match_case(word: &str, suggestion: &str) -> String {
    if word.chars().all(|c| !c.is_lowercase()) {
        return suggestion.to_uppercase();
    }
    if word.chars().next().is_some_and(char::is_uppercase) {
        let mut chars = suggestion.chars();
        if let Some(first) = chars.next() {
            return first.to_uppercase().collect::<String>() + chars.as_str();
        }
    }
    suggestion.to_string()
}

fn overlaps_any_span(offset: usize, len: usize, spans: &[(usize, usize)]) -> bool {
    let end = offset + len;
    spans.iter().any(|&(s, e)| offset < e && end > s)
}

/// Literal spans each token string occupies in `text` (verbatim port of the
/// TS `tokenSpans`: occurrences found via substring scan). Matches inside
/// these spans are skipped by the dialect check.
pub fn token_spans(text: &str, tokens: &[String]) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    for token in tokens {
        if token.is_empty() {
            continue;
        }
        let mut from = 0;
        while let Some(idx) = text[from..].find(token.as_str()) {
            let start = from + idx;
            spans.push((start, start + token.len()));
            from = start + token.len();
        }
    }
    spans
}

/// Words in `text` that belong to the dialect opposite `variant`.
/// `(term, suggestion, offset)` per hit.
pub fn detect(text: &str, variant: DialectVariant, exclude: &[(usize, usize)]) -> Vec<DialectHit> {
    let table = lookup_table(variant);
    let mut hits = Vec::new();
    for m in WORD_RE.find_iter(text) {
        let word = m.as_str();
        if overlaps_any_span(m.start(), word.len(), exclude) {
            continue;
        }
        if let Some(suggestion) = table.get(&word.to_lowercase()) {
            hits.push(DialectHit {
                term: word.to_string(),
                suggestion: match_case(word, suggestion),
                offset: m.start(),
            });
        }
    }
    hits
}

#[derive(Debug)]
pub struct DialectHit {
    pub term: String,
    pub suggestion: String,
    pub offset: usize,
}

pub struct DialectCheck {
    id: String,
    variant: DialectVariant,
}

impl DialectCheck {
    pub fn new(name: &str) -> Result<Self> {
        let variant = variant_for(name).ok_or_else(|| {
            Error::Config(format!(
                "dialect:{name}: cannot infer a variant; use dialect:american, dialect:british, or a locale like en-US / en-GB"
            ))
        })?;
        Ok(Self {
            id: format!("dialect:{name}"),
            variant,
        })
    }
}

impl Check for DialectCheck {
    fn id(&self) -> &str {
        &self.id
    }

    fn run(&self, _ctx: &CheckCtx, items: &[CheckItem]) -> Result<Vec<CheckFinding>> {
        let mut out = Vec::new();
        for item in items {
            let exclude = token_spans(&item.target, &crate::icu::extract_tokens(&item.target));
            for hit in detect(&item.target, self.variant, &exclude) {
                out.push(CheckFinding {
                    key: item.key.clone(),
                    check: self.id.clone(),
                    message: format!("'{}' should be '{}'", hit.term, hit.suggestion),
                });
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn us_locale_flags_british_spellings() {
        let hits = detect(
            "The colour of the favourite thing",
            DialectVariant::American,
            &[],
        );
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].term, "colour");
        assert_eq!(hits[0].suggestion, "color");
        assert_eq!(hits[1].term, "favourite");
    }

    #[test]
    fn gb_locale_flags_american_spellings() {
        let hits = detect("the color", DialectVariant::British, &[]);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].suggestion, "colour");
    }

    #[test]
    fn case_is_matched() {
        let hits = detect("Colour COLOUR colour", DialectVariant::American, &[]);
        assert_eq!(hits[0].suggestion, "Color");
        assert_eq!(hits[1].suggestion, "COLOR");
        assert_eq!(hits[2].suggestion, "color");
    }

    #[test]
    fn excluded_spans_are_skipped() {
        let text = "Pick {flavour} here";
        // {flavour} occupies a span; exclude it.
        let spans = vec![(5, 15)];
        assert!(detect(text, DialectVariant::American, &spans).is_empty());
        assert_eq!(detect(text, DialectVariant::American, &[]).len(), 1);
    }

    #[test]
    fn unknown_variant_errors() {
        assert!(DialectCheck::new("pt-BR").is_err());
        assert!(DialectCheck::new("en-US").is_ok());
    }
}
