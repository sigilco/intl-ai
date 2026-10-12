//! MDX extraction/merge tests: the invariants that make document
//! translation safe — skeleton byte-preservation, opaque constructs,
//! segment identity, link rewriting, lint findings.

use super::lint;
use super::mdx::Mdx;
use super::{DocFormat, SegKind};
use intl_ai_core::flatten::FlatMap;
use std::path::Path;

const DOC: &str = r#"---
title: Getting Started
description: Set up intl-ai in minutes.
nested:
  deep: keep
sidebar:
  - one
---

# Getting Started

intl-ai is a single binary that translates the **locale files** your i18n library reads.

::: warning The demo needs an _.illo.fyi origin
Be careful.
:::

::: tabs

== tab "install script"

```sh
curl -fsSL https://intl-ai.illo.fyi/install.sh | bash
```

== tab "Homebrew"

```sh
brew install intl-ai
```

:::

## Links

See [the guide](/guide/getting-started/) and [GitHub](https://github.com/sigilco/intl-ai).

| Name | Kind |
| ---- | ---- |
| `fmt` | builtin |

- item one
- item two

> quoted text

import X from "./x";

<Card title="hello">
</Card>
"#;

fn seg_values(doc: &super::Document) -> Vec<String> {
    doc.order.iter().map(|k| doc.segments[k].clone()).collect()
}

#[test]
fn extract_keeps_opaque_constructs_out() {
    let m = Mdx::new(vec!["title".into(), "description".into()]);
    let doc = m.extract(DOC, Path::new("t.md")).unwrap();
    let vals = seg_values(&doc);
    // code blocks are never segments
    assert!(!vals.iter().any(|v| v.contains("curl -fsSL")));
    assert!(!vals.iter().any(|v| v.contains("brew install")));
    // directive/tab markers are never segments
    assert!(!vals.iter().any(|v| v.contains(":::")));
    assert!(!vals.iter().any(|v| v.contains("== tab")));
    // mdx lines are never segments
    assert!(!vals.iter().any(|v| v.contains("import X")));
    assert!(!vals.iter().any(|v| v.contains("<Card")));
    // prose IS segments
    assert!(vals.iter().any(|v| v == "Getting Started"));
    assert!(vals.iter().any(|v| v.contains("single binary")));
    assert!(vals.iter().any(|v| v == "Be careful."));
    // frontmatter allowlist
    let fm: Vec<_> = doc
        .spans
        .iter()
        .filter(|s| s.kind == SegKind::Frontmatter)
        .collect();
    assert_eq!(fm.len(), 2);
    // list items: each item's paragraph is a segment
    assert!(vals.iter().any(|v| v == "item one"));
    assert!(vals.iter().any(|v| v == "item two"));
    // table cells
    assert!(vals.iter().any(|v| v == "Name"));
    // blockquote content
    assert!(vals.iter().any(|v| v == "quoted text"));
}

#[test]
fn merge_roundtrip_is_byte_identical() {
    let m = Mdx::new(vec!["title".into(), "description".into()]);
    let doc = m.extract(DOC, Path::new("t.md")).unwrap();
    let out = m.merge(&doc, &doc.segments, "").unwrap();
    assert_eq!(String::from_utf8(out.bytes).unwrap(), DOC);
}

#[test]
fn merge_splices_only_segments() {
    let m = Mdx::new(vec!["title".into(), "description".into()]);
    let doc = m.extract(DOC, Path::new("t.md")).unwrap();
    let mut target = FlatMap::new();
    for (k, v) in &doc.segments {
        target.insert(k.clone(), format!("ES::{v}"));
    }
    let out = m.merge(&doc, &target, "").unwrap();
    let s = String::from_utf8(out.bytes).unwrap();
    assert!(s.contains("ES::Getting Started"));
    assert!(s.contains("```sh\ncurl -fsSL"));
    assert!(s.contains("::: tabs"));
    assert!(s.contains("== tab \"install script\""));
    assert!(s.contains("# ES::Getting Started"));
    // frontmatter values spliced inside quotes structure preserved
    assert!(s.contains("title: ES::Getting Started"));
}

#[test]
fn link_rewrite_prepends_locale_prefix() {
    let m = Mdx::default();
    let doc = m
        .extract(
            "See [x](/guide/a) and [y](https://e.com) and [z](./rel).",
            Path::new("t"),
        )
        .unwrap();
    let mut target = FlatMap::new();
    for (k, v) in &doc.segments {
        target.insert(k.clone(), v.replace("x", "equis"));
    }
    let out = m.merge(&doc, &target, "/es").unwrap();
    let s = String::from_utf8(out.bytes).unwrap();
    assert!(s.contains("](/es/guide/a)"));
    assert!(s.contains("(https://e.com)"));
    assert!(s.contains("(./rel)"));
}

#[test]
fn reordered_source_produces_same_segids() {
    let m = Mdx::default();
    let a = m
        .extract("# A\n\nfoo text\n\nbar text\n", Path::new("a"))
        .unwrap();
    let b = m
        .extract("# A\n\nbar text\n\nfoo text\n", Path::new("b"))
        .unwrap();
    let ids_a: std::collections::BTreeSet<_> = a.segments.keys().collect();
    let ids_b: std::collections::BTreeSet<_> = b.segments.keys().collect();
    assert_eq!(ids_a, ids_b);
}

#[test]
fn lint_segment_catches_dropped_code_and_escapes() {
    assert!(lint::lint_segment("use `fmt` here", "use `fmt` aquí").is_empty());
    assert!(!lint::lint_segment("use `fmt` here", "use fmt aquí").is_empty());
    assert!(!lint::lint_segment("hello", "hello\\nworld").is_empty());
    assert!(!lint::lint_segment("see [x](/a)", "ver [x](/a/b)").is_empty());
    assert!(lint::lint_segment("see [x](/a)", "ver [x](/a)").is_empty());
}

#[test]
fn lint_document_flags_divergent_opaque_lines() {
    let m = Mdx::default();
    let src = "```sh\nrun it\n```\n\ntext\n";
    let out = "```sh\nrun it\n```\n\ntexto\n";
    let (sd, td) = docs(src, out, &m);
    assert!(lint::lint_document(src, &sd, out, &td, "").is_empty());
    let bad = "```sh\nrun it changed\n```\n\ntexto\n";
    let (_, bd) = docs(src, bad, &m);
    assert!(!lint::lint_document(src, &sd, bad, &bd, "").is_empty());
}

#[test]
fn lint_document_ignores_frontmatter_value_lines() {
    let m = Mdx::new(vec!["title".into()]);
    let src = "---\ntitle: Home\n---\n\nbody\n";
    let out = "---\ntitle: Inicio\n---\n\ncuerpo\n";
    let (sd, td) = docs(src, out, &m);
    assert!(lint::lint_document(src, &sd, out, &td, "").is_empty());
}

fn docs(
    src: &str,
    out: &str,
    m: &Mdx,
) -> (Vec<std::ops::Range<usize>>, Vec<std::ops::Range<usize>>) {
    let s = m.extract(src, Path::new("s")).unwrap();
    let t = m.extract(out, Path::new("t")).unwrap();
    (
        s.spans.iter().map(|x| x.range.clone()).collect(),
        t.spans.iter().map(|x| x.range.clone()).collect(),
    )
}

#[test]
fn lint_document_flags_unprefixed_links() {
    let m = Mdx::default();
    let src = "go [x](/guide/a)\n";
    let ok = "ve [x](/es/guide/a)\n";
    let (sd, td) = docs(src, ok, &m);
    assert!(lint::lint_document(src, &sd, ok, &td, "/es").is_empty());
    let bad = "ve [x](/guide/a)\n";
    let (_, bd) = docs(src, bad, &m);
    assert_eq!(lint::lint_document(src, &sd, bad, &bd, "/es").len(), 1);
}

#[test]
fn fully_protected_leafs_are_not_segments() {
    let text = "| `layout` | string | `\"flat\"` | Tree arrangement. |\n| `bool` | list | `[]` | Translatable cell. |\n\nbare `code` stays prose\n";
    let doc = Mdx::new(vec![]).extract(text, Path::new("t.md")).unwrap();
    let vals = seg_values(&doc);
    for code in ["`layout`", "string", "`\"flat\"`", "`bool`", "list", "`[]`"] {
        assert!(
            !vals.iter().any(|v| v == code),
            "code-only leaf must not be a segment: {code}"
        );
    }
    assert!(vals.iter().any(|v| v.contains("Tree arrangement")));
    assert!(vals.iter().any(|v| v.contains("Translatable cell")));
    assert!(vals.iter().any(|v| v.contains("bare `code` stays prose")));
    // Merge leaves the protected-only cells byte-identical: they were
    // never segments, so nothing is spliced over them.
    let m = Mdx::new(vec![]);
    let out = m.merge(&doc, &doc.segments, "").unwrap();
    assert_eq!(String::from_utf8(out.bytes).unwrap(), text);
}
