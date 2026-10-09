//! MDX/docmd markdown `DocFormat`.
//!
//! Extraction is two-pass: a line classifier marks every construct
//! CommonMark mis-parses or that must never translate (frontmatter,
//! fenced code incl. contents, `:::`/`==` directive markers, MDX
//! `import`/`export` lines, tag-led JSX lines) as opaque; the remaining
//! prose regions go through pulldown-cmark's offset iterator, and each
//! leaf block (paragraph, heading, table cell) contributes one segment
//! spanning its inline content — block markers (`##`, `-`, `>`) stay in
//! the skeleton and are byte-preserved by construction.
//!
//! Merge splices translated values into the recorded spans (descending
//! order) with locale link-prefix rewriting on `/`-rooted destinations.

use crate::doc::{DocFormat, Document, MergeOut, SegKind, Span};
use crate::{frontmatter, links};
use intl_ai_core::error::{Error, Result};
use intl_ai_core::flatten::FlatMap;
use intl_ai_core::hash::source_hash;
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use std::ops::Range;
use std::path::Path;

/// Markdown + MDX document format. `frontmatter_keys` are the dotted
/// frontmatter paths whose scalar values translate (`title`,
/// `description` by default).
#[derive(Debug, Default, Clone)]
pub struct Mdx {
    pub frontmatter_keys: Vec<String>,
}

impl Mdx {
    pub fn new(frontmatter_keys: Vec<String>) -> Self {
        Self { frontmatter_keys }
    }
}

fn options() -> Options {
    Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_HEADING_ATTRIBUTES
        | Options::ENABLE_MATH
}

/// Segment identity: content hash — gettext-style, stable under
/// reorders and shared across files (free cross-document TM).
pub fn segid(text: &str) -> String {
    format!("h:{}", &source_hash(text)[..12])
}

fn leaf_kind(t: &Tag) -> Option<SegKind> {
    match t {
        Tag::Paragraph => Some(SegKind::Paragraph),
        Tag::Heading { .. } => Some(SegKind::Heading),
        Tag::TableCell => Some(SegKind::Cell),
        Tag::DefinitionListTitle => Some(SegKind::Paragraph),
        _ => None,
    }
}

fn is_leaf_end(t: &TagEnd) -> bool {
    matches!(
        t,
        TagEnd::Paragraph | TagEnd::Heading(_) | TagEnd::TableCell | TagEnd::DefinitionListTitle
    )
}

/// Line-level opacity. An opaque line is never offered to the parser,
/// so its bytes can only survive verbatim into the skeleton.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Line {
    Prose,
    Opaque,
}

fn line_ranges(text: &str) -> Vec<Range<usize>> {
    let mut v = Vec::new();
    let mut off = 0;
    for l in text.split_inclusive('\n') {
        v.push(off..off + l.len());
        off += l.len();
    }
    if off < text.len() {
        v.push(off..text.len());
    }
    v
}

fn fence_close(t: &str, ch: char, n: usize) -> bool {
    let t = t.trim();
    t.chars().filter(|c| *c == ch).count() >= n && t.chars().all(|c| c == ch || c.is_whitespace())
}

/// Classify every line; returns the per-line classes plus the
/// frontmatter block range (already included in the opaque set).
pub fn classify(text: &str) -> (Vec<Line>, Vec<Range<usize>>) {
    let lines = line_ranges(text);
    let mut class = vec![Line::Prose; lines.len()];
    let fm = frontmatter::block_range(text);
    if let Some(r) = &fm {
        for (i, l) in lines.iter().enumerate() {
            if l.start < r.end {
                class[i] = Line::Opaque;
            }
        }
    }
    let fm_end = fm.as_ref().map(|r| r.end).unwrap_or(0);
    let mut in_fence: Option<(char, usize)> = None;
    let mut in_tag = false;
    for (i, r) in lines.iter().enumerate() {
        if r.start < fm_end {
            continue; // already opaque
        }
        let t = text[r.clone()].trim_end_matches(['\n', '\r']);
        let tt = t.trim_start();
        if let Some((ch, n)) = in_fence {
            class[i] = Line::Opaque;
            if fence_close(tt, ch, n) {
                in_fence = None;
            }
            continue;
        }
        if tt.starts_with("```") || tt.starts_with("~~~") {
            let ch = tt.chars().next().unwrap();
            let n = tt.chars().take_while(|c| *c == ch).count();
            if n >= 3 {
                class[i] = Line::Opaque;
                in_fence = Some((ch, n));
                continue;
            }
        }
        if tt.starts_with(":::") || tt.starts_with("==") {
            class[i] = Line::Opaque;
            continue;
        }
        if tt.starts_with("import ") || tt.starts_with("export ") {
            class[i] = Line::Opaque;
            continue;
        }
        if in_tag {
            class[i] = Line::Opaque;
            if tt.contains('>') {
                in_tag = false;
            }
            continue;
        }
        if tt.starts_with('<')
            && tt
                .as_bytes()
                .get(1)
                .is_some_and(|b| b.is_ascii_alphabetic() || *b == b'/')
        {
            class[i] = Line::Opaque;
            if !tt.contains('>') {
                in_tag = true;
            }
            continue;
        }
    }
    (class, lines)
}

