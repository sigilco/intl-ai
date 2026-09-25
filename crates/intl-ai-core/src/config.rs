use crate::diff::FindingKind;
use crate::error::{Error, Result};
use config::{Config, Environment, File, FileFormat};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const FILE_STEM: &str = "intl-ai";
const EXTENSIONS: [&str; 4] = ["toml", "json", "yaml", "yml"];
const MAX_EXTENDS_DEPTH: usize = 8;

/// One typed contract with `deny_unknown_fields` regardless of the file
/// format the user picked (plan section 7).
#[derive(Debug, Clone, Deserialize, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct IntlAiConfig {
    #[serde(default = "default_version")]
    pub version: u32,
    /// Consumed by the loader; kept in the struct so `config validate`
    /// can display it.
    #[serde(default)]
    pub extends: Option<StringOrList>,
    pub locale_dir: PathBuf,
    pub source: String,
    pub targets: Vec<String>,
    pub provider: ProviderConfig,
    #[serde(default)]
    pub check: CheckConfig,
    /// Fixed term -> translation pairs injected into every translate prompt.
    #[serde(default)]
    pub glossary: std::collections::BTreeMap<String, String>,
    /// Freeform style/dialect instruction per locale. Resolution: exact
    /// locale, then language subtag, then `*` as catch-all.
    #[serde(default)]
    pub locale_instructions: std::collections::BTreeMap<String, String>,
    /// Max source entries per translate request (plan 5.3 bugfix: honored
    /// on the initial pass too, not only refills). None = one request.
    #[serde(default)]
    pub batch_size: Option<usize>,
    /// Transport attempts per request before the batch fails (default 3,
    /// capped at 10).
    #[serde(default)]
    pub max_retries: Option<u32>,
    /// Placeholder contract hint sent to the model. `icu` selects the ICU
    /// MessageFormat hint; the ICU validator itself lands with W2 checks.
    #[serde(default)]
    pub processor: Option<ProcessorKind>,
    /// Locale file format minted for new files (plan 5.5: an existing
    /// file's own extension always wins over this preference).
    #[serde(default)]
    pub format: Option<intl_ai_formats::FileFormat>,
    /// Check entries (`[[checks]]`): builtins by `id`, declarative YAML
    /// specs by `spec`, external checkers by `exec` (v1 JSONL protocol).
    #[serde(default)]
    pub checks: Vec<CheckEntry>,
}

/// One `[[checks]]` entry: exactly one of `id`, `spec`, `exec`.
#[derive(Debug, Clone, Deserialize, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CheckEntry {
    /// Builtin check id: `icu`, `placeholder-parity`, `dialect:<locale>`,
    /// `judge`.
    pub id: Option<String>,
    /// Declarative YAML check spec, resolved against the config file's
    /// directory.
    pub spec: Option<PathBuf>,
    /// Exec check command (v1 JSONL protocol: one request line, one
    /// response line). Root config only, never via `extends` (same
    /// boundary as provider.command).
    pub exec: Option<String>,
    pub args: Option<Vec<String>>,
    /// Exec working directory, resolved against the config file's
    /// directory.
    pub cwd: Option<PathBuf>,
    /// Exec wall-clock budget (default 60_000 ms).
    pub timeout_ms: Option<u64>,
    /// Exec buffered stdout cap (default 1 MiB).
    pub max_stdout_bytes: Option<u64>,
}

impl CheckEntry {
    /// Human label for errors and reports.
    pub fn label(&self) -> String {
        self.id
            .clone()
            .or_else(|| self.spec.as_ref().map(|p| p.display().to_string()))
            .or_else(|| self.exec.clone())
            .unwrap_or_else(|| "<empty check entry>".into())
    }
}

#[derive(Debug, Clone, Copy, Deserialize, serde::Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProcessorKind {
    Passthrough,
    Icu,
}

