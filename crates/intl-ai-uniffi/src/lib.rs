//! intl-ai UniFFI bindings: `fill`, `check`, and `status` for foreign
//! languages (Swift, Kotlin, and anything else UniFFI generates).
//!
//! The core engine's `Transport` trait is synchronous (blocking HTTP via
//! `ureq`, subprocess commands, replay cassettes), so the plain
//! `fill`/`check`/`status` calls block the calling thread. The `*_async`
//! variants run the same pipeline on a dedicated worker thread and
//! stream `ProgressEvent`s to an `IntlAiProgress` observer; the crate
//! has no async runtime, so the bridge is `std::thread` + a oneshot
//! (see `progress.rs`).

mod error;
mod progress;
mod types;

pub use error::IntlAiError;
pub use progress::*;
pub use types::*;

use intl_ai_core::check::{self as core_check, CheckOptions as CoreCheckOptions};
use intl_ai_core::config::{self as core_config, ResolvedConfig};
use intl_ai_core::fill::{self as core_fill, FillOptions as CoreFillOptions};
use intl_ai_core::lockfile::{Origin, load_shard};
use intl_ai_core::progress::Progress as CoreProgress;
use intl_ai_core::selector::KeySelector;
use std::path::{Path, PathBuf};
use std::sync::Arc;

uniffi::setup_scaffolding!();

/// A resolved intl-ai configuration plus the methods that run against it.
/// Construct once per project; the object is immutable and safe to share
/// across threads (UniFFI wraps it in an `Arc`).
#[derive(uniffi::Object)]
pub struct IntlAi {
    /// `Arc` so the `*_async` methods can move the config onto a worker
    /// thread without cloning it.
    cfg: Arc<ResolvedConfig>,
}

#[uniffi::export]
impl IntlAi {
    /// Load `intl-ai.{toml,json,yaml}` from `config_path`, or discover it
    /// under `working_dir` when `config_path` is `None`. `working_dir`
    /// defaults to the process working directory; relative paths inside
    /// the config resolve against the config file's directory.
    #[uniffi::constructor]
    pub fn new(
        config_path: Option<String>,
        working_dir: Option<String>,
    ) -> Result<Arc<Self>, IntlAiError> {
        let cwd = resolve_cwd(working_dir)?;
        let cfg = core_config::load(config_path.as_deref().map(Path::new), &cwd)?;
        Ok(Arc::new(Self { cfg: Arc::new(cfg) }))
    }

    /// Parse an inline config string instead of reading a file. Paths
    /// inside it resolve against `working_dir` (default: process cwd).
    /// `extends` is rejected for inline configs (see `config::load_from_str`).
    #[uniffi::constructor]
    pub fn from_config_string(
        config: String,
        format: ConfigFormat,
        working_dir: Option<String>,
    ) -> Result<Arc<Self>, IntlAiError> {
        let cwd = resolve_cwd(working_dir)?;
        let cfg = core_config::load_from_str(&config, format.into(), &cwd)?;
        Ok(Arc::new(Self { cfg: Arc::new(cfg) }))
    }

    /// Configured source locale.
    pub fn source_locale(&self) -> String {
        self.cfg.config.source.clone()
    }

    /// Configured target locales.
    pub fn target_locales(&self) -> Vec<String> {
        self.cfg.config.targets.clone()
    }

    /// Absolute path of the resolved locale directory.
    pub fn locale_dir(&self) -> String {
        self.cfg.locale_dir().display().to_string()
    }

    /// Translate missing (or scoped stale/regenerated) keys into target
    /// locale files. Same behavior as `intl-ai fill`; blocking.
    pub fn fill(&self, options: FillOptions) -> Result<FillReport, IntlAiError> {
        Ok(Self::run_fill(&self.cfg, &options, None)?.into())
    }

    /// Same run as `fill`, returning the exact JSON the CLI emits with
    /// `--format json`.
    pub fn fill_json(&self, options: FillOptions) -> Result<String, IntlAiError> {
        Ok(serde_json::to_string_pretty(&Self::run_fill(
            &self.cfg, &options, None,
        )?)?)
    }

