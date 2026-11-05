# Dev 2.29 physical-device result

Date: 2026-11-04

Both supplied logs identify implementation
`94b8d061cc8563a8fbe7e07803cb97fb5fa6f924` and unsigned IPA workflow
`41899732869`. The permanent recorder and the bounded Dev 2.29 diagnostics
were active. This is verified device evidence from Dev 2.29, not an inference
from the picker label or IPA filename.

## Evidence

| File | Bytes | SHA-256 | Scope |
|---|---:|---|---|
| `3FE0A9AA-4C94-458B-A127-6CBE815B438E/1-Log.txt` | 1,350,914 | `e541f5ddcab238e728e1abcc1bf55ecac1446049547665d92273772c1debebb2` | 100 Chambers through Fieldrunners 2 |
| `3FE0A9AA-4C94-458B-A127-6CBE815B438E/2-Log.txt` | 3,211,680 | `dc29b18caff08f022d73599885640f328ff4658481e916033ddfa702c52862be` | Kick the Buddy through Wire Way, including second Labyrinth 2, MX Mayhem and Trism launches |

## Executive result

Three titles crossed into meaningful compatibility:

- **Perfect Cell** and **Trainyard** appear working in the user's tested scope.
- **Dismount** reaches and plays gameplay, but averages only about 15.4
  presentations per second and remains uncomfortably slow.

The pass also sharply reclassified several failures. Current is not losing
touches at the host/UIKit boundary. Kick the Buddy is not frozen and is not
missing input; it presents at about 54 Hz and audibly responds beneath an
all-black result. Fieldrunners 2 is a guest-wide wait before presentation.
Carnival's missing audio is specifically IMA4-compressed AIFC, whereas its
slowdown remains independent and unexplained. Trism now reaches gameplay and
then stops at an exact missing Core Graphics function.

No result justifies relaxing memory safety, fabricating framework success or
promoting a menu/splash-screen reach to full compatibility.

## Per-title results

