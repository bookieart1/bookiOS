# Dev 2.28 physical-device result

Date: 2026-11-04

All four supplied logs identify implementation
`5f421ee519024ee137ace6e70e54f4df03d79400` and unsigned IPA workflow
`41862962461`. The universal recorder and the Dev 2.28 bounded provenance
extensions were active. This is verified Dev 2.28 device evidence rather than
an inference from an archive name or picker label.

## Evidence

| File | SHA-256 | Scope |
|---|---|---|
| `444CC7A8-984C-43DD-B821-9F2CDC4BB189/1-Log.txt` | `d6f3d294168fe613938e529476f0bf6f6570e59aefdb6fa23612cb6079815277` | 100 Chambers through Fieldrunners 2 |
| `444CC7A8-984C-43DD-B821-9F2CDC4BB189/2-Log.txt` | `034a50c789158620162b8c1754e10129e1db60ddf68ae4a0ffad6aba6f82eebf` | JellyCar 1.5.4 through Peggle |
| `444CC7A8-984C-43DD-B821-9F2CDC4BB189/3-Log.txt` | `9c17f9d4dd48ce11970120de92f0ee1336a8cb5d4fc29da15c016b6cf09f8492` | Perfect Cell through Trism |
| `444CC7A8-984C-43DD-B821-9F2CDC4BB189/4-Log.txt` | `191365ee68f006d2714075badab478eb6e77faf79a6f0b18d1e5394356dcffdd` | Wire Way follow-up |

## Executive result

All fourteen direct Dev 2.28 contracts crossed the exact boundary they were
designed to remove. That is implementation progress, but not fourteen fixed
games: most immediately revealed a later independent missing contract or a
lifetime failure. Carnival and Labyrinth 2 run in the tested scope, but both
retain serious defects reported by the user.

The diagnostic half was also productive. Current is not failing because the
host loses taps; the EAGL view receives the touch stream and continues
presenting at about 59 Hz. Fieldrunners 2 and Trism stop servicing the
application event loop, which is why even the emulator Home control cannot be
handled. Wire Way continues to render and explicitly draws a full-screen black
primitive after the level transition; this is not a frozen compositor.

## Direct-contract results

| App | Dev 2.28 result | Current first boundary / decision |
|---|---|---|
| 100 Chambers | Crosses block-backed operation queue execution. | A `_touchHLE_NSMutableSet` instance is then treated as having a setter for `name`. Determine the historical KVC/dynamic-setter path and guest caller before changing NSObject semantics. |
| Bike Race Pro | Crosses one-argument URL path appending. | Missing `+[NSBlockOperation blockOperationWithBlock:]`; this is a bounded general Foundation candidate with copied-block lifetime tests. |
| Cannon Cadets | Crosses linear-gradient drawing. | Reaches a later bitmap assertion requiring the currently unsupported bitmap layout/format. Preserve the assertion until the exact image format is decoded. |
| Carnival | Launches and runs for 91.6 seconds with 10,999 main loops and 3,467 presentations; no crash. | The user observes load-related slowdowns severe enough to make play unenjoyable. Presentation pace falls from roughly 50 Hz to roughly 25 Hz late in the session. Numerous AIFC music/effect files fail real decoding with error `2003334207`. AIFC decoding is a concrete audio gap; it is not yet proven to be the sole performance cause. |
| Dismount | Crosses scanf assignment suppression. | Missing `+[NSSet setWithObjects:count:]`; implement only the real counted-object factory with correct nil/count/ownership behavior. |
| Kick the Buddy | Crosses `setWithSet:`. | Missing `_CFDictionaryContainsKey`; a narrow real Core Foundation contract is justified. |
| Labyrinth 2 | Runs for 117.9 seconds with 4,600 main loops and 1,982 presentations; no recorded panic. | The user observes the ball passing through walls, disappearing/reappearing, and an incorrect-looking menu. The timer blocker is removed, but compatibility is only partial. Correlate bounded accelerometer samples and simulation timesteps before modifying physics or motion semantics. |
| Legends of Ooo | Crosses URL path appending. | Missing `+[NSPersistentStoreCoordinator alloc]`. Core Data is a large subsystem, not a safe small stub. |
| MX Mayhem | Crosses recursive subpath enumeration. | Releases a deallocated/corrupted object at `0x30234af0`; use watched-address ownership provenance rather than suppressing the release. |
| ParaPanic HD | Crosses SHA-224. | Missing `_CC_SHA512`; real SHA-512 is a narrow, independently testable next contract. |
| Perfect Cell | Crosses signed scanning and `scanLongLong:`. | Missing `_fabsf`; add the real math symbol rather than an app-specific workaround. |
| The Sims 3 | Crosses `CGContextBeginPath`. | Missing `_CGContextFillPath`; implement actual path filling through the existing graphics machinery. |
| Trainyard | Crosses the mutable-copy ABI correction. | Missing `_glPointSizePointerOES`; implement only if the ES 1.1 extension contract and client-array lifetime can be modeled correctly. |
| Trism | Crosses cancellation of previous performs, but freezes while starting. All visible input and Home become unresponsive. | No final panic establishes a semantic fix. Capture a bounded all-thread/run-loop snapshot at the stall; do not add another speculative API patch. |

