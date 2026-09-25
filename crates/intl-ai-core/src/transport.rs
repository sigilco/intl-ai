use crate::error::Result;

/// One key awaiting translation.
#[derive(Debug, Clone)]
pub struct TranslationEntry {
    pub key: String,
    pub source: String,
}

#[derive(Debug, Default)]
pub struct TranslateRequest {
    pub source_locale: String,
    pub target_locale: String,
    pub entries: Vec<TranslationEntry>,
    /// Fixed term -> translation pairs injected into the prompt.
    pub glossary: std::collections::BTreeMap<String, String>,
    /// Freeform style/dialect instruction resolved for the target locale.
    pub locale_instruction: Option<String>,
    /// Reviewer notes per key, injected when refilling a rejected entry.
    /// Unused until the W2 quality loop lands.
    pub feedback: std::collections::BTreeMap<String, String>,
    /// Syntax hint for the prompt's placeholder contract (processor choice).
    pub syntax_hint: Option<String>,
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

/// One judged translation: a completed source/translation pair.
#[derive(Debug, Clone)]
pub struct JudgeItem {
    pub key: String,
    pub locale: String,
    pub source: String,
    pub translation: String,
}

#[derive(Debug, Default)]
pub struct JudgeRequest {
    pub items: Vec<JudgeItem>,
    /// Freeform style/dialect instruction resolved for the target locale.
    pub locale_instruction: Option<String>,
}

/// One adversarial score from the `judge` builtin.
#[derive(Debug, Clone)]
pub struct Judgement {
    pub key: String,
    /// 0..1, 1 = publishable.
    pub score: f64,
    pub reason: Option<String>,
    pub errors: Vec<String>,
}

/// Provider surface (W0: replay only; W1 adds OpenAI-compatible HTTP and
/// command transport in intl-ai-providers). Sync for now; async arrives with
/// the HTTP transport if profiling justifies it.
pub trait Transport {
    fn id(&self) -> &str;
    fn translate(&self, req: &TranslateRequest) -> Result<TranslateResponse>;
    /// Adversarial quality scoring for the `judge` builtin. Default reports
    /// the capability as unsupported so checks can skip cleanly.
    fn judge(&self, _req: &JudgeRequest) -> Result<Vec<Judgement>> {
        Err(crate::error::Error::Message(format!(
            "transport '{}' does not support judge",
            self.id()
        )))
    }
}
