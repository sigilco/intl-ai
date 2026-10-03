//! Locale file IO behind a pluggable surface: `Format` is the contract a
//! locale file format implements (parse file text into the corpus tree,
//! serialize the corpus back to canonical bytes, declare its extensions),
//! and `FormatRegistry` is the resolution point — `json`/`yaml` are
//! builtin registrations, `[[formats]] exec` entries register external
//! programs speaking the format protocol (see `exec.rs`). `resolve` finds
//! whichever file a locale already has (an existing file's format always
//! wins) or mints the configured preferred format for new files.

pub mod exec;
pub mod json;
pub mod yaml;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};

#[derive(Debug, thiserror::Error)]
pub enum FormatError {
    #[error("failed to read {path}: {source}")]
    Read { path: PathBuf, source: io::Error },
    #[error("failed to parse {path}: {source}")]
    Parse {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("failed to parse {path}: {source}")]
    ParseYaml {
        path: PathBuf,
        source: serde_yaml_ng::Error,
    },
    #[error("failed to write {path}: {source}")]
    Write { path: PathBuf, source: io::Error },
    /// An exec-backed format's subprocess failed or spoke off-protocol.
    #[error("exec format '{format}': {message}")]
    Exec { format: String, message: String },
    /// `format = "<name>"` named nothing the registry knows.
    #[error("unknown format '{name}' (registered: {known})")]
    UnknownFormat { name: String, known: String },
    /// Registry rule broken: duplicate name or extension.
    #[error("format registration: {0}")]
    Registry(String),
}

/// A locale file format: the corpus is the shared nested key -> string
/// tree (`serde_json::Value`); implementations translate between that
/// tree and the format's canonical file bytes.
pub trait Format: fmt::Debug + Send + Sync {
    /// Registry name; `format = "<name>"` selects it for minting new
    /// locale files.
    fn name(&self) -> &str;
    /// File extensions this format claims, canonical (minted) first.
    fn extensions(&self) -> Vec<&str>;
    /// Parses file text into the corpus tree.
    fn parse(&self, text: &str, path: &Path) -> Result<Value, FormatError>;
    /// Serializes the corpus tree into canonical file bytes.
    fn serialize(&self, value: &Value, path: &Path) -> Result<Vec<u8>, FormatError>;
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum FileFormat {
    #[default]
    Json,
    Yaml,
}

impl FileFormat {
    /// Every builtin registration, in registry order.
    pub const ALL: [Self; 2] = [Self::Json, Self::Yaml];

    /// Registry name; `format = "<name>"` selects this format.
    pub fn name(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Yaml => "yaml",
        }
    }

    /// Canonical extension used when creating a new locale file.
    pub fn extension(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Yaml => "yaml",
        }
    }

    pub fn for_extension(ext: &str) -> Option<Self> {
        match ext {
            "json" => Some(Self::Json),
            "yaml" | "yml" => Some(Self::Yaml),
            _ => None,
        }
    }
}

impl Format for FileFormat {
    fn name(&self) -> &str {
        FileFormat::name(*self)
    }

    fn extensions(&self) -> Vec<&str> {
        match self {
            Self::Json => vec!["json"],
            Self::Yaml => vec!["yaml", "yml"],
        }
    }

    fn parse(&self, text: &str, path: &Path) -> Result<Value, FormatError> {
        match self {
            Self::Json => json::parse(text, path),
            Self::Yaml => yaml::parse(text, path),
        }
    }

    fn serialize(&self, value: &Value, path: &Path) -> Result<Vec<u8>, FormatError> {
        match self {
            Self::Json => Ok(json::serialize(value)),
            Self::Yaml => yaml::serialize(path, value),
        }
    }
}

/// The resolution point for locale file IO: format name -> impl for
/// `format = "<name>"` minting, extension -> impl for existing files.
/// Registration order is resolve order (builtins first, so `fr.json` and
/// `fr.xml` on disk together pick the json).
#[derive(Clone, Default)]
pub struct FormatRegistry {
    ordered: Vec<Arc<dyn Format>>,
    by_name: HashMap<String, Arc<dyn Format>>,
    by_ext: HashMap<String, Arc<dyn Format>>,
}

impl fmt::Debug for FormatRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FormatRegistry")
            .field("formats", &self.names())
            .finish()
    }
}

impl FormatRegistry {
    /// `json` then `yaml` — the formats every intl-ai always speaks.
    pub fn builtins() -> Self {
        let mut registry = Self::default();
        registry
            .register(Arc::new(FileFormat::Json))
            .expect("builtin json registers");
        registry
            .register(Arc::new(FileFormat::Yaml))
            .expect("builtin yaml registers");
        registry
    }

