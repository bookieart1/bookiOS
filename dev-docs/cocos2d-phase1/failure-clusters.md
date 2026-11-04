# Cocos2d candidate corpus: causal map

Audit date: 2026-11-04. Baseline: Dev 2.23, `51a633428ec2b461761434036b6968d729c15594`, IPA workflow `41778866142`. This is investigation, not an implemented compatibility pass. [Manifest](manifest.md), [structured manifest](manifest.json), and [per-game cases](cases.md) supply exact package identities, original log lines, source paths, evidence, and next actions.

## What the complete corpus establishes

All 37 games have matching local IPAs and launch records. There are three original log files, not 37 separately attached files; their 37 launches are indexed individually. The fresh Clowning Around log fills that title's evidence gap. The user corrected Zombie Shock to **ZombieSmash** and supplied `ZombieSmash20.ipa`; it is no longer log-only.

All 37 ZIP archives pass CRC verification. **League of Evil and StickWars have encrypted executable slices and fail before guest CPU execution.** Their missing recorder is expected. All other 35 launches enable the universal recorder. ZIP integrity does not prove every resource is present or decodable, and matching bundle/version does not prove the on-phone package has the same hash as the local IPA: device logs do not contain a package hash.

Three games are user-confirmed working controls: **Dirt, Moonlights, Tiki Totems Premium**. Five have explicitly partial results: Hook Champ, iStunt Reloaded Pro, Love Catcher, Pro Zombie Soccer, Tap the Frog. The remaining 27 eligible launches have a captured blocking runtime failure. This is not a compatibility percentage for all cocos2d games: this collection was selected largely from unresolved titles.

## Engine attribution

- **32:** strong cocos scene-engine class-family evidence in executable metadata. This supports engine membership, not a precise upstream version or an unchanged engine fork.
- **Trenches / TrenchesVS:** CocosDenshion/CocosThunder audio integration is identifiable; a full cocos scene engine is unconfirmed.
- **Hook Champ and Love Catcher:** engine membership remains unconfirmed. Generic `Texture2D` or `director` names are insufficient.
- **League of Evil and StickWars:** encryption prevents reliable negative conclusions from missing executable markers.

Embedded source paths identify FastBall 2's 0.99.5, Kiko's 0.99.5-rc1, Mini Militia's 0.99.5-beta3, and Tap the Frog's 1.0.0-rc lineage. Other versions remain unknown. Older unprefixed `CocosNode`/`Director` and later `CCNode`/`CCDirector` families are recorded separately. Static marker hits are not proof that an SDK executes on the failing path; the curated manifest filters broad-string false positives.

The chosen ARM slices show fixed-function GL usage and no selected shader-function imports; the eligible logs select ES 1.1. This does not rule out dynamic/unused ES2 paths. PVR, compressed texture, font, atlas, map and audio filenames are indexed. None of those extensions alone establishes a failing decoder or corrupt asset. Box2D/Chipmunk presence likewise does not establish a physics defect.

## Causal clusters and separate workstreams

Counts below partition all 37 titles. A workstream containing several signatures is an implementation-planning grouping, **not** a claim that one bug explains all its games.

