# Dev 2.14 contracts and bounded causal origins

## Evidence and scope

Dev 2.14 follows verified Dev 2.13 source `aed3c769` and the two untouched
physical-device logs archived outside the checkout as:

- `outputs/device-logs/iDared-Dev-2.13-device-results-part1-2026-11-03.txt`
  (SHA-256
  `20A8A128D7E64D97EB8045C10A94C48F3224DE8CAE982249A1E7E1B6A9955B95`)
- `outputs/device-logs/iDared-Dev-2.13-device-results-part2-2026-11-03.txt`
  (SHA-256
  `A28FB6BCFD3374E502183BC6DDFC4E967AFDE2982F67473F23A9AFE9C5294EEF`)

Both logs embed `aed3c76` and hosted IPA workflow `41600662902`. The change
set attempts a safe path for every title in the focused list. It does not add
app-name branches, weaken invalid-object or memory checks, fabricate service
success, or suppress crashes. The universal recorder remains enabled.

## Per-title engineering decisions

| Title | Dev 2.13 decisive evidence | Dev 2.14 path and prediction |
|---|---|---|
| ADVENA | Missing `_CFRunLoopContainsTimer` before a usable launch. | Implement the Core Foundation query through the existing run-loop timer membership. Default/common modes report real membership; unsupported modes report false. |
| B'Blast | Reaches presentation, then `CAEAGLLayer` lacks `contentsScale`. | Add retained `CALayer.contentsScale` state, defaulting to 1.0. |
| Bungee Ball | Its historical private UIKit path sends `setAlpha:` to a real `UIColor`. | Add the bounded historical mutating color-alpha selector using the existing CGColor state. This is **provisional private API compatibility**, not new public UIKit behavior. |
| Crash Drive | Long gameplay/presentation and a clean Home return. | No compatibility behavior is changed. It is a working control for shared memory, UIKit, graphics, and picker behavior. |
| Decoloro | A NIB-created `UINavigationController` decodes without `UIView`; the app then runs 1,445 loops with zero EAGL presentations on a pink UIKit screen. | No guessed controller key is decoded. A one-time, 32-key archive summary records only key names/value kinds when this exact navigation archive boundary occurs. This should identify the historical child/view key without mutating decoding. |
| EPOCH | Missing `-[NSUserDefaults persistentDomainForName:]`. | Return an immutable snapshot for modeled application/global persistent domains and nil for an unknown or absent domain. |
| Flip's Escape | Hundreds of complete frames, but severe texture/sprite/audio corruption; prior framebuffer-only snapshots did not distinguish the producer. | Fixed presentation milestones now include blend state, active/client texture, texture environment, and vertex/texture-coordinate array descriptors. No per-frame tracing or GL behavior change. |
| Jack Lumber | Gameplay works and exits through Home. | No compatibility behavior is changed. It is a second working UIKit/graphics control. |
| JellyCar 3 | Missing class method `+[NSBundle pathsForResourcesOfType:inDirectory:]`. | Implement directory enumeration and extension filtering using the existing guest filesystem path machinery. |
| Junny | A NIB-created `UIBarButtonItem` lacks `initWithCoder:`. | Decode historical title, style, custom view, and width into the existing retained bar-item state. Target/action outlet connections remain owned by the NIB connection mechanism. |
| LOCV | `CGImageCreate` rejects 32-bit opaque RGB with `kCGImageAlphaNone`. | Accept that valid layout and normalize the unused final byte to opaque for iDared's internal RGBA image. Other unsupported bitmap layouts remain strict. |
| Peggle | Whole iDared termination is preceded by a strict Objective-C allocation collision at an address previously used as CGColor component storage. | Move guest-visible color components to a private framework heap so they cannot alias ordinary malloc allocations. Strict object collision checks remain. |
| Slingshot Racing | Missing `kill(1, 0)` existence probe. | Implement signal zero for the one synthetic PID; nonzero signals remain strict. |
| Stick Skater | Gameplay works but emits an error after manual Home. | No blanket cleanup suppression is added. The existing Home dump remains the discriminating teardown evidence. It is classified Playable with this limitation. |
| SUPERFALL | Long gameplay then an invalid resource-name string; the relevant lifetime record was evicted. | Expand the fixed four-way object-history table from 2,048 to 8,192 address slots. Lookup cost and eight-event depth remain fixed; memory is bounded. |
| Tiki Totems 2 | An invalid value from the wall/category path reaches dictionary insertion; its earlier lifetime was absent. | Recent array insert/add/replace values join the existing 16-address watched-lifetime set. This neither retains the value nor changes collection ownership. |
| Topple 2 | Missing games-catalog ETag produces nil, followed by invalid dictionary insertion. | **No behavior patch.** Native mutable dictionaries also reject nil, and inventing a network ETag would fabricate a dead service. Existing provenance already names the cause. |
| Unmechanical | Missing `persistentDomainForName:` at startup. | Same real persistent-domain snapshot contract as EPOCH. |
| Wind-up Knight | Missing Google Ads `GADRequest` class. | Route the `GAD` prefix through the existing unavailable-ad-SDK nil substitution. This represents an unavailable dead ad service; it does not report ad/network success. |
| ZENONIA2 | Same missing `_CFRunLoopContainsTimer` family as ADVENA. | Same real run-loop membership query, providing cross-title validation. |
| Spoing | Clean gameplay and Home return. | No compatibility behavior is changed. It remains the broad working regression control. |

The picker keeps these 21 titles first and all remaining titles alphabetical.
Crash Drive, Jack Lumber, and Stick Skater move to the user-facing Playable
filter based on the supplied device observations. SUPERFALL moves to Needs Work
because death still crashes. Classification is a convenience, not exhaustive
compatibility certification.

## General implementation boundaries

- CGColor component pointers remain stable for the color lifetime, but their
  framework-owned heap can no longer alias the application's ordinary malloc
  zone. No address quarantine or invalid-object tolerance is introduced.
- Run-loop timer membership, persistent defaults, bundle enumeration, layer
  scale, opaque image layout, bar-item decoding, and signal-zero probing are
  real contracts with focused host tests where the repository permits.
- The expanded lifetime history and array-value watch set are diagnostic-only,
  fixed-size, and preserve guest ownership and scheduling.
- The GL and NIB additions record only a handful of decisive milestones. They
  do not continuously trace instructions, messages, frames, or textures.
- Topple 2 receives no fabricated ETag. Stick Skater receives no swallowed
  teardown failure. Decoloro receives no guessed archive semantics.

## Validation and device predictions

Only hosted workflows are used. No local Rust, Cargo, CMake, preflight, or
build helper is run on the user's computer. Hosted compilation/test status,
skipped and allowed-to-fail checks, final commit, workflow identities, and IPA
verification are recorded after the workflows complete.

The focused device procedure preserves saves and stops each failing title at
its first reproducible terminal boundary. A former blocker disappearing means
only that the blocker was removed; gameplay and full compatibility still
require observation.
