//! `intl-ai-docs` configuration — a deliberately separate namespace
//! (`intl-ai-docs.{toml,json,yaml}`) so the corpus pipeline's config
//! never learns about document trees. Shape mirrors `intl-ai` config:
//! same provider block, glossary, per-locale instructions, batching,
//! `[[checks]]`, `[check]`/`[fill]`/`[quality]` sections — plus the
//! document-specific `source_dir`, `format`, `[links]`, `[frontmatter]`,
//! and the `experimental` gate.

use intl_ai_core::config::{
    CheckConfig, CheckEntry, FillConfig, ProviderConfig, QualityConfig, interpolate, resolve,
};
use intl_ai_core::error::{Error, Result};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const FILE_STEM: &str = "intl-ai-docs";
const EXTENSIONS: &[&str] = &["toml", "json", "yaml", "yml"];

#[derive(Debug, Clone, Deserialize, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DocsConfig {
    /// Config schema version; only 1 is understood.
    pub version: Option<u32>,
    /// Document support is experimental — every command refuses to run
    /// without this explicit opt-in.
    #[serde(default)]
    pub experimental: bool,
    /// Document tree root (e.g. "docs"), resolved against the config
    /// file's directory. Source documents live directly under it;
    /// each target locale gets a `<source_dir>/<locale>/` mirror.
    #[serde(default = "default_source_dir")]
    pub source_dir: PathBuf,
    /// Source locale id (its content is the tree under `source_dir`).
    #[serde(default = "default_source")]
    pub source: String,
    /// Target locales; each becomes `<source_dir>/<locale>/`.
    pub targets: Vec<String>,
    /// DocFormat name. Only "mdx" ships in v1 (markdown + MDX
    /// constructs); "markdown" is an accepted alias.
    #[serde(default = "default_format")]
    pub format: String,
    /// Tree layout: `flat` keeps the source documents directly under
    /// `source_dir` with targets at `source_dir/<locale>`; `locale-dirs`
    /// is the docmd i18n shape — the source tree lives at
    /// `source_dir/<source>` and every locale (source included) is a
    /// sibling directory under `source_dir`.
    #[serde(default)]
    pub layout: Layout,
    pub provider: ProviderConfig,
    #[serde(default)]
    pub glossary: BTreeMap<String, String>,
    /// Freeform style/dialect instructions per locale (exact, language
    /// subtag, then "*").
    #[serde(default)]
    pub locale_instructions: BTreeMap<String, String>,
    /// Max segments per translate request.
    pub batch_size: Option<usize>,
    /// Provider retries on retryable failures (default 3).
    pub max_retries: Option<u32>,
    #[serde(default)]
    pub links: LinksConfig,
    #[serde(default)]
    pub frontmatter: FrontmatterConfig,
    #[serde(default)]
    pub check: CheckConfig,
    #[serde(default)]
    pub fill: FillConfig,
    /// Corpus-style `[[checks]]` entries (spec/exec/judge/...), run
    /// alongside the always-on document builtins.
    #[serde(default)]
    pub checks: Vec<CheckEntry>,
    pub quality: Option<QualityConfig>,
    /// Glob patterns (vs document-relative paths) to exclude, on top of
    /// hidden dirs, `node_modules`, and the target locale dirs.
    #[serde(default)]
    pub exclude: Vec<String>,
}

/// Source/target tree arrangement.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, serde::Serialize, schemars::JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
pub enum Layout {
    /// Source files sit directly under `source_dir`; each target
    /// locale writes to `source_dir/<locale>`.
    #[default]
    Flat,
    /// docmd i18n shape: every locale is a sibling dir — source at
    /// `source_dir/<source>`, targets at `source_dir/<locale>`.
    LocaleDirs,
}

fn default_source_dir() -> PathBuf {
    PathBuf::from("docs")
}
fn default_source() -> String {
    "en".into()
}
fn default_format() -> String {
    "mdx".into()
}

#[derive(Debug, Clone, Deserialize, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LinksConfig {
    /// Insert the locale prefix into `/`-rooted link destinations.
    #[serde(default = "default_true")]
    pub rewrite: bool,
    /// Prefix template; `{locale}` is replaced per target locale.
    #[serde(default = "default_prefix")]
    pub prefix: String,
}

impl Default for LinksConfig {
    fn default() -> Self {
        Self {
            rewrite: true,
            prefix: default_prefix(),
        }
    }
}

fn default_prefix() -> String {
    "/{locale}".into()
}
fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FrontmatterConfig {
    /// Dotted frontmatter paths whose scalar values translate.
    #[serde(default = "default_fm_keys")]
    pub keys: Vec<String>,
}

impl Default for FrontmatterConfig {
    fn default() -> Self {
        Self {
            keys: default_fm_keys(),
        }
    }
}

fn default_fm_keys() -> Vec<String> {
    vec!["title".into(), "description".into()]
}

pub struct ResolvedDocsConfig {
    pub config: DocsConfig,
    pub config_dir: PathBuf,
    pub config_path: Option<PathBuf>,
}

