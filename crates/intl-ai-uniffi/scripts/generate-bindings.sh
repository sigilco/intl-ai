#!/usr/bin/env bash
# Regenerate Swift and Kotlin bindings into bindings/{swift,kotlin}.
# Run from anywhere; resolves the workspace root via cargo.
set -euo pipefail

crate_dir="$(cd "$(dirname "$0")/.." && pwd)"
workspace_root="$(cd "$crate_dir/../.." && pwd)"

cargo build -p intl-ai-uniffi --features cli --manifest-path "$workspace_root/Cargo.toml"

lib="$workspace_root/target/debug/libintl_ai_uniffi.so"
[ -f "$lib" ] || lib="$workspace_root/target/debug/libintl_ai_uniffi.dylib"

cargo run -p intl-ai-uniffi --features cli --bin uniffi-bindgen \
    --manifest-path "$workspace_root/Cargo.toml" -- \
    generate --library "$lib" \
    --language swift --language kotlin \
    --out-dir "$crate_dir/bindings"
