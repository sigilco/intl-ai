#!/usr/bin/env bash
# Build libintl_ai_uniffi for every Apple target and assemble
# swift/IntlAi/IntlAiFFI.xcframework, then refresh the vendored Swift
# bindings in swift/IntlAi/Sources/IntlAi. Run from anywhere.
set -euo pipefail

script_dir="$(cd "$(dirname "$0")" && pwd)"
package_dir="$(cd "$script_dir/.." && pwd)"
repo_root="$(cd "$package_dir/../.." && pwd)"
bindings_dir="$repo_root/crates/intl-ai-uniffi/bindings"

ios_device_targets=(aarch64-apple-ios)
ios_sim_targets=(aarch64-apple-ios-sim x86_64-apple-ios)
macos_targets=(aarch64-apple-darwin x86_64-apple-darwin)

all_targets=(
    "${ios_device_targets[@]}"
    "${ios_sim_targets[@]}"
    "${macos_targets[@]}"
)

# Match the package's deployment targets so the .a slices do not carry
# a newer minimum OS than the apps linking them (ld warns otherwise).
export IPHONEOS_DEPLOYMENT_TARGET="${IPHONEOS_DEPLOYMENT_TARGET:-13.0}"
export MACOSX_DEPLOYMENT_TARGET="${MACOSX_DEPLOYMENT_TARGET:-11.0}"

echo "==> Installing Rust targets"
rustup target add "${all_targets[@]}"

for target in "${all_targets[@]}"; do
    echo "==> cargo build --release --target $target"
    cargo build --release -p intl-ai-uniffi \
        --manifest-path "$repo_root/Cargo.toml" --target "$target"
done

staging="$(mktemp -d)"
trap 'rm -rf "$staging"' EXIT

lib_name="libintl_ai_uniffi.a"
ios_lib="$repo_root/target/${ios_device_targets[0]}/release/$lib_name"

# Universal binaries per xcframework slice: iOS device is arm64 only,
# simulator and macOS are arm64 + x86_64 fat libraries.
sim_libs=()
for target in "${ios_sim_targets[@]}"; do
    sim_libs+=("$repo_root/target/$target/release/$lib_name")
done
sim_lib="$staging/libintl_ai_uniffi_iossim.a"
lipo -create "${sim_libs[@]}" -output "$sim_lib"

mac_libs=()
for target in "${macos_targets[@]}"; do
    mac_libs+=("$repo_root/target/$target/release/$lib_name")
done
mac_lib="$staging/libintl_ai_uniffi_macos.a"
lipo -create "${mac_libs[@]}" -output "$mac_lib"

# SwiftPM resolves the FFI module through this headers directory: the
# UniFFI-generated modulemap must be named module.modulemap there.
headers_dir="$staging/headers"
mkdir -p "$headers_dir"
cp "$bindings_dir/intl_ai_uniffiFFI.h" "$headers_dir/"
cp "$bindings_dir/intl_ai_uniffiFFI.modulemap" "$headers_dir/module.modulemap"

output="$package_dir/IntlAiFFI.xcframework"
rm -rf "$output"
echo "==> xcodebuild -create-xcframework"
xcodebuild -create-xcframework \
    -library "$ios_lib" -headers "$headers_dir" \
    -library "$sim_lib" -headers "$headers_dir" \
    -library "$mac_lib" -headers "$headers_dir" \
    -output "$output"

# Keep the vendored bindings in sync with the crate output.
cp "$bindings_dir/intl_ai_uniffi.swift" "$package_dir/Sources/IntlAi/intl_ai_uniffi.swift"

echo "==> Wrote $output"
xcodebuild -create-xcframework -help >/dev/null 2>&1 || true
ls "$output"
