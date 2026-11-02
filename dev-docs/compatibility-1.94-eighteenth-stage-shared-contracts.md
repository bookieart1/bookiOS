# Dev 1.94: eighteenth-stage shared contracts

## Device evidence

The Dev 1.93 combined device log is preserved outside the source checkout as
`outputs/device-logs/iDared-Dev-1.93-device-results.txt` (333,149 bytes,
SHA-256 `38f1889454b7561091a9a5545b4766529f1fcc55b2f233c6ef89632fe8a9712e`).
It embeds source `3083d76c8884dbcacfe0c536c64b349e1e897f92`, workflow
`41377242331`, and the permanent universal recorder.

All three Dev 1.93 blockers were removed even though the visible launch or
comic endpoints did not yet become functional:

- Junny passes the fixed-capacity Core Foundation array path and now stops at
  the missing C API `_CGColorGetColorSpace`, before its first main loop.
- Rolando passes aggregate request-header assignment. Its worker performs two
  unavailable network requests, drains an autorelease pool, creates the next
  pool, and passes the earlier pointer `0x3646c100` as the `beforeDate:` to
  `NSRunLoop`. That object has already lost its isa, so the emulator correctly
  rejects `timeIntervalSince1970`. The pointer's early allocation and reuse
  are consistent with an SDK-cached `[NSDate distantFuture]` object.
- Toy Bot 2 reaches its comic transition after 25.038 seconds, 2,223 main-loop
  iterations and 673 EAGL returns. The subtype property now works. The
  compositor then incorrectly sends `isCumulative` to `CATransition` while
  applying the animation.
- Toy Bot plays and returns cleanly through emulator Home after 15.046 seconds,
  1,259 loops and 425 EAGL returns.
- Spoing plays and returns cleanly through emulator Home after 15.020 seconds,
  661 loops and 310 EAGL returns.

## Contract decisions

### Core Graphics color-space getter

`CGColorGetColorSpace` is a Get-rule function: it returns the color space
associated with a color without transferring ownership. The current color
host object records a canonical color-space name because the compositor copies
color components by value. Dev 1.94 adds process-lifetime canonical RGB and
gray color-space objects and returns the matching borrowed object. It does not
manufacture a caller-owned object, relax null/object checks, or add unsupported
color models. The current RGB-only `CGColorCreate` boundary remains explicit.

### Foundation boundary-date lifetime

`NSDate.distantFuture` and `distantPast` are stable boundary-date objects on
the native Foundation reference host. Old networking SDK workers cache these
objects for repeated run-loop calls. The previous emulator implementation
allocated and autoreleased a new object on each call, making an SDK-cached
pointer invalid at the next pool drain. Dev 1.94 keeps one process-lifetime
object per boundary date and returns it on subsequent calls. Ordinary date
factory methods retain their existing ownership behavior; invalid objects are
not tolerated and `NSRunLoop` safety checks remain strict.

The timestamp calculations are deliberately unchanged in this stage. The fix
is lifetime/identity, not a new calendar approximation.

### Core Animation class dispatch

Apple's hierarchy makes `CATransition` a direct subclass of `CAAnimation`.
`isCumulative`, `keyPath`, `fromValue`, `toValue`, and `byValue` belong to
property-animation subclasses. The compositor previously handled groups and
keyframes, then assumed every remaining animation was a `CABasicAnimation`.
Dev 1.94 recognizes transition animations before that property path. Their
timing, delegate and removal lifecycle still runs, while the already-updated
model-layer state remains visible. Visual cross-fade/push rendering is still
out of scope and is not fabricated.

## Validation

Focused library tests cover:

- stable, borrowed RGB color-space identity/model/lifetime after source-color
  release;
- distant-past/future identity and usability across autorelease-pool drains;
- application of a `CATransition` without property-animation selector sends.

The guest test source also calls `CGColorGetColorSpace` through its C ABI and
checks the borrowed result after releasing both source colors. The native
macOS Foundation fixture independently checks boundary-date identity and
lifetime across an inner autorelease-pool drain. The local Windows host has no
Rust toolchain, so no local Rust compilation result is claimed. Hosted macOS
and Windows library tests are the compile/test gate; lint, formatting, the full
guest executable and iOS packaging must be reported according to the workflow
steps that actually run.

## Device prediction and focused retest

1. **Junny:** should pass color-space lookup and reveal either UI or the next
   genuine blocker.
2. **Rolando:** should reuse the stable distant-future date safely and reveal
   either UI or its next genuine blocker. Offline requests remain offline.
3. **Spoing:** play briefly and return through emulator Home as the broad
   regression control.
4. **Toy Bot:** load the existing save, play briefly, use menu/resume, and
   return through emulator Home.
5. **Toy Bot 2:** continue through the comic/Next sequence and attempt to enter
   gameplay.

Do not delete saves. Cube, Glowfish and other working checkpoints do not need
an unrelated replay. Fragger remains a separate whole-host/state investigation
that requires its matching iOS Analytics report, not a speculative patch in
this build.

## Remaining limitations

- Core Graphics still supports only the color models already represented by
  the emulator; this patch does not claim broad color-management fidelity.
- The boundary dates keep their previous approximate values. Only their
  process lifetime and repeated-call identity change.
- CATransition type/subtype state and lifecycle are supported, but the visual
  transition effect, start/end progress and filters are not rendered.
- Passing a first blocker does not establish launch, gameplay or full-game
  compatibility.
