# Dev 2.05: broad compatibility contracts and causal diagnostics

Dev 2.05 is a deliberately broad batch selected from the current compatibility
ledger. It combines independent, evidence-backed framework contracts so one
hosted IPA build can test substantially more than one title. It does not make
app-name checks or relax strict memory, object, collection, or ABI validation.

## Direct compatibility contracts

| Primary app evidence | General change | Prediction and boundary |
|---|---|---|
| B'Blast | Complete the 32-bit guest `CFNumberCreate`/`CFNumberGetValue` surface used by signed 64-bit, `long`, `CFIndex`, `NSInteger`, `double`, and `CGFloat` values. Extend lossless-conversion checks to the same types. | Removes the exact type-11 `CFNumberGetValue` blocker. Numeric narrowing still reports loss rather than fabricating a value. |
| Junny | Implement retained `CGGradientCreateWithColors`, `CGGradientRetain`, and `CGGradientRelease`, snapshotting color stops and optional locations. | Removes the exact missing gradient export. It supports RGB colors and creation/ownership only; gradient drawing remains governed by existing context support. |
| Mos Speedrun | Implement `UIApplication canOpenURL:` for the host bridge's actually supported standard schemes. | Standard web, mail, and telephone schemes report available; malformed and custom third-party schemes remain unavailable. No external service success is fabricated. |
| Peggle | Accept historical RemoteIO 32-bit little-endian float PCM and convert bounded samples to PCM16 for OpenAL. | Removes the selected-ASBD rejection while preserving channel, endian, packing, and format validation. This does not implement the Apple 3D Mixer. |
| Piyo Blocks | Implement `NSKeyedArchiver encodeInteger:forKey:` using the existing keyed archive state. | Removes the exact save/Home selector blocker and preserves 32-bit `NSInteger` ABI. |
| SPYmouse | Implement Unicode decimal-digit membership for `NSCharacterSet decimalDigitCharacterSet`. | Removes the exact missing selector while excluding non-decimal numeric symbols. |
| SUPERFALL and Silverfish | Store and return `NSNumberFormatter formatterBehavior`. | Removes SUPERFALL's exact missing setter and supplies the same historical formatter state used by Silverfish. It does not claim every locale/formatter behavior is implemented. |
| Toy Bot 2 | Preserve nil UIControl targets and resolve their action through the active responder chain when the event fires. | Replaces the earlier ignored registration with UIKit's target-action contract. An unresolved action is logged and ignored as UIKit would; no app-specific target is invented. |
| Trism | Implement retained per-state `UIButton` title-shadow colors and apply the current value to the title label. | Removes the missing state contract while preserving normal-state fallback and ownership. |
| ZA! Second Wave | Implement pending/deferred `pthread_cancel`, delivery at `pthread_testcancel`, and self-only immediate delivery for asynchronous cancellation. | Removes the exit-time missing export without terminating another guest coroutine unsafely. Other POSIX cancellation points remain out of scope. |

## Bounded causal improvements

The following titles still lack enough evidence for a safe behavioral change.
Their safety checks remain strict; Dev 2.05 changes the evidence captured at
the existing failure boundary.

| App | Diagnostic refinement | Question it should answer |
|---|---|---|
| Legendary War | Retain more property-list dictionary origins and record the guest PC/LR that created each tracked container. | Was the immutable nested dictionary produced by an option that should have made descendants mutable, or is the guest mutating a legitimately immutable object? |
| Tiki Totems 2 | Record bounded invocation selector names and target classes with the existing argument-set mask. | Which concrete invocation contract reaches `invoke` without argument index 2, and is the omission tied to a specific target/selector path? |
| Trenches | Preserve 16 guest instruction bytes around each recorded array mutation/access caller. | Do the valid removals and later out-of-range access come from one guest routine/loop, allowing the earlier semantic error to be located without weakening bounds? |
| ZA! Second Wave | Report absolute presentation cadence from recorder start in addition to transition-triggered snapshots. | Is the title slow from its first frame, explaining why the existing healthy-to-slow transition detector never fired? |

All additions remain fixed-size. The universal recorder remains enabled for
every eligible app, but it still performs no unbounded per-frame or instruction
trace.

## Regression scope

Spoing is deliberately unchanged and is the primary cross-subsystem canary.
The device run should also verify apps that reach each changed contract rather
than replaying unrelated known-working titles.

## Local and hosted validation boundary

Focused unit coverage was added for 64-bit/guest-sized CF numbers, Unicode
decimal digits, formatter behavior, gradient stop snapshotting, and float PCM
conversion. Source formatting and whitespace checks run locally. Native local
compilation remains disabled after the documented Windows host-resource
incident; hosted Windows/macOS tests and the macOS arm64 IPA build are the
authoritative compile checks. Their skipped and allowed-to-fail steps must be
reported separately.

## Focused device order

1. B'Blast: launch and wait for its normal startup result.
2. Junny: launch and try the first playable path.
3. Legendary War: launch until its first reproducible failure.
4. Mos Speedrun: launch, navigate the opening interface, and try Play.
5. Peggle: launch and wait through the logo/audio initialization.
6. Piyo Blocks: play briefly, exit with emulator Home, reopen, and check save state.
7. Silverfish: launch and try to reach gameplay.
8. SPYmouse: launch and try the first playable path.
9. SUPERFALL: launch and try the first playable path.
10. Tiki Totems 2: launch until its first reproducible failure.
11. Toy Bot 2: press Next through the intro and try to enter gameplay.
12. Trenches: launch until its first reproducible failure.
13. Trism: launch and navigate the first interface/gameplay path.
14. ZA! Second Wave: enter gameplay for roughly 20 seconds, open/close the pause menu, then exit with emulator Home.
15. Spoing: play briefly, exit with emulator Home, and reopen.

Stop an individual title after its first reproducible failure. No save reset is
requested.
