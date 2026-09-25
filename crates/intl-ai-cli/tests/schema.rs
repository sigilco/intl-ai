use std::path::Path;

/// The committed schema file must stay byte-identical to what
/// `intl-ai config schema` generates (same drift contract the old TS
/// schema-drift CI job enforced, now against the Rust contract).
#[test]
fn committed_schema_matches_generated() {
    let generated =
        serde_json::to_string_pretty(&intl_ai_core::config::json_schema()).unwrap() + "\n";
    let committed_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/public/schema/intl-ai.schema.json");
    let committed = std::fs::read_to_string(&committed_path).unwrap_or_else(|e| {
        panic!(
            "read {}: {e} (regenerate with `intl-ai config schema`)",
            committed_path.display()
        )
    });
    assert_eq!(
        generated, committed,
        "docs/public/schema/intl-ai.schema.json drifted; regenerate with `intl-ai config schema > docs/public/schema/intl-ai.schema.json`"
    );
}
