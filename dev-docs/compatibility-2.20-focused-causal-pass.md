# Dev 2.20 focused causal pass

Dev 2.20 follows the Dev 2.19 device review. It deliberately requests only
titles that receive changed compatibility code, a new bounded diagnostic, or a
directly relevant regression control.

## Included changes and predictions

| Title | Classification | Change | Prediction / falsifier |
| --- | --- | --- | --- |
| B'Blast | Diagnostic-only | At the first five-second flight-recorder checkpoint, emit the newest 12 boundary events once. | A run that later disappears without a terminal dump should still retain immediate lead-up context. If the process ends before five seconds or the snapshot has no relevant boundary, the experiment is inconclusive. |
| Peggle | Diagnostic-only | Preserve the newest 32 guest draw/framebuffer calls and print them only when a GL error is observed by the guest or Core Animation compositor. | The next `0x506` failure should identify the candidate call sequence preceding the sticky error. The history narrows causality; it does not by itself prove which call was invalid. |
| Ragdoll 3 | Semantic correction | `hw.l2cachesize` returns the historical absent-name result (`-1`, `ENOENT`) for the emulated iPhone1,1 instead of panicking. | Ragdoll 3 should cross this query or handle the absent optional cache. A repeat at this same query falsifies the implementation. |
| Spoing | Regression control | No compatibility change. | Short launch/gameplay/exit verifies the recorder and GL bookkeeping did not disturb a healthy OpenGL ES title. |

The app picker displays **Dev 2.20**, places these four titles first, and no
longer jumps to page one after a long-press Playable/Not Playable override.
Changing the actual All/Playable/Needs Work filter still resets to its first
page intentionally.

## Explicit exclusions

- Topple 2 directly passes a `UIColor` where `CALayer.backgroundColor` requires
  a `CGColorRef`; globally accepting that mismatch would hide invalid API use.
- Trenches VS mutates the receiving array from inside
  `makeObjectsPerformSelector:` even though Foundation forbids that mutation;
  snapshotting globally would invent semantics.
- SUPERFALL and Tiki Totems 2 now have exact producer classes from Dev 2.19;
  their next step is focused binary/caller analysis, not another unchanged
  device replay.
- EPOCH and Unmechanical need a real media-readiness model rather than more
  status logging. Parachute Ninja needs callback-value provenance. Decoloro and
  Toy Bot 2 need a specific presentation/state theory. None receives a
  speculative patch in this build.

## Validation boundary

Focused unit tests cover absent L2-cache behavior, the bounded GL candidate
history, and one-time flight-recorder checkpoint output. The permanent
universal recorder, strict invalid-memory behavior, saves, networking, media,
and app packages are unchanged. Hosted validation and package verification are
recorded separately after the single reviewed push.
