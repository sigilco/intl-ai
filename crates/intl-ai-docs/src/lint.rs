//! Document integrity lints — the corruption signatures from
//! whole-string-translate attempts, plus the invariants merge itself
//! cannot express (a translated segment value may still break the
//! document):
//!
//! - `lint_segment`: per-segment checks. Protected-span parity (code,
//!   math, link/image destinations, inline HTML, footnote refs,
//!   emphasis delimiter counts — the model must keep these
//!   byte-identical or count-identical) plus JSON/escape leakage and
//!   directive markers leaking into prose.
//! - `lint_document`: merged-output checks. Every classified-opaque
//!   line in the source must appear identical and in order (fences,
//!   directives, frontmatter delimiters), and `/`-rooted link
//!   destinations must carry the locale prefix.

use crate::links;
use crate::mdx;
use pulldown_cmark::{Event, Options, Parser, Tag};
use std::collections::BTreeMap;

pub(crate) fn parse_options() -> Options {
    Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_HEADING_ATTRIBUTES
        | Options::ENABLE_MATH
}

/// Protected inline content of one segment as a multiset:
/// `kind:payload` -> count. Emphasis/strike/superscript markers count
/// only (which words they wrap may legitimately change); payloads like
/// code text and link destinations must survive byte-identical.
pub fn protected(text: &str) -> BTreeMap<String, usize> {
    let mut m: BTreeMap<String, usize> = BTreeMap::new();
    let mut add = |k: String| *m.entry(k).or_default() += 1;
    for ev in Parser::new_ext(text, parse_options()) {
        match ev {
            Event::Code(t) => add(format!("code:{t}")),
            Event::InlineMath(t) | Event::DisplayMath(t) => add(format!("math:{t}")),
            Event::Html(t) | Event::InlineHtml(t) => {
                // Whitespace inside a tag is safe to normalize for
                // parity — attributes may wrap lines.
                add(format!("html:{}", t.split_whitespace().collect::<String>()))
            }
            Event::FootnoteReference(n) => add(format!("fnref:{n}")),
            Event::TaskListMarker(b) => add(format!("task:{b}")),
            Event::Start(Tag::Link { dest_url, .. }) => add(format!("link:{dest_url}")),
            Event::Start(Tag::Image { dest_url, .. }) => add(format!("img:{dest_url}")),
            Event::Start(Tag::Strong) => add("fmt:strong".into()),
            Event::Start(Tag::Emphasis) => add("fmt:em".into()),
            Event::Start(Tag::Strikethrough) => add("fmt:strike".into()),
            Event::Start(Tag::Superscript) => add("fmt:super".into()),
            Event::Start(Tag::Subscript) => add("fmt:sub".into()),
            _ => {}
        }
    }
    m
}

/// A segment is translatable only when it carries prose text. A leaf
/// that is entirely protected content (code spans, math, inline HTML,
/// a bare link destination) or only punctuation has nothing for the
/// model to render — merging it back byte-identical is both safer and
/// cheaper than sending it to a provider that may "translate" the
/// identifier anyway.
pub fn translatable(text: &str) -> bool {
    for ev in Parser::new_ext(text, parse_options()) {
        if let Event::Text(t) = ev {
            if !t.trim().is_empty() {
                return true;
            }
        }
    }
    false
}

/// Per-segment lint: translated value vs its source. `src` may differ
/// positionally from `tgt` — comparisons are multiset-level.
pub fn lint_segment(src: &str, tgt: &str) -> Vec<String> {
    let mut issues = Vec::new();
    // Escape leakage: a model conditioned on JSON emits literal
    // backslash sequences a markdown value never gains legitimately.
    for pat in ["\\n", "\\\"", "\\{", "\\}", "\\[", "\\]"] {
        if tgt.contains(pat) && !src.contains(pat) {
            issues.push(format!("added literal escape {pat:?}"));
        }
    }
    if tgt.contains(":::") && !src.contains(":::") {
        issues.push("directive marker `:::` added to prose".to_string());
    }
    let (a, b) = (protected(src), protected(tgt));
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

/// Merged-document lint: opaque-line parity against the source plus
/// locale-prefix coverage on `/`-rooted link destinations. `src` is the
/// original document, `out` the merged (or on-disk translated) text;
/// `src_spans`/`out_spans` are each side's segment byte ranges — lines
/// containing a segment are exempt from parity (their span is the part
/// that legitimately changed).
pub fn lint_document(
    src: &str,
    src_spans: &[std::ops::Range<usize>],
    out: &str,
    out_spans: &[std::ops::Range<usize>],
    link_prefix: &str,
) -> Vec<String> {
    let mut issues = Vec::new();
    let (a, b) = (
        mdx::opaque_lines(src, src_spans),
        mdx::opaque_lines(out, out_spans),
    );
    if a != b {
        let pos = a
            .iter()
            .zip(&b)
            .position(|(x, y)| x != y)
            .map(|i| format!("line {}: {:?} -> {:?}", i + 1, a[i], b[i]))
            .unwrap_or_else(|| format!("{} opaque lines -> {}", a.len(), b.len()));
        issues.push(format!("opaque document lines diverge from source ({pos})"));
    }
    if !link_prefix.is_empty() {
        for dest in links::unprefixed(out, link_prefix) {
            issues.push(format!(
                "internal link {dest:?} lacks locale prefix {link_prefix:?}"
            ));
        }
    }
    issues
}
