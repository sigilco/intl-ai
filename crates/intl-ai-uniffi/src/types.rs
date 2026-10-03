//! UniFFI-facing request/result types. They mirror the serde types in
//! `intl-ai-core` (which are not FFI-shaped: `usize`, `BTreeMap`, borrowed
//! strs) and convert field-by-field so the generated Swift/Kotlin API
//! stays close to the CLI's `--format json` contract.

use intl_ai_core::diff as core_diff;
use intl_ai_core::error::ErrorType;
use intl_ai_core::fill as core_fill;
use intl_ai_core::lockfile::Origin;
use serde::Serialize;
use std::collections::HashMap;

/// Format of an inline config string passed to `IntlAi::from_config_string`.
#[derive(Debug, Clone, Copy, uniffi::Enum)]
pub enum ConfigFormat {
    Toml,
    Json,
    Yaml,
}

impl From<ConfigFormat> for config::FileFormat {
    fn from(f: ConfigFormat) -> Self {
        match f {
            ConfigFormat::Toml => Self::Toml,
            ConfigFormat::Json => Self::Json,
            ConfigFormat::Yaml => Self::Yaml,
        }
    }
}

/// Origin scope for `CheckOptions.origin` (mirrors `check --origin`).
#[derive(Debug, Clone, Copy, uniffi::Enum)]
pub enum OriginFilter {
    Ai,
    Human,
}

impl From<OriginFilter> for Origin {
    fn from(o: OriginFilter) -> Self {
        match o {
            OriginFilter::Ai => Self::Ai,
            OriginFilter::Human => Self::Human,
        }
    }
}

/// Finding bucket names (mirrors `check --fail-on` values).
#[derive(Debug, Clone, Copy, uniffi::Enum)]
pub enum FindingKind {
    Missing,
    Stale,
    Invalid,
    Modified,
    Extra,
    Unreviewed,
}

impl From<FindingKind> for core_diff::FindingKind {
    fn from(k: FindingKind) -> Self {
        match k {
            FindingKind::Missing => Self::Missing,
            FindingKind::Stale => Self::Stale,
            FindingKind::Invalid => Self::Invalid,
            FindingKind::Modified => Self::Modified,
            FindingKind::Extra => Self::Extra,
            FindingKind::Unreviewed => Self::Unreviewed,
        }
    }
}

/// Failure classification on `FillFailure.kind` (mirrors core `ErrorType`;
/// serializes snake_case like the CLI JSON).
#[derive(Debug, Clone, Copy, Serialize, uniffi::Enum)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    RateLimit,
    Http,
    SpawnFailure,
    Timeout,
    ProcessExit,
    ParseError,
    OutputTruncated,
    Validation,
    Empty,
    Config,
    Lockfile,
    Io,
    Auth,
    Unknown,
}

impl From<ErrorType> for ErrorKind {
    fn from(t: ErrorType) -> Self {
        match t {
            ErrorType::RateLimit => Self::RateLimit,
            ErrorType::Http => Self::Http,
            ErrorType::SpawnFailure => Self::SpawnFailure,
            ErrorType::Timeout => Self::Timeout,
            ErrorType::ProcessExit => Self::ProcessExit,
            ErrorType::ParseError => Self::ParseError,
            ErrorType::OutputTruncated => Self::OutputTruncated,
            ErrorType::Validation => Self::Validation,
            ErrorType::Empty => Self::Empty,
            ErrorType::Config => Self::Config,
            ErrorType::Lockfile => Self::Lockfile,
            ErrorType::Io => Self::Io,
            ErrorType::Auth => Self::Auth,
            ErrorType::Unknown => Self::Unknown,
        }
    }
}

/// Options for `IntlAi.fill` (mirrors `intl-ai fill` flags).
#[derive(Debug, Clone, uniffi::Record)]
pub struct FillOptions {
    /// Target locales; empty = config targets.
    pub locales: Vec<String>,
    /// Key glob specs (e.g. "auth.*"); empty = all keys.
    pub keys: Vec<String>,
    /// File with one key glob per line; relative paths resolve against
    /// the config directory. Merged with `keys`.
    pub keys_file: Option<String>,
    /// Re-fill AI-owned keys whose source hash changed.
    pub stale: bool,
    /// Overwrite AI-owned existing values. Needs a `keys` scope or
    /// `yes = true`.
    pub regenerate: bool,
    /// With `regenerate`: also overwrite human-owned values.
    pub include_human: bool,
    /// Compute and report without writing locale files or shards.
    pub dry_run: bool,
    /// Skip the stat-cache read/write for this run.
    pub no_cache: bool,
    /// Validation-gate check names for this run (overrides the config
    /// `fill.validate` list). Ignored when `no_validate` is set.
    pub validate: Vec<String>,
    /// Disable the configured `fill.validate` gate for this run.
    pub no_validate: bool,
    /// Override the threshold of any judge check in the gate (0..=1).
    pub judge_threshold: Option<f64>,
    /// Confirm an unscoped `regenerate` (no key scope given).
    pub yes: bool,
}

/// Options for `IntlAi.check` (mirrors `intl-ai check` flags).
#[derive(Debug, Clone, uniffi::Record)]
pub struct CheckOptions {
    /// Target locales; empty = config targets.
    pub locales: Vec<String>,
    /// Key glob specs; empty = all keys.
    pub keys: Vec<String>,
    /// File with one key glob per line; relative paths resolve against
    /// the config directory. Merged with `keys`.
    pub keys_file: Option<String>,
    /// Scope stale/modified/unreviewed/invalid to one effective origin.
    pub origin: Option<OriginFilter>,
    /// Finding kinds that set `has_issues`. `None` = config
    /// `check.fail_on`; `Some(empty)` clears the gate.
    pub fail_on: Option<Vec<FindingKind>>,
    /// Skip the stat-cache read/write for this run.
    pub no_cache: bool,
}

