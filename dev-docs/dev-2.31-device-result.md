# Dev 2.31 physical-device result

Date: 2026-11-05

The three supplied logs identify implementation
`62aefdd42441ead701cf79e72f8589fc965ac8b3` and unsigned IPA workflow
`41968860889`. This is verified device evidence from the final Dev 2.31
candidate, not an inference from the picker label or IPA filename.

## Evidence

| File | Bytes | SHA-256 | Scope |
|---|---:|---|---|
| `BF32821B-F3FF-4173-9984-91DFA7B49067/1-Log.txt` | 1,768,889 | `bcc3d06f886b065fedc4d2dfdba02752e3d6cbe2bba730239285df7a6059bc2d` | First portion of the Dev 2.31 physical-device pass |
| `BF32821B-F3FF-4173-9984-91DFA7B49067/2-Log.txt` | 1,980,823 | `08ad452d4ec122bf8ce80c5814f4c5e618b635da99629671419b1043146c8126` | Continuation of the compatibility and diagnostic pass |
| `BF32821B-F3FF-4173-9984-91DFA7B49067/3-Log.txt` | 461,342 | `e7cd4384935c2a30b36f67186b961ee34ec9fe07f0f3a5288f16801f821de6c3` | Final launches, including repeated Wire Way behavior |

## Executive result

Dev 2.31 produced several real boundary crossings but did not make the major
remaining problem titles fully compatible:

- **Bike Race Pro, Labyrinth 2, Legends of Ooo, ParaPanic HD, The Sims 3 and
  Trism** cross the exact contract implemented for this pass and expose a later
  failure or impairment.
- **Carnival's IMA4 correction did not reach the decoder for the real package.**
  The real AIFC `COMM` chunks report a sample-size field of zero; the new
  decoder currently accepts only 16. The continuing `ima4` failures therefore
  falsify the test fixture's assumption without establishing that zero is legal
  historical behavior. Audio and intermittent slowdown remain separate issues.
- **Fieldrunners 2** is visually unchanged, but its diagnostic advanced: the
  recurring blocked guest PCs now include instruction bytes. Those addresses
  can be symbolized before any scheduler change is considered.
- **Current, Dismount, Kick the Buddy, MX Mayhem, 100 Chambers and Wire Way**
  remain in their established failure families. Their logs do not support a
  new broad semantic correction yet.
- **Spoing** remains the healthy logged control in the supplied scope.

No result justifies weakening memory safety, accepting arbitrary bitmap
layouts, suppressing exceptions, fabricating internet connectivity or treating
a splash/menu reach as functional compatibility.

## Per-title reconciliation

