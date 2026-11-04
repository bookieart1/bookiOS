# Dev 2.24 — shared cocos candidate contracts

Prepared 2026-11-04 against tested commit `51a633428ec2b461761434036b6968d729c15594`, workflow `41778866142` (Dev 2.23). The user's later scope expansion superseded the initial rank-1-only request. Eleven affected titles and two relevant controls are included; no unrelated compatibility cluster is patched. No game is yet declared fixed by this implementation.

## Evidence and predictions

The original logs, hashes, executable metadata and causal distinctions remain in the phase-1 corpus. `manifest.json` preserves each original observed outcome and adds a separate `dev_2_24` field. `dev-2.24-scope.json` is the exact picker/test set. Raw logs and bulk inspection caches are kept locally rather than uploading user evidence or game payloads with the source.

| Games | Confirmed boundary and change | Prediction / falsifier |
|---|---|---|
| Continuity 2 | `C2D-KVC-01`: object setter `@` incorrectly tried to unbox an NSValue. Inspect destination type; pass the object unchanged. Direct object ivars retain the new value and release the old value. | Same legal object setter should no longer panic. A scalar receives its numeric value, not its object address. Another invalid-value origin remains possible. |
| Infinity Field | `C2D-NUMBER-01`: `numberWithUnsignedChar:` absent. Add factory, initializer and unsigned-byte retrieval using existing unsigned number storage. | 128–255 remain positive; proceed past the factory. `objCType` reflects class-cluster storage, not necessarily construction type. |
| Super Stick Golf | `C2D-ARCHIVE-01`: primitive Boolean in keyed archive rejected. Decode NSNumber Boolean using the existing UID cache and ownership. | Existing archive reaches its next stage; repeated UID retains identity. Do not reset saves. |
| Stick Stunt Biker | `C2D-DATE-01`: valid 1970 date goes through an unsigned Duration in description. Reuse signed Gregorian date arithmetic, retaining fractional seconds and strict timezone limitations. | No negative-Duration panic. Pre-2001 dates stay correct rather than clamped. |
| AbstractWar 2.0 | `C2D-ALPHA-01`: `CCTexture2D` requests 128×128, 8-bit alpha-only, NULL color space. Handle single-channel storage, padding, drawing/blending and image conversion. | Cross legal bitmap creation and show correct textures/text. A newly visible but corrupt texture fails validation. |
| Addicus; Kiko | `C2D-STREAM-01`: analytics TCP pair returned NULL, followed by missing read-stream scheduling. Create owned offline stream objects; open fails with POSIX ENETUNREACH. Register context retain/release, default/common-mode schedules, one deferred error notification, status/error, unschedule/close. | Both leave analytics initialization through failure handling. A retry storm or same scheduling failure disproves adequacy. No TCP, HTTP, SSL or DNS success is fabricated. |
| Flow Free; Mini Militia | `C2D-IAD-01`: ADBannerView allocation absent. Add UIView-backed configuration, historical content sizes, copied configuration, iOS 4 nonretained delegate, false loaded/action state, one deferred ADErrorServerFailure. | Pass banner setup; delegate can hide the unavailable banner and continue. No loaded/action callback is invented. |
| Wispin; Super Turbo Action Pig | `C2D-FBO-01`: GL error 0x506 observed later during compositor cleanup despite complete FBO at that later point. New bounded pre-draw/clear completeness probe captures first incomplete state and current context/thread/caller/attachments. | A captured incomplete FBO identifies an earlier producer. If all 512 probes are complete, the cause lies outside the sampled boundaries or later in execution; this does not prove healthy graphics. |
| Dirt | Relevant unchanged graphics/input/audio/save control. | Existing color textures and controls remain correct; no new corruption or slowdown. |
| Tiki Totems Premium | Relevant unchanged Foundation/UIKit/run-loop/transition control. | Existing gameplay, menus and return/relaunch remain correct. |

These are shared Apple-framework defects/partial contracts, not proof cocos itself is faulty. Stream and ad implementations remain **partial offline contracts**, not complete framework emulation. Graphics probes are **diagnostic-only**. The other five corrections target well-established value/bitmap contracts and still require build and device evidence.

## Boundaries and risks

