use intl_ai_core::error::Result;
use intl_ai_core::flatten::FlatMap;
use std::fmt;
use std::ops::Range;
use std::path::Path;

/// Translatable leaf classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegKind {
    Heading,
    Paragraph,
    Cell,
    Frontmatter,
}

/// A splice site in `source`: `merge` replaces `range` with the target
/// value for `seg` (or keeps the source text when untranslated).
#[derive(Debug, Clone)]
pub struct Span {
    pub range: Range<usize>,
    pub seg: String,
    pub kind: SegKind,
}

/// Extracted document: original text + every replacement site + the
/// FlatMap the engine runs on.
#[derive(Debug)]
pub struct Document {
    /// Original document text.
    pub source: String,
    /// Replacement sites in document order. Duplicate seg ids are
    /// expected: identical source text shares one segment (and one
    /// translation), so a repeated paragraph splices from the same key.
    pub spans: Vec<Span>,
    /// seg id -> source segment text (raw markdown incl. inline markup).
    pub segments: FlatMap,
    /// seg ids in first-appearance order.
    pub order: Vec<String>,
    /// Non-fatal extraction observations (kept opaque by design).
    pub warnings: Vec<String>,
}

pub struct MergeOut {
    pub bytes: Vec<u8>,
    /// Segments with no target value — source text kept (po4a-style
    /// fallback so a partially translated doc stays complete).
    pub fell_back: Vec<String>,
}

/// A document surface: `extract` produces skeleton + segment FlatMap,
/// `merge` splices a translated FlatMap back into the skeleton.
pub trait DocFormat: fmt::Debug + Send + Sync {
    fn name(&self) -> &str;
    fn extensions(&self) -> &'static [&'static str];
    fn extract(&self, text: &str, path: &Path) -> Result<Document>;
    /// Splice `target` values into the skeleton. `link_prefix` (e.g.
    /// "/es") is prepended to `/`-rooted link destinations; empty
    /// disables rewriting.
    fn merge(&self, doc: &Document, target: &FlatMap, link_prefix: &str) -> Result<MergeOut>;
}
