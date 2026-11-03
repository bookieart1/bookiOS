# Dev 2.09 expanded general contracts

Dev 2.09 combines the six later boundaries exposed by the Dev 2.08 device
test with three concrete contracts from the current inventory and two bounded
diagnostic improvements. It does not introduce app-name compatibility
branches, tolerate invalid objects, weaken memory checks, or fabricate service
success.

## Compatibility contracts

- ADVENA and ZENONIA2: export the historical `kCFURLFileLength` URL resource
  property key used by their shared resource manager.
- B'Blast: implement the real `NSError **` success/failure contract for
  `NSFileManager attributesOfItemAtPath:error:`.
- Bungee Ball: permit NSNumber property-list scalar values in
  `descriptionInStringsFileFormat`, preserving quoted NSString behavior and
  rejecting arbitrary object types.
- Junny: return the historical component count for the existing monochrome and
  RGB color-space models.
- Peggle: implement `OSAtomicOr32` and its barrier variant using the emulator's
  cooperative guest-thread model.
- BouncyMouse: add the retained view relationship, target-action lifecycle,
  state, scale, velocity, and touch delivery for `UIPinchGestureRecognizer`.
- DH Bowling: add `UIWindow resignKeyWindow`, including key-window state and
  the historical resign notification.
- Flip's Escape: accept floating-point drift in the homogeneous row after
  multiplying two mathematically affine transforms. This does not accept a
  general non-affine matrix.
- The Legend of Crystal Valley: implement the read-only zlib `gz*` stream
  family through the guest filesystem, including gzip decompression,
  transparent uncompressed reads, EOF, rewind, position, and cleanup.

## Bounded diagnostics

- Roboto: preserve the largest single presentation gap and its preceding
  bounded API history, complementing the existing sustained-slow capture.
- Unmechanical: preserve the newest 32 unresolved relocation summaries in the
  terminal flight-recorder report so launch evidence survives log truncation.

## Device-test layout

The app picker places the focused targets first in this order: ADVENA,
B'Blast, BouncyMouse, Bungee Ball, DH Bowling, Flip's Escape, Junny, The Legend
of Crystal Valley, Peggle, Roboto, Unmechanical, ZENONIA2, and Spoing. All
other installed apps remain alphabetized. The ordering changes only picker
presentation; it does not alter the games.

## Validation boundary

The standard local preflight is intentionally not run for this iteration due
to the prior Windows workstation incident and the user's hosted-only
instruction. Hosted lint/tests and the unsigned iOS IPA workflow are the
authoritative checks. Physical-device results remain required before any
title is promoted from “blocker removed” to a functional compatibility claim.
