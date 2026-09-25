/// Timestamp source for lockfile `updated_at` and report names.
/// `INTL_AI_NOW` is a test-only override (cassettes must stay
/// byte-deterministic); never set it in production — a frozen clock
/// poisons every `updated_at` it touches.
pub fn now() -> String {
    if let Ok(v) = std::env::var("INTL_AI_NOW") {
        return v;
    }
    jiff::Timestamp::now()
        .strftime("%Y-%m-%dT%H:%M:%SZ")
        .to_string()
}