| App | Verified Dev 2.31 result | Interpretation and next discriminating action |
|---|---|---|
| 100 Chambers | Still ends when `_touchHLE_NSMutableSet` receives KVC key `name`; the trace retains value `Terrain`. | No visual or causal advance beyond the concrete KVC boundary. Identify the guest caller/producer before adding any property behavior to sets. |
| Bike Race Pro | Crosses `NSDictionary writeToURL:atomically:` and next raises the app's `Problem with internet connection` exception. | The Foundation serialization contract advanced. Do not suppress the app exception or fabricate connectivity; determine the historical offline path separately. |
| Cannon Cadets | The user sees black with no gameplay. The same strict bitmap-context assertion rejects the package's unsupported pixel layout before a main loop or EAGL presentation. | No advance. Inspect the exact layout/color-space request and implement only a historically supported format. |
| Carnival | Runs for 64.5 seconds with 2,121 presentations (32.9 Hz average), but the user hears no music and no button sounds, only rare clean effects, with brief intense slowdowns. Real files still report `AIFC compression=ima4 ... bits=0` and are rejected before decoding. | The intended audio correction was not exercised by the package. Research historical IMA4 `COMM` sample-size semantics before accepting zero. Profile slow frames separately; the log does not prove audio causes the slowdown. |
| Current | Continues presenting at about 58.5 Hz and receiving taps at its game view, but no control reacts visually or audibly. | Host input delivery is not the blocker. Symbolize the first guest/cocos2d callback and its coordinates rather than changing generic UIKit hit testing. |
| Dismount | Gameplay and all expected controls work, but the run averages about 15.6 presentations per second and is extremely slow. | The former compatibility contract remains crossed. The blocker is performance; use bounded per-subsystem frame timing or hot guest-PC sampling. |
| Fieldrunners 2 | Pure silent black; Home cannot be serviced. Six guest threads remain blocked before the main loop or first presentation. Dev 2.31 now preserves instruction bytes at recurring PCs including `0x002a8c10`, `0x002a8478`, `0x002a8ce8` and `0x002a8e68`. | Visual behavior is unchanged, but the diagnostic progressed. Symbolize these exact PCs/instructions and identify their wait primitives before altering scheduling. |
| Kick the Buddy | Still presents a black scene at about 54.6 Hz. Blind taps occasionally trigger interactive sounds. | Input and game activity continue beneath a content/rendering defect. Capture bounded draw/resource state at the first black frame; do not treat it as a freeze. |
| Labyrinth 2 | Calls the new `hidesWhenStopped` contract and runs, then later terminates after OpenGL error `0x506` despite a complete framebuffer. The intermittent ball/wall leak was not reliably resolved by this contract. | The UIKit boundary crossed. Identify the first producer of GL `0x506`; keep the intermittent physics/motion defect as a separate investigation. |
| Legends of Ooo | Real `NSManagedObjectModel` allocation succeeds; the next immediate blocker is missing `initWithContentsOfURL:`. | The class-identity partial contract advanced exactly as predicted. Implement real model initialization/parsing only when its historical behavior is established. |
| MX Mayhem | Still attempts to release an object whose `isa` is nil, now at another runtime address. | Same invalid-object/lifetime family. The changing address further rejects a pointer-specific workaround; interpret allocation/ownership history. |
| ParaPanic HD | `processIdentifier` succeeds and execution reaches a later null-page access. | The Foundation contract crossed. Symbolize the new null producer; do not make address zero readable. |
| Spoing | Brief run and Home return complete without a terminal panic. | Healthy control remains intact in the logged scope; this is not a full long-play compatibility certification. |
| The Sims 3 | Proceeds much farther after Clear blending, then panics in `ns_string.rs` on an out-of-range string index (`len 6`, index `9`). | Graphics correction advanced the title. Inspect the guest caller and historical string/range contract before changing bounds behavior. |
| Trism | Crosses `CGContextClosePath` and reaches gameplay, but later freezes; Home is not serviced. A recurring blocked guest PC remains available for symbolization. | The graphics contract advanced, but the later guest stall is distinct. Symbolize the recurring PC/wait before changing scheduling. |
| Wire Way | Two runs remain active around 29 Hz, but pause or level completion turns the output permanently black and silent. | Same guest-driven transition failure. A future capture must freeze on the fade/audio-stop transition rather than enlarge a generic rolling history. |

## Compatibility classification after Dev 2.31

- **Playable but materially impaired:** Dismount (severe slowdown), Carnival
  (missing most audio and episodic slowdown), Labyrinth 2 (intermittent physics
  defect and later GL error), Wire Way (gameplay until pause/level completion).
- **Exact contract crossed; later blocker exposed:** Bike Race Pro, Legends of
  Ooo, ParaPanic HD, The Sims 3, Trism.
- **Active but visually blocked:** Kick the Buddy.
- **Input blocked after delivery to the game view:** Current.
- **Guest initialization wait/deadlock:** Fieldrunners 2.
- **Strict graphics-format boundary:** Cannon Cadets.
- **Invalid-object/lifetime origin unresolved:** MX Mayhem.
- **KVC target/producer origin unresolved:** 100 Chambers.
- **Implemented audio hypothesis disproved by real package metadata:** Carnival
  IMA4 sample-size assumption.
- **Healthy logged control:** Spoing.

## Preserved next actions

This result authorizes documentation only. It does not begin another source
change, push, workflow or IPA build. The highest-information next actions are:

1. Verify historical AIFC/Apple IMA4 `COMM` sample-size-zero behavior using a
   primary/reference source, then update the decoder only if that representation
   is valid.
2. Symbolize Fieldrunners 2's recorded blocked PCs and Trism's recurring PC
   against their exact executables before changing scheduling.
3. Capture Kick the Buddy's first-black-frame graphics/resource state and Wire
   Way's exact fade/audio-stop transition rather than increasing generic rings.
4. Profile Dismount and Carnival with bounded subsystem/frame-time evidence;
   their input paths are not the performance blocker.
5. Trace The Sims 3's string caller, ParaPanic HD's later null producer and
   Legends of Ooo's actual model-loading needs before implementing their next
   contracts.