/// Opaque lines' raw content (trimmed), excluding any line that
/// overlaps a segment span — `lint_document` compares these between
/// source and merged output, and a line holding a translated span is
/// only partially skeleton (a `title:` line's value legitimately
/// diverges).
pub fn opaque_lines(text: &str, spans: &[Range<usize>]) -> Vec<String> {
    let (class, lines) = classify(text);
    class
        .iter()
        .zip(&lines)
        .filter(|(c, r)| {
            **c == Line::Opaque && !spans.iter().any(|s| s.start < r.end && r.start < s.end)
        })
        .map(|(_, r)| text[r.clone()].trim_end().to_string())
        .collect()
}

/// Block-level tags — everything not an inline construct (Link, Image,
/// and the emphasis family are inline; they extend an open leaf rather
/// than bounding it).
fn is_block_tag(t: &Tag) -> bool {
    !matches!(
        t,
        Tag::Link { .. }
            | Tag::Image { .. }
            | Tag::Strong
            | Tag::Emphasis
            | Tag::Strikethrough
            | Tag::Superscript
            | Tag::Subscript
    )
}

fn flush_leaf(
    cur: &mut Option<(usize, usize, SegKind, bool)>,
    spans: &mut Vec<(Range<usize>, SegKind)>,
) {
    if let Some((s, e, k, _)) = cur.take()
        && s != usize::MAX
        && e > s
    {
        spans.push((s..e, k));
    }
}

/// Leaf-block content spans inside one prose region. Explicit leaves
/// (Paragraph/Heading/TableCell/DefinitionListTitle) span their inner
/// events — the block's own Start range is excluded so `#`/`{-}` markers
/// stay skeleton. Tight list items emit inline content without a
/// Paragraph wrapper (pulldown-cmark drops it), so text directly under
/// an `Item` opens an implicit leaf that closes on `End(Item)` and
/// flushes early if a nested block appears (loose items keep their
/// paragraph segments).
#[derive(Clone, Copy, PartialEq, Eq)]
enum Open {
    Item,
    Block,
    Inline,
}

fn leaf_spans(text: &str, region: Range<usize>) -> Vec<(Range<usize>, SegKind)> {
    let mut spans = Vec::new();
    let mut stack: Vec<Open> = Vec::new();
    // (min_start, max_end, kind, implicit-item leaf)
    let mut cur: Option<(usize, usize, SegKind, bool)> = None;
    // Tight-list test: innermost open *block* tag is an Item.
    let in_tight_item =
        |stack: &[Open]| stack.iter().rev().find(|o| **o != Open::Inline) == Some(&Open::Item);
    let body = &text[region.clone()];
    for (ev, r) in Parser::new_ext(body, options()).into_offset_iter() {
        let r = (r.start + region.start)..(r.end + region.start);
        match &ev {
            Event::Start(t) => {
                if is_block_tag(t) {
                    if cur.is_some_and(|c| c.3) {
                        flush_leaf(&mut cur, &mut spans);
                    }
                    if cur.is_none()
                        && let Some(k) = leaf_kind(t)
                    {
                        cur = Some((usize::MAX, 0, k, false));
                        stack.push(Open::Block);
                        continue;
                    }
                    stack.push(if matches!(t, Tag::Item) {
                        Open::Item
                    } else {
                        Open::Block
                    });
                } else {
                    stack.push(Open::Inline);
                    if cur.is_none() && in_tight_item(&stack) {
                        cur = Some((usize::MAX, 0, SegKind::Paragraph, true));
                    }
                    if let Some((s, e, _, _)) = &mut cur {
                        *s = (*s).min(r.start);
                        *e = (*e).max(r.end);
                    }
                }
            }
            Event::End(t) => {
                let closes = match &cur {
                    Some((_, _, _, true)) => matches!(t, TagEnd::Item),
                    Some(_) => is_leaf_end(t),
                    None => false,
                };
                stack.pop();
                if closes {
                    flush_leaf(&mut cur, &mut spans);
                    continue;
                }
            }
            _ => {
                if cur.is_none() && in_tight_item(&stack) {
                    cur = Some((usize::MAX, 0, SegKind::Paragraph, true));
                }
                if let Some((s, e, _, _)) = &mut cur {
                    *s = (*s).min(r.start);
                    *e = (*e).max(r.end);
                }
            }
        }
    }
    flush_leaf(&mut cur, &mut spans);
    spans
}

