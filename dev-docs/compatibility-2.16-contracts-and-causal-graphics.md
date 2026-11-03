# Dev 2.16 contracts and causal graphics diagnostics

## Evidence

Dev 2.16 follows Dev 2.15 commit `9fd3fb59`. Two device logs embed that
commit and hosted run `41642274758`. The supplied Decoloro screenshot shows
the real title artwork and heard music between pink regions, not a uniformly
blank launch. The previously supplied Flip's Escape recording was inspected at
eight frames per second: a correct menu is replaced after input by repeated
sprite-atlas rows and independently moving misplaced sprites while frames keep
presenting. That is active rendering corruption, not a freeze.

## General compatibility changes

- `NSString initWithContentsOfURL:encoding:error:` now returns `nil` with a
  standard offline `NSError` and failing-URL metadata. It does not fabricate
  network content. ADVENA's dead AdTilt URL is the primary test.
- `NSProcessInfo.environment` returns an immutable snapshot of the emulated
  guest environment, including guest `setenv`/`unsetenv` changes. It never
  exposes host process variables. EPOCH and Unmechanical are the primary tests.
- A decoded `UIBarButtonItem` accepts the historical UIControl-shaped
  target/action reconnection selector while retaining the ordinary single
  target/action contract. Junny is the primary test.
- `CALayer.borderColor` and `borderWidth` store and return historical property
  state. Border rasterization is not claimed. Wind-up Knight is the primary
  test.
- `+[NSException raise:format:]` formats the guest exception and then fails
  strictly with its real name and reason. Objective-C exception unwinding is
  still unavailable, so this is diagnostic/partial rather than a compatibility
  success claim. ZENONIA2 is the primary test.

## Bounded causal diagnostics

- Peggle's first name-entry interaction creates a root-layer compositor error
  (`GL_INVALID_FRAMEBUFFER_OPERATION`). Dev 2.16 keeps the failure strict and
  checks framebuffer/error state at layer entry, transform, background,
  texture bind/upload, and immediately around the texture draw. This should
  identify the first compositor operation that makes the framebuffer invalid.
- Bungee Ball and Decoloro receive a 24-view, depth-eight UIKit hierarchy
  snapshot only when emulator Home is pressed. This distinguishes a missing,
  hidden, off-screen, or noninteractive interface from a run-loop stall.
- Flip's Escape and Slingshot Racing receive at most 32 pre-draw GL snapshots
  per app run and eight per input window. Each records draw kind, viewport,
  active/client texture units, texture and buffer bindings, matrix mode, blend,
  and texturing state. No GL error is consumed and no rendering state changes.

## Literal per-title action

| Title | Dev 2.16 action |
|---|---|
| ADVENA | Compatibility change: standard offline URL error semantics. |
| B'Blast | No new code path. Its null-page origin remains unresolved; changing memory behavior would be unsafe. |
| Bungee Ball | Diagnostic-only: bounded view hierarchy at manual exit. |
| Crash Drive | No change; relevant working rendering/input control. |
| Decoloro | Diagnostic-only: bounded view hierarchy at manual exit. |
| EPOCH | Compatibility change: guest `NSProcessInfo.environment`. |
| Flip's Escape | Diagnostic-only: bounded pre-draw GL state history. |
| Jack Lumber | No change; relevant working control. |
| JellyCar 3 | No new patch. The former missing-class blocker is gone; the new early null pointer has no proven producer yet. |
| Junny | Compatibility change: decoded bar-button target/action reconnection. |
| LOCV | No change; relevant working image/rendering control. |
| Peggle | Diagnostic-only: exact compositor/FBO failure stage. A native `.ips` is still valuable if iOS terminates before this assertion reaches the log. |
| Slingshot Racing | Diagnostic-only: bounded pre-draw GL state history. |
| Stick Skater | No change and not present in the supplied Dev 2.15 logs. |
| SUPERFALL | No new patch. The death path consumes a deallocated string, but provenance was evicted; accepting the stale object would be unsafe. |
| Tiki Totems 2 | No new patch. The stale-object retain lacks a proven producer; tolerating it would weaken object safety. |
| Topple 2 | No patch. Its dead-service ETag is nil; fabricating it or allowing nil dictionary values would be incorrect. |
| Unmechanical | Compatibility change: guest `NSProcessInfo.environment`. |
| Wind-up Knight | Partial compatibility change: CALayer border property state; visible border drawing remains unsupported. |
| ZENONIA2 | Diagnostic/partial contract: formatted strict exception report; no exception swallowing. |
| Spoing | No change; intermittent first-launch null fault remains unproven, and the successful second launch is retained as a control. |

## Validation boundary

Focused unit tests cover offline URL metadata, guest process environment,
bar-button connection state and ownership, layer border state, and formatted
exception reporting. Hosted workflows are required for compilation and tests;
no local Rust, Cargo, CMake, preflight, or build helper is run on the user's
computer. Physical results remain necessary before any title is promoted.
