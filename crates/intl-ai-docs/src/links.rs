//! Link destination rewriting applied to translated segment values at
//! merge time: `/`-rooted destinations (site-internal links) get the
//! target-locale prefix (`/guide/x` -> `/es/guide/x`) so a translated
//! tree links within itself. External (`https://`), protocol-relative
//! (`//`), anchor (`#`) and relative (`./`, `../`) links are untouched.
//! Idempotent: destinations already carrying the prefix are skipped.

/// Rewrite `](dest)` and `](<dest>)` destinations in one translated
/// segment. `prefix` like "/es"; empty disables.
pub fn rewrite(text: &str, prefix: &str) -> String {
    if prefix.is_empty() {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len() + 8);
    let mut rest = text;
    while let Some(i) = rest.find("](") {
        out.push_str(&rest[..i + 2]);
        rest = &rest[i + 2..];
        if let Some(stripped) = rest.strip_prefix('<') {
            out.push('<');
            rest = stripped;
        }
        if rest.starts_with('/') && !rest.starts_with("//") && !rest.starts_with(prefix) {
            out.push_str(prefix);
        }
    }
    out.push_str(rest);
    out
}

/// Inverse of `rewrite`: drop `prefix` from destinations that carry it,
/// mapping on-disk rendered text back to the logical value the shard
/// stores. A destination matches only on a full segment boundary
/// (`/es` itself or `/es/...`), never on `/español`.
pub fn strip(text: &str, prefix: &str) -> String {
    if prefix.is_empty() {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(i) = rest.find("](") {
        out.push_str(&rest[..i + 2]);
        rest = &rest[i + 2..];
        if let Some(stripped) = rest.strip_prefix('<') {
            rest = stripped;
        }
        if rest == prefix
            || rest
                .strip_prefix(prefix)
                .is_some_and(|r| r.starts_with('/'))
        {
            rest = &rest[prefix.len()..];
            // `/es` alone degenerates to `/`, which `rewrite` would
            // re-prefix anyway — keep the stripped form for parity.
        }
    }
    out.push_str(rest);
    out
}

/// `/`-rooted link destinations present in `text` that lack `prefix` —
/// used by `lint_document` on the merged output so unprefixed internal
/// links in a localized tree are findings, not silent leaks.
/// Parse-aware: destinations inside code spans and fenced blocks are
/// literals (examples, snippets) and legitimately keep their raw form.
pub fn unprefixed(text: &str, prefix: &str) -> Vec<String> {
    use pulldown_cmark::{Event, Options, Parser, Tag};
    let opts = Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_HEADING_ATTRIBUTES
        | Options::ENABLE_MATH;
    let mut bad = Vec::new();
    for ev in Parser::new_ext(text, opts) {
        if let Event::Start(Tag::Link { dest_url, .. })
        | Event::Start(Tag::Image { dest_url, .. }) = ev
        {
            let d = dest_url.as_ref();
            if d.starts_with('/') && !d.starts_with("//") && !d.starts_with(prefix) {
                bad.push(d.to_string());
            }
        }
    }
    bad
}