/// One rule-level violation reported by a configured check.
#[derive(Debug, Clone, Serialize, uniffi::Record)]
pub struct CheckFinding {
    pub key: String,
    /// Check id that produced the violation (`icu`, spec id, exec name).
    pub check: String,
    pub message: String,
    /// Replayed from the incremental check cache, not re-run this time.
    pub cached: bool,
}

impl From<core_diff::CheckFinding> for CheckFinding {
    fn from(f: core_diff::CheckFinding) -> Self {
        Self {
            key: f.key,
            check: f.check,
            message: f.message,
            cached: f.cached,
        }
    }
}

/// Per-key aggregate quality score when `[quality]` is configured.
#[derive(Debug, Clone, Serialize, uniffi::Record)]
pub struct QualityScore {
    pub score: f64,
    pub band: String,
}

/// Per-locale structural diff plus rule violations.
#[derive(Debug, Clone, Serialize, uniffi::Record)]
pub struct LocaleDiff {
    pub missing: Vec<String>,
    pub stale: Vec<String>,
    pub modified: Vec<String>,
    pub extra: Vec<String>,
    pub unreviewed: Vec<String>,
    pub invalid: Vec<CheckFinding>,
    /// Empty unless `[quality]` is configured.
    pub quality: HashMap<String, QualityScore>,
}

impl From<core_diff::LocaleDiff> for LocaleDiff {
    fn from(d: core_diff::LocaleDiff) -> Self {
        Self {
            missing: d.missing,
            stale: d.stale,
            modified: d.modified,
            extra: d.extra,
            unreviewed: d.unreviewed,
            invalid: d.invalid.into_iter().map(Into::into).collect(),
            quality: d
                .quality
                .into_iter()
                .map(|(k, q)| {
                    (
                        k,
                        QualityScore {
                            score: q.score,
                            band: q.band.to_string(),
                        },
                    )
                })
                .collect(),
        }
    }
}

/// Result of `IntlAi.check`.
#[derive(Debug, Clone, Serialize, uniffi::Record)]
pub struct CheckReport {
    pub locales: HashMap<String, LocaleDiff>,
    /// Check-level failures (spec load, exec spawn, provider error).
    pub errors: Vec<String>,
    /// A `fail_on` kind or a check-level error fired.
    pub has_issues: bool,
    /// Any finding at all, independent of `fail_on`.
    pub has_findings: bool,
}

/// A fill-time failure; `key` is `None` for locale-level failures.
#[derive(Debug, Clone, Serialize, uniffi::Record)]
pub struct FillFailure {
    pub locale: String,
    pub key: Option<String>,
    pub kind: ErrorKind,
    pub message: String,
}

/// Per-locale outcome of `IntlAi.fill`.
#[derive(Debug, Clone, Serialize, uniffi::Record)]
pub struct LocaleFillReport {
    pub requested: u64,
    /// Provider answered for an in-chunk key.
    pub translated: u64,
    /// Keys adopted into file + shard.
    pub written: u64,
    /// Values regenerated that were human-owned (`include_human` tier).
    pub regenerated_human: u64,
    /// Shard entries pruned (key gone from source and target).
    pub pruned: u64,
    pub adopted_human: u64,
    pub reconciled_human: u64,
    /// Scoped keys with an existing AI-owned value left alone.
    pub skipped_existing: u64,
    /// Same set, human-owned subset.
    pub skipped_human: u64,
    /// In-chunk keys the provider omitted from its response.
    pub omitted: Vec<String>,
    /// Existing values destroyed by a write (leaf/shape replacement).
    pub clobbered: Vec<String>,
    /// Keys sent through a corrective round by the validation gate.
    pub refilled: u64,
    /// Gate findings still open after the last corrective round; the
    /// values were adopted anyway and live on the shard entry's
    /// `quality.unresolved`.
    pub unresolved: Vec<CheckFinding>,
}

impl From<core_fill::LocaleFillResult> for LocaleFillReport {
    fn from(r: core_fill::LocaleFillResult) -> Self {
        Self {
            requested: r.requested as u64,
            translated: r.translated as u64,
            written: r.written as u64,
            regenerated_human: r.regenerated_human as u64,
            pruned: r.pruned as u64,
            adopted_human: r.adopted_human as u64,
            reconciled_human: r.reconciled_human as u64,
            skipped_existing: r.skipped_existing as u64,
            skipped_human: r.skipped_human as u64,
            omitted: r.omitted,
            clobbered: r.clobbered,
            refilled: r.refilled as u64,
            unresolved: r.unresolved.into_iter().map(Into::into).collect(),
        }
    }
}

/// Result of `IntlAi.fill`.
#[derive(Debug, Clone, Serialize, uniffi::Record)]
pub struct FillReport {
    pub locales: HashMap<String, LocaleFillReport>,
    /// Per-key/per-locale failures; a failed batch marks every key in it.
    pub failures: Vec<FillFailure>,
    pub dry_run: bool,
}

/// One locale's inventory for `IntlAi.status` (same counts as
/// `intl-ai status --format json`).
#[derive(Debug, Clone, Serialize, uniffi::Record)]
pub struct LocaleStatus {
    pub locale: String,
    /// Total lockfile shard entries.
    pub entries: u64,
    pub ai: u64,
    pub human: u64,
    pub reviewed: u64,
    /// Deliberately untranslated tombstones.
    pub absent: u64,
    pub missing: u64,
    pub stale: u64,
    pub modified: u64,
    pub extra: u64,
    pub unreviewed: u64,
}
