#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != Darwin ]]; then
    echo "Build PackOBF for macOS on a Mac." >&2
    exit 1
fi

mode="${1:-debug}"
case "$mode" in
    debug|test) profile=dev; configuration=debug; swift_optimization=-g ;;
    release) profile=release; configuration=release; swift_optimization=-O ;;
    *) echo "Usage: $0 [debug|release|test]" >&2; exit 2 ;;
esac

platform_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
gui_dir="$(cd "$platform_dir/../.." && pwd)"
cd "$gui_dir"

rust_target="$(rustc -vV | sed -n 's/^host: //p')"
case "$rust_target" in
    aarch64-apple-darwin) swift_arch=arm64 ;;
    x86_64-apple-darwin) swift_arch=x86_64 ;;
    *) echo "Unsupported Rust host target: $rust_target" >&2; exit 1 ;;
esac

export MACOSX_DEPLOYMENT_TARGET=15.0
rust_dir="$gui_dir/target/macos-rust"
build_dir="$gui_dir/target/macos/$swift_arch/$configuration"
app="$build_dir/PackOBF.app"
frameworks="$app/Contents/Frameworks"
dylib=libpackobf_gui.dylib
cargo_args=(--locked --features ffi --target "$rust_target" --target-dir "$rust_dir")

cargo rustc "${cargo_args[@]}" --lib --profile "$profile" --crate-type cdylib -- \
    -C "link-arg=-Wl,-install_name,@rpath/$dylib"

mkdir -p "$app/Contents/MacOS" "$frameworks"
cp "$rust_dir/$rust_target/$configuration/$dylib" "$frameworks/$dylib"
cp "$platform_dir/Info.plist" "$app/Contents/Info.plist"
package_version="$(sed -n 's/^version = "\([^"]*\)"$/\1/p' "$gui_dir/Cargo.toml")"
/usr/libexec/PlistBuddy -c "Add CFBundleShortVersionString string ${package_version%%-*}" \
    -c "Add CFBundleVersion string ${package_version%%-*}" "$app/Contents/Info.plist"

swift_args=(
    -swift-version 6
    -target "$swift_arch-apple-macosx$MACOSX_DEPLOYMENT_TARGET"
    -module-cache-path "$build_dir/swift-module-cache"
    -import-objc-header "$gui_dir/include/packobf_gui.h"
    -L "$frameworks" -lpackobf_gui
)
xcrun swiftc "${swift_args[@]}" "$swift_optimization" -module-name PackOBF \
    -Xlinker -rpath -Xlinker '@executable_path/../Frameworks' \
    "$platform_dir"/Sources/*.swift -o "$app/Contents/MacOS/PackOBF"
codesign --force --sign - "$frameworks/$dylib"
codesign --force --sign - "$app"
echo "Built $app"

if [[ "$mode" == test ]]; then
    cargo test "${cargo_args[@]}"
    xcrun swiftc "${swift_args[@]}" -g -module-name PackOBFTests \
        -Xlinker -rpath -Xlinker '@executable_path/PackOBF.app/Contents/Frameworks' \
        "$platform_dir/Sources/Optimizer.swift" "$platform_dir/Sources/PackModel.swift" \
        "$platform_dir/Tests/OptimizationTest.swift" -o "$build_dir/PackOBFTests"
    "$build_dir/PackOBFTests"
fi