impl ProcessorKind {
    pub fn syntax_hint(&self) -> &'static str {
        match self {
            Self::Passthrough => {
                "Preserve any placeholders like {variable} exactly as they appear."
            }
            Self::Icu => {
                "ICU MessageFormat: Use {variable} for placeholders, e.g., \"Hello {name}\". Supports plural/select syntax."
            }
        }
    }
}

fn default_version() -> u32 {
    1
}

#[derive(Debug, Clone, Deserialize, serde::Serialize, schemars::JsonSchema)]
#[serde(untagged)]
pub enum StringOrList {
    One(String),
    Many(Vec<String>),
}

impl StringOrList {
    pub fn as_vec(&self) -> Vec<String> {
        match self {
            Self::One(s) => vec![s.clone()],
            Self::Many(v) => v.clone(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, serde::Serialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProviderConfig {
    Replay(ReplayProvider),
    Http(HttpProvider),
    Command(CommandProvider),
}

#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReplayProvider {
    /// Cassette JSON: `{ "<locale>": { "<source value>": "<translation>" } }`.
    /// Resolved against the config file's directory.
    pub file: PathBuf,
}

impl serde::Serialize for ReplayProvider {
    fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        s.collect_map([("file", &self.file)])
    }
}

/// OpenAI-compatible chat-completions surface (plan 5.1.9): every surveyed
/// tool converges on api-url + api-key + model. `provider` selects the wire
/// shape; only "openai" is wired in W1.
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HttpProvider {
    #[serde(default = "default_http_provider")]
    pub provider: String,
    pub model: String,
    /// Resolved at load via `${env:VAR}`/`${file:PATH}` interpolation.
    /// Serialized masked so `config validate` never leaks it.
    pub api_key: String,
    /// Optional override; defaults to the provider's canonical URL.
    pub base_url: Option<String>,
    /// Extra request-body params, spread LAST so user params win (plan 5.1.9).
    #[serde(default)]
    pub model_params: Option<serde_json::Map<String, serde_json::Value>>,
}

fn default_http_provider() -> String {
    "openai".into()
}

impl serde::Serialize for HttpProvider {
    fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        s.collect_map([
            ("provider", serde_json::json!(self.provider)),
            ("model", serde_json::json!(self.model)),
            ("api_key", serde_json::json!("********")),
            ("base_url", serde_json::json!(self.base_url)),
            ("model_params", serde_json::json!(self.model_params)),
        ])
    }
}

/// Headless coding-agent CLI as a transport (the v1 differentiator: keyless
/// dev). Either `agent` (a preset) or `command`+optional `args`; `command`
/// wins when both are set (plan 5.1.6).
#[derive(Debug, Clone, Deserialize, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CommandProvider {
    /// Preset name from the agent table (kebab-case, see providers/presets).
    pub agent: Option<AgentPreset>,
    /// Free-form command. Allowed only in the root config file, never via
    /// `extends` (plan 5.1.6: data files from a dependency must not spawn
    /// arbitrary programs).
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    /// Where the prompt travels. Default stdin; `argv` appends it as the
    /// last arg (crush-style CLIs whose reader ignores stdin).
    pub prompt_via: Option<PromptVia>,
    /// Working directory for the spawned agent, resolved against the
    /// config file's directory.
    pub cwd: Option<PathBuf>,
    /// Wall-clock budget before SIGTERM -> 5s -> SIGKILL (default 300_000).
    pub timeout_ms: Option<u64>,
    /// Buffered stdout cap before the process is killed (default 10 MiB).
    pub max_stdout_bytes: Option<u64>,
}

#[derive(Debug, Clone, Copy, Deserialize, serde::Serialize, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum AgentPreset {
    ClaudeCode,
    Opencode,
    Codex,
    Crush,
    Gemini,
}

#[derive(Debug, Clone, Copy, Deserialize, serde::Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PromptVia {
    Stdin,
    Argv,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CheckConfig {
    #[serde(default = "default_fail_on")]
    pub fail_on: Vec<FindingKind>,
}

