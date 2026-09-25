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
#[derive(Debug, Clone, Deserialize, serde::Serialize)]
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
}

fn default_version() -> u32 {
    1
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
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

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProviderConfig {
    Replay(ReplayProvider),
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReplayProvider {
    /// Cassette JSON: `{ "<locale>": { "<source value>": "<translation>" } }`.
    /// Resolved against the config file's directory.
    pub file: PathBuf,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
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
    vec![FindingKind::Stale, FindingKind::Invalid]
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
        }
    }
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
    EXTENSIONS
        .iter()
        .map(|ext| cwd.join(format!("{FILE_STEM}.{ext}")))
        .find(|p| p.is_file())
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
    finish(builder, cwd.to_path_buf(), None)
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
    let mut visiting = HashSet::new();
    walk_extends(path, config_dir, &mut visiting, &mut ordered, 0)?;
    Ok(ordered)
}

fn walk_extends(
    path: &Path,
    config_dir: &Path,
    visiting: &mut HashSet<PathBuf>,
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
    if !visiting.insert(canon.clone()) {
        return Err(Error::Config(format!(
            "extends cycle at {}",
            path.display()
        )));
    }
    let cfg = Config::builder()
        .add_source(File::from(path.to_path_buf()))
        .build()
        .map_err(|e| Error::Config(format!("{}: {e}", path.display())))?;
    // config-rs errors on absent keys even for Option<T>; treat absent as None.
    let extends: Option<StringOrList> = cfg.get::<Option<StringOrList>>("extends").ok().flatten();
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
            walk_extends(&base_path, config_dir, visiting, ordered, depth + 1)?;
        }
    }
    ordered.push(path.to_path_buf());
    Ok(())
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
    Ok(ResolvedConfig {
        config,
        config_dir,
        config_path,
    })
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
            "locale_dir = \"${env:INTL_AI_TEST_DIR}\"\nsource = \"en\"\ntargets = []\nprovider = { kind = \"replay\", file = \"${env:MISSING:-fallback.json}\" }\n",
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
            "extends = \"./base.toml\"\nlocale_dir = \"l\"\nsource = \"en\"\ntargets = []\nprovider = { kind = \"replay\", file = \"c\" }\n",
        );
        let cfg = load(None, dir.path()).unwrap();
        assert_eq!(cfg.config.source, "en");
        assert_eq!(cfg.config.check.fail_on, vec![FindingKind::Stale]);
    }
}
