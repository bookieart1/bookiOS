# Dev 2.08: file-stream, runtime, graphics, and serialization contracts

Dev 2.08 follows the Dev 2.07 physical-device run. It contains only general
compatibility contracts supported by a decisive recorded blocker. It does not
weaken object validity, collection bounds, mutability, memory protection, or
exception handling. It does not fabricate network or social-service success.
The universal flight recorder remains enabled.

The two exact Dev 2.07 inputs are retained as:

- `outputs/device-logs/iDared-Dev-2.07-combined-device-results-part1-2026-11-02.txt`
  (396,506 bytes, SHA-256
  `516aeb9f9064d30b91227209dca5c1a065f9a1a141a02dcb8c03f64aa2d39b3d`)
- `outputs/device-logs/iDared-Dev-2.07-combined-device-results-part2-2026-11-02.txt`
  (1,770,392 bytes, SHA-256
  `25ef2f73080b62a5c42996f518e2e943a81c9e41df6bb7ce8e1620d4b6cd4d33`)

## Selected compatibility changes

| Apps and evidence | General implementation | Contract boundary and prediction |
|---|---|---|
| ADVENA and ZENONIA2 both stop at `_CFReadStreamCreateWithFile`. | Implement file-backed Core Foundation read streams by toll-free bridging to the existing `NSInputStream` file implementation. Export open, read, byte-availability, status, and close from Core Foundation. | File URLs produce owned streams and non-file URLs fail. This predicts passage beyond the shared missing symbol; it does not add network streams. |
| B'Blast stops at `+[TWTweetComposeViewController canSendTweet]`. | Add the legacy Twitter framework and availability query. | The removed host Twitter account service is truthfully unavailable, so the query returns false. No tweet composer or successful service is fabricated. |
| Bungee Ball stops at `-[NSDictionary descriptionInStringsFileFormat]`. | Serialize NSString-keyed/string-valued dictionaries using the historical quoted `.strings` representation with escaping. | Ordering is unspecified. Non-string entries remain rejected rather than being silently coerced. This predicts passage beyond the exact selector. |
| Junny stops at `_CGColorGetComponents`. | Give each created RGB color stable guest-visible RGBA component storage and return its borrowed pointer. Release that storage with the owning color. | The existing generic-RGB color model is preserved. This does not claim support for arbitrary color spaces. |
| Peggle advances through hundreds of loops and frames, then stops at `_chmod`. | Implement path-based `chmod` through the existing guest file descriptor and `fchmod` behavior. | Existing read-only bundle and filesystem errors remain authoritative. This predicts passage beyond the missing POSIX symbol, not successful permission changes on read-only content. |

These are shared subsystem corrections. There are no app identifiers or
per-title behavior branches.

## Every other Dev 2.07 title was assessed

The following titles are intentionally not included as unchanged physical-test
targets in Dev 2.08. A behavior-changing guess at their present boundary would
be unsafe or would provide no meaningful new evidence:

- **Legendary War:** the guest tries to set `height` through KVC on an immutable
  property-list dictionary. Making immutable dictionaries mutable globally
  would violate Foundation semantics and could corrupt unrelated games.
- **Oddy Smog:** the recorded session ends during normal gameplay without an
  emulator panic or recorder terminal dump. The report that the whole host app
  closed therefore needs the corresponding iOS `.ips` termination report;
  changing guest behavior cannot be justified from this log.
- **SPYmouse:** nil reaches mutable NSString borrowing during resource/path
  work. Treating nil as a valid string would hide the producing error and
  contaminate all Foundation callers.
- **SUPERFALL:** a deallocated or corrupted pointer is used as an NSString
  resource name after extended gameplay. Accepting the pointer would weaken
  memory safety; its retained lifetime origin is required first.
- **Super Slyder:** the next boundary is Objective-C exception throwing.
  Correct exception unwinding is a runtime/ABI project, while swallowing the
  exception would alter program control flow and conceal the real failure.
- **Tiki Totems 2:** a deallocated object is inserted into a mutable dictionary
  while starting a level. Retaining an object whose `isa` is already nil would
  tolerate use-after-free and risk host corruption.
- **Toy Bot 2:** the intro remains visually nonresponsive but exits cleanly and
  has no decisive missing API or crash. Inventing a target/action or automatic
  transition could skip legitimate game state.
- **Trenches:** the guest accesses index 3 after reducing an array to length 2.
  Returning a fabricated element or disabling bounds checks would be incorrect.
- **ZA! 2ndWave:** the app loops near 60 Hz but presents near 5 Hz from startup.
  The existing bounded snapshot does not identify whether the time is guest
  simulation, rendering, or content behavior; changing global scheduling would
  risk every working game.
- **Trism:** it was not launched in the supplied Dev 2.07 logs, so no new
  Dev 2.08 hypothesis can be based on this run.

**Silverfish** completed a long run and returned through emulator Home cleanly.
**Space Monkey** also returned cleanly. **Spoing** had one intermittent startup
null read followed by a normal saved-progress run; it remains a regression
control, not a title receiving an arbitrary compatibility change.

## Validation boundaries

Focused host tests cover file-stream ownership and URL rejection, Twitter
availability, `.strings` quoting/escaping, stable color-component storage, and
the new `chmod` guest call. The standard hosted test workflow and unsigned IPA
workflow are inspected separately; skipped and allowed-to-fail steps are not
inferred from a workflow badge. Package verification must confirm the embedded
implementation commit, arm64 executable, development bundle identity, unsigned
state, and universal-recorder marker.

## Focused Dev 2.08 device test

Only apps with an actual compatibility change, plus one broad regression, are
requested:

1. ADVENA: launch and continue as far as possible.
2. B'Blast: launch and continue as far as possible.
3. Bungee Ball: launch and try the first playable path.
4. Junny: launch and try the first playable path.
5. Peggle: pass the logo/menu and try starting gameplay.
6. ZENONIA2: launch and continue as far as possible.
7. Spoing: play briefly, use emulator Home, and reopen once.

Stop each failing title after its first reproducible failure. Do not reset or
delete saves.
