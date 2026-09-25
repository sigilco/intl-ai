use serde_json::Value;
use std::fs;
use std::io;
use std::path::Path;

pub use crate::FormatError;

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

/// Canonical JSON bytes (2-space indent, insertion order preserved,
/// trailing newline). Split from `write` so callers can byte-compare
/// before touching disk.
pub fn serialize(value: &Value) -> Vec<u8> {
    format!(
        "{}\n",
        serde_json::to_string_pretty(value).unwrap_or_default()
    )
    .into_bytes()
}

/// Writes canonical JSON via atomic tmp+rename.
pub fn write(path: &Path, value: &Value) -> Result<(), FormatError> {
    write_atomic(path, &serialize(value))
}

pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), FormatError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| FormatError::Write {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    // Unique tmp per process: two concurrent writers must not share a
    // staging file (the per-locale ShardLock serializes shard writers;
    // locale files still get the last-writer-wins rename either way).
    let tmp = path.with_extension(format!("tmp-intl-ai-{}", std::process::id()));
    fs::write(&tmp, bytes).map_err(|source| FormatError::Write {
        path: tmp.clone(),
        source,
    })?;
    fs::rename(&tmp, path).map_err(|source| FormatError::Write {
        path: path.to_path_buf(),
        source,
    })
}
