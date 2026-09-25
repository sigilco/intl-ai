/// Timestamp source for lockfile `updated_at` and report names.
/// `INTL_AI_NOW` overrides it so tests and cassettes stay byte-deterministic.
pub fn now() -> String {
    if let Ok(v) = std::env::var("INTL_AI_NOW") {
        return v;
    }
    jiff::Timestamp::now()
        .strftime("%Y-%m-%dT%H:%M:%SZ")
        .to_string()
}
