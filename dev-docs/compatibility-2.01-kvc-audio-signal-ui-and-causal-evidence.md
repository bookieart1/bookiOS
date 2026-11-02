# Dev 2.01: KVC, Audio Unit, signal-stack, UI state and causal evidence

## Evidence and scope

The complete Dev 2.00 physical-device log is preserved as
`outputs/device-logs/iDared-Dev-2.00-foundation-posix-segmented-batch-2026-11-02.txt`
(707,680 bytes; SHA-256
`b08b13d42567b0f38f1eca08fb5b36212c8059e000ab1475749b3939ad2b8b21`).
It embeds final Dev 2.00 source `a736f6f487e299262ae8a0c5366f7ef71b276248`,
workflow `41431665435`, and the universal recorder for every eligible launch.

Every Dev 2.00 compatibility contract passed its former blocker. Solomon's
Boneyard reached working gameplay in the tested scope. LegendaryWar, Peggle,
Tiki Totems 2, TrenchesVS and ZA! 2ndWave exposed five later, exact, standard
contracts. Dev 2.01 implements those contracts together. It also adds bounded
causal evidence for the two maintained unresolved return targets, 8bitninja
1.2 and DoDoFly, without altering their emulation behavior.

## Implemented contracts

- **Foundation KVC assignment:** `NSObject setValue:forKeyPath:` resolves each
  relationship through `valueForKey:` and delegates the final assignment to
  the existing `setValue:forKey:` machinery. Accessor search, scalar unboxing,
  direct-ivar behavior and undefined-key handling therefore remain shared.
  `NSMutableDictionary setValue:forKey:` maps non-nil values to entries and
  removes the entry for nil, preserving Foundation's dictionary KVC behavior.
- **Audio Unit render notifications:** `AudioUnitAddRenderNotify` and
  `AudioUnitRemoveRenderNotify` store the callback/refcon pair as the listener
  identity. Registered listeners receive the documented pre-render and
  post-render flags around the existing render callback. A nonzero render
  result adds the post-render-error flag; notification return codes do not
  fabricate audio success.
- **dyld executable-path negotiation:** `_NSGetExecutablePath` now treats the
  required byte count as path length plus the NUL terminator. An undersized
  buffer returns `-1`, writes the required size, and leaves the buffer alone;
  a successful call leaves the caller's size unchanged.
- **POSIX alternate signal stack:** `sigaltstack` stores validated state per
  emulated thread, supports query and disable, rejects unsupported flags,
  enforces Darwin's 32 KiB minimum, and returns standard `EINVAL`, `ENOMEM` or
  `EPERM`. Signal delivery remains outside this contract.
- **Segmented-control tint state:** `UISegmentedControl` retains a stable
  `tintColor`, returns it, replaces it with normal ownership, and releases it
  at teardown. This does not claim pixel-accurate archived segment rendering.

Primary behavior references were Apple's KVC documentation, dyld's
`_NSGetExecutablePath` implementation and manual, XNU's Darwin signal ABI, and
Apple's Audio Unit render-notification documentation.

## Bounded causal evidence

- Failed `NSBundle` candidate paths are retained in a fixed 16-entry ring and
  emitted only with a recorder dump. This should distinguish an incorrect
  8bitninja lookup from a genuinely absent packaged resource without changing
  lookup order or nil behavior.
- Ownership-relevant operations on callback-backed Core Foundation dictionaries
  are retained in a fixed 48-entry ring: lookup, insertion, removal and
  deallocation, together with the four custom key/value retain/release callback
  addresses. Ordinary dictionary traffic is deliberately excluded. This is not
  a global retain/release trace and does not change object lifetime. It is
  intended to identify where DoDoFly's deallocated returned value entered or
  left its specific dictionary.

The universal recorder remains enabled for every eligible app. Both new rings
are bounded, allocation-free during recording, and silent during normal play.

## Validation plan

Focused guest tests cover nested and single-component KVC assignment, exact
dyld size negotiation, alternate-stack enable/query/disable and failure
semantics, render-notification registration identity/removal, and tint-state
ownership/identity. Compilation and automated testing run only on GitHub-hosted
runners: the local native preflight remains disabled after its November 2
resource-saturation incident. Hosted library tests, skipped or allowed-to-fail
steps, iOS arm64 compilation and package verification must be reported
separately.

## Device predictions and boundaries

| App | Dev 2.00 boundary | Dev 2.01 prediction |
|---|---|---|
| LegendaryWar | valid dictionary missing `setValue:forKeyPath:` | passes KVC assignment and exposes a later boundary or reaches further UI |
| Peggle | missing `AudioUnitAddRenderNotify` after logo and audio initialization | listener registration and render notifications execute; later audio/game boundary remains possible |
| Tiki Totems 2 | `_NSGetExecutablePath` asserted on an undersized buffer | size negotiation succeeds and startup continues |
| TrenchesVS | missing `_sigaltstack` before a loop/frame | alternate-stack registration succeeds; later signal APIs may remain |
| ZA! 2ndWave | valid segmented control missing `setTintColor:` | tint state passes and later UI initialization is exposed |
| 8bitninja 1.2 | nil resource later passed to CFURL conversion | behavior remains strict; crash dump names recent exact failed candidate paths |
| DoDoFly | callback dictionary returns a deallocated value | behavior remains strict; crash dump preserves recent dictionary/value/callback lifecycle events |

Blot, Solomon's Boneyard, Space Monkey and Spoing are focused regressions.
No save reset, app-identifier branch, resource fabrication, audio-success
fabrication, invalid-pointer tolerance or memory-check weakening is included.