impl ResolvedDocsConfig {
    /// The directory holding source documents: `source_dir` for flat
    /// layout, `source_dir/<source>` for locale-dirs.
    pub fn source_dir(&self) -> PathBuf {
        let base = resolve(&self.config_dir, &self.config.source_dir);
        match self.config.layout {
            Layout::Flat => base,
            Layout::LocaleDirs => base.join(&self.config.source),
        }
    }

    /// `source_dir/<locale>/` for flat layout (sibling of the source
    /// files); `source_dir/<locale>/` at the tree root for locale-dirs
    /// (sibling of `source_dir/<source>`).
    pub fn target_dir(&self, locale: &str) -> PathBuf {
        resolve(&self.config_dir, &self.config.source_dir).join(locale)
    }

    /// `intl-ai-docs.lock.d/` next to the config file.
    pub fn lock_dir(&self) -> PathBuf {
        self.config_dir.join("intl-ai-docs.lock.d")
    }

    /// Per-document shard: `intl-ai-docs.lock.d/<locale>/<relpath>.toml`.
    pub fn shard_path(&self, locale: &str, relpath: &str) -> PathBuf {
        self.lock_dir().join(locale).join(format!("{relpath}.toml"))
    }

    /// Incremental check cache shared by all document runs.
    pub fn check_cache_path(&self) -> PathBuf {
        self.lock_dir().join("check-cache.json")
    }

    /// Style/dialect instruction for a target locale: exact, then
    /// language subtag, then "*" (same rule as the corpus config).
    pub fn instruction_for(&self, locale: &str) -> Option<String> {
        let m = &self.config.locale_instructions;
        m.get(locale)
            .or_else(|| locale.split(['-', '_']).next().and_then(|lang| m.get(lang)))
            .or_else(|| m.get("*"))
            .cloned()
    }

    pub fn max_retries(&self) -> u32 {
        self.config.max_retries.unwrap_or(3)
    }

    /// Link prefix for a locale ("" disables rewriting).
    pub fn link_prefix(&self, locale: &str) -> String {
        if self.config.links.rewrite {
            self.config.links.prefix.replace("{locale}", locale)
        } else {
            String::new()
        }
    }
}

/// Finds `intl-ai-docs.{toml,json,yaml,yml}` walking ancestors from
/// `cwd` (a `.git` dir stops the walk), or honors `--config`.
pub fn discover(cwd: &Path) -> Option<PathBuf> {
    let mut dir = Some(cwd);
    while let Some(d) = dir {
        if let Some(p) = EXTENSIONS
            .iter()
            .map(|ext| d.join(format!("{FILE_STEM}.{ext}")))
            .find(|p| p.is_file())
        {
            return Some(p);
        }
        if d.join(".git").exists() {
            return None;
        }
        dir = d.parent();
    }
    None
}

pub fn load(config_arg: Option<&Path>, cwd: &Path) -> Result<ResolvedDocsConfig> {
    let path = match config_arg {
        Some(p) => resolve(cwd, p),
        None => discover(cwd).ok_or_else(|| {
            Error::Config(format!(
                "no intl-ai-docs.{{toml,json,yaml,yml}} found in {}",
                cwd.display()
            ))
        })?,
    };
    let config_dir = path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| cwd.to_path_buf());

    let built = config::Config::builder()
        .add_source(config::File::from(path.clone()))
        .add_source(
            config::Environment::with_prefix("INTL_AI_DOCS")
                .separator("__")
                .try_parsing(true),
        )
        .build()
        .map_err(|e| Error::Config(format!("{}: {e}", path.display())))?;
    // ${env:}/${file:} interpolation on the raw tree (same rule as the
    // corpus config: secrets never sit in the file verbatim).
    let mut raw: serde_json::Value = built
        .try_deserialize()
        .map_err(|e| Error::Config(format!("{}: {e}", path.display())))?;
    interpolate(&mut raw, &config_dir)?;
    let config: DocsConfig = serde_json::from_value(raw)
        .map_err(|e| Error::Config(format!("{}: {e}", path.display())))?;

    let resolved = ResolvedDocsConfig {
        config,
        config_dir,
        config_path: Some(path),
    };
    resolved.validate()?;
    Ok(resolved)
}

impl ResolvedDocsConfig {
    fn validate(&self) -> Result<()> {
        let c = &self.config;
        if c.version.is_some_and(|v| v != 1) {
            return Err(Error::Config(format!(
                "intl-ai-docs: unsupported version {}; expected 1",
                c.version.unwrap()
            )));
        }
        if !c.experimental {
            return Err(Error::Config(
                "intl-ai-docs: document support is experimental — set `experimental = true` to opt in"
                    .into(),
            ));
        }
        if c.targets.is_empty() {
            return Err(Error::Config(
                "intl-ai-docs: `targets` must list at least one locale".into(),
            ));
        }
        if !matches!(c.format.as_str(), "mdx" | "markdown") {
            return Err(Error::Config(format!(
                "intl-ai-docs: unknown format {:?} (supported: mdx)",
                c.format
            )));
        }
        if c.targets.iter().any(|t| t == &c.source) {
            return Err(Error::Config(
                "intl-ai-docs: `source` must not also appear in `targets`".into(),
            ));
        }
        Ok(())
    }
}
