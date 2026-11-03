# Dev 2.13 batched contracts, provenance, and picker correction

## Evidence and scope

Dev 2.13 follows the verified Dev 2.12 source `33508cbf` and the seven
physical-device logs archived in `outputs/device-logs/` on November 3, 2026.
It intentionally combines independent, general contracts so one hosted build
can test a broad batch without mixing in app-name branches or permissive memory
behavior. The universal flight recorder remains enabled for every eligible
launch.

The picker classification is a user-facing convenience, not a claim that every
mechanic in a title has been exhaustively verified. The user's observed
playability decisions are authoritative there. Glowfish HD, Splode, and Toy Bot
2 move to Needs Work. AB Buster, Air Penguin, ChuChu Rocket, CW-Gunner,
Daredevil Dave, Diversion, Flick Buddies, Floe, Kosmo Spin, Mini Beasts HD,
Solomon's Boneyard, Simple Physics, and Ski Jumping move to Playable.
Sticky remains Playable, consistent with its multi-level and saved-progress
device history.

## General corrections

| Family | Change | Prediction and boundary |
|---|---|---|
| Crash Drive, Jack Lumber, Slingshot Racing, Wind-up Knight | Export the historical `UIApplicationLaunchOptionsLocalNotificationKey` NSString. | The common unresolved relocation no longer supplies zero to their shared startup path. A later blocker would split the cluster. |
| Stick Skater | Implement the historical `kAudioSessionProperty_AudioInputAvailable` UInt32 query and report false because iDared has no input-capture path. | The property query returns a real supported value; it does not fabricate microphone capture. |
| ADVENA, ZENONIA2 | Export `CFRunLoopTimerIsValid` through the existing bridged `NSTimer` state. | The exact missing Core Foundation boundary is removed while timer validity remains controlled by `invalidate`. |
| B'Blast | Return POSIX errors for invalid socket family/type/protocol instead of asserting. | `socket(0,0,0)` fails normally; no socket or network success is fabricated. |
| LOCV | Treat EOF before base autodetection as a normal failed numeric conversion. | The empty/short conversion no longer trips an internal base assertion. Invalid explicit bases remain strict. |
| Unmechanical, EPOCH | Treat `host_statistics` count as capacity, reject undersized buffers, and report the count written. | Capacity 60 accepts the 15-word VM statistics result. |
| Bungee Ball | Give pushed view controllers a weak, observable `navigationController` relationship. | The exact missing selector returns the real containing controller, not a constant placeholder. |
| Junny, Decoloro, JellyCar 3 | Add a retained, stateful `UINavigationBar` stack and connect it to `UINavigationController`. | Basic item-stack, identity, style and tint calls proceed. Rendering and animations remain unclaimed. |
| Peggle | Make `malloc_zone_free` remove an Objective-C host registration before its guest address becomes reusable, matching ordinary `free`. | The observed zone-free/object-table collision cannot recur through this path. Genuine stale-object use still fails. |

## Bounded diagnostics

- Every non-nil Objective-C boundary result enters a 512-entry, four-way
  compact last-return table. It survives the 96-event recent ring and
  deallocation, allowing SUPERFALL and Tiki Totems 2 terminal values to name a
  likely API producer when retained. It is not a guest-store or allocation
  trace, and absence remains inconclusive.
- EAGL snapshots are recorded only at presentation milestones 1, 2, 3, 8, 16,
  32, 64, and 128. They add framebuffer completeness, viewport/scissor,
  texture binding and matrix mode for Flip's Escape without per-frame logging.
- Topple 2 already identifies the exact missing defaults value and subsequent
  nil dictionary insertion. Native mutable dictionaries also reject nil; no
  unsafe semantic accommodation is added. Its next result tests only whether
  the other general corrections change the path.

## Focused device batch

The picker places these titles first: ADVENA, B'Blast, Bungee Ball, Crash
Drive, Decoloro, EPOCH, Flip's Escape, Jack Lumber, JellyCar 3, Junny, LOCV,
Peggle, Slingshot Racing, Stick Skater, SUPERFALL, Tiki Totems 2, Topple 2,
Unmechanical, Wind-up Knight, ZENONIA2, and Spoing. Spoing is the working
control. Existing saves must remain intact.

## Explicit limits

No network service, microphone, navigation-bar drawing, Objective-C exception
suppression, invalid-memory tolerance, global extra retain, or app-specific
branch is introduced. BouncyMouse's missing pan recognizer is excluded because
a correct gesture lifecycle is a larger independent contract. Topple 2 receives
no behavior patch because the evidence presently identifies guest nil input,
not an emulator semantic correction.
