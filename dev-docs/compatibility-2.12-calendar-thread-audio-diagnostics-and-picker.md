# Dev 2.12: calendar, thread, Audio Unit, causal-value, and picker work

## Evidence selected from Dev 2.11

The Dev 2.11 device log embeds implementation `d13d547c` and workflow
`41586511172`. The permanent recorder and bounded object history operated in
all eight eligible launches without a user-observed slowdown.

- Crash Drive and Jack Lumber resolved to the same missing data import:
  `_NSBuddhistCalendar`. This is no longer an unidentified null-page family.
- Peggle reached an exact Audio Unit query: stream format, input scope, input
  bus 1 on a 3D mixer. The existing implementation treated all stream formats
  as bus zero and asserted before it could return the bus's format.
- Topple 2's `NSThread` object was released and its storage reused as a pthread
  mutex before `_touchHLE_NSThreadInvocationHelper` ran. The started thread did
  not own an execution-lifetime retain.
- SUPERFALL and Tiki Totems 2 still consumed invalid values after enough object
  traffic to evict their address histories. Their final consumed addresses are
  known, but the bounded history did not identify a producer.
- Anomaly HD remained an ownership ambiguity. No semantic change is made for
  it in this build.

## General corrections

1. Foundation exports the historical `NSCalendar` identifier constants with
   their documented string values. This is identifier availability only;
   non-Gregorian date arithmetic is not claimed.
2. A started `NSThread` now holds one balanced logical retain until execution
   finishes. Its target and argument remain retained through execution and are
   released exactly once at exit. Lazily created facades for POSIX threads do
   not fabricate an execution retain.
3. A 3D mixer's input element count now owns one independent stream-format
   slot per input bus. Stream-format and sample-rate queries validate the
   requested scope/element and return `paramErr` for an invalid bus instead of
   asserting or silently aliasing bus zero.

These changes preserve strict invalid-memory and invalid-object failures. They
do not fabricate calendar calculations, microphone capture, audio conversion,
networking, or successful gameplay.

## Bounded causal-value correlation

At a terminal panic, the recorder now checks whether any of the final consumed
addresses was returned by one of the already retained recent API/Objective-C
boundaries. It prints at most eight producer hints. No new event ring, global
message trace, guest-store trace, object retention, or per-frame work is added.
The hint is correlation, not proof of allocation or lifetime origin. Its first
purpose is to discriminate the SUPERFALL and Tiki Totems 2 invalid-value cases;
absence remains inconclusive.

## Native picker changes

The default remains **All Games**. A compact **Games** menu in the bottom bar
can show **Playable** (working plus useful near-working paths) or **Needs Work**.
Filtering maps back to each app's original Rust index, so it cannot launch a
different title. The selection is display-only and persisted by the native
picker; it does not alter saves, diagnostics, launch options, or emulation.
Unknown titles are conservatively placed in Needs Work. The embedded grouping
must be revised when a physical-device result changes a title's status.

The active Dev 2.12 device batch remains first. The unresolved Dev 2.10
collection (ADVENA, B'Blast, BouncyMouse, Bungee Ball, Flip's Escape, Junny,
The Legend of Crystal Valley, Unmechanical, and ZENONIA2) follows as a separate
visible collection before the remaining alphabetical library. This restores
that earlier work without pretending those titles received new semantic fixes
in Dev 2.12.

## Predictions and falsifiers

| Primary test | Prediction | Falsifier / remaining limit |
|---|---|---|
| Crash Drive, Jack Lumber | Both advance past `_NSBuddhistCalendar`. | The same unresolved import means the export path is wrong. Any later blocker is separate. |
| Peggle | The bus-1 stream-format query succeeds and initialization advances. | The same query/assertion falsifies the bus model. Later audio or gameplay is unproven. |
| Topple 2 | The thread receiver remains an `NSThread` until its entry helper runs. | Reuse despite the logical retain rejects premature receiver deallocation as the full cause. |
| SUPERFALL, Tiki Totems 2 | A repeated crash may identify the last retained boundary that returned the consumed address. | No hint is expected when the producer is older than the ring or came from a guest store; do not expand logging automatically. |
| Slingshot Racing, Stick Skater, Wind-up Knight | The now-identified calendar family can be checked across the related startup code. | A different import or failure splits the former engine-shaped cluster. |
| Spoing | Brief gameplay remains healthy. | A repeatable regression requires subsystem-specific review. |
| Picker filter | All/Playable/Needs Work show the intended subsets and launch the tapped title. | Wrong title, lost page state, or hidden known-playable title is a picker bug, not a game compatibility result. |

## Validation boundary

Focused unit tests cover identifier values, execution-lifetime ownership,
independent mixer input buses, recent-return correlation, and conservative
picker classification. Following the workstation-safety instruction, no local
Rust, Cargo, CMake, preflight, or build helper is run. Hosted macOS/Windows
tests and the hosted unsigned iOS archive are the compilation gates. Skipped
and allowed-to-fail steps remain separate from actual passes.

## Hosted validation and delivered package

The implementation is `33508cbf20b7243d5bb6771bf2411f669764ab2c` (the
preceding `9d4ed4cc` commit had only a compile-time diagnostic format-string
error; no contract changed in the correction).

- Test run [41591043189](https://github.com/bookieart1/iDared32bit/actions/runs/41591043189)
  passed the native Foundation reference, lint, and **222/222 library tests**
  on both macOS and Windows. macOS also passed three focused diagnostic
  reproducer invocations. Android failed while installing NDK r28c before its
  lint step. Formatting, the full guest-executable suite, and native sanity
  were branch-gated/skipped and are not claimed as passes.
- IPA run [41591043208](https://github.com/bookieart1/iDared32bit/actions/runs/41591043208)
  compiled and packaged the arm64 iOS app successfully.
- The verified IPA is 20,219,652 bytes, SHA-256
  `8b5c6f9071f8b1e3e574c8d4ed466e0596ca42afbb7f1dff9538f1c66b091a08`.
  It embeds the implementation commit/workflow, uses
  `com.bookieart1.idared32bit.dev` / `iDared Dev`, targets iOS 16.0, and has
  no provisioning profile or code-signature load command/directory.
- The preserved native executable and dSYM UUID is
  `5C3569AB-5D80-3EE0-A58A-A2AFBFBE0E19` (arm64). The matching archive is only
  valid for `.ips` reports from this exact package.

Physical-device compatibility and picker acceptance remain pending. Hosted
compilation does not promote any game to working.
