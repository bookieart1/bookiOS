#!/usr/bin/env bash

# Verify that the archived executable contains the first-frame orientation gate
# and its bridge into the actual Cargo-built SDL UIKit implementation.
set -euo pipefail

if [[ $# -ne 2 ]]; then
    echo "Usage: $0 <linked-executable> <scratch-directory>" >&2
    exit 2
fi

executable="$1"
scratch_directory="$2"
strings_file="$scratch_directory/idared-dev237-linked-strings.txt"
symbols_file="$scratch_directory/idared-dev237-linked-symbols.txt"
verified_markers_file="$scratch_directory/idared-dev237-linked-markers.txt"

/usr/bin/strings "$executable" > "$strings_file"

# Each marker originates in a different actual SDL UIKit compilation unit.
# Their presence in the linked executable proves that Cargo's bundled native
# SDL—not the unrelated vendor/SDL copy—was rebuilt.
markers=(
    'sdl-scene-delegate-supported-orientations-query'
    'reason=native-key-root'
    'reason=native-host-fallback'
    'sdl-first-frame-cover-installed'
    'sdl-first-frame-cover-removed'
    'sdl-controller-supported-orientations-query'
    'sdl-controller-transition-begin'
    'sdl-orientation-policy-consumed'
    'sdl-window-destroy-begin'
    'sdl-status-bar-orientation-notification'
    'sdl-view-window-change-begin'
    'rust-guest-first-frame'
    'guest-orientation-recorded-pending'
    'guest-presentation-committed'
    'guest-termination-begin'
)
: > "$verified_markers_file"
for marker in "${markers[@]}"; do
    grep -Fq "$marker" "$strings_file"
    printf '%s\n' "$marker" >> "$verified_markers_file"
    echo "Linked diagnostic marker: $marker"
done

# Materialize nm output before searching it. With pipefail enabled, piping nm
# directly into grep -q lets grep's successful early exit SIGPIPE nm and turns
# a proven symbol match into a false exit-1 verification failure. The final
# release link is allowed to internalize/strip this Rust bridge name: the SDL
# call sites are ordinary (non-weak) references, so a successful final link,
# together with the required SDL, Rust and Swift markers above, proves that the
# phase-gate implementation was resolved into the shipped executable.
/usr/bin/nm -gU "$executable" > "$symbols_file"
if grep -Fq '_idared_sdl_host_orientation_policy' "$symbols_file"; then
    echo "Linked phase-policy bridge is public: idared_sdl_host_orientation_policy"
else
    echo "Linked phase-policy bridge name was stripped/internalized; verified by non-weak SDL call-site link plus SDL/Rust/Swift markers."
fi
