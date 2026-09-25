use intl_ai_core::error::{Error, ErrorType, Result};
use intl_ai_core::transport::{TranslateRequest, TranslateResponse, Translated, Transport};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// Deterministic transport serving recorded translations from a cassette
/// JSON file: `{ "<locale>": { "<source value>": "<translation>" } }`.
/// Golden-path e2e never touches a real API (plan section 8). A missing
/// cassette entry is a hard error so fixtures stay honest.
pub struct ReplayTransport {
    id: String,
    cassettes: BTreeMap<String, BTreeMap<String, String>>,
}

impl ReplayTransport {
    pub fn load(path: &Path) -> Result<Self> {
        let text = fs::read_to_string(path).map_err(|source| Error::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let cassettes = serde_json::from_str(&text).map_err(|e| {
            Error::transport(
                ErrorType::ParseError,
                format!("replay cassette {}: {e}", path.display()),
            )
        })?;
        Ok(Self {
            id: "replay".into(),
            cassettes,
        })
    }
}

impl Transport for ReplayTransport {
    fn id(&self) -> &str {
        &self.id
    }

    fn translate(&self, req: &TranslateRequest) -> Result<TranslateResponse> {
        let table = self.cassettes.get(&req.target_locale).ok_or_else(|| {
            Error::transport(
                ErrorType::Empty,
                format!("replay: no cassette for locale {}", req.target_locale),
            )
        })?;
        let mut translations = Vec::with_capacity(req.entries.len());
        for e in &req.entries {
            match table.get(&e.source) {
                Some(v) => translations.push(Translated {
                    key: e.key.clone(),
                    value: v.clone(),
                }),
                None => {
                    return Err(Error::transport(
                        ErrorType::Empty,
                        format!(
                            "replay: cassette {} has no entry for key {} ('{}')",
                            req.target_locale, e.key, e.source
                        ),
                    ));
                }
            }
        }
        Ok(TranslateResponse {
            translations,
            model: "replay".into(),
        })
    }
}