impl DocFormat for Mdx {
    fn name(&self) -> &str {
        "mdx"
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["md", "mdx"]
    }

    fn extract(&self, text: &str, path: &Path) -> Result<Document> {
        let (class, lines) = classify(text);
        let mut spans: Vec<Span> = Vec::new();
        let mut segments: FlatMap = FlatMap::new();
        let mut order = Vec::new();
        let mut warnings = Vec::new();
        let mut add = |kind: SegKind, range: Range<usize>| {
            let seg_text = &text[range.clone()];
            if seg_text.trim().is_empty() {
                return;
            }
            let id = segid(seg_text);
            if !segments.contains_key(&id) {
                segments.insert(id.clone(), seg_text.to_string());
                order.push(id.clone());
            }
            spans.push(Span {
                range,
                seg: id,
                kind,
            });
        };

        // Frontmatter scalar spans first (document order: it leads).
        if let Some(fm) = frontmatter::block_range(text) {
            for (_, r) in frontmatter::scalar_spans(text, fm.clone(), &self.frontmatter_keys) {
                add(SegKind::Frontmatter, r);
            }
        }

        // Prose regions between opaque lines, parsed per region so an
        // opaque line inside a container splits the parse cleanly.
        let mut prose: Option<usize> = None;
        let mut regions = Vec::new();
        for (i, r) in lines.iter().enumerate() {
            match (class[i], prose) {
                (Line::Prose, None) => prose = Some(r.start),
                (Line::Opaque, Some(s)) => {
                    regions.push(s..r.start);
                    prose = None;
                }
                _ => {}
            }
        }
        if let Some(s) = prose {
            regions.push(s..text.len());
        }
        for region in regions {
            for (r, k) in leaf_spans(text, region) {
                add(k, r);
            }
        }
        let _ = path;
        let _ = &mut warnings;
        Ok(Document {
            source: text.to_string(),
            spans,
            segments,
            order,
            warnings,
        })
    }

    fn merge(&self, doc: &Document, target: &FlatMap, link_prefix: &str) -> Result<MergeOut> {
        let mut out = doc.source.clone();
        let mut fell_back = Vec::new();
        let mut spans = doc.spans.clone();
        // Splice back-to-front so earlier ranges stay valid.
        spans.sort_by_key(|s| std::cmp::Reverse(s.range.start));
        for sp in spans {
            let Some(val) = target.get(&sp.seg) else {
                fell_back.push(sp.seg.clone());
                continue;
            };
            let mut repl = links::rewrite(val, link_prefix);
            if sp.kind == SegKind::Frontmatter {
                repl = fm_escape(&doc.source, &sp.range, &repl);
            }
            out.replace_range(sp.range.clone(), &repl);
        }
        Ok(MergeOut {
            bytes: out.into_bytes(),
            fell_back,
        })
    }
}

/// Frontmatter scalar safety: a value spliced between existing quotes
/// must have its quote char escaped; an unquoted value that would need
/// quoting under YAML gets wrapped in double quotes.
fn fm_escape(source: &str, range: &Range<usize>, val: &str) -> String {
    let quote = source
        .as_bytes()
        .get(range.start.saturating_sub(1))
        .copied();
    match quote {
        Some(b'"') => val.replace('\\', "\\\\").replace('"', "\\\""),
        Some(b'\'') => val.replace('\'', "''"),
        _ => {
            let needs_quotes = val.contains(": ")
                || val.starts_with(|c: char| "\"'[{&*!|>%@`#-?".contains(c))
                || val.trim() != val;
            if needs_quotes {
                format!("\"{}\"", val.replace('\\', "\\\\").replace('"', "\\\""))
            } else {
                val.to_string()
            }
        }
    }
}

impl Mdx {
    /// Convenience for tests and pipelines: extract + merge roundtrip.
    pub fn splice(&self, text: &str, target: &FlatMap, link_prefix: &str) -> Result<String> {
        let doc = self.extract(text, Path::new("<memory>"))?;
        let m = self.merge(&doc, target, link_prefix)?;
        String::from_utf8(m.bytes)
            .map_err(|e| Error::Message(format!("merged document is not utf-8: {e}")))
    }
}
