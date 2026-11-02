# Dev 2.06: runtime, UIKit, audio graph, and causal contracts

Dev 2.06 follows the fifteen-title Dev 2.05 device batch. It attempts a
general compatibility correction for each newly exact blocker and adds bounded
causal evidence where changing behavior would still be unsafe. It contains no
app-name checks, memory-fault suppression, collection-bounds weakening, fake
network success, or save reset. The universal flight recorder remains enabled.

## Dev 2.05 evidence

| App | Device result and first decisive boundary |
|---|---|
| B'Blast | Launch reaches missing `performSelector:onThread:withObject:waitUntilDone:modes:`. |
| Junny | The prior gradient blocker is removed. Startup now reaches `NGGradientWindow initWithFrame:` and a null read after resolving historical status-bar/keyboard constants that the emulator did not export. |
| Legendary War | It still attempts KVC mutation of an immutable nested property-list dictionary. The bounded origin table was overwritten before the fault. |
| Mos Speedrun | Works in the tested scope. |
| Peggle | Startup requests Apple's historical 3D Mixer Audio Component. The emulator returns null, nevertheless accepted the null component, and later panicked because the resulting output unit had no render callback. |
| Piyo Blocks | Works in the tested scope. |
| Silverfish | One launch returns cleanly; the next reaches missing `NSNumberFormatter setLocale:`. |
| SPYmouse | Startup progresses to missing `NSURL query`. |
| SUPERFALL | Not present in the supplied run; Dev 2.05's formatter fix remains unverified on device. |
| Tiki Totems 2 | The guest creates an invocation for `eventPlay:`, sets its target and selector, but leaves the object argument slot untouched before invoking it. |
| Toy Bot 2 | Intro controls repeatedly fail to resolve nil-target `performAction`; the title does not progress before a clean Home exit. |
| Trenches | The same guest path requests array index 3 at count 2; the earlier bounded array history is insufficient to preserve the full lead-up. |
| Trism | Launch reaches missing `UIButton setTitleShadowOffset:`. |
| ZA! 2ndWave | Gameplay remains abnormally slow and text is corrupted; the log repeatedly fails to map the exact historical font name `American Typewriter`. |
| Spoing | Works in the tested scope and remains the regression control. |

## Compatibility changes

| Primary evidence | General change | Boundary |
|---|---|---|
| B'Blast | Implement the five-argument `performSelector:onThread:withObject:waitUntilDone:modes:` using the existing thread/run-loop queue and wait behavior. | The emulator currently has one effective run-loop mode, so the supplied mode list does not create separate queues. |
| Junny | Export the historical status-bar orientation/frame notification and user-info string constants, keyboard animation duration/curve keys, and `UIEdgeInsetsZero`. | This supplies real constants only; it does not fabricate an orientation transition. |
| Peggle | Reject null/unknown Audio Components correctly; recognize Apple's 3D Mixer; retain a single source-to-destination Audio Unit connection; and render the connected callback through the existing strict PCM path. | This is a partial one-source graph, not arbitrary multi-bus mixing, spatialization, or effects. Unsupported graphs still fail. |
| Silverfish | Add copied/retained `NSNumberFormatter locale` state and normal teardown. | Locale-aware numeric formatting beyond the existing bounded formatter implementation is not claimed. |
| SPYmouse | Implement `NSURL query`, preserving the URL's percent-encoded query text and returning nil when absent. | No URL decoding or network success is fabricated. |
| Tiki Totems 2 | Materialize untouched object argument slots as nil, matching zero-initialized native invocation storage. | Untouched scalar/struct arguments remain strict errors; this is not blanket argument fabrication. |
| Toy Bot 2 | If the active first-responder chain cannot resolve a nil-target control action, fall back to the sending control's responder chain, with both walks bounded. | Unresolvable actions remain logged and ignored; no target is invented. |
| Trism | Add `UIButton titleShadowOffset` state and forward it to the existing title label. | Existing title-shadow color ownership and state fallback are preserved. |
| ZA! 2ndWave | Recognize the exact spaced `American Typewriter` family name as the existing monospace substitute. | This predicts corrected text layout; it does not claim to fix the remaining presentation-rate slowdown. |

## Bounded causal changes

- Legendary War's fixed dictionary-origin table grows from 256 to 1,024
  entries so startup churn is less likely to overwrite the relevant immutable
  dictionary's creation evidence. Immutability remains enforced.
- Trenches' fixed array-event history grows from 48 to 256 events so the next
  strict bounds failure retains substantially more of the preceding mutation
  sequence. Bounds remain enforced.
- The overall recorder test ceiling grows from 192 to 320 KiB to cover those
  fixed arrays. This remains a bounded recorder, not unbounded logging.

## Focused validation

- Rust coverage exercises encoded `NSURL` query extraction and retained/copied
  formatter locale state.
- Guest coverage exercises absent URL queries, omitted object invocation
  arguments, and invalid/null Audio Component construction.
- Local native compilation remains disabled after the November 2 Windows host
  resource incident. Only low-cost diff checks run locally; hosted Windows and
  macOS jobs provide compile/test/package evidence.

## Device order

1. B'Blast: launch and wait for the first result.
2. Junny: launch and try to enter the first playable path.
3. Legendary War: launch to the first reproducible failure.
4. Mos Speedrun: open and play briefly.
5. Peggle: launch through the logo and try to reach the menu/game.
6. Piyo Blocks: open, play briefly, Home, and reopen.
7. Silverfish: launch twice and try to reach gameplay.
8. SPYmouse: launch and try the first playable path.
9. SUPERFALL: launch and try the first playable path; this is the missing Dev 2.05 retest.
10. Tiki Totems 2: launch and try Play.
11. Toy Bot 2: swipe/press through the intro and try to enter gameplay.
12. Trenches: launch to its first reproducible failure.
13. Trism: launch and try to enter gameplay.
14. ZA! 2ndWave: enter gameplay for about 20 seconds and inspect speed/text.
15. Spoing: play briefly, Home, and reopen.

Stop an individual title after its first reproducible failure. Do not delete
or reset saves.
