//! Locale file IO. JSON and YAML behind one API: reads dispatch on the
//! file's extension, `resolve` finds whichever file a locale already has
//! (an existing file's format always wins) or mints the configured
//! preferred format for new files.

pub mod json;
pub mod yaml;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io;
use std::path::{Path, PathBuf};

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

/// Resolves the on-disk file for `locale`. An existing `.json`, `.yaml`,
/// or `.yml` file wins in that order; otherwise the path is minted with
/// the preferred format's canonical extension.
pub fn resolve(locale_dir: &Path, locale: &str, preferred: FileFormat) -> PathBuf {
    for ext in ["json", "yaml", "yml"] {
        let candidate = locale_dir.join(format!("{locale}.{ext}"));
        if candidate.exists() {
            return candidate;
        }
    }
    locale_dir.join(format!("{locale}.{}", preferred.extension()))
}

/// Dispatching read: the extension on `path` picks the parser, with a
/// JSON fallback for extension-less paths (fixtures and tests).
pub fn read(path: &Path) -> Result<Option<Value>, FormatError> {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .and_then(FileFormat::for_extension)
        .unwrap_or_default()
    {
        FileFormat::Json => json::read(path),
        FileFormat::Yaml => yaml::read(path),
    }
}

/// Dispatching write: format follows the extension on `path`.
pub fn write(path: &Path, value: &Value) -> Result<(), FormatError> {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .and_then(FileFormat::for_extension)
        .unwrap_or_default()
    {
        FileFormat::Json => json::write(path, value),
        FileFormat::Yaml => yaml::write(path, value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