| App | Verified Dev 2.29 result | Current first boundary / interpretation |
|---|---|---|
| 100 Chambers | The bounded KVC trace records `setValue:forKey:` with value `"Terrain"` and key `"name"` on `_touchHLE_NSMutableSet`, followed by `setValue:forUndefinedKey:` and the existing missing-setter panic. | The final exception is now attributable to a concrete KVC operation, but the guest producer/caller that supplied a set as the target still needs symbolization. A global mutable-set `name` property would be incorrect. |
| Bike Race Pro | Crosses block-operation creation and reaches missing `-_touchHLE_NSMutableDictionary writeToURL:atomically:`. | New exact Foundation serialization boundary; the block lifetime correction did its intended job. |
| Cannon Cadets | Reaches the same strict bitmap-context assertion before presenting a frame. | The assertion remains the correct boundary. Use the captured bitmap/color-space parameters to implement only the actual historical layout; do not accept arbitrary layouts. |
| Carnival | Runs for 41.6 seconds with 1,021 presentations (24.6 Hz average) and exits cleanly. The user hears no music and only sparse, otherwise clean effects. The log records 119 AIFC decode failures for compression `ima4`. | Dev 2.29's uncompressed AIFC support is not the codec this game primarily needs. Real IMA4 decoding is the audio gap. Slowdown persists independently; the frozen slow-present capture began without a healthy baseline, so it did not identify a healthy-to-slow transition or prove audio causation. |
| Current | Rendering remains healthy: 1,661 presentations in 28.6 seconds (58.2 Hz). All 152 recorded touch lifecycle events reach the interactive `EAGLView`, but the user observes no response from Play or any other control. | Host input, UIKit hit testing to the EAGL view and presentation are working. The failure is later in guest/cocos2d coordinate conversion, touch claiming, menu selection or callback invocation. |
| Dismount | The game now reaches and plays gameplay without a panic. It produces 1,052 presentations in 68.2 seconds (15.4 Hz average). | The counted-set contract is validated on device. Compatibility remains partial because gameplay is objectively slow; performance, not the former Foundation exception, is now the blocker. |
| Fieldrunners 2 | No frame presents and no main-loop iteration begins. Periodic snapshots show six guest threads all sleeping or blocked; the main thread repeatedly remains at guest PC `0x002a8c10`, wait detail `0x3b`. Repeated Home requests are queued but cannot be serviced. | Strong guest initialization wait/deadlock evidence, not a picker/Home defect or ordinary rendering failure. The recurring PCs and wait objects need symbolization before changing scheduling. |
| Kick the Buddy | The screen remains black, but blind taps produce game sounds. The log records 1,497 presentations in 27.7 seconds (54.1 Hz) and 360 touch lifecycle events. | The Core Foundation key-presence blocker is crossed. This is an active rendering/content-state defect: input, game logic and presentation continue underneath the black output. |
| Labyrinth 2 | First launch reaches 122 presentations and then finds missing `UIActivityIndicatorView.hidesWhenStopped`. A restored/second launch runs for 191 seconds with 3,542 presentations (18.5 Hz) and no panic. The user reproduces a ball/wall collision leak only once. | The physics defect is intermittent and the supplied run does not correlate the exact wall crossing with a decisive motion/timer anomaly. The missing getter is a separate exact UIKit gap, and launch/restored-state path differences must remain distinct. |
| Legends of Ooo | Crosses persistent-store-coordinator allocation and next stops because `NSManagedObjectModel` is absent at `+alloc`. | Exact next Core Data class boundary. No managed-object-model or persistence behavior should be fabricated. |
| MX Mayhem | Two launches still dispatch/release nil-`isa` receivers (`0x30234e40` and `0x30234b30`) after roughly three to four seconds. | Lifetime/invalid-object origin remains the problem. The two different addresses argue against hard-coding one pointer; retained allocation/owner history must be interpreted before a semantic change. |
| ParaPanic HD | Crosses SHA-512 and next stops at missing `NSProcessInfo.processIdentifier`. | Exact small Foundation contract candidate. |
| Perfect Cell | User reaches working gameplay. The run lasts 107.2 seconds with 3,195 presentations (29.8 Hz) and exits cleanly. | `_fabsf` is validated on device and the title is working within the observed scope. Longer gameplay, save/relaunch and later content remain unproven. |
| Spoing | Recorder completes a brief run and clean Home return without a panic. | Relevant healthy control remains intact in the logged scope; the user supplied no new extended gameplay observation in this pass. |
| The Sims 3 | `CGContextFillPath` is reached, then bitmap drawing stops at unsupported blend mode `16` (`kCGBlendModeClear`). | The fill-path contract crossed its former boundary. Clear-blend semantics are the exact next graphics contract. |
| Trainyard | User reaches working gameplay. The run lasts 21.5 seconds with 616 presentations (28.7 Hz) and exits cleanly. | `glPointSizePointerOES` is validated on the physical ES 1.1 path. Working within the observed scope; longer levels and save/relaunch remain unproven. |
| Trism | A later launch reaches gameplay and then stops at exact missing function `_CGContextClosePath`. | The earlier apparent transition stall is not the current first blocker in this run. Implementing real close-path behavior is a bounded Core Graphics candidate. |
| Wire Way | Gameplay still reaches level completion. A short sound fragment loops, then audio stops and the guest fades to permanent black. The process remains active for 64.2 seconds with 1,873 presentations (29.2 Hz); the final visual history again contains an opaque black full-screen triangle strip. | This remains guest-driven black rendering rather than a frozen compositor. The enlarged generic selector ring was dominated by presentation/layout events and did not retain the decisive level-complete scene/audio callback. A future capture must freeze on the fade/audio-stop transition rather than merely enlarge a continuously rolling history. |

## Compatibility classification after Dev 2.29

- **Working in tested scope:** Perfect Cell, Trainyard.
- **Partial/playable but materially impaired:** Dismount (slow), Carnival
  (slow and mostly silent), Labyrinth 2 (intermittent collision defect), Wire
  Way (first level only; completion transition black).
- **Active but visually blocked:** Kick the Buddy.
- **Input blocked after successful delivery to the game view:** Current.
- **Guest initialization wait/deadlock:** Fieldrunners 2.
- **Exact later missing contract:** Bike Race Pro, Legends of Ooo, ParaPanic
  HD, The Sims 3, Trism.
- **Strict graphics-format boundary:** Cannon Cadets.
- **Invalid-object/lifetime origin unresolved:** MX Mayhem.
- **KVC target/producer origin unresolved:** 100 Chambers.
- **Healthy logged control:** Spoing.

## Preserved next actions

This result deliberately does not begin another implementation round. When the
batch is resumed, the evidence supports these bounded next steps:

1. Implement only well-defined historical contracts after source review:
   mutable-dictionary URL writing, `NSProcessInfo.processIdentifier`,
   `CGContextClosePath`, and clear blend mode. Core Data model behavior is a
   larger separate milestone.
2. Symbolize Fieldrunners 2's repeated wait PCs and wait object, and Current's
   guest touch/menu callback path.
3. Add real IMA4 AIFC decoding for Carnival separately from performance
   profiling; do not assume one fixes the other.
4. Trigger/freeze Wire Way evidence at the fade or audio-stop boundary so the
   decisive scene callback cannot be overwritten by later presentation events.
5. Correlate Labyrinth 2 motion/timestep state only when a wall-crossing event
   is actually reproduced.

This checkpoint changes documentation only. It does not authorize or contain a
compatibility implementation, commit, push, workflow run or new IPA.