impl Default for CheckConfig {
    fn default() -> Self {
        Self {
            fail_on: default_fail_on(),
        }
    }
}

fn default_fail_on() -> Vec<FindingKind> {
    // `missing` is gated by default: a fill that omits keys must not ship
    // green (fail-first posture, plan 5.6).
    vec![
        FindingKind::Missing,
        FindingKind::Stale,
        FindingKind::Invalid,
    ]
}

#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    pub config: IntlAiConfig,
    /// Directory the config file lives in; relative paths resolve here.
    pub config_dir: PathBuf,
    pub config_path: Option<PathBuf>,
}

impl ResolvedConfig {
    pub fn locale_dir(&self) -> PathBuf {
        resolve(&self.config_dir, &self.config.locale_dir)
    }

    pub fn replay_file(&self) -> Option<PathBuf> {
        match &self.config.provider {
            ProviderConfig::Replay(r) => Some(resolve(&self.config_dir, &r.file)),
            _ => None,
        }
    }

    /// Agent `cwd`, resolved against the config file's dir (plan 5.1.6).
    pub fn command_cwd(&self) -> Option<PathBuf> {
        match &self.config.provider {
            ProviderConfig::Command(c) => c.cwd.as_ref().map(|p| resolve(&self.config_dir, p)),
            _ => None,
        }
    }

    /// Style/dialect instruction for a target locale: exact, then language
    /// subtag ("fr-CA" -> "fr"), then "*" catch-all (plan 5.4).
    pub fn instruction_for(&self, locale: &str) -> Option<String> {
        let m = &self.config.locale_instructions;
        m.get(locale)
            .or_else(|| locale.split(['-', '_']).next().and_then(|lang| m.get(lang)))
            .or_else(|| m.get("*"))
            .cloned()
    }

    pub fn syntax_hint(&self) -> &'static str {
        self.config
            .processor
            .as_ref()
            .map(|p| p.syntax_hint())
            .unwrap_or_else(|| ProcessorKind::Passthrough.syntax_hint())
    }

    pub fn max_retries(&self) -> u32 {
        self.config.max_retries.unwrap_or(3)
    }

    /// Preferred format for minting new locale files (default JSON).
    pub fn file_format(&self) -> intl_ai_formats::FileFormat {
        self.config.format.unwrap_or_default()
    }

    /// Locale file path for `locale`: an existing file's extension wins,
    /// else the configured `format` mints it (plan 5.5).
    pub fn locale_path(&self, locale: &str) -> PathBuf {
        intl_ai_formats::resolve(&self.locale_dir(), locale, self.file_format())
    }

    /// Every on-disk file this locale could resolve to, in resolve order.
    /// `len() > 1` means shadowed siblings (e.g. fr.json AND fr.yaml);
    /// callers warn once per locale rather than silently picking.
    pub fn shadowed_locale_files(&self, locale: &str) -> Vec<PathBuf> {
        ["json", "yaml", "yml"]
            .iter()
            .map(|ext| self.locale_dir().join(format!("{locale}.{ext}")))
            .filter(|p| p.is_file())
            .collect()
    }

    /// Gitignored stat-cache path, `.intl-ai/cache.json` next to the
    /// config file.
    pub fn cache_path(&self) -> PathBuf {
        self.config_dir.join(".intl-ai/cache.json")
    }
}

/// JSON Schema for the typed config contract. Generated, so `config
/// validate` and the committed schema file can never drift apart.
pub fn json_schema() -> serde_json::Value {
    serde_json::to_value(schemars::schema_for!(IntlAiConfig))
        .expect("schema serialization is infallible")
}

fn resolve(base: &Path, p: &Path) -> PathBuf {
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        base.join(p)
    }
}

