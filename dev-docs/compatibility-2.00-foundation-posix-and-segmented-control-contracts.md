# Dev 2.00: Foundation, POSIX and segmented-control contracts

## Evidence and scope

The Dev 1.99 device log is preserved as
`outputs/device-logs/iDared-Dev-1.99-shared-contract-batch-2026-11-02.txt`
(655,664 bytes; SHA-256
`6e36cf47ccb698b801c737ac150ada868def43b6bd0e777fd72bb36bff37c344`).
It embeds final Dev 1.99 source `e22df8ddd963b256622113ed1260b4b8925690ea`,
workflow `41418393785`, and the permanent universal recorder.

Dev 1.99 advanced five apps to standard, independently testable contracts and
identified the same directory-attributes contract in two unrelated apps. Dev
2.00 implements those bounded contracts together. It does not alter the two
remaining ambiguous lifetime/resource failures in 8bitninja or DoDoFly.

## Implemented contracts

- **KVC key paths:** `NSObject valueForKeyPath:` traverses dot-separated keys
  through the existing `valueForKey:` machinery. This preserves dictionary
  lookup, normal accessor search, direct-ivar lookup, scalar boxing and nil
  short-circuit behavior rather than adding a dictionary-only shortcut.
- **Explicit localized bundle lookup:**
  `pathForResource:ofType:inDirectory:forLocalization:` searches global
  resources first and then the explicitly named `.lproj`, including the
  existing device-family resource preference. It does not silently run the
  ordinary preferred-language fallback.
- **POSIX `rmdir`:** directory removal uses the virtual filesystem's genuine
  empty-directory checks and reports Darwin-compatible `ENOENT`, `ENOTDIR`,
  `ENOTEMPTY`, `EACCES`, `EROFS`, `EFAULT` or `EIO` outcomes. It does not fake
  success for a file or nonempty directory.
- **Directory creation attributes:** `NSFileManager` accepts and decodes the
  historical NSNumber value for `NSFilePosixPermissions` instead of rejecting
  every non-nil attributes dictionary. The virtual filesystem still does not
  persist permission bits, and that limitation is logged rather than
  represented as a false permission model.
- **Segmented-control state:** `UISegmentedControl` now has a bounded host
  object, `initWithItems:`, title/image segments, insertion/removal, selection,
  momentary mode, per-segment enabled/width state, ownership/teardown, and
  basic equal-width touch selection with value-change delivery. Full native
  segmented-control drawing and archived `UISegment` rendering remain out of
  scope.

The universal recorder, strict invalid-object and memory checks, network
offline behavior, app picker and unrelated compatibility code are unchanged.

## Predicted device advances

| App | Dev 1.99 first decisive blocker | Dev 2.00 prediction |
|---|---|---|
| LegendaryWar | missing `valueForKeyPath:` on a mutable dictionary | KVC traversal passes and exposes a later boundary or reaches further UI |
| Peggle | missing localized `NSBundle` resource lookup overload | resource lookup passes when the packaged resource exists, otherwise returns genuine nil |
| Solomon's Boneyard | missing POSIX `rmdir` after Play and save handling | empty save-directory removal passes with real failure semantics preserved |
| Tiki Totems 2 / TrenchesVS | directory creation rejected a standard 0755 attributes dictionary | directory creation proceeds without fabricating permission persistence |
| ZA! 2ndWave | missing `UISegmentedControl initWithItems:` | control initialization/state passes and exposes later UI initialization |

These are blocker-removal predictions, not gameplay claims.

## Validation plan

- Focused guest tests cover nested KVC traversal, global-first explicit
  localization lookup, `rmdir` file/nonempty/success behavior, directory
  creation with POSIX permissions, and segmented-control ownership/state.
- Compilation and automated tests run only on GitHub-hosted machines. The local
  native preflight is intentionally not used after its dependency rebuild
  caused unacceptable host resource saturation on November 2, 2026.
- Hosted test results, skipped/allowed-to-fail steps, arm64 iOS compilation and
  package verification are reported separately.
- Device targets are LegendaryWar, Peggle, Solomon's Boneyard, Tiki Totems 2,
  TrenchesVS and ZA! 2ndWave. Blot, Space Monkey and Spoing are the focused
  regressions because the batch touches Foundation/UIKit and filesystem paths.

## Remaining limitations

Dev 2.00 does not diagnose or tolerate DoDoFly's deallocated dictionary value,
does not decide why 8bitninja's requested resource becomes nil, does not store
filesystem mode bits, and does not claim pixel-accurate segmented-control
rendering. Those boundaries remain explicit so later evidence can discriminate
their causes.
