use serde_json::Value;
use std::fs;
use std::io;
use std::path::Path;

use crate::FormatError;
use crate::json::write_atomic;

/// Reads a locale YAML file. `Ok(None)` means the file does not exist;
/// a corrupt file is an error, never silently treated as empty.
pub fn read(path: &Path) -> Result<Option<Value>, FormatError> {
    match fs::read_to_string(path) {
        Ok(text) => {
            // YAML is self-describing enough to decode straight into the
            // shared JSON value tree; locale keys stay strings either way.
            serde_yaml_ng::from_str(&text)
                .map(Some)
                .map_err(|source| FormatError::ParseYaml {
                    path: path.to_path_buf(),
                    source,
                })
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(FormatError::Read {
            path: path.to_path_buf(),
            source,
        }),
    }
}

/// Writes YAML via atomic tmp+rename. serde_yaml_ng emits plain block
/// mappings with stable insertion order for our ordered Value tree.
pub fn write(path: &Path, value: &Value) -> Result<(), FormatError> {
    let body = serde_yaml_ng::to_string(value).map_err(|e| FormatError::ParseYaml {
        path: path.to_path_buf(),
        source: e,
    })?;
    write_atomic(path, body.as_bytes())
}
