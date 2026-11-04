# Dev 2.26 physical-device result

## Identity and evidence

All three supplied logs identify implementation
`eeb7775480cb8bd7b76089fdcdf64d9f16db1b3f` from unsigned IPA workflow
`41816961881`. This is therefore the delivered Dev 2.26 build, not an inference
from a filename or picker label.

| File | SHA-256 | Launches |
|---|---|---|
| attachment `45F1DE08-AD1B-4EBD-8595-A79379C70576/1-Log.txt` | `c6fcb556f41d48b0737a49f68d332622f16befd3d2ea9c7c5c193c4b221248d2` | Cannon Cadets, Clowning Around, Cordy, Current, Guerrilla Bob, Kick the Buddy, Labyrinth 2, Mini Militia, Muffin Knight |
| attachment `45F1DE08-AD1B-4EBD-8595-A79379C70576/2-Log.txt` | `563188d21fdef1081022a57cb002ba50991b7c447c00c0a83f606c38dbefd584` | Sonic Dash |
| attachment `45F1DE08-AD1B-4EBD-8595-A79379C70576/3-Log.txt` | `522adb25076680e84a36534f4ad9b48a34850f2516174889ae79bd6f3ca3f6e5` | Trainyard, Spoing, ZombieSmash |

The universal recorder is enabled for every eligible launch. The user's visual
observations and the recorder are complementary: a missing visible improvement
does not mean the former contract was not crossed, and a presentation count is
not proof of correct graphics or gameplay.

## Per-title result

| Title | Dev 2.26 result | Interpretation and next discriminating action |
|---|---|---|
| Cannon Cadets | The Dev 2.25 gradient-object boundary is crossed. Startup now stops at missing `_CGContextDrawLinearGradient`, before a main-loop frame. | **Verified later contract.** Implement historical linear-gradient drawing over the existing immutable gradient stops, clip, transform, color space and blend state. A no-op would be false success. |
| Clowning Around | The progress-view class boundary is crossed; 111 loops and 112 presentations occur. It then panics in `maybe_initialize_class` because an Objective-C receiver is absent from the host-object table while beginning the stage-select animation. | **Origin not yet proved.** Add one bounded diagnostic at the failed class-initialization lookup recording the receiver, caller and recent lifetime generation. Do not weaken the lookup or accept unregistered objects. |
| Cordy | The run lasts 23.2 seconds with 1,560 loops and 740 presentations, then a local-file `NSURLResponse` receives `statusCode` at guest PC `0x008f6320`. | **Standards-correct emulator boundary, incorrect/ambiguous guest assumption.** Do not add HTTP methods to every file response. Symbolize the caller and determine why its file load enters an HTTP-only path. |
| Current | The user confirms the menu is visible but no button reacts. The recorder captured 162 touch events and shows touches reaching the full-screen interactive `EAGLView`; 1,965 frames present before clean Home return. | **Basic host hit testing is disproved as the cause.** Next capture the guest `touchesBegan:`/`touchesEnded:` entry arguments and coordinate conversion for one tap, plus overlapping-touch rejection. Do not globally force multitouch. |
| Guerrilla Bob | User reports working play. The log runs 73.1 seconds, presents 2,621 frames and returns cleanly through Home. The former alpha-0.4 view is hidden. | **Working in this tested scope.** Dev 2.26 did not change hit testing, so do not claim the diagnostic itself fixed the title. Preserve it as a UIKit/graphics control; no immediate retest is needed. |
| Kick the Buddy | Failable image decoding crosses the previous error. After 657 frames it reaches missing `+[NSMutableSet setWithSet:]`. | **Verified small Foundation contract.** Implement class-cluster factory/copy semantics with correct element ownership; test mutable independence from the source set. |
| Labyrinth 2 | Visible-cell lookup crosses the previous boundary. After 71 presentations it reaches missing `+[NSTimer scheduledTimerWithTimeInterval:invocation:repeats:]`. | **Verified general timer contract.** Implement invocation retention, current-run-loop scheduling, repeat/invalidation behavior and teardown; do not call the invocation synchronously. |
| Mini Militia | It still renders in the wrong orientation and reports no Internet. After the user exits, teardown reaches unimplemented `+[GKMatchmaker sharedMatchmaker]`. | **Dropped at the user's request.** It now depends on orientation plus obsolete online/GameKit behavior. Do not keep it in routine batches or fabricate service availability. Preserve the evidence for a future explicit GameKit project. |
| Muffin Knight | Logo then black screen; Home cannot be processed. Offline `connect` to `83.221.146.11:80` fails twice at PC `0x019787a0`; from 10–60 seconds the main thread remains in `0x015b3b50..0x015b3be0`, with loops/presents fixed at 90/4. | **Guest-side offline polling loop confirmed.** Symbolize/disassemble the connect caller and fixed loop from the exact IPA. Do not fabricate network success or add a watchdog escape. |
| Sonic Dash | It runs 52.4 seconds and presents 819 frames, but graphics are severely corrupted. It requests OpenGL ES 2, receives `nil` because only ES1 is implemented, then uses an ES1 path. | **Strong ES2/fallback diagnosis.** This is no longer a `fcntl` problem. Inspect the IPA to determine whether its ES1 fallback is intended and complete. Full ES2/shader support is broad, high-cost work; no small safe visual patch is established. The visible FPS text is likely guest-side, not a recorder overlay, but that detail is not yet proved. |
| Trainyard | The former run-loop source boundary is crossed, but `NSMutableData initWithBytes:length:` now fails strict typed dispatch immediately. It also attempts a writable open of bundled `trainyardStatic.db` and receives access denied. | **The Dev 2.26 mutable initializer has an ABI/type-registration defect.** Correct the host method signature/encoding and test dynamic dispatch on both `NSData` and `NSMutableData`; only then reassess whether the database path is a second blocker. |
| ZombieSmash | XML-parser cancellation crosses the former boundary and 35 frames render. It then calls `CGBitmapContextCreate` with non-null data and `bytesPerRow == 0`, hitting the strict assertion. | **Historical contract requires research before patching.** Establish whether the targeted iPhone OS versions derive the row size when caller storage is supplied. If confirmed, calculate and validate the minimum aligned row size without making undersized buffers accessible. |
| Spoing | User reports normal operation; 113 presentations and clean Home return. | **Relevant regression control passed.** |

