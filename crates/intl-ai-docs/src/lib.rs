//! intl-ai-docs: AI localization for document trees (experimental).
//!
//! A `DocFormat` splits a document into a skeleton (everything that must
//! survive byte-identical: code fences, directives, frontmatter
//! delimiters, markup) plus a `FlatMap` of translatable segments. The
//! shared FlatMap engine (`fill_flat`/`check_flat`, transport, gate,
//! lockfile, checks) then operates on segments exactly as it does on
//! locale catalogs, and `merge` splices translated values back into the
//! skeleton — document structure can never be translated by
//! construction.
//!
//! Config lives in `intl-ai-docs.{toml,json,yaml}` and shards under
//! `intl-ai-docs.lock.d/`, deliberately separate namespaces from the
//! corpus pipeline so `intl-ai` never learns about document trees.

pub mod checks;
pub mod config;
pub mod doc;
pub mod frontmatter;
pub mod links;
pub mod lint;
pub mod mdx;
pub mod pipeline;
pub mod rekey;

pub use doc::{DocFormat, Document, MergeOut, SegKind, Span};

#[cfg(test)]
mod mdx_tests;