## Diagnostic-title results

| App | Verified result | What the experiment established / next action |
|---|---|---|
| Clowning Around | Panics when Objective-C message handling unwraps an absent host object. | The endpoint is stricter, but the producing callback/object generation is not yet proved. Extend only the watched receiver's registration/deallocation history. |
| Current | Menu remains visually active but universally unresponsive. The EAGL view receives touches and presentation continues at about 58.7 Hz for 38.5 seconds. | Reject host-to-UIKit input loss and renderer freeze. Next trace the guest hit-test/coordinate conversion and target/action callback selected for one Play tap. |
| Fieldrunners | Still consumes a null value as a mutable NSString. | The final string operation remains downstream. Use the captured nil-return history to select and watch the exact producer; never make nil a valid string. |
| Fieldrunners 2 | Remains in startup without entering a usable application loop; Home cannot be serviced and iDared must be closed. | Confirms a guest initialization wait/loop, not a broken picker Home button. Symbolize recurring thread PCs and capture all thread wait states once. |
| JellyCar 1.5.4 | Releases a deallocated/corrupted object at `0x30020bd0`. | The previous autorelease/reuse theory remains plausible but unconfirmed. Watch allocation, autorelease-pool enqueue/drain, reuse and final release for this address. |
| NinJump | Still reaches a strict null-page read. | The diagnostic did not yet expose a safe semantic correction; symbolize the immediate guest caller and its nil-return producer. |
| Peggle | Reaches OpenGL error `0x0506` while the framebuffer reports complete (`0x8cd5`). | This is an exact graphics boundary: capture the first call that sets `0x0506`, not merely the later compositor failure. |
| Scribblenauts | Still reaches a strict null-page read. | Preserve strict memory checks; focus the nil producer/caller trace at the final consumer. |
| SUPERFALL | Still consumes an invalid/deallocated NSString at `0x3646e6f0`. | Watch the resource-name object's creation, retain/autorelease ownership, pool drain, reuse and death. |
| Super Hexagon | Still reaches a pre-frame null access. | Static caller symbolization remains necessary; no safe runtime semantic change is established. |
| Tiki Totems 2 | Retains a deallocated/corrupted object at `0x3044b300`. | Track allocation/class/ownership/reuse for exactly this watched address; do not globally tolerate invalid objects. |
| Wire Way | Gameplay works until level completion, then sound stops and the screen fades to an unresponsive black state. Rendering continues, the framebuffer remains complete and the final history contains a full-screen black triangle-strip draw using `glColor4f(0,0,0,1)`. | The guest is actively drawing black; this is not a compositor failure. Symbolize the level-complete scene/audio transition and the callback that should advance out of the black overlay. |
| Spoing | Healthy control: 1,038 main loops, 444 presentations and clean Home return in 29.4 seconds. | No relevant regression observed. |

## User-observed compatibility classification

- **Carnival:** partial. It runs, but fluctuating load-related slowdown prevents
  enjoyable play.
- **Current:** blocked by nonfunctional menu interaction despite delivered
  touches and normal presentation cadence.
- **Fieldrunners 2:** blocked by an initialization/event-loop stall; even Home
  is not serviced.
- **Labyrinth 2:** partial. Gameplay exists, but collision/motion corruption and
  menu defects are serious.
- **Trism:** blocked by a start transition stall and unserviced input/Home.
- **Wire Way:** partial. First-level gameplay works; its completion transition
  stops audio and remains black/unresponsive.

## Highest-value follow-up

1. Combine the small exact next contracts that have independent, testable
   semantics: `NSBlockOperation blockOperationWithBlock:`,
   `NSSet setWithObjects:count:`, `_CFDictionaryContainsKey`, real SHA-512,
   `_fabsf`, and `CGContextFillPath`. Treat `glPointSizePointerOES` separately
   unless its pointer lifetime can be verified.
2. Symbolize Wire Way's level-complete transition and Current's Play-button
   hit-test/target-action route. Both now have sharply bounded questions.
3. Add one bounded all-thread/run-loop snapshot for the Fieldrunners 2 and
   Trism stalls rather than more API logging.
4. Add real AIFC decoding for Carnival as an independently correct audio
   capability, while separately profiling the presentation-rate collapse so
   the audio gap is not incorrectly blamed for all slowdown.
5. Correlate Labyrinth 2's accelerometer sample stream with simulation
   timesteps before changing motion or timer behavior.

No source implementation, commit, push or new IPA is authorized or produced by
this evidence review.
