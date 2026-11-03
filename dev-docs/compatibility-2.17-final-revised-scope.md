# Dev 2.17 final revised scope — 2026-11-03

## Baseline and decision

This revision builds on, rather than replaces, the five-game implementation at
`dc6bf51bf9dccd2e29c1af8ad240dad916b8e98a`. The original B'Blast, JellyCar 3,
SUPERFALL, Tiki Totems 2 and Topple 2 analysis remains valid. Its diagnostics
are bounded and preserve the failing behavior; the two semantic corrections
are narrow and evidence-backed. The earlier candidate workflows predate this
combined scope and are historical candidates, not the final Dev 2.17 package.

The two Dev 2.16 device logs identify commit `c84fdd9` and IPA workflow
`41648810896`. They exposed three exact later boundaries: AVURLAsset creation
in EPOCH and Unmechanical, `UIEdgeInsetsFromString` in Junny, and
`CFMakeCollectable` in Wind-up Knight. The current ledger was then screened for
small independent general contracts and exact secondary users. Rolando and
Ragdoll 3 qualified; the other reviewed cases did not.

## Inclusion decision table

| Game | Latest verified behavior | Decisive blocker | Change / classification | Prediction and falsifier | Regression exposure / decision |
| --- | --- | --- | --- | --- | --- |
| B'Blast | Dev 2.15: brief audio/black result; 48 presents before failed parsing and later null fault | Failed `strstr` result is transformed into a bad length, then `strlen(NULL)` | New bounded diagnostic: newest 16 failed searches, caller/thread and 80-byte prefixes | The next crash should include the searched input/pattern. Unrelated content or no captured failure weakens the resource/path theory. | Diagnostic-only; no guest result changes. **Include.** |
| EPOCH | Dev 2.16: advances to missing `+[AVURLAsset URLAssetWithURL:options:]` | Missing AVFoundation factory | Partial general AVURLAsset construction contract | The selector failure must disappear and the app must reach a later operation. The same selector failure falsifies linkage/dispatch. | URL/options are retained; no media success is fabricated. **Include.** |
| JellyCar 3 | Dev 2.15: time conversion faults after unresolved epoch symbol | Missing CF absolute-time epoch data | General data-symbol correction | The unresolved symbol and immediate time-conversion fault disappear. The same fault with resolved data falsifies the diagnosis. | Two documented constant exports only. **Include.** |
| Junny | Dev 2.16: one frame, then missing `_UIEdgeInsetsFromString` | Missing UIKit parser and 32-bit struct-return ABI | General semantic correction | The missing symbol disappears and initialization advances. The same symbol failure falsifies export/ABI integration. | Malformed/nil input produces zero insets; no layout is fabricated. **Include.** |
| Ragdoll 3 | Dev 2.12: startup stops at `sysctlbyname("hw.activecpu")` | Missing Darwin CPU-count key | Small general system-information correction | It must pass `hw.activecpu` and expose its next real boundary. The same missing-key panic falsifies map/export coverage. | Returns one, matching the emulated single-core device and `hw.ncpu`. **Include.** |
| Rolando | Dev 2.12: splash setup stops at `_CGPathGetCurrentPoint` | Missing Core Graphics path query | General semantic correction | Splash initialization must pass the symbol and reach a later boundary. The same missing symbol falsifies export integration. | Read-only path state; empty path is `{0,0}`, closed path returns subpath start. **Include.** |
| Spoing | Dev 2.16: 862 presentations and clean Home return | None | Relevant regression control | Brief gameplay, Home and relaunch should remain normal. Any new fault is a regression signal, not a predicted beneficiary. | Single healthy control covers recorder/input/render/audio/relaunch. **Include.** |
| SUPERFALL | Dev 2.15: smooth gameplay until death; stale resource-name candidate after large history eviction | Origin of watched resource-name address is lost | New persistent bounded lifetime provenance | Death should still fail, but allocation/class/free/reuse history should survive ordinary-table eviction. Address drift or no prior record is inconclusive. | Diagnostic-only; no retain/quarantine. **Include.** |
| Tiki Totems 2 | Dev 2.15: level transition retains a zero-isa value | Unknown raw allocation/object origin | New bounded raw-allocation history | Failure evidence should identify a live raw block, freed/reused object, or no bounded match. A fresh non-object refutes a simple deallocation-only theory. | Diagnostic-only; dictionary validity remains strict. **Include.** |
| Topple 2 | Latest evidence: nil cached ETag enters request construction | `addValue:nil` incorrectly attempted dictionary insertion | Partial request semantic correction, reference-validated | Request construction should pass; a later offline failure is separate. Repeating the same nil insertion falsifies the path. | Removes only the named header; no network/cache success. **Include.** |
| Unmechanical | Dev 2.16: same missing AVURLAsset factory as EPOCH | Missing AVFoundation factory | Secondary validation of the same partial contract | It must pass the identical selector and reveal a later real boundary. Same failure falsifies the shared implementation. | Shares EPOCH's exact API, no extra patch. **Include.** |
| Wind-up Knight | Dev 2.16: missing `_CFMakeCollectable` after layer-state progress | Missing reference-counted Core Foundation compatibility function | Verified historical no-op/identity contract | It must pass the symbol without a retain-count change. Same symbol failure or ownership drift falsifies the implementation. | No garbage collector or invalid-pointer tolerance. **Include.** |

