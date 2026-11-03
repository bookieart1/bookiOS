# Dev 2.15 contracts, UIKit archives, and post-input graphics evidence

## Evidence and boundaries

Dev 2.15 follows verified Dev 2.14 commit `5d2559a9` and the two supplied
physical-device logs, preserved outside the checkout as:

- `outputs/device-logs/iDared-Dev-2.14-device-results-part1-2026-11-03.txt`
  (SHA-256
  `B9E311FE1812F558F9BF874FAFF65D7EB9AAC5B29D79F1DC1045DBEE2E524E45`)
- `outputs/device-logs/iDared-Dev-2.14-device-results-part2-2026-11-03.txt`
  (SHA-256
  `ACD9FB8F0FC34CAF2E40994F8CEAE70EDED3B09821E3E1F5596FD3B73E4A047F`)

The supplied Flip's Escape screen recording is preserved as
`outputs/device-logs/iDared-Dev-2.14-Flips-Escape-visual-corruption-2026-11-03.mp4`
(SHA-256
`055C2569FF72A299FB99031724973F11CB1BB5DA5246D22A5440A0D31D655AA4`).
It was inspected at eight frames per second. The menu initially renders
correctly. After the gameplay transition, presentation continues smoothly but
the visible scene becomes repeated sprite-atlas rows and misplaced tiles while
some game objects continue moving. That falsifies a freeze or blank-frame
theory and points to texture-coordinate/client-array/binding state.

The permanent universal recorder remains enabled. No app-name behavior,
invalid-memory tolerance, swallowed exception, fabricated service response, or
resource invention is added.

## Per-title path

| Title | Dev 2.14 evidence | Dev 2.15 action or bounded next discriminator |
|---|---|---|
| ADVENA / ZENONIA2 | Both progress through timer membership, then stop at missing `_CFRunLoopRemoveTimer`. | Export removal through the existing run-loop timer list. Default/common modes remove real membership; absent or unsupported membership is a no-op. |
| B'Blast | Null reaches `strlen` after numerous failed bundle-resource candidates, including malformed-looking guest-generated names. | No resource or path is fabricated. Existing failed-path/candidate-directory evidence is retained for a legitimate-package comparison; this is presently safer than changing filesystem semantics. |
| Bungee Ball | Play reaches a solid yellow frame; a subsequent touch belongs to no view and the move/end path dispatches to nil. | Do not dispatch move/end responder or gesture messages for touches that hit no view. Continue tracking them until end as UIKit does. |
| Crash Drive / Jack Lumber | Working in the supplied scope. | Focused controls for touch, navigation, GL presentation, and clean Home behavior; no compatibility mutation. |
| Decoloro | Pink screen, no EAGL presents; archive keys identify `UINavigationBar`, `UIViewControllers`, and `UIChildViewControllers`. | Reconstruct a nib-created navigation controller's retained bar and controller stack from those historical archive keys. The bar still has no claimed rendering/animation implementation. |
| EPOCH / Unmechanical | Both stop at `NSSearchPathDirectory` value 100. | Implement `NSAllApplicationsDirectory` using iDared's single modeled user applications root. |
| Flip's Escape / Slingshot Racing | Active presentation continues during severe atlas/magenta/block corruption. | Record at most 24 detailed frames per app run, in eight-frame windows immediately after input: pointers, client arrays, buffer bindings, matrices, texture unit/environment, blend, viewport, scissor, and framebuffer status. This is diagnostic-only and makes no GL state change. |
| JellyCar 3 / Wind-up Knight | Dynamic lookup of absent `iSimulate` / `GADRequest` classes panics even though `NSClassFromString` is defined to return nil. | Correct `NSClassFromString` to lazily return a known class and return nil for an unknown class. No fake class is introduced. |
| Junny | A decoded title-only `UIBarButtonItem` panics because the host invents and then looks up an unregistered `undefinedSelector`. | A bar item with no target/action now remains actionless. The fallback accessor registers a stable selector only if queried. |
| LOCV | User-observed menu/game path now works in the tested scope. | Add to the picker Playable view; no compatibility mutation. |
| Peggle | Host closes at the keyboard path; the strict compositor later reports `GL_INVALID_FRAMEBUFFER_OPERATION` without a producing layer. | Preserve the strict panic and, for the bounded post-input window only, check each compositor layer so the first producing layer/stage is named. |
| Stick Skater | Gameplay works; manual Home is followed by a teardown memory error. | No cleanup error is suppressed. Existing Home/thread/object histories remain the path; the focused retest determines whether the unrelated UIKit changes affect teardown. |
| SUPERFALL | Death still consumes an invalid string after a very large mutation/lifetime history. | Preserve strict failure and the expanded bounded lifetime provenance. No global retention or string coercion is justified by the current evidence. |
| Tiki Totems 2 | Level selection advances farther, then a stale category object is retained. | Preserve the watched array-transfer lifetime path; no stale object is tolerated or retained artificially. |
| Topple 2 | A dead online catalog ETag is nil and later inserted into a dictionary. | No patch: native dictionaries reject nil and a network ETag cannot be invented. The concrete next comparison is an offline native-device/reference path, not permissive collection behavior. |
| Spoing | Working control. | No compatibility mutation; validates broad input/render/audio/save behavior. |

## Implementation classification

- **General semantic contracts:** `CFRunLoopRemoveTimer`,
  `NSAllApplicationsDirectory`, and `NSClassFromString` nil-on-unknown.
- **UIKit semantic corrections:** actionless decoded bar items, no delivery of
  unowned touches, and historical navigation-controller archive state.
- **Diagnostic-only:** post-input EAGL snapshots and post-input compositor
  layer error attribution. Both have fixed 24-frame-per-run budgets.
- **Evidence-only/no mutation:** B'Blast, SUPERFALL, Tiki Totems 2, Topple 2,
  and Stick Skater. Each already has a specific next discriminator; unsafe
  success fabrication or lifetime masking is deliberately excluded.

## Predictions

- ADVENA and ZENONIA2 should pass their current run-loop removal boundary.
- EPOCH and Unmechanical should pass directory 100.
- JellyCar 3 and Wind-up Knight should observe nil for absent optional classes
  instead of terminating in the runtime lookup helper.
- Junny should pass title-only bar-item decoding.
- Bungee Ball and Decoloro should no longer crash merely because a touch has no
  target; Decoloro may additionally expose its archived root controller.
- Flip's Escape and Slingshot Racing may look unchanged, but the log should now
  show the state transition immediately after Play/input.
- Peggle may still terminate, but the first invalid compositor layer/stage
  should replace the undifferentiated final GL assertion.

Passing one boundary is not full compatibility. Physical-device results must
separately establish menus, gameplay, transitions, audio, save/relaunch, and
clean exit.

