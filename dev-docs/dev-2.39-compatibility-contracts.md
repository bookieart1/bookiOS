# Dev 2.39 compatibility contracts

Dev 2.39 starts from the device-validated Phase 2 persistent-index commit
`8ec92ed06487ef0b6705d7ec1ecdd91b179ccfb9`. It preserves the persistent
cold-start index, process-lifetime retained catalog, real selected-package
runtime validation and full-scan recovery path.

## Implemented contracts

- `UIImagePickerController sourceType` now preserves the historical default
  and per-controller getter/setter state while continuing to report unavailable
  host camera/photo sources truthfully. Targets: Pollushot and Karoshi.
- `NSString cStringUsingEncoding:` explicitly supports a one-byte terminator
  for Windows CP1252 and writes the terminator rather than relying on allocator
  initialization. Target: Crazy Golf.
- `CATransform3DMakeRotation` exports the existing, tested general 3D rotation
  implementation. Target: 1112 Episode 1.
- `NSString stringWithContentsOfFile:usedEncoding:error:` and the corresponding
  initializer reuse the existing BOM/default-encoding and file-error machinery,
  report the detected encoding, and report an inapplicable-encoding error on a
  decode failure. Target: Catacombs.
- Objective-C runtime `class_isMetaClass` exposes the runtime's existing class,
  metaclass, substituted-class and unimplemented-class identity. Target:
  Demolish.
- `pthread_attr_setscope` accepts Darwin's supported system contention scope,
  returns `EINVAL` for an invalid attribute or scope and `ENOTSUP` for the
  recognized but unsupported process scope. Target: Collision Effect.

These implementations remove exact recorded first blockers. They do not claim
that any target is fully compatible until physical-device testing reaches and
verifies meaningful gameplay.

## Focused validation

- Image-picker default/source-type state and truthful source unavailability.
- CP1252 encoding and explicit NUL termination.
- File-encoding detection and successful `usedEncoding:` out-parameter.
- Rotation export equivalence with the existing transform operation.
- Class versus metaclass versus `Nil` runtime identity.
- Supported, unsupported and invalid pthread scope behavior.
- Existing persistent-index/retained-catalog test coverage remains part of the
  hosted full library test gate.