/// Finds `intl-ai.{toml,json,yaml,yml}` in `cwd` (first hit wins) or honors
/// `--config`. `--config -` reads TOML from stdin.
pub fn discover(cwd: &Path) -> Option<PathBuf> {
    // Walk ancestors so `fill`/`check` work from any subdirectory; a `.git`
    // dir marks the repo boundary and stops the walk (M5).
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

pub fn load(config_arg: Option<&Path>, cwd: &Path) -> Result<ResolvedConfig> {
    let path = match config_arg {
        Some(p) => resolve(cwd, p),
        None => discover(cwd).ok_or_else(|| {
            Error::Config(format!(
                "no intl-ai.{{toml,json,yaml,yml}} found in {}",
                cwd.display()
            ))
        })?,
    };
    let config_dir = path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| cwd.to_path_buf());

    let bases = collect_extends(&path, &config_dir)?;
    let mut builder = Config::builder();
    for base in &bases {
        builder = builder.add_source(File::from(base.clone()));
    }
    builder = builder
        .add_source(File::from(path.clone()))
        .add_source(env_overlay());
    finish(builder, config_dir, Some(path))
}

pub fn load_from_str(text: &str, format: FileFormat, cwd: &Path) -> Result<ResolvedConfig> {
    let builder = Config::builder()
        .add_source(File::from_str(text, format))
        .add_source(env_overlay());
    let resolved = finish(builder, cwd.to_path_buf(), None)?;
    if resolved.config.extends.is_some() {
        return Err(Error::Config(
            "extends needs a config file on disk; --config - cannot resolve relative extends"
                .into(),
        ));
    }
    Ok(resolved)
}

fn env_overlay() -> Environment {
    Environment::with_prefix("INTL_AI")
        .separator("__")
        .try_parsing(true)
}

/// Depth-first extends chain, ordered so later sources override earlier
/// (base files first, the extending file last). Cycles and depth overruns
/// are hard config errors.
fn collect_extends(path: &Path, config_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut ordered = Vec::new();
    // `stack` is the current recursion path (true cycles); `seen` dedups
    // files already merged (a diamond `extends` is a DAG, not a cycle).
    let mut stack = HashSet::new();
    let mut seen = HashSet::new();
    walk_extends(path, config_dir, &mut stack, &mut seen, &mut ordered, 0)?;
    Ok(ordered)
}

fn walk_extends(
    path: &Path,
    config_dir: &Path,
    stack: &mut HashSet<PathBuf>,
    seen: &mut HashSet<PathBuf>,
    ordered: &mut Vec<PathBuf>,
    depth: usize,
) -> Result<()> {
    if depth > MAX_EXTENDS_DEPTH {
        return Err(Error::Config(format!(
            "extends chain deeper than {MAX_EXTENDS_DEPTH} at {}",
            path.display()
        )));
    }
    let canon = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    // `stack` is the active recursion path (true cycles); `seen` dedups files
    // already fully merged (a diamond is a DAG, not a cycle).
    if stack.contains(&canon) {
        return Err(Error::Config(format!(
            "extends cycle at {}",
            path.display()
        )));
    }
    if !seen.insert(canon.clone()) {
        return Ok(());
    }
    stack.insert(canon.clone());
    let cfg = Config::builder()
        .add_source(File::from(path.to_path_buf()))
        .build()
        .map_err(|e| Error::Config(format!("{}: {e}", path.display())))?;
    let raw: Value = cfg
        .try_deserialize()
        .map_err(|e| Error::Config(format!("{}: {e}", path.display())))?;
    // A base file is data, never code: it must not spawn programs, redirect
    // the provider endpoint (which carries api_key as a Bearer token), or
    // interpolate ${env:}/${file:} secrets into prompt-bound fields
    // (plan 5.1.6 boundary).
    if depth > 0 {
        if let Some(field) = first_interpolated(&raw) {
            return Err(Error::Config(format!(
                "{}: {field} uses ${{...}} interpolation, only allowed in the root config file, not via extends",
                path.display()
            )));
        }
        if let Some(provider) = raw.get("provider") {
            for field in ["command", "agent", "base_url"] {
                if provider.get(field).is_some_and(|c| !c.is_null()) {
                    return Err(Error::Config(format!(
                        "{}: provider.{field} is only allowed in the root config file, not via extends",
                        path.display()
                    )));
                }
            }
        }
        if let Some(checks) = raw.get("checks") {
            let has_exec = checks.as_array().is_some_and(|arr| {
                arr.iter()
                    .any(|c| c.get("exec").is_some_and(|e| !e.is_null()))
            });
            if has_exec {
                return Err(Error::Config(format!(
                    "{}: [[checks]] exec is only allowed in the root config file, not via extends",
                    path.display()
                )));
            }
        }
    }
    // Absent or mistyped `extends` is treated as absent.
    let extends: Option<StringOrList> = raw
        .get("extends")
        .cloned()
        .and_then(|v| serde_json::from_value(v).ok());
    let parent_dir = path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| config_dir.to_path_buf());
    if let Some(list) = extends {
        for base in list.as_vec() {
            let base_path = resolve(&parent_dir, Path::new(&base));
            if !base_path.is_file() {
                return Err(Error::Config(format!(
                    "extends target {} does not exist (from {})",
                    base_path.display(),
                    path.display()
                )));
            }
            walk_extends(&base_path, config_dir, stack, seen, ordered, depth + 1)?;
        }
    }
    stack.remove(&canon);
    ordered.push(path.to_path_buf());
    Ok(())
}

