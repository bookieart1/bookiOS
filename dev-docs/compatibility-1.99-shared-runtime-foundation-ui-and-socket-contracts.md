# Dev 1.99: shared runtime, Foundation, UIKit and socket contracts

## Evidence and scope

The Dev 1.98 device log is preserved as
`outputs/device-logs/iDared-Dev-1.98-shared-contract-batch-2026-11-02.txt`
(658,877 bytes; SHA-256
`56ba0e4e0d459704fc756fde28b884603fc226e1ecb9cbcab0d4f50392f51282`).
It embeds source `b703edaeb1104f78f121ae06642f311be0a0dc61`, workflow
`41414382461`, and the permanent universal flight recorder.

The run verifies that every Dev 1.98 target contract was reached or passed. It
also supplies seven independent next contracts. Dev 1.99 implements those
contracts together to make one physical-device cycle useful without merging
unrelated final symptoms or adding app-name branches.

## Implemented contracts

- **Darwin IPv4 socket input:** `bind`, `connect`, and `sendto` now treat their
  explicit `socklen_t` argument as authoritative when decoding a guest
  `sockaddr`. The family and explicit minimum size remain strict. This matches
  Darwin's copied-in address contract and permits 8bitninja's otherwise valid
  broadcast address whose `sin_len` byte was uninitialized. Network-offline
  behavior remains a genuine `ENETUNREACH`; no connection success is faked.
- **CFDictionary callback independence:** key and value callback pointers are
  installed independently. A dictionary with null key callbacks and standard
  retaining value callbacks now retains values as Core Foundation requires.
  This addresses the deallocated UITouch value retrieved by DoDoFly.
- **Objective-C protocol naming:** `NSStringFromProtocol` reads the name from
  the compiler-emitted 32-bit Objective-C 2 protocol layout and returns an
  NSString. Null input returns nil.
- **CFNumber identity:** `CFNumberGetTypeID` returns one stable opaque type ID,
  and `CFGetTypeID` assigns that same ID to the toll-free-bridged NSNumber class
  family.
- **POSIX file creation:** `creat` has its standard create/truncate/write
  behavior through the existing virtual filesystem. Permission bits remain
  unmodeled, consistently with `open`.
- **Foundation file metadata key:** `_NSFilePosixPermissions` is exported with
  its historical string value. This removes the shared PLCrashReporter startup
  boundary in Tiki Totems 2 and TrenchesVS without changing crash handling.
- **UILabel highlight state:** `highlightedTextColor` and `highlighted` have
  retained state, getters/setters, redraw invalidation, teardown, and render
  selection. This supplies ZA! 2ndWave's missing selector as a real small UIKit
  contract.

The universal recorder, memory-fault checks, invalid-object checks, offline
network semantics, and app picker are unchanged. DizzyWizards and Marblenauts
receive no speculative rendering patch.

## Predicted device advances

| App | Dev 1.98 first decisive blocker | Dev 1.99 prediction |
|---|---|---|
| 8bitninja 1.2 | valid AF_INET bytes rejected because `sa_len` was `0xa0` despite syscall length 16 | address parsing reaches the normal offline result or a later contract |
| DoDoFly | `intValue` sent to a deallocated CFDictionary value after the menu | touch dictionary retains its value; Play reaches later initialization |
| LegendaryWar | missing `_NSStringFromProtocol` | protocol naming passes |
| Peggle | missing `_CFNumberGetTypeID` | number type identity passes |
| Solomon's Boneyard | missing `_creat` | file creation passes |
| Tiki Totems 2 / TrenchesVS | unresolved `_NSFilePosixPermissions` caused null-page access in PLCrashReporter | constant resolution passes while strict crash reporting stays intact |
| ZA! 2ndWave | valid UILabel lacked `setHighlightedTextColor:` | label setup passes |

These are blocker-removal predictions, not gameplay claims.

## Validation plan

- Guest tests cover independent CFDictionary callbacks, CFNumber type identity,
  `creat`, `NSStringFromProtocol`, the Foundation constant, and UILabel state.
- A Rust unit test proves an input sockaddr with a garbage embedded length but
  explicit syscall length 16 decodes as the intended IPv4 broadcast address.
- Hosted development-library tests and lint are inspected separately from the
  arm64 iOS build and unsigned-package verification.
- Device regressions are Blot, Space Monkey and Spoing. The target apps stop at
  their first reproducible failure so the recorder preserves the next boundary.

## Remaining limitations

This work does not implement network service success, filesystem permission
storage, protocol mutation APIs, every CF type ID accessor, UILabel animation,
or any app-specific bypass. Newly exposed failures must be assessed as new
contracts rather than evidence that these implementations failed.
