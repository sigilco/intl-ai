//! YAML frontmatter scan: the `---`-delimited block at offset 0 is
//! opaque except allowlisted `path: scalar` lines, whose value spans
//! become Frontmatter segments. Block scalars, sequences and nested
//! maps stay opaque — only flat scalars translate.

use std::ops::Range;

/// Byte range of the whole frontmatter block (both `---` delimiters
/// included), or None when the document does not start with one.
pub fn block_range(text: &str) -> Option<Range<usize>> {
    let first_nl = if text.starts_with("---\r\n") {
        5
    } else if text.starts_with("---\n") {
        4
    } else {
        return None;
    };
    let mut off = first_nl;
    for line in text[off..].split_inclusive('\n') {
        let t = line.trim_end();
        if t == "---" || t == "..." {
            return Some(0..off + line.len());
        }
        off += line.len();
    }
    // Unterminated frontmatter is just a horizontal rule + text; do not
    // swallow the document.
    None
}

/// Scalar value spans for allowlisted dotted keys inside the
/// frontmatter body (`text[body]`, absolute offsets). Returns
/// (dotted key, value range) pairs; the range excludes surrounding
/// quotes so a translation splices inside them.
pub fn scalar_spans(
    text: &str,
    body: Range<usize>,
    allow: &[String],
) -> Vec<(String, Range<usize>)> {
    let mut path: Vec<(usize, String)> = Vec::new();
    let mut spans = Vec::new();
    let mut off = body.start;
    for line in text[body.clone()].split_inclusive('\n') {
        let raw = line.trim_end_matches(['\n', '\r']);
        let indent = raw.len() - raw.trim_start().len();
        let t = raw.trim_start();
        while path.last().is_some_and(|(i, _)| *i >= indent) {
            path.pop();
        }
        if t.is_empty() || t.starts_with('#') || t.starts_with('-') {
            off += line.len();
            continue;
        }
        if let Some((name, rest)) = t.split_once(':') {
            let name = name.trim();
            let val = rest.trim_start();
            let dotted = {
                let mut p: Vec<&str> = path.iter().map(|(_, n)| n.as_str()).collect();
                p.push(name);
                p.join(".")
            };
            if val.is_empty() || val.starts_with('#') {
                // Nested map opens here; an empty value is a container.
                path.push((indent, name.to_string()));
            } else if !matches!(val.chars().next(), Some('|' | '>' | '[' | '{')) {
                let vstart = off + indent + (t.len() - rest.len()) + (rest.len() - val.len());
                let vraw = val.trim_end();
                let mut end = vstart + vraw.len();
                if let Some(ci) = vraw.find(" #") {
                    end = vstart + ci;
                }
                let mut start = vstart;
                let inner = &text[start..end];
                if inner.len() >= 2
                    && ((inner.starts_with('"') && inner.ends_with('"'))
                        || (inner.starts_with('\'') && inner.ends_with('\'')))
                {
                    start += 1;
                    end -= 1;
                }
                if allow.iter().any(|a| a == &dotted) && end > start {
                    spans.push((dotted, start..end));
                }
            }
        }
        off += line.len();
    }
    spans
}
