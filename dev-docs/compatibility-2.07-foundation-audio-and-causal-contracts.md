# Dev 2.07: Foundation, audio topology, and causal contracts

Dev 2.07 continues the unresolved Dev 2.06 batch and adds four low-risk
contracts identified by the fresh library sweep. The new titles extend the
batch; they do not replace the earlier targets. The build contains no app-name
branches, permissive object-pointer handling, collection-bounds weakening,
fake network success, or save reset. The universal flight recorder remains
enabled.

## Device evidence and selected changes

| Primary evidence | General implementation | Contract boundary |
|---|---|---|
| B'Blast | Add process-wide `NSSetUncaughtExceptionHandler` / `_NSGetUncaughtExceptionHandler` round-trip state. | The callback is stored exactly; this does not swallow exceptions or install a host exception policy. |
| Junny | Allow `UINib` to retain and resolve resources from the bundle supplied to `nibWithNibName:bundle:`. | Nil still means the main bundle. NIB decoding behavior itself is unchanged. |
| Peggle | Implement Audio Unit property 11 (`kAudioUnitProperty_ElementCount`) for the 3D mixer input scope. | The existing one-source partial 3D-mixer graph remains the supported topology; this is not arbitrary multi-bus spatial mixing. |
| Silverfish and Trism | Add copied `NSNumberFormatter groupingSeparator` state and use it in bounded formatting/parsing. | Locale-complete number formatting is not claimed. |
| SUPERFALL | Add the matching copied `currencyGroupingSeparator` state. | Currency symbols, locale rules, and arbitrary format patterns remain outside the existing partial formatter. |
| Tiki Totems 2 | Implement `NSMutableDictionary removeObjectsForKeys:` through strict per-key removal. | Dictionary mutability and key validity remain enforced. |
| ADVENA | Populate the requested `NSError**` when `NSString initWithContentsOfFile:encoding:error:` cannot find a file. | Decode failures still need a separate error mapping; no missing resource is fabricated. |
| Oddy Smog | Implement `NSString longLongValue` with whitespace/sign/prefix parsing and bounded overflow behavior. | This removes the exact selector blocker; it is not presented as a fix for the separately observed visual corruption. |
| Super Slyder and Bungee Ball | Implement `+[NSURLConnection canHandleRequest:]` according to the transport the emulator can actually start. | Local-file requests return true. Network requests return false; network success is not fabricated. |
| ZENONIA2 | Map historical Core Foundation encoding `0x422` to NSString's DOS Korean encoding and the CP949/EUC-KR codec. | Valid CP949 input is supported; this does not guess malformed byte sequences. |
| ZA! 2ndWave and Roboto diagnostics | Freeze one bounded presentation snapshot after three sustained slow intervals even when the title was slow from its first frame. | Existing healthy-to-slow capture remains. No scheduling, frame pacing, or rendering behavior changes. |

## Evidence retained without speculative behavior changes

- Legendary War still needs historical immutable property-list KVC semantics
  and guest caller symbolization before a safe correction is known.
- SPYmouse still needs a narrow record of the nil NSString producer; nil
  handling remains strict.
- Toy Bot 2's intro is a noncrashing visual/interaction stall. No target or
  action is invented without evidence.
- Trenches still performs a stale index-3 access at count 2. Array bounds remain
  strict while the existing expanded history preserves the mutation sequence.
- Anomaly HD, Flip's Escape, Paper Toss, and Unmechanical remain evidence-first
  investigations. Modern Combat 2 is an obsolete-service track, Potty 2 is
  encrypted, and Jungle is inactive at the user's request.

## Focused validation

- Unit tests cover uncaught-handler state, Audio Unit element-count round-trip,
  formatter separator ownership/use, dictionary bulk removal, NSString numeric
  parsing, missing-file NSError metadata, Korean encoding conversion, truthful
  URL capability reporting, and both slow-capture modes.
- Local preflight uses the pinned Rust toolchain, reusable target cache, and a
  disposable detached worktree. Hosted tests and iOS packaging are inspected
  separately; a green workflow badge is not treated as proof of skipped jobs.
- The IPA must retain the development bundle identity and permanent universal
  recorder marker, contain an arm64 executable, and remain unsigned without a
  provisioning profile.

## Device test order

1. ADVENA: launch to the first result.
2. B'Blast: launch to the first result.
3. Bungee Ball: launch and try the first playable path.
4. Junny: launch and try the first playable path.
5. Legendary War: launch to the first reproducible failure.
6. Oddy Smog: play briefly and exercise the path that previously reached the numeric selector.
7. Peggle: launch through its logo and try to reach the menu/game.
8. Silverfish: launch twice and try gameplay.
9. SPYmouse: launch to the first reproducible failure.
10. SUPERFALL: launch and try the first playable path.
11. Super Slyder: launch and try the first playable path.
12. Tiki Totems 2: launch and try Play.
13. Toy Bot 2: scroll through the intro and try to enter gameplay.
14. Trenches: launch to its first reproducible failure.
15. Trism: launch and try the first playable path.
16. ZA! 2ndWave: enter gameplay for about 20 seconds so the slow-from-start snapshot can freeze.
17. ZENONIA2: launch and try the first playable path.
18. Spoing: play briefly, Home, and reopen as the regression control.

Stop an individual title after its first reproducible failure. Do not delete
or reset saves.