    /// Names and extensions are unique: two formats claiming one name
    /// would make `format = "<name>"` ambiguous, and two claiming one
    /// extension would make dispatch on an existing file ambiguous.
    pub fn register(&mut self, format: Arc<dyn Format>) -> Result<(), FormatError> {
        let name = format.name().to_string();
        if self.by_name.contains_key(&name) {
            return Err(FormatError::Registry(format!(
                "format name '{name}' is already registered"
            )));
        }
        for ext in format.extensions() {
            if let Some(existing) = self.by_ext.get(ext) {
                return Err(FormatError::Registry(format!(
                    "extension '{ext}' is already claimed by format '{}'",
                    existing.name()
                )));
            }
        }
        for ext in format.extensions() {
            self.by_ext.insert(ext.to_string(), Arc::clone(&format));
        }
        self.by_name.insert(name, Arc::clone(&format));
        self.ordered.push(format);
        Ok(())
    }

    pub fn format(&self, name: &str) -> Option<&Arc<dyn Format>> {
        self.by_name.get(name)
    }

    pub fn names(&self) -> Vec<&str> {
        self.ordered.iter().map(|f| f.name()).collect()
    }

    /// Every claimed extension in resolve order (each format's canonical
    /// extension first, then its aliases).
    pub fn extensions(&self) -> Vec<&str> {
        self.ordered.iter().flat_map(|f| f.extensions()).collect()
    }

    /// Resolves the on-disk file for `locale`. An existing file under any
    /// registered extension wins (registration order); otherwise the path
    /// is minted with `preferred`'s canonical extension.
    pub fn resolve(
        &self,
        locale_dir: &Path,
        locale: &str,
        preferred: &str,
    ) -> Result<PathBuf, FormatError> {
        for format in &self.ordered {
            for ext in format.extensions() {
                let candidate = locale_dir.join(format!("{locale}.{ext}"));
                if candidate.exists() {
                    return Ok(candidate);
                }
            }
        }
        let format = self
            .format(preferred)
            .ok_or_else(|| FormatError::UnknownFormat {
                name: preferred.to_string(),
                known: self.names().join(", "),
            })?;
        Ok(locale_dir.join(format!("{locale}.{}", format.extensions()[0])))
    }

    fn dispatch(&self, path: &Path) -> Arc<dyn Format> {
        path.extension()
            .and_then(|e| e.to_str())
            .and_then(|ext| self.by_ext.get(ext))
            .cloned()
            // Extension-less paths (fixtures, tests) keep the historical
            // JSON fallback; so do extensions nobody claimed.
            .unwrap_or_else(|| Arc::new(FileFormat::Json))
    }

    /// Dispatching read: the extension on `path` picks the format.
    /// `Ok(None)` means the file does not exist (no subprocess spawned
    /// for an exec format on a missing file).
    pub fn read(&self, path: &Path) -> Result<Option<Value>, FormatError> {
        let format = self.dispatch(path);
        match std::fs::read_to_string(path) {
            Ok(text) => format.parse(&text, path).map(Some),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(source) => Err(FormatError::Read {
                path: path.to_path_buf(),
                source,
            }),
        }
    }

    /// Dispatching write: format follows the extension on `path`. Skips
    /// the write entirely when the serialized bytes already match the
    /// file — unchanged locale files keep their mtime and trigger no
    /// watchers (M2).
    pub fn write(&self, path: &Path, value: &Value) -> Result<(), FormatError> {
        let body = self.dispatch(path).serialize(value, path)?;
        if std::fs::read(path).ok().as_deref() == Some(body.as_slice()) {
            return Ok(());
        }
        json::write_atomic(path, &body)
    }
}

static BUILTINS: LazyLock<FormatRegistry> = LazyLock::new(FormatRegistry::builtins);

/// Resolves the on-disk file for `locale` against the builtin registry
/// (json, yaml, yml; an existing file wins, else mint `preferred`).
/// Config-resolved callers use `FormatRegistry::resolve` so `[[formats]]`
/// exec registrations count too.
pub fn resolve(locale_dir: &Path, locale: &str, preferred: FileFormat) -> PathBuf {
    BUILTINS
        .resolve(locale_dir, locale, preferred.name())
        .expect("builtin formats always register")
}

/// Reads `path` with the builtin registry (extension dispatches, JSON
/// fallback). Config-resolved callers use `FormatRegistry::read`.
pub fn read(path: &Path) -> Result<Option<Value>, FormatError> {
    BUILTINS.read(path)
}