/// First dotted path whose string value contains `${`, or None. Used to
/// keep interpolation (secrets) out of non-root config sources.
fn first_interpolated(v: &Value) -> Option<String> {
    fn walk(v: &Value, path: &str) -> Option<String> {
        match v {
            Value::String(s) if s.contains("${") => Some(path.to_string()),
            Value::Array(items) => items
                .iter()
                .enumerate()
                .find_map(|(i, item)| walk(item, &format!("{path}[{i}]"))),
            Value::Object(map) => map
                .iter()
                .find_map(|(k, val)| walk(val, &format!("{path}.{k}"))),
            _ => None,
        }
    }
    walk(v, "")
}

/// Merge -> raw tree -> string interpolation -> typed deserialize.
/// env: interpolation is custom (config-rs' env source overrides whole keys,
/// it does not interpolate inside values).
fn finish(
    builder: config::ConfigBuilder<config::builder::DefaultState>,
    config_dir: PathBuf,
    config_path: Option<PathBuf>,
) -> Result<ResolvedConfig> {
    let merged = builder
        .build()
        .map_err(|e| Error::Config(format!("load: {e}")))?;
    let mut raw: Value = merged
        .try_deserialize()
        .map_err(|e| Error::Config(format!("merge: {e}")))?;
    interpolate(&mut raw, &config_dir)?;
    let config: IntlAiConfig =
        serde_json::from_value(raw).map_err(|e| Error::Config(format!("schema: {e}")))?;
    validate(&config)?;
    Ok(ResolvedConfig {
        config,
        config_dir,
        config_path,
    })
}

/// Config format version currently understood. Newer files are refused
/// (same fail-closed forward-compat rule as lockfile shards, plan 5.5).
pub const CONFIG_VERSION: u32 = 1;

/// Locale ids become filenames (`<locale>.json`, `<locale>.toml`):
/// anything path-unsafe is a config error, not a silent write elsewhere.
fn valid_locale_id(id: &str) -> bool {
    !id.is_empty()
        && !id.contains("..")
        && !id
            .bytes()
            .any(|b| matches!(b, b'/' | b'\\' | 0) || b < 0x20)
}