| Signature(s), count | Games | Earliest actionable evidence | Interpretation / next discriminator |
|---|---|---|---|
| ALPHA-01, 1 | AbstractWar 2.0 | Null color space passed with 128×128, 8-bit alpha-only bitmap; caller maps to CCTexture2D image conversion | Confirmed legal input mishandled by Core Graphics. Implement the complete one-channel path, not a null-pointer exception. |
| STREAM-01, 2 | Addicus; Kiko | GANTCPSocket/Google Analytics, NULL socket-stream pair, then absent read-stream scheduling | Strong shared third-party transport boundary. Model creation/open/error/callback lifecycle; a scheduling no-op is insufficient. |
| IAD-01, 2 | Flow Free; Mini Militia | `ADBannerView alloc` fails | Confirmed shared class gap. Historical unavailable-ad behavior needs proper view/delegate state, not fabricated ad success. |
| Foundation value workstream, 7 | Continuity 2; Infinity Field; Super Stick Golf; Stick Stunt Biker; Doodle Grub; Junk Jack Retro; Lumi | Respectively KVC object unboxing, unsigned-byte boxing, Boolean archive object, negative date conversion, UIColor coding, dictionary block enumeration, aggregate method encoding | Seven distinct signatures. First four are the best bounded starting batch; last three require archive-format or guest ABI work. |
| UIKit contracts, 3 | Cannon Cadets; Clowning Around; iStunt Reloaded Pro | Animation transaction flag; programmatic alert dismissal; tableView returning plain UIView after failed nib fallback | Separate contracts. iStunt's producer is wrong: adding `setRowHeight:` to UIView would conceal the error. |
| DLOPEN-01, 1 | Contre Jour HD | Assertion on unknown dynamic-library request | Recover requested path before deciding optional-library failure versus actual loading support. |
| RUNLOOP-01, 1 | Trainyard | `CFRunLoopSourceCreate` absent after worker connection setup | Decode source context/version. Source lifecycle is not a timer stub or a successful network connection. |
| SHA1-01, 1 | ZombieSmash | `CC_SHA1_Init` absent; imported by matching unencrypted binary | Real incremental digest implementation with guest context-layout tests. No evidence yet of a cocos-specific cause. |
| ORIENT-01, 1 | FastBall 2 | SDL orientation/display-mode size mismatch | Host resize/order investigation; not yet the same cause as the clipped games. |
| VISUAL-02/03, 2 | Pro Zombie Soccer; Tap the Frog | Continuing presentations, wrong visible scale/orientation and no usable interaction | Possible draw/touch-coordinate or overlay-state relationship. Compare transforms and hit-test delivery before sharing a correction. |
| INPUT-01, 1 | Hook Champ | Name-entry page remains stuck while presentations continue | Capture one field tap, first-responder request and Back tap; distinguish undelivered touch from guest UI state. |
| VISUAL-01, 1 | Love Catcher | White picture with user-observed audio; ~1,005 presentations then repeated guest sleep path | Geometry/scene state or a guest wait remain candidates. Not a proven stopped renderer. |
| FBO-01, 2 | Super Turbo Action Pig; Wispin | Earlier GL 0x506 noticed at compositor cleanup, queried current framebuffer complete | Shared detector, not yet a shared cause. Reconstruct earlier binding/attachment/draw history; first-error capture only if existing history cannot decide. |
| MUTABILITY-01, 1 | Legendary Wars | Immutable property-list dictionary receives KVC `height` assignment | Resolve plist creation options and native historical behavior; no global mutable-container accommodation. |
| ARRAY-01, 1 | Trenches / TrenchesVS | Removal callbacks shrink count 5→4→3→2 during fixed-count selector iteration, then index 3 fails | Immediate chain confirmed. Native mutation contract still needs a reference test before selecting snapshot/live semantics. |
| PROFILE-01, 1 | Robokill | Empty profile array consumed at index 0 | Trace profile creation/defaults producer. Distinct from Trenches; never invent a profile. |
| PLUS-01, 1 | Blockoban Premium | Class NGDataModel receives `domain` after offline error handling | Track NSError out-slot, branch and receiver origin. Not a missing `domain` method on the game class. |
| NULL-01, 1 | Mega Jump | Null read precedes CPU MemoryError; upstream producer absent | Symbolize exact fault and trace source register/call return. Recent resource names do not prove resource failure. |
| LIFETIME-01/02, 2 | Silent Swords ZERO; Thief Lupin | Invalid hash receiver versus missing mutable-string host object | Two unresolved producers, not a proven shared release bug. Reuse address-generation history; targeted watches only after choosing a producer. |
| PACKAGE-01, 2 | League of Evil; StickWars | Encrypted executable rejected before CPU start | External package evidence required. No emulator patch, decryption or acquisition attempted. |
| CONTROL-01, 3 | Dirt; Moonlights; Tiki Totems Premium | User-observed gameplay, ongoing presentation and manual Home | Working only in stated tested scope; choose controls by affected subsystem. |

## What is new versus already known

- Addicus's analytics path was already documented at Dev 1.68–1.70 and in the standstill review. **Kiko provides a new concrete second member**, and the current failure is read-stream scheduling, after the earlier write-stream surface. The known NULL-creation accommodation has not solved the lifecycle. Cordy/Current in the Dev 2.23 review provide outside-corpus validation candidates; ORBITAL is a historical candidate, not a fresh result here.
- Continuity's current failure is **object-valued KVC dispatch**, beyond the method-token parser work in commit `fbc5652e`. It is not evidence that the new tokenizer itself should be reverted. The setter still selects unboxing from the incoming NSValue class instead of the required target type.
- Stick Stunt Biker's negative-duration panic is now connected to **1970 date creation → news-feed property → NSLog → NSDate description → signed Apple-epoch conversion**. Calling it a generic timing or scheduling bug would target the wrong subsystem.
- AbstractWar's apparent null-object failure is a **valid alpha-only bitmap request** with a symbolized cocos texture caller. It does not belong in an arbitrary invalid-object bucket.
- Clowning Around's fresh log reconfirms the older alert-dismissal blocker; it is not a newly introduced regression.
- Legendary Wars and Trenches still hit previously documented strict boundaries. This pass strengthens current identity and event sequences, but does not establish native mutation/exception behavior or authorize weakening it.
- Tap the Frog's recovery panic happens **after** the visible/input defect, during Home. Fixing that lifetime fault alone would not explain or correct the zoomed image.

## Remaining gaps

No native reference execution, new guest tests, or local guest execution was performed. Aggregate returns do not measure frame pacing, correct pixels, input usability, audio quality, save integrity, or long sessions. No new native `.ips` report is present or required by a demonstrated whole-host termination in these particular 37 launches. If a future run closes native iDared without a captured panic, obtain its matching report rather than infer the cause from a truncated log.

There is no evidence here for a corpus-wide OpenAL, Box2D, Chipmunk or ARM-instruction defect. Those remain possible in individual later paths, not prioritized findings. Unknown origins are explicitly retained in each case; absence of a recorded allocation event is not proof of absence of deallocation or corruption.
