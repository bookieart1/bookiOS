# Dev 2.19: later contracts, causal provenance, and picker status overrides

Dev 2.19 follows physical Dev 2.18 (`9566691`, workflow `41703580910`). It
combines four small historical contracts with bounded provenance aimed at the
eleven standstill titles intentionally omitted from Dev 2.18. No diagnostic
relaxes a fault, retains a guest object, changes touch delivery, changes
scheduling, invents a resource, or fabricates media/network success.

## General compatibility corrections

| Contract | Primary apps | Boundary and limit |
| --- | --- | --- |
| XML property-list serialization | ADVENA | `NSPropertyListSerialization` now emits XML when format 100 is requested, continues to emit binary for format 200, and returns a real error description for unsupported formats. |
| `AVPlayerItem.status` / `error` | EPOCH, Unmechanical | A newly constructed item reports historical `Unknown` and nil error. It does not claim readiness, decoding, tracks, duration, or playback. |
| `hw.l1dcachesize` | Ragdoll 3 | Reports the original iPhone ARM1176 16 KiB L1 data-cache size through the existing 64-bit sysctl ABI. |
| Historical private `UIAlertView` other-buttons argument | Topple 2 | Accepts either an NSArray of titles or the one title string observed from the early caller, while reusing the existing ownership/button model. |

## Bounded causal additions

- Every `strstr` result, not only misses, is retained in a 32-entry ring. This
  can show how B'Blast's bundle-character name search advances before the
  terminal miss.
- A 64-entry touch lifecycle records begin/move/end delivery, nil hit views,
  active-touch count and point. This targets Toy Bot 2's apparently stuck
  ComicView touch and Decoloro's out-of-container landscape controls.
- A 64-entry delayed-selector ring records scheduling and delivery with target,
  selector, argument and delay. This targets Parachute Ninja's wrong callback
  argument and provides ordering evidence without changing callbacks.
- Collection evidence now records the first three raw words and observed class
  status of the inserted value. This targets Tiki Totems 2's 12-byte non-object
  structure without accepting it as an Objective-C object.
- Watched lifetime evidence now identifies the exact autorelease pool that
  consumes a watched object. This targets SUPERFALL's proven resource-name
  use-after-autorelease without globally retaining names.
- Array events now include array and transferred-object classes. This narrows
  Trenches VS's mutation/alias path without permitting the invalid index.
- Terminal dumps include 32 bounded bytes around the actual guest PC and image
  identity. This supplies a symbolizable producer/consumer site for JellyCar 3,
  Splode, and other near-null or invalid-object failures; it is not instruction
  tracing.

Flip's Escape and Peggle continue to use the permanent post-input GLES and
framebuffer evidence. Dev 2.19 adds exact touch/fault correlation but makes no
speculative GL-state correction. Bungee Ball, Junny, Rolando, Wind-up Knight,
and ZENONIA 2 receive the same terminal instruction/callback evidence after
their Dev 2.18 boundaries; no offline service or missing resource is faked.

## Picker behavior

- The visible header label is `Dev 2.19`.
- The meaningful test batch is placed before the alphabetized remainder.
- Long-pressing an icon opens **Mark as Playable**, **Mark as Not Playable**,
  and, after an override, **Use Built-in Status**.
- The override is keyed by bundle identifier, stored in `UserDefaults`, and
  affects only the Playable/Needs Work filters. It survives relaunch and normal
  upgrades and does not modify the engineering ledger or emulator behavior.

## Validation boundary

Focused host tests cover XML serialization, AVPlayerItem state, the cache-size
sysctl, the historical alert argument, and bounded diagnostic storage. Hosted
tests, iOS compilation, unsigned-package verification, and physical-device
results remain separate evidence. The universal recorder remains enabled. Its
compile-time test ceiling is now 384 KiB after adding the fixed-size histories;
the device-tested recorder remains bounded and showed no measurable slowdown.