## Additional candidates considered and omitted

| Candidate | Evidence / possible action | Decision |
| --- | --- | --- |
| Peggle | Native iDared terminates without an emulator panic or termination marker. | **Omit.** A matching iOS `.ips` report is required before an emulator change can be justified. |
| ADVENA / ZENONIA 2 | Guest host-not-found/socket exceptions after unavailable services. | **Omit.** Historical offline recovery and Objective-C exception-unwinding behavior are not yet established; success must not be fabricated. |
| Flip's Escape / Slingshot Racing | Severe corruption despite complete FBOs and sane captured viewport. | **Omit.** Current snapshots reject the simple FBO/viewport theory but do not identify a specific wrong state. More generic logging would not yet discriminate the cause. |
| Bungee Ball / Decoloro | Static solid-color/presentation results with bounded hierarchy evidence. | **Omit.** No concrete hierarchy ownership, layout or presentation contract is yet shown wrong. |
| Donguri / FallDown / Mini Militia | Historical missing `ADBannerView`. | **Omit.** A safe implementation needs a coherent unavailable-ad lifecycle and delegate behavior, not an allocation stub; too broad for this batch. |
| Dynamite Jack | Historical missing `_localeconv`. | **Omit.** Stable guest `lconv` storage and locale field semantics need a dedicated implementation and evidence pass. |
| Truckers | Historical missing `UIColor colorWithPatternImage:`. | **Omit.** Pattern-color rendering affects broad drawing behavior and needs its own reference and regression set. |

No unchanged broken title is included merely to see whether it still crashes.
The six games added after the initial five-game investigation all have changed
compatibility code. Spoing is the only physical-device regression control.

## Implemented contracts and boundaries

### AVAsset / AVURLAsset

`AVAsset` and `AVURLAsset` now exist. `+URLAssetWithURL:options:` performs the
normal alloc/init/autorelease sequence; `-initWithURL:options:` retains the URL
and options, and `-URL` returns the identical URL. Teardown balances both
retains. A nil URL remains a strict invalid precondition. No tracks, duration,
metadata, decoding or playback capability is claimed.

### UIEdgeInsetsFromString

The parser accepts the historical braced four-component representation with
whitespace and floating-point values. Nil or malformed strings return zero
insets. `UIEdgeInsets` now uses the correct large-struct guest return path for
the 32-bit ABI. Focused tests cover valid/malformed formats and the hidden
memory-return classification.

### CFMakeCollectable

In iPhone's reference-counted environment, `CFMakeCollectable` returns the
same object and changes no ownership. Null remains null. It does not introduce
garbage collection or make invalid pointers acceptable. A retain-count test
guards this boundary.

### CGPathGetCurrentPoint

The query returns the last move/line point, the subpath start after close, and
zero for an empty path. It uses existing path state and does not change drawing
or flattening. Tests cover all four states.

### hw.activecpu

The Darwin key returns a 32-bit integer value of one, consistent with the
emulated single-core iPhone and existing `hw.ncpu`. The normal sysctl sizing,
buffer validation and error path remain unchanged.

## Validation boundary

Focused hosted tests cover AVURLAsset identity/retention, edge-inset parsing and
guest struct-return ABI, CF ownership neutrality, CGPath current-point state,
the 32-bit active-CPU query, both CF epoch constants, Topple's request behavior,
and every bounded diagnostic capacity/failure-preservation rule. The universal
flight recorder remains permanent and strict invalid memory/object checks stay
fatal. There are no app-identifier branches in compatibility code.

Local Rust, Cargo, CMake, preflight and build helpers are intentionally not run.
Only source review and `git diff --check` are performed locally. Hosted test and
IPA outcomes, skipped/allowed-to-fail steps, artifact identity and hashes are
recorded after the final single implementation push.

## Primary references

- Apple archived Garbage Collection Programming Guide, reference-counted
  `CFMakeCollectable` behavior:
  https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/GarbageCollection/Articles/gcAPI.html
- Apple open-source Core Foundation `CFMakeCollectable` implementation:
  https://github.com/opensource-apple/CF/blob/master/CFRuntime.c
- Apple AVURLAsset documentation:
  https://developer.apple.com/documentation/avfoundation/avurlasset
- Apple archived AVFoundation Programming Guide, URL asset construction:
  https://developer.apple.com/library/archive/documentation/AudioVideo/Conceptual/AVFoundationPG/Articles/01_UsingAssets.html
- Apple UIKit geometry conversion documentation:
  https://developer.apple.com/documentation/uikit/uiedgeinsets
- Apple Core Graphics path documentation:
  https://developer.apple.com/documentation/coregraphics/cgpath/getcurrentpoint()
