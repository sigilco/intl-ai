use crate::error::Result;

/// One key awaiting translation.
#[derive(Debug, Clone)]
pub struct TranslationEntry {
    pub key: String,
    pub source: String,
}

#[derive(Debug)]
pub struct TranslateRequest {
    pub source_locale: String,
    pub target_locale: String,
    pub entries: Vec<TranslationEntry>,
}

#[derive(Debug)]
pub struct Translated {
    pub key: String,
    pub value: String,
}

#[derive(Debug)]
pub struct TranslateResponse {
    pub translations: Vec<Translated>,
    /// Model/transport label recorded on lockfile entries.
    pub model: String,
}

/// Provider surface (W0: replay only; W1 adds OpenAI-compatible HTTP and
/// command transport in intl-ai-providers). Sync for now; async arrives with
/// the HTTP transport if profiling justifies it.
pub trait Transport {
    fn id(&self) -> &str;
    fn translate(&self, req: &TranslateRequest) -> Result<TranslateResponse>;
}
