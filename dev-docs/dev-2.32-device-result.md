# Dev 2.32 physical-device result

Date: 2026-11-05

The two supplied logs identify implementation
`9d568fc7526acc08f4ce6ee9185ca8fbff5e2cea` and unsigned IPA workflow
`41990584956`. This is verified device evidence from the final Dev 2.32
candidate, not an inference from its visible label or filename.

## Evidence

| File | Bytes | SHA-256 | Scope |
|---|---:|---|---|
| `B5C0A060-54DC-46DE-9300-050588F2F7CB/1-Log.txt` | 1,651,141 | `eada876077bd0a3f525953ebe541a66e45529455327c031564a8ce28b2d8b87e` | 1112 ep.01 through Sky Burger |
| `B5C0A060-54DC-46DE-9300-050588F2F7CB/2-Log.txt` | 1,060,368 | `f690178358dec4964fae12dc80033602ab6a82bc00e47517844723b0f482bb13` | Stick Stunt Biker, Tiki Totems Premium and TowerBloxx |

## Executive result

Dev 2.32 produced real progress, but no newly tested problem title is yet
verified fully compatible:

- **Highway Rider** now reaches sustained, responsive gameplay. Its mechanics
  work, but major scene content is absent: no road, gray planes and invisible
  collision hazards. The notification contract crossed; the remaining blocker
  is a graphics/resource presentation failure.
- **iStunt Reloaded Pro** now enters the level-pack view instead of failing on a
  plain `UIView`. The new root is a real `UITableView`, but the level buttons do
  not appear and repeated interaction eventually stops producing guest-visible
  responses. The table-controller correction crossed; list population and
  touch lifecycle remain separate.
- **Sky Burger** now reaches its name-entry prompt. Tapping that prompt closes
  the entire iDared host. The recorder contains no emulator panic or normal
  return; it ends after eight rejected frees of the same unknown address and an
  `NSData initWithContentsOfURL:` TODO. A native iOS `.ips` report is required
  before attributing the host termination to any one of those events.
- **TowerBloxx** now reaches playable gameplay, verifying that the picker
  contract crossed. When the native Home control is used, guest execution
  deadlocks and cleanup then panics in autorelease-pool teardown. Gameplay
  advancement and exit reliability must remain distinct.
- **Tiki Totems Premium** remains fully working in the user's tested scope and
  returns through Home cleanly. The relevant UIKit/Foundation control passed.

The other launches expose exact later contracts in the log but have no new
user-supplied visual assessment. They are recorded as log-derived boundaries,
not functional conclusions.

## Per-title reconciliation

| App | Verified Dev 2.32 result | Interpretation and next discriminating action |
|---|---|---|
| 1112 ep.01 | Reaches OpenGL setup, then stops at unimplemented text line-break mode `2`. | `contentMode` is no longer the first blocker. Research/implement the historical UIKit text-wrapping mode with focused layout tests. |
| Doodle Grub | A keyed unarchive returns an `NSString`, after which the game sends it `CGColor`. | The new color encoder does not prove existing archived data decodes correctly. Identify the archive key and whether this is old saved data before changing `NSString` or weakening dispatch. |
| Highway Rider | Runs for about a minute and exits through Home normally. The user confirms responsive mechanics but missing road/scene graphics and invisible hazards. | Notification state crossed. The log shows a complete framebuffer and active ES1 drawing but no decisive GL error. Correlate a screenshot/video with bounded texture/resource uploads; generic sidecar-file probes alone do not prove package damage. |
| iStunt Reloaded Pro | Enters and switches among level packs. The root hierarchy contains a real `UITableView`, but level buttons are absent; repeated taps eventually stop affecting guest UI. Near the end, new touches are repeatedly ignored because the view is treated as already tracking a touch. | The table-controller correction crossed. Separately capture data-source component/row/cell calls and touch begin/end ownership; do not invent level rows or globally enable multitouch. |
| Junk Jack Retro | Crosses dictionary block enumeration and next stops at missing `UITapGestureRecognizer` allocation. | General UIKit gesture-recognizer implementation is the next exact contract; preserve target/action and touch-state semantics. |
| R.Racing GTI | Crosses Objective-C property lookup and stops at missing `MPMediaPickerController initWithMediaTypes:`. | The runtime metadata correction advanced. A historically accurate media-picker unavailable/cancel path is required; do not fabricate a music selection. |
| Real Racing | Same later `MPMediaPickerController initWithMediaTypes:` boundary as R.Racing GTI. | This is a real two-game shared MediaPlayer/UIKit contract, independent of property metadata. |
| Sky Burger | Reaches the name prompt, then tapping it closes iDared. The log ends abruptly without a recorder panic, after repeated rejected frees of address `0x3020a030`. | The action-sheet boundary advanced, but the host termination's origin is unverified. Obtain the matching native `.ips`; retain strict allocator behavior and do not suppress the invalid frees. Unsupported IMA4 AIFC remains a separate audio limitation. |
| Stick Stunt Biker | Two launches still hit the ASCII-only assertion in `ns_string.rs` after frames begin presenting. | The MacRoman/Latin-1 correction did not cover the actual entry path. Capture the bytes, requested encoding and guest caller at this assertion before broadening conversion behavior. |
| Tiki Totems Premium | User reports perfect operation; clean Home return is recorded. | Healthy regression control passes in the tested scope. This remains a scoped observation, not exhaustive certification. |
| TowerBloxx | The user reaches and plays gameplay. Native Home then produces `No active threads, program has deadlocked!`, followed by an autorelease-pool/coroutine cleanup panic. | Picker behavior crossed. Preserve the gameplay result, but investigate Home/teardown scheduling and autorelease-pool balance before promoting the title to fully compatible. |

## Compatibility classification after Dev 2.32

- **Gameplay reached but visually unusable:** Highway Rider.
- **Gameplay reached with exit/teardown failure:** TowerBloxx.
- **Menu/level selection advanced but incomplete and later input-stalled:**
  iStunt Reloaded Pro.
- **Host termination requiring native evidence:** Sky Burger.
- **Exact later contracts exposed:** 1112 ep.01, Junk Jack Retro, R.Racing
  GTI and Real Racing.
- **Serialization/save-origin uncertainty:** Doodle Grub.
- **Implemented encoding hypothesis incomplete:** Stick Stunt Biker.
- **Healthy control:** Tiki Totems Premium.

## Preserved next actions

This result is a documentation checkpoint only. It does not authorize a source
change, push, workflow or IPA build. The highest-information next evidence is:

1. Sky Burger: matching native iOS `.ips` for the host termination.
2. Highway Rider: one screenshot or short recording correlated with bounded
   first-corrupt-frame texture/resource state.
3. iStunt: bounded table data-source and touch-lifecycle capture.
4. TowerBloxx: isolate Home/guest-thread teardown independently of gameplay.
5. Implement the explicit shared/direct contracts only after their historical
   semantics are established: UIKit line-break mode, tap recognizers and the
   unavailable media-picker path.
