# Dev 2.38 Phase 2 persistent cold-start index

This loading-performance revision extends Dev 2.34's process-lifetime retained
`LibraryCatalog` across complete iDared process launches. It does not include
the paused Dev 2.35-and-later orientation experiments or game-compatibility
changes.

## Architecture

The native iOS picker stores a disposable, versioned binary-plist index in the
application container's `Library/Caches/iDaredPickerIndex` directory. Processed
picker icons are separate PNG files under `icons-v1`. The index records the
package-relative path, package type, inexpensive source fingerprint, localized
picker metadata, display-only compatibility classification, icon reference and
either the successful inspection result or a bounded failure description.

On startup, iDared enumerates direct `Games` entries and compares their cheap
fingerprints with the index. Unchanged successful records load their processed
icons without reopening their IPA archives. Unchanged rejected packages use a
negative cache record. New, renamed, replaced, repaired, ambiguous or changed
packages alone use the existing full package inspection path. Loose `.app`
directories receive a recursive metadata-only digest because child changes do
not reliably update the parent directory timestamp. IPA payload bytes are not
hashed during ordinary startup.

The index is atomically replaced and carries schema, picker-metadata,
localization and integrity boundaries. A missing, truncated, corrupt,
incompatible or altered index is disposable: the current scanner rebuilds it.
A missing or corrupt cached icon causes focused reinspection of that package.
The OS may purge the entire cache without affecting correctness.

## Correctness boundaries

Cached picker data is never launch authority. The selected `AppInfo` retains
the current enumerated host path, and ordinary `Environment` creation still
reopens and validates the real package before guest execution. No guest
filesystem, save state, memory, threads, graphics state or emulator
`Environment` is persisted.

The Dev 2.34 in-memory catalog remains the first return path after normal exit,
recoverable crash and failed launch. Its unchanged-library check and full-scan
fallback are preserved; the disk index is for process cold starts and reloads,
not a replacement for the faster retained return.

## Invalidation and measurement

File packages use normalized relative path, type, size, high-resolution
modification time and filesystem identity. `.app` directories additionally use
recursive entry count, byte count and a SHA-256 digest of child paths and
metadata. These fingerprints are change detectors, not cryptographic package
identity. Any ambiguous fingerprint is deliberately inspected for the current
launch and not persisted.

Bounded `Log.txt` summaries distinguish index absence, reuse, schema or
localization invalidation, corruption, successful and negative hits, new and
changed misses, icon misses, focused inspections, deletions, read/decode,
reconciliation, icon work, index write, archive-entry work and the existing
Rust-to-Swift/first-visible-picker timing. The first index-building launch may
remain near the old full-scan cost; physical-device testing must establish the
subsequent unchanged-start improvement.

## Validation scope

Focused tests cover initial creation, unchanged success and failure reuse,
incremental additions/replacements/repairs/deletions, current-path
reconstruction, missing icons, truncated and integrity-invalid indexes, schema
and preferred-language invalidation, nested `.app` changes and loose IPA import
behavior. Hosted macOS preflight remains the authoritative full library-test
and iOS archive gate. Device testing is still required for actual cold-start
timing, icon/UI fidelity, Files-app mutation behavior and retained-return
performance with the full installed library.