## What the pass established

Dev 2.26 did not make most affected titles visibly playable, but it was not an
empty pass. Cannon Cadets, Clowning Around, Kick the Buddy, Labyrinth 2,
Trainyard and ZombieSmash all crossed the exact boundary changed in Dev 2.26.
Current's recorder disproves a basic app-picker/hit-test failure, Muffin Knight's
black screen is now tied to a precise offline guest loop, and Sonic Dash is
separated into an OpenGL ES 2/fallback problem. Guerrilla Bob and Spoing are
the two functional observations in this run.

## Recommended next queue

1. **Direct general contracts:** `CGContextDrawLinearGradient` (Cannon Cadets),
   `NSMutableSet setWithSet:` (Kick the Buddy), invocation-backed scheduled
   timers (Labyrinth 2), and the mutable-data dispatch ABI correction
   (Trainyard).
2. **One historically researched contract:** zero `bytesPerRow` bitmap creation
   for ZombieSmash, only if the old iPhone contract and buffer-size safety are
   established.
3. **Bounded causal work:** failed class-initialization receiver provenance for
   Clowning Around and guest callback/coordinate tracing for Current.
4. **Offline analysis before another phone test:** symbolize Muffin Knight's
   two exact PC ranges and inspect Sonic Dash's requested ES2/fallback path.
5. **Excluded from the next routine batch:** Mini Militia by user request;
   Cordy until its caller is symbolized; Guerrilla Bob because it currently
   works; Spoing only if a changed subsystem needs a control.

No compatibility code, build, commit or workflow was authorized by this result
review.