/// Cross-field rules the type shape can't express.
fn validate(config: &IntlAiConfig) -> Result<()> {
    if config.version > CONFIG_VERSION {
        return Err(Error::Config(format!(
            "config version {} exceeds supported {CONFIG_VERSION}",
            config.version
        )));
    }
    if config.targets.is_empty() {
        return Err(Error::Config("targets must not be empty".into()));
    }
    if config.targets.iter().any(|t| t == &config.source) {
        return Err(Error::Config(
            "targets must not contain the source locale".into(),
        ));
    }
    for id in std::iter::once(&config.source).chain(config.targets.iter()) {
        if !valid_locale_id(id) {
            return Err(Error::Config(format!(
                "invalid locale id '{id}' (used as a filename; no path separators or '..')"
            )));
        }
    }
    match &config.provider {
        ProviderConfig::Replay(_) => {}
        ProviderConfig::Http(h) => {
            if h.provider != "openai" {
                return Err(Error::Config(format!(
                    "provider '{}': only \"openai\" is wired in W1 (anthropic follows)",
                    h.provider
                )));
            }
            if h.api_key.is_empty() {
                return Err(Error::Config("provider.api_key must not be empty".into()));
            }
        }
        ProviderConfig::Command(c) => {
            if c.agent.is_none() && c.command.is_none() {
                return Err(Error::Config(
                    "provider kind=command needs `agent` (preset) or `command`".into(),
                ));
            }
            if c.agent.is_some() && c.command.is_some() {
                return Err(Error::Config(
                    "provider: `agent` and `command` are mutually exclusive (keep one)".into(),
                ));
            }
        }
    }
    if let Some(r) = config.max_retries {
        if r > 10 {
            return Err(Error::Config("max_retries is capped at 10".into()));
        }
    }
    if let Some(b) = config.batch_size {
        if b == 0 {
            return Err(Error::Config("batch_size must be >= 1".into()));
        }
    }
    for (i, entry) in config.checks.iter().enumerate() {
        let kinds = [
            entry.id.is_some(),
            entry.spec.is_some(),
            entry.exec.is_some(),
        ]
        .iter()
        .filter(|p| **p)
        .count();
        if kinds != 1 {
            return Err(Error::Config(format!(
                "checks[{i}]: exactly one of `id`, `spec`, `exec` is required"
            )));
        }
        if let Some(cmd) = &entry.exec {
            if cmd.trim().is_empty() {
                return Err(Error::Config(format!(
                    "checks[{i}]: exec must not be empty"
                )));
            }
        }
        if let Some(id) = &entry.id {
            if id.trim().is_empty() {
                return Err(Error::Config(format!("checks[{i}]: id must not be empty")));
            }
        }
    }
    Ok(())
}

