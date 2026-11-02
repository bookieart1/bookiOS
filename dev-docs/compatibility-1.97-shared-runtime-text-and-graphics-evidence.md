# Dev 1.97: shared runtime/text contracts and bounded graphics evidence

## Evidence and selection

The Dev 1.93 portfolio sweep and the Dev 1.96 focused device run identify
several independent general boundaries suitable for one device iteration:

- 8bitninja 1.2 and DoDoFly call `CFDictionaryCreateMutable` with a positive
  capacity. Apple's open-source Core Foundation header describes this value as
  a non-negative sizing hint that an implementation may ignore.
- LegendaryWar needs `class_getClassMethod`; TrenchesVS needs `class_getName`;
  Peggle needs `CFGetTypeID`.
- SpaceMnkey and ZA! 2ndWave need restartable `mbsrtowcs` conversion.
- DizzyWizards presents frames but produces almost no visible image, while
  Marblenauts runs without presenting. The former needs bounded first-present
  GL state and the latter needs an exit control that is not itself dependent on
  a guest presentation.

These are not one crash family. They are combined only to avoid spending a
physical-device/build cycle on a single title.

## Implementation

- Accept non-negative mutable-dictionary capacity hints without imposing a
  fixed bound. Existing key/value callback and ownership behavior is unchanged.
- Implement inherited class-method lookup through the metaclass and stable
  class-name C strings, including the documented empty string for `Nil`.
- Implement opaque, stable Core Foundation type identity from the dynamic
  toll-free-bridged Objective-C class.
- Implement bounded UTF-8 `mbsrtowcs` count, truncation, source-update,
  termination and `EILSEQ` behavior. UTF-8 has no shift state; handler/locale
  expansion beyond the emulator's existing UTF-8 environment is out of scope.
- Record renderbuffer allocation and at most three presentation-state snapshots
  per EAGL context (renderbuffer size, framebuffer/status, viewport and
  scissor). No per-frame trace is added and no GL error state is consumed.
- On iOS, place the emulator Home control in a native UIKit overlay and route
  its callback back through the normal event queue. It no longer depends on a
  guest `presentRenderbuffer:` call. Other hosts retain the existing OpenGL
  decoration.

The universal flight recorder, null-page protection, API failure behavior and
game-independent dispatch remain unchanged. There are no bundle-identifier or
title-specific branches.

## Validation boundary

Focused guest checks cover positive dictionary capacity and growth,
Core Foundation type-ID stability, class method/name contracts, and
`mbsrtowcs` restart/count behavior. Hosted library tests and the iOS archive
build remain required. Package verification cannot establish physical-device
rendering, native-control placement, or app compatibility.

The device batch is 8bitninja 1.2, Blot, Cube, DizzyWizards, DoDoFly,
LegendaryWar, Marblenauts, Peggle, SpaceMnkey, Spoing, TrenchesVS and
ZA! 2ndWave. Blot, Cube and Spoing are regressions rather than targets.
