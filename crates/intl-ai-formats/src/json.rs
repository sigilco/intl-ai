use serde_json::Value;
use std::fs;
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
    #[error("failed to write {path}: {source}")]
    Write { path: PathBuf, source: io::Error },
}

/// Reads a locale JSON file. `Ok(None)` means the file does not exist;
/// a corrupt file is an error, never silently treated as empty.
pub fn read(path: &Path) -> Result<Option<Value>, FormatError> {
    match fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text)
            .map(Some)
            .map_err(|source| FormatError::Parse {
                path: path.to_path_buf(),
                source,
            }),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(FormatError::Read {
            path: path.to_path_buf(),
            source,
        }),
    }
}

/// Writes canonical JSON (2-space indent, insertion order preserved,
/// trailing newline) via atomic tmp+rename.
pub fn write(path: &Path, value: &Value) -> Result<(), FormatError> {
    let body = format!(
        "{}\n",
        serde_json::to_string_pretty(value).unwrap_or_default()
    );
    write_atomic(path, body.as_bytes())
}

pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), FormatError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| FormatError::Write {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let tmp = path.with_extension("tmp-intl-ai");
    fs::write(&tmp, bytes).map_err(|source| FormatError::Write {
        path: tmp.clone(),
        source,
    })?;
    fs::rename(&tmp, path).map_err(|source| FormatError::Write {
        path: path.to_path_buf(),
        source,
    })
}
