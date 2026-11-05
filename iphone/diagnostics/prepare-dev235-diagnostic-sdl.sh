#!/usr/bin/env bash

# Resolve the SDL source that Cargo will actually compile for the iPhone
# target, then replace only that nested SDL checkout with the pinned,
# observation-only Diagnostic 2 commit stored in this repository.

set -euo pipefail

readonly EXPECTED_SDL2_SYS_SOURCE='git+https://github.com/iDared32bit-emu/rust-sdl2?branch=touchhle-ios-host#50a369c061496c25f32f965172d67282d4623b63'
readonly EXPECTED_RUST_SDL2_COMMIT='50a369c061496c25f32f965172d67282d4623b63'
readonly EXPECTED_SDL_BASE_COMMIT='701a47dc7e626b3142ccdfc26a07db2c43a91c02'
readonly EXPECTED_SDL_DIAGNOSTIC_COMMIT='07318edcc53abdc0ffa1ac5d53e585ba5489cf28'
readonly EXPECTED_BUNDLE_SHA256='9d99602243e725c38b45e6c92ab1bc7f4196abba99b5287616e446f924f25d18'
readonly EXPECTED_BRIDGE_PATCH_BLOB='1d663ad7125255b9f79aacc6ef1b5cfaf07b1d7f'
readonly DIAGNOSTIC_REF='refs/heads/bookios/dev235-presentation-diagnostics'

workspace="${GITHUB_WORKSPACE:?GITHUB_WORKSPACE must identify the checked-out iDared repository}"
runner_temp="${RUNNER_TEMP:?RUNNER_TEMP must be available}"
bundle="$workspace/iphone/diagnostics/dev235-presentation-diagnostics.bundle"
bridge_patch="$workspace/iphone/diagnostics/dev235-diagnostic-sdl-rust-bridge.patch"
metadata="$runner_temp/idared-ios-cargo-metadata.json"

actual_bundle_sha256="$(shasum -a 256 "$bundle" | awk '{ print $1 }')"
test "$actual_bundle_sha256" = "$EXPECTED_BUNDLE_SHA256"
actual_bridge_patch_blob="$(
    git -C "$workspace" rev-parse HEAD:iphone/diagnostics/dev235-diagnostic-sdl-rust-bridge.patch
)"
test "$actual_bridge_patch_blob" = "$EXPECTED_BRIDGE_PATCH_BLOB"
actual_bridge_patch_sha256="$(shasum -a 256 "$bridge_patch" | awk '{ print $1 }')"

# This is target-specific on purpose: it proves which sdl2-sys instance the
# aarch64 iPhone dependency graph resolves, rather than assuming that the
# repository's separate vendor/SDL checkout is a build input.
cargo metadata \
    --locked \
    --filter-platform aarch64-apple-ios \
    --format-version 1 \
    > "$metadata"

manifest_query='.packages[] | select(.name == "sdl2-sys" and .version == "0.37.0" and .source == $source) | .manifest_path'
manifest_count="$(
    jq -r --arg source "$EXPECTED_SDL2_SYS_SOURCE" "$manifest_query" "$metadata" |
        awk 'NF { count += 1 } END { print count + 0 }'
)"
if [[ "$manifest_count" -ne 1 ]]; then
    echo "Expected exactly one pinned iPhone sdl2-sys package, found $manifest_count." >&2
    exit 1
fi

sdl2_sys_manifest="$(
    jq -r --arg source "$EXPECTED_SDL2_SYS_SOURCE" "$manifest_query" "$metadata"
)"
sdl2_sys_dir="$(dirname "$sdl2_sys_manifest")"
rust_sdl2_dir="$(dirname "$sdl2_sys_dir")"
sdl_dir="$sdl2_sys_dir/SDL"

actual_rust_sdl2_commit="$(git -C "$rust_sdl2_dir" rev-parse HEAD)"
test "$actual_rust_sdl2_commit" = "$EXPECTED_RUST_SDL2_COMMIT"

