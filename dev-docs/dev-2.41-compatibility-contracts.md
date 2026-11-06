# Dev 2.41 compatibility contracts and focused diagnostics

Dev 2.41 continues from the device-tested Phase 2 persistent-index lineage and
Dev 2.40. It does not alter the persistent cold-start index, the retained
in-process catalog, package launch validation, orientation policy, or guest
teardown.

## Direct semantic corrections

- OpenGLES now exports the historical `EAGLGetVersion` C function. It reports
  the modeled EAGL 1.0 implementation and accepts either output pointer as
  null. The exact Dev 2.40 failures for 1000: Find'Em, Glyder, Glyder 2 and
  JumpO'Clock all stopped at this missing export.
- `NSKeyedUnarchiver` now treats reserved archive UID 0 as nil rather than
  decoding the `$null` marker into an `NSString`. In 1112 Episode 1 the
  `RenderNodeChilds` field referenced UID 0 and the app then sent `count` to
  the incorrectly materialized marker string.

## Bounded evidence, without speculative behavior changes

- `UITableView` records at most eight reload layouts and 24 selection mappings,
  including section row counts, bounds, content size, touch point, scroll
  offset, flat row and selected index path. This targets Bungee Ball's
  disappearing level-grid state after selection.
- `NSBundle` records bounded resource-search summaries after a complete miss,
  plus bounded ownership evidence for `bundlePath` and `resourcePath` returns.
  This targets Catacombs' nil resource/property-list boundary and Pollushot's
  later stale path-string dispatch without weakening either contract.
- Objective-C runtime method lookup records at most 24 complete superclass
  chains when `class_getInstanceMethod` finds no method. The implementation
  already searches inherited methods, so Demolish receives diagnostics instead
  of an unjustified second lookup implementation.

Karoshi is unchanged and remains a passive regression control for the previous
stdio correction. BLOT is a passive working OpenGL control. The target and
control packages are pinned to the picker front in this alphabetical order:
1000: Find'Em, 1112 Episode 1, BLOT, Bungee Ball, Catacombs, Demolish, Glyder,
Glyder 2, JumpO'Clock, Karoshi and Pollushot.

## Device predictions

The four OpenGLES consumers should cross `_EAGLGetVersion`. 1112 should no
longer fail because `RenderNodeChilds` is the literal `$null` string. These are
blocker-removal predictions, not claims of playability. The other four
unresolved targets should reproduce their existing behavior while leaving
bounded evidence that discriminates the next semantic correction. Karoshi and
BLOT should not regress. Cold launch, normal return and recoverable failure
return must continue using the Phase 2 index and retained catalog.
