use crate::error::{Error, Result};
use intl_ai_formats::json::write_atomic;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Committed shared truth, one flat TOML file per locale
/// (`<locale_dir>/intl-ai.lock.d/<locale>.toml`). Hard properties per plan
/// 5.5: deterministic bytes, idempotent writes, strict parse + fail-closed,
/// atomic tmp+rename, forward-compatible (unknown fields preserved verbatim,
/// newer `version` refused).
pub const LOCKFILE_DIR: &str = "intl-ai.lock.d";
pub const CURRENT_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum Origin {
    #[default]
    Ai,
    Human,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Entry {
    pub value: String,
    pub source_hash: String,
    #[serde(default)]
    pub origin: Origin,
    #[serde(default)]
    pub reviewed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality: Option<toml::Value>,
    /// Unknown fields from newer binaries round-trip verbatim.
    #[serde(flatten)]
    pub extra: BTreeMap<String, toml::Value>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Shard {
    pub version: u32,
    #[serde(default)]
    pub entries: BTreeMap<String, Entry>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, toml::Value>,
}

// Deserialize manually so `version` is checked before accepting a shard:
// fail-closed on newer versions, missing means pre-versioned (0) and loads.
impl<'de> Deserialize<'de> for Shard {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Raw {
            #[serde(default)]
            version: u32,
            #[serde(default)]
            entries: BTreeMap<String, Entry>,
            #[serde(flatten)]
            extra: BTreeMap<String, toml::Value>,
        }
        let raw = Raw::deserialize(deserializer)?;
        if raw.version > CURRENT_VERSION {
            return Err(serde::de::Error::custom(format!(
                "lockfile shard version {} exceeds supported {CURRENT_VERSION}",
                raw.version
            )));
        }
        Ok(Shard {
            version: raw.version,
            entries: raw.entries,
            extra: raw.extra,
        })
    }
}

pub fn shard_path(locale_dir: &Path, locale: &str) -> PathBuf {
    locale_dir.join(LOCKFILE_DIR).join(format!("{locale}.toml"))
}

/// Canonical serialization: `version` header, then `[entries."<key>"]`
/// blocks sorted by key (BTreeMap order), entry fields in struct order.
/// Identical content produces identical bytes; this is `lockfile fmt`.
pub fn serialize(shard: &Shard) -> Result<String> {
    let mut stamped = shard.clone();
    stamped.version = CURRENT_VERSION;
    let mut out =
        toml::to_string(&stamped).map_err(|e| Error::Lockfile(format!("serialize shard: {e}")))?;
    if !out.ends_with('\n') {
        out.push('\n');
    }
    Ok(out)
}

/// Missing shard -> empty v1 shard. Corrupt/unreadable -> hard error
/// (origin data is unrecoverable, never silently regenerate).
pub fn load_shard(locale_dir: &Path, locale: &str) -> Result<Shard> {
    let path = shard_path(locale_dir, locale);
    match fs::read_to_string(&path) {
        Ok(text) => {
            toml::from_str(&text).map_err(|e| Error::Lockfile(format!("{}: {e}", path.display())))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Shard::default()),
        Err(source) => Err(Error::Io { path, source }),
    }
}

/// Idempotent write: skips when serialized bytes already match on-disk.
/// Returns whether the file was (re)written.
pub fn save_shard(locale_dir: &Path, locale: &str, shard: &Shard) -> Result<bool> {
    let path = shard_path(locale_dir, locale);
    let body = serialize(shard)?;
    if let Ok(existing) = fs::read_to_string(&path) {
        if existing == body {
            return Ok(false);
        }
    }
    write_atomic(&path, body.as_bytes()).map_err(Error::Format)?;
    Ok(true)
}

/// Validates every `*.toml` in `intl-ai.lock.d`. Returns per-file entry
/// counts; parse/version failures surface as hard lockfile errors.
pub fn check_shards(locale_dir: &Path) -> Result<Vec<(String, usize)>> {
    let dir = locale_dir.join(LOCKFILE_DIR);
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|source| Error::Io {
        path: dir.clone(),
        source,
    })? {
        let entry = entry.map_err(|source| Error::Io {
            path: dir.clone(),
            source,
        })?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }
        let text = fs::read_to_string(&path).map_err(|source| Error::Io {
            path: path.clone(),
            source,
        })?;
        let shard: Shard = toml::from_str(&text)
            .map_err(|e| Error::Lockfile(format!("{}: {e}", path.display())))?;
        let locale = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();
        out.push((locale, shard.entries.len()));
    }
    out.sort();
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn sample_shard() -> Shard {
        let mut entries = BTreeMap::new();
        entries.insert(
            "b.key".to_string(),
            Entry {
                value: "v".into(),
                source_hash: "h".into(),
                origin: Origin::Ai,
                reviewed: false,
                model: Some("replay".into()),
                updated_at: Some("2026-09-25T00:00:00Z".into()),
                ..Default::default()
            },
        );
        entries.insert(
            "a.key".to_string(),
            Entry {
                value: "human".into(),
                source_hash: "h2".into(),
                origin: Origin::Human,
                reviewed: true,
                ..Default::default()
            },
        );
        Shard {
            version: 1,
            entries,
            extra: BTreeMap::new(),
        }
    }

    #[test]
    fn roundtrip_sorted_deterministic() {
        let shard = sample_shard();
        let text = serialize(&shard).unwrap();
        assert!(text.contains("version = 1"));
        let a = text.find("\"a.key\"").unwrap();
        let b = text.find("\"b.key\"").unwrap();
        assert!(a < b);
        let parsed: Shard = toml::from_str(&text).unwrap();
        assert_eq!(serialize(&parsed).unwrap(), text);
    }

    #[test]
    fn idempotent_write() {
        let dir = tempdir().unwrap();
        let shard = sample_shard();
        assert!(save_shard(dir.path(), "fr", &shard).unwrap());
        assert!(!save_shard(dir.path(), "fr", &shard).unwrap());
    }

    #[test]
    fn unknown_entry_fields_preserved() {
        let text = r#"version = 1

[entries."k"]
value = "v"
source_hash = "h"
future_field = { nested = 1 }
"#;
        let shard: Shard = toml::from_str(text).unwrap();
        assert_eq!(
            shard.entries["k"].extra["future_field"]["nested"],
            toml::Value::Integer(1)
        );
        let out = serialize(&shard).unwrap();
        assert!(out.contains("future_field"));
    }

    #[test]
    fn newer_version_refused() {
        let text = "version = 2\n";
        assert!(toml::from_str::<Shard>(text).is_err());
    }
}