# Cargo normally checks out this submodule with the git dependency. Initialize
# it only inside Cargo's resolved dependency checkout if a cold source cache
# did not already contain it.
if [[ ! -e "$sdl_dir/CMakeLists.txt" ]]; then
    git -C "$rust_sdl2_dir" submodule update --init --recursive -- sdl2-sys/SDL
fi

actual_sdl_base="$(git -C "$sdl_dir" rev-parse HEAD)"
test "$actual_sdl_base" = "$EXPECTED_SDL_BASE_COMMIT"
git -C "$sdl_dir" diff --quiet

# The repository bundle is a complete, pinned history whose diagnostic commit
# has the exact SDL 2.26.4 submodule revision as its parent. This makes the
# source override reproducible on a clean hosted runner without maintaining a
# second external SDL fork.
git -C "$sdl_dir" fetch "$bundle" "$DIAGNOSTIC_REF"
test "$(git -C "$sdl_dir" rev-parse FETCH_HEAD)" = "$EXPECTED_SDL_DIAGNOSTIC_COMMIT"
test "$(git -C "$sdl_dir" rev-parse "$EXPECTED_SDL_DIAGNOSTIC_COMMIT^")" = "$EXPECTED_SDL_BASE_COMMIT"
git -C "$sdl_dir" checkout --detach "$EXPECTED_SDL_DIAGNOSTIC_COMMIT"
test "$(git -C "$sdl_dir" rev-parse HEAD)" = "$EXPECTED_SDL_DIAGNOSTIC_COMMIT"
git -C "$sdl_dir" apply --check "$bridge_patch"
git -C "$sdl_dir" apply "$bridge_patch"

# Assert that Cargo's own build script compiles CARGO_MANIFEST_DIR/SDL and that
# the exact native observation points expected by Diagnostic 2 are present.
grep -Fq 'CARGO_MANIFEST_DIR' "$sdl2_sys_dir/build.rs"
grep -Fq '.join("SDL")' "$sdl2_sys_dir/build.rs"
grep -Fq 'sdl-scene-delegate-supported-orientations-query' "$sdl_dir/src/video/uikit/SDL_uikitappdelegate.m"
grep -Fq 'sdl-controller-supported-orientations-query' "$sdl_dir/src/video/uikit/SDL_uikitviewcontroller.m"
grep -Fq 'sdl-orientation-policy-consumed' "$sdl_dir/src/video/uikit/SDL_uikitwindow.m"
grep -Fq 'sdl-status-bar-orientation-notification' "$sdl_dir/src/video/uikit/SDL_uikitevents.m"
grep -Fq 'idared_sdl_presentation_diagnostic_event' "$sdl_dir/src/video/uikit/SDL_uikitdiagnostics.h"

echo "Resolved iPhone sdl2-sys manifest: $sdl2_sys_manifest"
echo "Resolved rust-sdl2 commit: $actual_rust_sdl2_commit"
echo "Resolved bundled SDL source: $sdl_dir"
echo "Bundled SDL base commit: $actual_sdl_base"
echo "Applied SDL diagnostic commit: $EXPECTED_SDL_DIAGNOSTIC_COMMIT"
echo "Diagnostic bundle SHA-256: $actual_bundle_sha256"
echo "Diagnostic Rust-bridge patch Git blob: $actual_bridge_patch_blob"
echo "Diagnostic Rust-bridge patch SHA-256: $actual_bridge_patch_sha256"

if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
    {
        echo "sdl2_sys_manifest=$sdl2_sys_manifest"
        echo "sdl_dir=$sdl_dir"
        echo "rust_sdl2_commit=$actual_rust_sdl2_commit"
        echo "sdl_base_commit=$actual_sdl_base"
        echo "sdl_diagnostic_commit=$EXPECTED_SDL_DIAGNOSTIC_COMMIT"
        echo "bundle_sha256=$actual_bundle_sha256"
        echo "bridge_patch_blob=$actual_bridge_patch_blob"
        echo "bridge_patch_sha256=$actual_bridge_patch_sha256"
    } >> "$GITHUB_OUTPUT"
fi