    /// Async `fill`: the pipeline runs on a dedicated worker thread and
    /// `observer` (when set) receives every `ProgressEvent` in order on
    /// that thread. The returned future never blocks the foreign
    /// executor thread.
    pub async fn fill_async(
        &self,
        options: FillOptions,
        observer: Option<Arc<dyn IntlAiProgress>>,
    ) -> Result<FillReport, IntlAiError> {
        let cfg = Arc::clone(&self.cfg);
        let observer = observer.map(progress::core_observer);
        let report =
            progress::run_blocking(move || Self::run_fill(&cfg, &options, observer)).await??;
        Ok(report.into())
    }

    /// Report missing/stale/modified/unreviewed findings without writing.
    /// Same behavior as `intl-ai check`; blocking.
    pub fn check(&self, options: CheckOptions) -> Result<CheckReport, IntlAiError> {
        Ok(Self::run_check(&self.cfg, &options, None)?.into())
    }

    /// Same run as `check`, returning the exact JSON the CLI emits with
    /// `--format json`.
    pub fn check_json(&self, options: CheckOptions) -> Result<String, IntlAiError> {
        Ok(serde_json::to_string_pretty(&Self::run_check(
            &self.cfg, &options, None,
        )?)?)
    }

    /// Async `check`: same worker-thread/observer contract as
    /// `fill_async`.
    pub async fn check_async(
        &self,
        options: CheckOptions,
        observer: Option<Arc<dyn IntlAiProgress>>,
    ) -> Result<CheckReport, IntlAiError> {
        let cfg = Arc::clone(&self.cfg);
        let observer = observer.map(progress::core_observer);
        let report =
            progress::run_blocking(move || Self::run_check(&cfg, &options, observer)).await??;
        Ok(report.into())
    }

    /// Per-locale inventory counts (same as `intl-ai status`). Read-only:
    /// no configured checks run and no transport is resolved. `locales`
    /// empty = config targets.
    pub fn status(&self, locales: Vec<String>) -> Result<Vec<LocaleStatus>, IntlAiError> {
        self.run_status(&locales)
    }

    /// Same run as `status`, returning `{"locales": [...]}` like
    /// `intl-ai status --format json`.
    pub fn status_json(&self, locales: Vec<String>) -> Result<String, IntlAiError> {
        let statuses = self.run_status(&locales)?;
        Ok(serde_json::to_string_pretty(
            &serde_json::json!({ "locales": statuses }),
        )?)
    }
}

impl IntlAi {
    fn run_fill(
        cfg: &ResolvedConfig,
        options: &FillOptions,
        observer: Option<Arc<dyn CoreProgress>>,
    ) -> Result<core_fill::FillReport, IntlAiError> {
        let selector = Self::selector(cfg, &options.keys, options.keys_file.as_deref())?;
        // Destructive-tier guard (same as the CLI): rewriting every
        // AI-owned value needs an explicit scope or an explicit yes.
        if options.regenerate && selector.is_any() && !options.yes {
            return Err(IntlAiError::Validation(
                "regenerate overwrites every AI-owned value in the selected locales; \
                 scope it with keys/keys_file, or set yes to confirm"
                    .into(),
            ));
        }
        let gate_names = if !options.validate.is_empty() {
            options.validate.clone()
        } else if options.no_validate {
            Vec::new()
        } else {
            cfg.config.fill.validate.clone()
        };
        if let Some(t) = options.judge_threshold {
            if !(0.0..=1.0).contains(&t) {
                return Err(IntlAiError::Validation(format!(
                    "judge_threshold must be in 0..=1 (got {t})"
                )));
            }
        }
        let gate = intl_ai_checks::build_gate(cfg, &gate_names, options.judge_threshold)?;
        let transport = intl_ai_providers::build_transport(cfg)?;
        let opts = CoreFillOptions {
            locales: non_empty(&options.locales),
            selector,
            stale_only: options.stale,
            regenerate: options.regenerate,
            include_human: options.include_human,
            dry_run: options.dry_run,
            no_cache: options.no_cache,
            observer,
        };
        Ok(core_fill::fill(
            cfg,
            transport.as_ref(),
            &opts,
            gate.as_ref(),
        )?)
    }

