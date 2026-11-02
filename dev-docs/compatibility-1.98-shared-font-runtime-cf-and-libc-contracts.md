# Dev 1.98: shared font, runtime, Core Foundation and libc contracts

## Evidence and selection

The complete Dev 1.97 device log is preserved outside this checkout as
`outputs/device-logs/iDared-Dev-1.97-shared-contract-batch-2026-11-02.txt`
(726,513 bytes; SHA-256
`257e39fa791b5459919de98db4aa00060fd3bda3fbaf1bb03e17a3c5bd456e3b`).
It embeds final source `6eb54d93a74645cff52c6eceae1ea8ebb1b5dc17`,
workflow `41410063791`, and the permanent bounded recorder.

Dev 1.97 removed every contract blocker it targeted. SpaceMnkey reached
working gameplay, pause/menu/re-entry and clean Home return. Blot, Cube and
Spoing remained healthy. The native Home control also recovered Marblenauts
despite zero guest presentations. Six other titles advanced to later,
independent boundaries:

| App | First Dev 1.97 boundary | Dev 1.98 action |
|---|---|---|
| DoDoFly | Missing `+[UIFont familyNames]`. | Implement the historical installed-family table already represented by iDared's font mapper, plus `fontNamesForFamilyName:`. |
| LegendaryWar | Missing `_method_exchangeImplementations`. | Exchange two canonical runtime `Method` implementations while preserving their owning class/selector identities. |
| Peggle | Missing `_CFStringGetTypeID`. | Add a stable CFString-family identity and make `CFGetTypeID` return it for constant, mutable and ordinary NSString subclasses. |
| TrenchesVS | Missing `+[NSNumber numberWithUnsignedLong:]`. | Implement constructor, initializer and accessor using the 32-bit guest `unsigned long` ABI. |
| ZA! 2ndWave | Missing `_strcasestr`. | Implement C/POSIX-locale ASCII case-insensitive substring search; this also targets Solomon's Boneyard. |
| 8bitninja 1.2 | IPv4 `sockaddr.sa_len` is neither 16 nor zero, but the actual bytes are absent from the panic. | Preserve strict validation and include length, family and all 16 bytes in the one-time terminal diagnostic. Do not guess at or accept an invalid layout. |

These are combined to make the physical-device cycle useful; they are not one
failure family.

## Implementation boundaries

- `UIFont` returns immutable arrays for the seventeen historical iOS 2 font
  families and their known PostScript names. The table reuses the names
  already accepted by `fontWithName:size:`. Bundled-font family metadata is
  not synthesized.
- `method_exchangeImplementations` requires two valid non-null Method handles,
  swaps their IMPs through the existing canonical swizzling machinery and
  treats exchanging a method with itself as a no-op.
- `CFStringGetTypeID` returns one opaque process-local value. `CFGetTypeID`
  recognizes every NSString subclass as that same toll-free-bridged family;
  unrelated families retain their existing dynamic identity until their own
  public type accessor is implemented.
- `NSNumber` unsigned-long behavior is deliberately 32-bit, matching armv7
  iPhone OS rather than the 64-bit host.
- `strcasestr` follows the emulator's existing C/POSIX-locale ASCII folding,
  including empty-needle and not-found behavior.
- The app picker no longer draws repeated `UNOFFICIAL` labels. Its base remains
  black, while the fork identity remains in the version text, logs and About
  material.
- The 8bitninja change is diagnostic-only. It does not relax memory or socket
  validation and cannot itself make that title compatible.

The universal recorder, rendering, saves, network-access policy, resource
lookup and guest failure behavior are unchanged. There are no bundle-identifier
or title-specific branches.

## Validation and predictions

Focused checks cover method exchange/restore/no-op behavior, CFString type
identity across class-cluster members, 32-bit unsigned-long preservation,
`strcasestr` edge cases, and UIFont family/name discovery. Local Rust
compilation cannot complete on the current Windows host because the MSVC
linker is unavailable; hosted library tests and iOS compilation are therefore
required and must be reported separately.

Predictions for device testing:

- DoDoFly, LegendaryWar, Peggle, TrenchesVS and ZA! 2ndWave should cross their
  exact Dev 1.97 blockers; that is not a promise of menu or gameplay.
- Solomon's Boneyard should cross the same previously catalogued `strcasestr`
  boundary.
- 8bitninja should either cross its socket assertion naturally or report the
  exact unsupported address bytes needed for a defensible next decision.
- Blot, SpaceMnkey and Spoing are focused regressions. No save reset is needed.

Physical-device compatibility remains pending until the Dev 1.98 IPA is built
and tested.
