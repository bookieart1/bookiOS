# Dev 2.29 follow-up contracts and focused diagnostics

Date: 2026-11-04

This is the implementation record for the full Dev 2.28 follow-up requested by
the user. It covers all fourteen titles whose Dev 2.28 contracts crossed their
former first blocker, plus the focused investigations for Current, Wire Way,
Fieldrunners 2, Trism, Carnival and Labyrinth 2. Carnival, Labyrinth 2 and
Trism occur in both sets, so there are seventeen unique affected titles.
Spoing is the only unchanged device regression control.

No implementation makes invalid memory accessible, tolerates invalid Objective-C
objects, fabricates a network/media/persistence result, or branches on an app
identifier. The permanent universal flight recorder remains enabled.

## Compatibility contracts

| App | Dev 2.28 later boundary | Dev 2.29 implementation | Status and prediction |
|---|---|---|---|
| 100 Chambers | KVC path treats a mutable set as if it had `name` state | Bounded KVC selector/argument history | **Diagnostic-only.** The next log should identify the producer/caller before any general KVC behavior is changed. |
| Bike Race Pro | Missing `+[NSBlockOperation blockOperationWithBlock:]` | Real block-operation factory with owned guest-block copy, compiler copy/dispose helpers and existing synchronous queue execution | **Partial contract.** It should cross the missing selector without retaining a stale stack pointer. General concurrent operation scheduling remains out of scope. |
| Cannon Cadets | Unsupported bitmap-layout assertion | Assertion now reports the exact color space, bitmap flags, dimensions, component size, row size and data pointer | **Diagnostic-only.** It should expose the real unsupported layout; no format is accepted speculatively. |
| Carnival | Uncompressed AIFC files fail decoding; presentation rate falls from about 50 to 25 Hz | Decode uncompressed AIFC `NONE`, `twos` and `sowt`; identify other AIFC compression exactly; freeze the first sustained 50/60-to-25 Hz boundary history | **Partial contract plus diagnostic.** Supported uncompressed sounds should play; compressed codecs remain explicit. The slowdown capture separates audio success from frame-cadence collapse. |
| Dismount | Missing `+[NSSet setWithObjects:count:]` | Counted factory with exact count, nil rejection, deduplication and owned elements | **Verified contract candidate.** It should cross the missing selector. |
| Kick the Buddy | Missing `_CFDictionaryContainsKey` | Toll-free-bridged key-presence query, including presence independent of returned value | **Verified contract candidate.** It should cross the missing symbol. |
| Labyrinth 2 | Intermittent collision/motion corruption | Bounded real/simulated accelerometer sample history and scheduled/fired timer interval history | **Diagnostic-only.** The next run can correlate motion samples and simulation cadence without altering physics. |
| Legends of Ooo | Missing `+[NSPersistentStoreCoordinator alloc]` | Correct Core Data framework/class identity and inherited NSObject allocation only | **Strict partial contract.** It should cross class allocation and reveal the first actual Core Data method required. No persistent store or managed-object behavior is fabricated. |
| MX Mayhem | Release/dispatch of a deallocated or corrupted object | Nil-`isa` dispatch now dumps the watched address's retained producer and allocation/lifetime history before preserving the panic | **Diagnostic-only.** It should identify an earlier owner/free/reuse boundary rather than tolerate the object. |
| ParaPanic HD | Missing `_CC_SHA512` | Real one-shot SHA-512 using the existing cryptographic implementation | **Verified contract candidate.** The standard `abc` vector is covered. |
| Perfect Cell | Missing `_fabsf` | Real single-precision absolute value, including signed zero and NaN behavior | **Verified contract candidate.** It should cross the missing symbol. |
| The Sims 3 | Missing `_CGContextFillPath` | Nonzero-winding bitmap path fill through existing path, transform, color, clip and blend machinery; path resets after drawing | **Partial graphics contract.** It should cross the missing symbol; unsupported path elements remain strict. |
| Trainyard | Missing `_glPointSizePointerOES` | ES 1.1 point-size client-array entry point, buffer-offset/guest-pointer translation and visual history | **Partial graphics contract.** Native iOS ES 1.1 should cross the symbol. The desktop GL2 compatibility backend remains strict because it has no equivalent fixed-function array. |
| Trism | Startup transition stalls and Home is not serviced | Each bounded five-second checkpoint now includes up to sixteen live guest thread PCs, stack/link registers and wait reasons | **Diagnostic-only.** A stalled run should identify a recurring instruction or lock/wait state. |

## Additional focused investigations

| App | Question | Dev 2.29 evidence added |
|---|---|---|
| Current | Where does a Play tap stop after reaching the EAGL view? | Larger bounded touch history plus decoded `hitTest`, `pointInside`, coordinate-conversion, `locationInView`, control and target/action Objective-C boundaries. |
| Fieldrunners 2 | Is startup spinning, sleeping or waiting on synchronization? | Bounded all-thread snapshots at periodic recorder checkpoints. |
| Wire Way | Which callback/scene state draws and then fails to leave the black transition? | Larger bounded selector history focused on scene, transition, level, audio and stop callbacks. |
| Carnival | Is the slowdown a presentation collapse, audio decode pressure or something earlier? | A real uncompressed-AIFC correction, exact remaining AIFC codec metadata and a frozen sustained slow-presentation transition. |
| Labyrinth 2 | Are wall crossings correlated with wrong motion conversion or simulation cadence? | Bounded motion values/timestamps plus timer scheduling/firing intervals. |
| Trism | Which guest thread or wait state prevents both gameplay and Home handling? | Bounded all-thread register and blocking-state snapshots. |

## Device test set

The app picker pins these entries first, alphabetically:

1. 100 Chambers
2. Bike Race Pro
3. Cannon Cadets
4. Carnival
5. Current
6. Dismount
7. Fieldrunners 2
8. Kick the Buddy
9. Labyrinth 2
10. Legends of Ooo
11. MX Mayhem
12. ParaPanic HD
13. Perfect Cell
14. Spoing (control)
15. The Sims 3
16. Trainyard
17. Trism
18. Wire Way

The source label and picker header are Dev 2.29. A hosted build and physical
results are separate later evidence; this record does not claim that a title is
fixed before device testing.

## Validation boundary

Focused host tests cover counted sets, dictionary key presence, SHA-512,
single-precision absolute value, uncompressed AIFC decoding and Core Data class
identity/allocation. Existing recorder tests cover bounded histories and
sustained-presentation capture. Compilation, formatting, the complete hosted
test matrix and iOS packaging must be reported separately after the GitHub
workflows run. Local Rust/Cargo/CMake/preflight processes are deliberately not
run on the user's computer because the user previously required hosted-only
validation after a local build attempt destabilized the workstation.

## Physical-device result

The verified Dev 2.29 device evidence, log hashes, per-title outcomes and
preserved next actions are recorded in `dev-2.29-device-result.md`. That result
is an evidence checkpoint only and does not authorize another implementation or
build.
