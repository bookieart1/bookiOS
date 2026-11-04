# Dev 2.28 locked 27-title contracts and provenance

Date: November 4, 2026

Dev 2.28 is the user-authorized maximum coherent follow-up to the verified Dev
2.27 pass. All 27 requested titles are pinned first in strict alphabetical
order. Super Hexagon is matched through display name, bundle identifier and IPA
filename so its empty display-name metadata cannot exclude it again. The app
header and SwiftUI picker identify this build as Dev 2.28.

## Compatibility corrections

| Title | General implementation | Prediction and boundary |
|---|---|---|
| 100 Chambers | `NSOperationQueue addOperationWithBlock:` executes within the queue's existing deterministic synchronous partial model. | The missing selector disappears; real asynchronous Block-copy/queue concurrency remains out of scope. |
| Bike Race Pro | One-argument `NSURL URLByAppendingPathComponent:` delegates to the historical two-argument path operation with `isDirectory:NO`. | Crosses the missing selector without changing URL identity or filesystem results. |
| Cannon Cadets | Core Graphics linear-gradient rasterization uses the context transform, clip, blend mode and gradient extension flags. | Crosses `_CGContextDrawLinearGradient`; rendering, not fabricated success, is observable. |
| Carnival | Stable local/Darwin CF notification centers with synchronous add/post/remove delivery and bounded ownership. | Crosses `_CFNotificationCenterGetDarwinNotifyCenter`; no external Darwin service is fabricated. |
| Dismount | `scanf` assignment suppression (`%*...`) consumes matching input without consuming a vararg or incrementing the assignment count. | Removes the parser panic while preserving matching failures and EOF behavior. |
| Kick the Buddy | `NSSet`/`NSMutableSet setWithSet:` creates membership-equivalent sets with appropriate mutability. | Crosses the missing factory without sharing mutable backing state. |
| Labyrinth 2 | Invocation-backed scheduled timers retain the invocation, join the current run loop and call `invoke` without a timer argument. | Crosses the missing timer selector while preserving timer invalidation/repeat behavior. |
| Legends of Ooo | Same one-argument URL contract as Bike Race Pro. | Same falsifier: the title must reach a later boundary. |
| MX Mayhem | Recursive `subpathsOfDirectoryAtPath:error:` uses the existing directory enumerator and returns a Cocoa error for a missing directory. | Crosses the missing selector; traversal remains inside the guest filesystem. |
| ParaPanic HD | One-shot CommonCrypto SHA-224 produces the real 28-byte digest. | Crosses `_CC_SHA224`; a standard `abc` vector is tested. |
| Perfect Cell | `NSScanner scanLongLong:` and corrected signed-prefix parsing consume only one optional leading sign and decimal digits, with bounded overflow. | Crosses the missing selector and removes the old permissive `+`/`-` parsing. |
| The Sims 3 | `CGContextBeginPath` releases the old current path and installs an empty mutable path. | Crosses the missing symbol without drawing or altering graphics state. |
| Trainyard | `NSData mutableCopyWithZone:` now passes the historical const-byte ABI to `NSMutableData initWithBytes:length:`. | Removes the strict typed-dispatch mismatch; denied writable paths remain visible. |
| Trism | One-argument `cancelPreviousPerformRequestsWithTarget:` removes all matching pending run-loop requests, releasing retained arguments and waking waiters. | Crosses the missing class selector without cancelling other targets. |

## Bounded diagnostic experiments

The universal recorder remains permanent and strict. Dev 2.28 adds three
general, fixed-size evidence sources: 16 recent Objective-C nil returns, 24
autorelease enqueues with pool/object/class/refcount/caller context, and focused
input, presentation, callback/invocation boundaries. Recording does not call
guest code, retain objects, make invalid memory readable or change API results.

| Title | Question the new evidence is meant to answer |
|---|---|
| Clowning Around | Which callback/invocation boundary first consumes the absent host object? |
| Current | Which guest input boundary receives each delivered touch before the menu stays inert? |
| Fieldrunners | Which retained nil-return producer precedes the mutable-string consumption? |
| Fieldrunners 2 | Do focused presentation/callback boundaries stop before its existing six-thread initialization wait? |
| JellyCar 1.5.4 | Which pool enqueues and later drains the reused `NSNumber` address? |
| NinJump | Which invocation/operation boundary and nil producer precede the null-page read? |
| Peggle | Does the longer producer/lifetime history survive its intermittent host-versus-guest termination path? |
| Scribblenauts | Which retained nil producer precedes the final invalid consumption? |
| SUPERFALL | Which pool owns the resource-name object that later dies? |
| Super Hexagon | What nil/input/presentation boundary exists immediately before its pre-frame null access? |
| Tiki Totems 2 | Whether the suspicious raw value ever enters an Objective-C autorelease/lifetime path. |
| Wire Way | Which presentation transition draws the still-interactive black pause/end state? |

Spoing is the sole healthy control. It has no compatibility change and is
included only to detect recorder, input, rendering, audio, exit or relaunch
regressions.

## Safety and remaining limits

- No bundle-identifier compatibility branches are present; bundle/file identity
  is used only to arrange the requested picker order.
- No networking, media, service, graphics or audio success is fabricated.
- Strict invalid-object and invalid-memory failures remain enabled.
- The operation-queue block path is explicitly a synchronous partial contract,
  consistent with the existing queue model; it is not a claim of GCD-quality
  concurrent scheduling.
- Device testing is still required. Crossing a former blocker is not proof of
  gameplay compatibility.