- Preserve strict invalid-object, invalid-pointer and exception handling. No bundle checks in compatibility code; bundle identifiers are used only to prioritize the picker.
- Streams support the existing default/common run-loop model, not arbitrary modes or NSStream delegate delivery. Error callbacks are deferred out of the public API; client context is protected during callback replacement. The error survives close; close suppresses queued notification. No automated reconnect is created. One callback per stream does not prevent guest code itself from retrying; watch for repeated stream-pair creation in the phone log.
- iAd covers the observed iOS 4 banner surface. Error timing is an offline approximation, refresh/interstitials are unsupported, and modern zeroing weak delegates are not claimed. Callback delivery is finite per banner and can be cancelled by clearing its delegate.
- The alpha bitmap's image consumer converts mask coverage to existing RGBA backing. Full color-space, mask metadata and arbitrary bitmap-format support are not claimed. Padding and caller ownership are tested.
- Framebuffer diagnostics require the existing recorder to be enabled, query at most 512 guest draw/clear boundaries per launch, and freeze on the first incomplete result. They do not call glGetError, bind state, or suppress the failing draw. Healthy runs emit only the first/last sampling marker. The permanent universal recorder remains enabled.
- Baseline `Cargo.lock`, `cg_context.rs` working-copy differences and user-owned `artifacts/` are excluded from this implementation commit.

## Validation

Added `cocos_contract_*` tests exercise real ARM guest calls, including signed-double/struct returns; NSNumber 0/127/128/255 and archive round-trips; object setter identity and signed/unsigned scalar setters; Boolean archive UID identity across decoder teardown; alpha-only bitmap row padding, clear and caller-owned storage; stream callback arguments/context retention and finite delivery; iAd size/configuration and deferred error callback; finite framebuffer budget/freeze. Callback selectors are invoked explicitly in headless fixtures because fake environments have no window; physical run-loop timing remains a device test.

No local Rust/Cargo/CMake/preflight is run. Lightweight diff/source checks precede hosted compilation. The existing test workflow includes macOS and Windows library tests; development formatting and full TestApp integration/native sanity steps are skipped, and lint is allowed to fail. Inspect actual results independently from IPA packaging. Baseline's known plist XML test failure must not be hidden. Hosted and package results will be appended after completion; currently pending.

## Physical test, alphabetical, without deleting saves

1. AbstractWar 2.0: launch, start a round, inspect text and textures.
2. Addicus: launch, wait for the menu, try starting play; note repeated loading or slowdown.
3. Continuity 2: start or continue a level, try movement and one transition.
4. Dirt: brief movement/pause/resume, exit and reopen existing save (control).
5. Flow Free: start a puzzle, draw connections, return to menu; note any empty ad area blocking input.
6. Infinity Field: reach menu, start a round and use controls.
7. Kiko: reach level selection, start a puzzle; note repeated loading or slowdown.
8. Mini Militia: attempt an offline/local game; online multiplayer is not expected.
9. Stick Stunt Biker: launch and start a level.
10. Super Stick Golf: use existing progress, select a hole, take a shot, exit/reopen.
11. Super Turbo Action Pig: launch and try Play once; diagnostic test even if it still fails.
12. Tiki Totems Premium: brief level/menu/exit/reopen check (control).
13. Wispin: launch and try Play once; diagnostic test even if it still fails.

Stop each failed title after its first reproducible failure. Send the complete log after the batch. If iDared itself closes, save the current/previous log before it is overwritten and send both parts. Report menu/gameplay, input, graphics, sound, transitions and retained progress; frame counts alone do not establish any of those.

Next milestone: reconcile this same batch's fresh device outcomes and the new FBO boundary before advancing other clusters. Remaining corpus titles retain their previous explicit blockers in the manifest; none is silently promoted or forgotten.

## Primary contract references

- [Apple KVC non-object values](https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/KeyValueCoding/DataTypes.html): destination object versus scalar conversion.
- [Apple Quartz context formats](https://developer.apple.com/library/archive/documentation/GraphicsImaging/Conceptual/drawingwithquartz2d/dq_context/dq_context.html): supported NULL-space 8-bit alpha-only bitmap.
- [Apple CFStream implementation](https://github.com/apple-oss-distributions/CF/blob/main/CFStream.c) and [archived stream guide](https://developer.apple.com/library/archive/documentation/Networking/Conceptual/CFNetwork/CFStreamTasks/CFStreamTasks.html): creation/open separation, context ownership, deferred events and terminal error state.
- [Apple-authored iAd header, SDK mirror](https://github.com/theos/sdks/blob/master/iPhoneOS9.3.sdk/System/Library/Frameworks/iAd.framework/Headers/ADBannerView.h): iOS 4 error codes and unavailable-content delegate contract. Later features are not imported into the iOS 4 subset.
