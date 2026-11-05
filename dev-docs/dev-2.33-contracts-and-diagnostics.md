# Dev 2.33 contracts and bounded diagnostics

Date: 2026-11-05

Dev 2.33 is one coherent seventeen-title device pass. Five titles exercise
three general compatibility corrections, ten titles receive bounded evidence
capture at their established unresolved subsystem, and two working titles are
unchanged regression controls. No bundle-identifier branches, fabricated
media selection, invalid-pointer tolerance, crash suppression or app-specific
success path is included.

## Physical-test scope

| App | Classification | Dev 2.33 change | Prediction / falsifier |
|---|---|---|---|
| 1112 ep.01 | Direct correction | Implement historical `UILineBreakModeClip` as one unwrapped line whose reported/drawn width is clipped to the constraint. | The former mode-2 exception disappears. The theory is falsified if mode 2 remains the first failure. |
| BouncyMouse | Direct correction | Add the pre-iOS-5 `UIPanGestureRecognizer` target/action lifecycle, translation, velocity and touch-count properties. | Pan allocation and drag callbacks advance. A missing class/selector at the same boundary falsifies the implementation. |
| Bungee Ball | Diagnostic | Preserve bounded table reload, row construction and selection events. | A level-selection failure can be separated into missing data-source rows, missed selection or later presentation. |
| CrazyGolf | Regression control | No compatibility change. It exercises input, ES1 rendering, transitions and Home return after gesture/graphics instrumentation. | Any regression is exposure from the shared UIKit/OpenGL changes, not an intended app fix. |
| Doodle Grub | Diagnostic | Record keyed-unarchive keys, missing keys and returned object pointers/classes in the bounded selector history. | The next log should identify which archive key produces the string later treated as a color. |
| Flip's Escape | Diagnostic | Extend the bounded OpenGL visual-call history to compressed texture and framebuffer-copy texture paths. | A corrupt transition should retain its last texture-upload/copy state; no change in evidence falsifies this capture point. |
| Highway Rider | Diagnostic | Same bounded texture/copy history, alongside the permanent resource and presentation recorder. | The missing road can be correlated with texture creation/copy state rather than guessed from gray output. |
| iStunt Reloaded Pro | Diagnostic | Record table reload begin/end, materialized-row count, selection hit/miss and delegate delivery. | The absent level buttons should resolve to a data-source population boundary or a later guest-side transition. |
| JellyCar 1.5.4 | Diagnostic | Watch duplicate same-pool autorelease enqueues, preserving their later release/deallocation history while retaining the existing bounded enqueue tail. | A repeat double-scheduling failure should retain the object/pool sequence instead of only the terminal invalid receiver. |
| Junk Jack Retro | Direct correction | Add `UITapGestureRecognizer`, including historical defaults, target/action recognition and required-tap/touch properties. | The former class-allocation blocker disappears and taps can invoke the registered target. |
| Kick the Buddy | Diagnostic | Extend the bounded texture/copy history for the audibly interactive black scene. | The log should distinguish absent texture population from a later complete black composition. |
| MX Mayhem | Diagnostic | Add the bounded duplicate-autorelease watcher described above. | A double-scheduled corrupted/deallocated receiver should retain producer/pool history; normal ownership remains strict. |
| R.Racing GTI | Direct correction / partial service contract | Implement `MPMediaPickerController initWithMediaTypes:` plus media types, assign delegate, multiple-selection flag and copied prompt state. No music selection is fabricated. | The constructor blocker disappears. A later unavailable-service/cancel boundary remains visible. |
| Real Racing | Direct correction / partial service contract | Same shared media-picker construction/state contract. | Same prediction and boundary as R.Racing GTI. |
| SUPERFALL | Diagnostic | Watch duplicate autorelease scheduling so resource-name lifetime can be correlated with the pool that consumes it. | The next failure should preserve whether the name was scheduled twice; normal one-retain autoreleases are deliberately not promoted as suspicious. |
| Tiki Totems Premium | Regression control | No compatibility change. It checks working UIKit input, graphics, transitions and clean Home return. | It should remain working in the same tested scope. |
| TowerBloxx | Diagnostic | Preserve duplicate autorelease scheduling and table/transition history through the gameplay-to-Home path. | The next teardown failure should retain an earlier double-scheduling signal if one exists; no invalid cleanup is suppressed. |

## Implementation boundaries

- Clip mode is intentionally separate from the still-provisional tail
  truncation behavior.
- Tap recognition currently supports the one-tap/one-touch contract used by
  the evidence. Higher tap/touch requirements fail rather than being reported
  as successful.
- Pan recognition begins only after a bounded movement threshold and reports
  began/changed/ended target actions. It does not invent inertial scrolling.
- The media picker models object construction and properties only. The host
  media library remains unavailable, and no selected item or successful media
  service is fabricated.
- Every diagnostic ring is bounded and observational. Strict ownership,
  invalid-object and memory behavior is unchanged.

Host compilation and tests are implementation evidence only. None of the five
correction titles is promoted until the verified IPA is exercised on the
physical device.
