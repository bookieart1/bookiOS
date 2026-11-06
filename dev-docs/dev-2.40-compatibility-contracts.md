# Dev 2.40 compatibility contracts

Dev 2.40 continues from the device-tested Phase 2 persistent-index lineage and
the Dev 2.39 source checkpoint. It does not alter persistent-index semantics,
the retained in-process catalog, guest teardown, or package launch validation.

## Focused contracts

- Detached `UIView` point and rectangle conversion to or from a nil view uses
  the existing `CALayer` root-coordinate conversion path. This targets the
  exact Level 3 call paths in 1000: Find'Em, Glyder, Glyder 2 and JumpO'Clock.
- `+[UIImage imageNamed:nil]` returns nil before bundle lookup. Bungee Ball's
  exact level-construction path treats the image name as optional and already
  checks the result.
- `_CATransform3DConcat` exports the existing matrix-composition operation for
  1112 Episode 1.
- `+[NSBundle allBundles]` returns the modeled main application bundle for
  Catacombs; framework bundles are not separately represented.
- `_objc_getMetaClass` returns the metaclass for a registered class, or Nil for
  an unknown class, for Demolish.
- `_setvbuf` accepts the three standard buffering modes against touchHLE's
  direct-I/O `FILE` model and rejects invalid modes for Karoshi.
- An unimplemented framework-class placeholder permits the inherited
  `+instancesRespondToSelector:` capability query while retaining fail-fast
  behavior for ordinary unsupported use. Pollushot's exact query for the absent
  `initWithContentViewController` selector therefore returns false.

The ten target packages are pinned to the beginning of the picker by exact
bundle identifier in this alphabetical order: 1000: Find'Em, 1112 Episode 1,
Bungee Ball, Catacombs, Demolish, Glyder, Glyder 2, JumpO'Clock, Karoshi and
Pollushot. Other packages remain alphabetized after this focused batch.

## Deliberate exclusions

Cut the Buttons' suspected Blocks ABI/lifetime issue remains too broad for this
batch. Crazy Golf's post-entry property-list failure and Collision Effect's nil
string comparison require focused runtime provenance; Dev 2.40 does not mask
either failure. No title is promoted to playable until physical-device testing
demonstrates the resulting behavior.

## Physical predictions

Each target should cross its recorded first blocker. The four detached-view
titles must no longer fail merely because NIB construction precedes window
attachment. Bungee Ball should cross its optional nil-theme-image boundary.
The remaining five API consumers should either progress or expose a later,
truthful blocker. A working control should still launch, play and return, and
the indexed cold launch plus retained-catalog return paths should remain fast.