/// Writes `path` with the builtin registry. Config-resolved callers use
/// `FormatRegistry::write` so exec-registered extensions serialize right.
pub fn write(path: &Path, value: &Value) -> Result<(), FormatError> {
    BUILTINS.write(path, value)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A registry-registered format that never touches a subprocess:
    /// corpus serialized with a `DEMO:` prefix.
    #[derive(Debug)]
    struct DemoFormat;
    impl Format for DemoFormat {
        fn name(&self) -> &str {
            "demo"
        }
        fn extensions(&self) -> Vec<&str> {
            vec!["demo", "dmo"]
        }
        fn parse(&self, text: &str, _path: &Path) -> Result<Value, FormatError> {
            serde_json::from_str(text.trim_start_matches("DEMO:")).map_err(|source| {
                FormatError::Parse {
                    path: PathBuf::new(),
                    source,
                }
            })
        }
        fn serialize(&self, value: &Value, _path: &Path) -> Result<Vec<u8>, FormatError> {
            Ok(format!("DEMO:{}", serde_json::to_string(value).unwrap()).into_bytes())
        }
    }

    fn demo_registry() -> FormatRegistry {
        let mut r = FormatRegistry::builtins();
        r.register(Arc::new(DemoFormat)).unwrap();
        r
    }

    #[test]
    fn file_format_extensions() {
        assert_eq!(FileFormat::Json.extension(), "json");
        assert_eq!(FileFormat::Yaml.extension(), "yaml");
        assert_eq!(FileFormat::for_extension("yml"), Some(FileFormat::Yaml));
        assert_eq!(FileFormat::for_extension("yaml"), Some(FileFormat::Yaml));
        assert_eq!(FileFormat::for_extension("json"), Some(FileFormat::Json));
        assert_eq!(FileFormat::for_extension("toml"), None);
    }

    #[test]
    fn registry_rejects_duplicate_names_and_extensions() {
        let mut r = FormatRegistry::builtins();
        // Name collision with a builtin.
        assert!(r.register(Arc::new(FileFormat::Json)).is_err());
        r.register(Arc::new(DemoFormat)).unwrap();
        // Name collision with a custom registration.
        assert!(r.register(Arc::new(DemoFormat)).is_err());

        #[derive(Debug)]
        struct ExtSquatter;
        impl Format for ExtSquatter {
            fn name(&self) -> &str {
                "squatter"
            }
            fn extensions(&self) -> Vec<&str> {
                vec!["yml"]
            }
            fn parse(&self, _t: &str, _p: &Path) -> Result<Value, FormatError> {
                unreachable!()
            }
            fn serialize(&self, _v: &Value, _p: &Path) -> Result<Vec<u8>, FormatError> {
                unreachable!()
            }
        }
        // Extension collision with a builtin alias.
        let err = r.register(Arc::new(ExtSquatter)).unwrap_err();
        assert!(err.to_string().contains("yml"));
    }

    #[test]
    fn resolve_prefers_existing_file() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::write(dir.path().join("fr.yaml"), "a: b\n").unwrap();
        std::fs::write(dir.path().join("de.json"), "{}").unwrap();
        // Existing yaml wins over the json default.
        assert_eq!(
            resolve(dir.path(), "fr", FileFormat::Json),
            dir.path().join("fr.yaml")
        );
        // Existing json wins over a yaml preference.
        assert_eq!(
            resolve(dir.path(), "de", FileFormat::Yaml),
            dir.path().join("de.json")
        );
        // Nothing on disk -> mint with preferred extension.
        assert_eq!(
            resolve(dir.path(), "es", FileFormat::Yaml),
            dir.path().join("es.yaml")
        );
    }

    #[test]
    fn resolve_covers_custom_extensions() {
        let dir = tempfile::TempDir::new().unwrap();
        let r = demo_registry();
        std::fs::write(dir.path().join("fr.dmo"), "DEMO:{}").unwrap();
        // An existing custom-alias file wins over the json default.
        assert_eq!(
            r.resolve(dir.path(), "fr", "json").unwrap(),
            dir.path().join("fr.dmo")
        );
        // Minting uses the custom format's canonical extension.
        assert_eq!(
            r.resolve(dir.path(), "de", "demo").unwrap(),
            dir.path().join("de.demo")
        );
        // Unknown preferred name is a clear error.
        let err = r.resolve(dir.path(), "es", "bogus").unwrap_err();
        assert!(err.to_string().contains("unknown format 'bogus'"));
        assert!(err.to_string().contains("demo"));
    }

    #[test]
    fn yaml_roundtrip_dispatches_on_extension() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("fr.yaml");
        let value = serde_json::json!({"greeting": "Salut", "nav": {"home": "Accueil"}});
        write(&path, &value).unwrap();
        let back = read(&path).unwrap().unwrap();
        assert_eq!(back["greeting"], "Salut");
        assert_eq!(back["nav"]["home"], "Accueil");
        // The file on disk is YAML, not JSON.
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("greeting: Salut"));
    }

    #[test]
    fn custom_format_roundtrips_through_registry() {
        let dir = tempfile::TempDir::new().unwrap();
        let r = demo_registry();
        let path = dir.path().join("fr.demo");
        let value = serde_json::json!({"greeting": "Salut"});
        r.write(&path, &value).unwrap();
        // The demo format's own bytes, not json serialization.
        assert!(std::fs::read_to_string(&path).unwrap().starts_with("DEMO:"));
        let back = r.read(&path).unwrap().unwrap();
        assert_eq!(back["greeting"], "Salut");
    }
}
