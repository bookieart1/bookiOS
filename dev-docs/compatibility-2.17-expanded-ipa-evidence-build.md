# Dev 2.17 expanded IPA-evidence build — 2026-11-03

This is the final implementation scope layered on the reviewed Dev 2.17
candidate at `0a9144d8`. It preserves the five-game causal investigation and
the Dev 2.16 contracts, then incorporates the later executable/resource/log
review. No compatibility path branches on a bundle identifier.

## New general work in this revision

- **ADVENA and ZENONIA 2:** implement the historical 32-bit Darwin `hostent`
  result for numeric IPv4 literals. This is local parsing, not DNS or network
  success; later offline socket behavior remains genuine.
- **Bungee Ball:** implement bounded, data-source-driven `UITableView` cell
  construction and row-selection delivery. The previous implementation only
  counted rows, explaining the empty visible menu and inert touches.
- **Flip's Escape:** post-input draw snapshots now include the enabled vertex
  and texture-coordinate arrays, buffer bindings, sizes, types, strides and
  pointers. This distinguishes bad geometry sources from otherwise sane FBO,
  viewport and presentation state without reading pixels or changing GL state.
- **Trenches VS:** `makeObjectsPerformSelector:` now records receiver count and
  object identity before and after every callback. Enumeration behavior stays
  strict; mutation is neither hidden nor tolerated.
- **Picker:** the focused batch is placed first and the header permanently
  displays `Dev 2.17` below the public version.

## Preserved earlier Dev 2.17 work

| Test | Meaningful change |
| --- | --- |
| B'Blast | Bounded failed-`strstr` provenance; diagnostic only. |
| EPOCH / Unmechanical | Partial `AVURLAsset` construction preserving URL/options identity; no fabricated media success. |
| JellyCar 3 | Core Foundation epoch constants. |
| Junny | Historical `UIEdgeInsetsFromString` parsing with 32-bit struct-return ABI. |
| Ragdoll 3 | `hw.activecpu` for the emulated single-core device. |
| Rolando | `CGPathGetCurrentPoint`. |
| SUPERFALL | Persistent watched resource-name lifetime provenance. |
| Tiki Totems 2 | Bounded raw-allocation provenance. |
| Topple 2 | Nil HTTP-header value removes that header. |
| Wind-Up Knight | Ownership-neutral `CFMakeCollectable`. |

## Diagnostic-only tests from the later evidence review

| Test | Exact question this build can answer |
| --- | --- |
| Decoloro | Input-triggered CA/EAGL snapshots and the permanent ObjC history show whether the partial intro changes render state or never establishes an interactive hierarchy. |
| Parachute Ninja | The permanent typed ObjC call history preserves the NG callback argument and scheduled/direct ordering at the first invalid `domain` receiver. |
| Peggle | Existing bounded allocator/object history preserves the producer and reuse history for a strict collision or the later compositor/keyboard failure; no fault is suppressed. |
| Splode | Existing callback/notification history is retained around the level-completion transition; no OpenFeint result is fabricated. |
| Toy Bot 2 | Touch-driven graphics snapshots plus ObjC delivery history distinguish an inert hit path from a scene that accepts input but fails to advance. |

## Control and explicit omissions

Spoing is the single regression control. Hook World and Legendary War remain
omitted because the IPA review points to guest mutation of immutable property
list/defaults data and a native historical comparison is still required.
Slingshot Racing needs a substantially broader GLES2/shader investigation than
this GLES1-focused build. Mage Gauntlet's exact IPA was unavailable. Retesting
any of those here would not produce attributable evidence.

## Validation boundary

Focused source tests cover numeric `hostent` layout/content and table section
row mapping. Existing tests cover every preserved Dev 2.17 semantic contract
and bounded recorder capacity. Invalid memory, invalid objects, immutable
collections and offline services remain strict. Hosted results and artifact
identity are recorded separately after the single push; no local Rust/Cargo,
CMake, preflight or packaging helper is run.