    fn run_check(
        cfg: &ResolvedConfig,
        options: &CheckOptions,
        observer: Option<Arc<dyn CoreProgress>>,
    ) -> Result<core_check::CheckReport, IntlAiError> {
        let checks = intl_ai_checks::build(cfg)?;
        let transport = if checks.iter().any(|c| c.needs_transport()) {
            Some(intl_ai_providers::build_transport(cfg)?)
        } else {
            None
        };
        let opts = CoreCheckOptions {
            locales: non_empty(&options.locales),
            origin_filter: options.origin.map(Into::into),
            fail_on: options
                .fail_on
                .clone()
                .map(|kinds| kinds.into_iter().map(Into::into).collect()),
            selector: Self::selector(cfg, &options.keys, options.keys_file.as_deref())?,
            no_cache: options.no_cache,
            observer,
        };
        Ok(core_check::check(
            cfg,
            &opts,
            &checks,
            transport.as_deref(),
        )?)
    }

    fn run_status(&self, locales: &[String]) -> Result<Vec<LocaleStatus>, IntlAiError> {
        let opts = CoreCheckOptions {
            locales: non_empty(locales),
            origin_filter: None,
            fail_on: Some(vec![]),
            selector: KeySelector::any(),
            no_cache: false,
            observer: None,
        };
        let report = core_check::check(&self.cfg, &opts, &[], None)?;

        let mut out = Vec::new();
        for (locale, d) in &report.locales {
            let shard = load_shard(&self.cfg.locale_dir(), locale)?;
            let mut ai = 0u64;
            let mut human = 0u64;
            let mut reviewed = 0u64;
            let mut absent = 0u64;
            for e in shard.entries.values() {
                match e.origin {
                    Origin::Ai => ai += 1,
                    Origin::Human => human += 1,
                }
                if e.reviewed {
                    reviewed += 1;
                }
                if e.absent {
                    absent += 1;
                }
            }
            out.push(LocaleStatus {
                locale: locale.clone(),
                entries: shard.entries.len() as u64,
                ai,
                human,
                reviewed,
                absent,
                missing: d.missing.len() as u64,
                stale: d.stale.len() as u64,
                modified: d.modified.len() as u64,
                extra: d.extra.len() as u64,
                unreviewed: d.unreviewed.len() as u64,
            });
        }
        Ok(out)
    }

    /// Merge `keys` specs with an optional `keys_file` (relative paths
    /// resolve against the config directory).
    fn selector(
        cfg: &ResolvedConfig,
        keys: &[String],
        keys_file: Option<&str>,
    ) -> Result<KeySelector, IntlAiError> {
        let mut sel = KeySelector::from_specs(keys)?;
        if let Some(path) = keys_file {
            let path = cfg.config_dir.join(path);
            sel = sel.merge(KeySelector::from_file(&path)?)?;
        }
        Ok(sel)
    }
}

fn resolve_cwd(working_dir: Option<String>) -> Result<PathBuf, IntlAiError> {
    match working_dir {
        Some(d) => Ok(PathBuf::from(d)),
        None => std::env::current_dir().map_err(|e| IntlAiError::Io(e.to_string())),
    }
}

fn non_empty(locales: &[String]) -> Option<Vec<String>> {
    (!locales.is_empty()).then(|| locales.to_vec())
}

impl From<core_check::CheckReport> for CheckReport {
    fn from(r: core_check::CheckReport) -> Self {
        Self {
            locales: r.locales.into_iter().map(|(k, v)| (k, v.into())).collect(),
            errors: r.errors,
            has_issues: r.has_issues,
            has_findings: r.has_findings,
        }
    }
}

impl From<core_fill::FillReport> for FillReport {
    fn from(r: core_fill::FillReport) -> Self {
        Self {
            locales: r.locales.into_iter().map(|(k, v)| (k, v.into())).collect(),
            failures: r
                .failures
                .into_iter()
                .map(|f| FillFailure {
                    locale: f.locale,
                    key: f.key,
                    kind: f.kind.into(),
                    message: f.message,
                })
                .collect(),
            dry_run: r.dry_run,
        }
    }
}
