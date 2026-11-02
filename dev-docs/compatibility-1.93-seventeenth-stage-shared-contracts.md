# Dev 1.93: seventeenth-stage shared contracts

## Device evidence

The Dev 1.92 device log is preserved outside the source checkout as
`outputs/device-logs/iDared-Dev-1.92-device-results.txt` (451,419 bytes,
SHA-256 `3632a2581c9892f1cf03cb23dc112e64387108c959514889ff79c03f1764c9a2`).
It embeds source `8a2f523564c7799be75b44577dc90aa91a60ecbe`, workflow
`41373651546`, and the permanent universal recorder.

- Cube now has working gun audio, including after changing weapons, and no
  longer displays its failed-sample message. This verifies the bounded PCM WAV
  fallback in the tested scope.
- Junny handles the unavailable Plus+ service, then calls
  `CFArrayCreateMutable(NULL, 2, NULL)`. The emulator rejects the lawful
  positive capacity before the first main loop.
- Rolando handles the unavailable Plus+ service, starts its worker, then sends
  `setAllHTTPHeaderFields:` to a valid `NSMutableURLRequest` before the first
  presented frame.
- Spoing plays and returns cleanly through emulator Home after 8.694 seconds,
  247 main loops and 116 EAGL returns.
- Toy Bot loads its existing save, plays, and returns cleanly through Home
  after 7.045 seconds, 544 loops and 188 EAGL returns.
- Toy Bot 2 reaches its comic transition after 20.922 seconds, 1,686 loops and
  463 EAGL returns. The Dev 1.92 subtype constants are resolved; the next
  blocker is the standard `CATransition setSubtype:` property.
- Fragger is intentionally outside this build. Its whole-host termination and
  saved state require the separate iOS Analytics report investigation.

## Contract decisions

### Core Foundation mutable-array capacity

Historical Core Foundation documentation defines capacity zero as variable
size and a positive capacity as a fixed safe maximum. It is not equivalent to
Foundation's `NSMutableArray initWithCapacity:` storage hint. Dev 1.93 accepts
non-negative capacities, records a positive fixed bound in the existing array
host object, and checks growth against that bound. The null-callback array
remains non-retaining. Negative capacity and growth beyond the declared bound
remain strict failures; callback structures remain unsupported.

This is a general Core Foundation correction. There is no Junny branch and the
emulator does not silently turn a fixed collection into an unlimited one.

### Mutable request aggregate headers

`NSMutableURLRequest.allHTTPHeaderFields` is a nullable copy property. Dev 1.93
implements `setAllHTTPHeaderFields:` by replacing the private backing fields
with an independent mutable copy. Native macOS verification established that
passing nil clears the aggregate property to an empty dictionary rather than
making the getter return nil. The getter continues to return a snapshot, so
caller mutations cannot alias request state.
No network success is fabricated and the existing offline behavior is
unchanged.

### Core Animation transition state

`CATransition` now has its own host object rather than borrowing
`CABasicAnimation` storage. It initializes `type` to the documented fade
default, stores copied `type` and nullable `subtype` strings, exposes both
getters, participates in animation copying, and releases owned state during
teardown. The existing animation/presentation engine remains unchanged: this
implements property state, not a claim that every transition effect is
visually rendered.

## Validation

Focused development-library tests cover:

- positive fixed CFArray capacity, removal/reuse, over-capacity rejection and
  negative-capacity rejection;
- aggregate request-header replacement, copy isolation, native nil-clearing
  behavior, and per-field mutation after clearing;
- CATransition defaults, type/subtype state and animation-copy preservation.

The macOS native Foundation reference fixture also exercises lawful positive
CFArray capacity and request-header copy/nil behavior. The local Windows host
has no Rust toolchain installed, so no local Rust compilation result is
claimed. Hosted macOS and Windows library tests are the compile/test gate; lint,
formatting, guest executable tests and iOS packaging must be reported
separately according to the actual workflow steps.

## Device prediction and focused retest

The prediction is limited to removal of these three exact blockers:

1. **Junny:** should pass the capacity-2 array creation and reveal either UI or
   its next genuine blocker.
2. **Rolando:** should pass aggregate HTTP-header replacement and reveal either
   UI or its next genuine blocker.
3. **Toy Bot:** load the existing save, play briefly, use menu/resume, and exit
   through emulator Home. This is the closest transition/persistence
   regression.
4. **Toy Bot 2:** continue through the comic/Next sequence and attempt to enter
   gameplay.
5. **Spoing:** play briefly and return through emulator Home as the broad
   regression control.

Cube is a verified Dev 1.92 checkpoint and need not be replayed for these
unrelated changes. The universal recorder remains enabled unchanged. Saves
must not be deleted.

## Remaining limitations

- CFArray callback structures other than null remain unsupported.
- HTTP header-name canonicalization and multi-value delimiter behavior remain
  separate work; Dev 1.93 does not change those paths.
- CATransition start/end progress, filters, and visual effect fidelity are not
  claimed by this patch.
- Passing a first blocker does not establish launch or gameplay compatibility.