/// `${env:VAR}` / `${env:VAR:-default}` / `${env:VAR:?}` / `${file:PATH}` /
/// `${VAR}` inside string values. Bare `$VAR` is deliberately unsupported
/// (dollar signs are common in translated text).
fn interpolate(v: &mut Value, base_dir: &Path) -> Result<()> {
    match v {
        Value::String(s) => {
            *s = interpolate_str(s, base_dir)?;
        }
        Value::Array(items) => {
            for item in items {
                interpolate(item, base_dir)?;
            }
        }
        Value::Object(map) => {
            for (_, val) in map.iter_mut() {
                interpolate(val, base_dir)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn interpolate_str(s: &str, base_dir: &Path) -> Result<String> {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(start) = rest.find("${") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let Some(end) = after.find('}') else {
            return Err(Error::Config("unterminated ${ in config value".into()));
        };
        let inner = &after[..end];
        out.push_str(&resolve_expr(inner, base_dir)?);
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

fn resolve_expr(inner: &str, base_dir: &Path) -> Result<String> {
    if let Some(env_spec) = inner.strip_prefix("env:") {
        resolve_env(env_spec)
    } else if let Some(file) = inner.strip_prefix("file:") {
        let path = resolve(base_dir, Path::new(file.trim()));
        fs::read_to_string(&path)
            .map(|s| s.trim_end_matches(['\n', '\r']).to_string())
            .map_err(|source| Error::Io { path, source })
    } else {
        resolve_env(inner)
    }
}

fn resolve_env(spec: &str) -> Result<String> {
    let (name, default, required) = if let Some((n, d)) = spec.split_once(":-") {
        (n, Some(d), false)
    } else if let Some(n) = spec.strip_suffix(":?") {
        (n, None, true)
    } else {
        (spec, None, false)
    };
    match env::var(name) {
        Ok(v) => Ok(v),
        Err(_) if default.is_some() => Ok(default.unwrap().to_string()),
        Err(_) if required => Err(Error::Config(format!("required env var {name} is not set"))),
        Err(_) => Err(Error::Config(format!(
            "env var {name} referenced by config is not set"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    fn write(dir: &Path, name: &str, body: &str) -> PathBuf {
        let p = dir.join(name);
        let mut f = fs::File::create(&p).unwrap();
        f.write_all(body.as_bytes()).unwrap();
        p
    }

    #[test]
    fn loads_toml_and_resolves_paths() {
        let dir = tempdir().unwrap();
        write(
            dir.path(),
            "intl-ai.toml",
            r#"locale_dir = "locales"
source = "en"
targets = ["fr"]
[provider]
kind = "replay"
file = "cassette.json"
"#,
        );
        let cfg = load(None, dir.path()).unwrap();
        assert_eq!(cfg.config.targets, vec!["fr"]);
        assert_eq!(cfg.locale_dir(), dir.path().join("locales"));
        assert_eq!(cfg.replay_file().unwrap(), dir.path().join("cassette.json"));
    }

    #[test]
    fn json_and_yaml_parse_same_shape() {
        for (name, body) in [
            (
                "intl-ai.json",
                r#"{"locale_dir":"locales","source":"en","targets":["de"],
                   "provider":{"kind":"replay","file":"c.json"}}"#,
            ),
            (
                "intl-ai.yaml",
                "locale_dir: locales\nsource: en\ntargets: [de]\nprovider:\n  kind: replay\n  file: c.json\n",
            ),
        ] {
            let dir = tempdir().unwrap();
            write(dir.path(), name, body);
            let cfg = load(None, dir.path()).unwrap();
            assert_eq!(cfg.config.source, "en");
        }
    }

    #[test]
    fn unknown_field_rejected() {
        let dir = tempdir().unwrap();
        write(
            dir.path(),
            "intl-ai.toml",
            "locale_dir = \"l\"\nsource = \"en\"\ntargets = []\nbogus = 1\nprovider = { kind = \"replay\", file = \"c\" }\n",
        );
        assert!(load(None, dir.path()).is_err());
    }

    #[test]
    fn env_interpolation() {
        unsafe {
            env::set_var("INTL_AI_TEST_DIR", "from-env");
        }
        let dir = tempdir().unwrap();
        write(
            dir.path(),
            "intl-ai.toml",
            "locale_dir = \"${env:INTL_AI_TEST_DIR}\"\nsource = \"en\"\ntargets = [\"fr\"]\nprovider = { kind = \"replay\", file = \"${env:MISSING:-fallback.json}\" }\n",
        );
        let cfg = load(None, dir.path()).unwrap();
        assert_eq!(cfg.config.locale_dir, Path::new("from-env"));
        unsafe {
            env::remove_var("INTL_AI_TEST_DIR");
        }
    }

    #[test]
    fn extends_layering() {
        let dir = tempdir().unwrap();
        write(
            dir.path(),
            "base.toml",
            "source = \"es\"\ncheck = { fail_on = [\"stale\"] }\n",
        );
        write(
            dir.path(),
            "intl-ai.toml",
            "extends = \"./base.toml\"\nlocale_dir = \"l\"\nsource = \"en\"\ntargets = [\"fr\"]\nprovider = { kind = \"replay\", file = \"c\" }\n",
        );
        let cfg = load(None, dir.path()).unwrap();
        assert_eq!(cfg.config.source, "en");
        assert_eq!(cfg.config.check.fail_on, vec![FindingKind::Stale]);
    }
}
