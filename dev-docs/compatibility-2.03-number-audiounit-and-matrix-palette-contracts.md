# Dev 2.03: number lifetime, Audio Unit errors and matrix palettes

## Dev 2.02 device evidence

The complete 861,054-byte device log embeds source `f239dd7`, IPA workflow
`41441045328`, and the permanent universal recorder for every eligible launch.
Its SHA-256 is
`c005999a830f357c0eac888260571d341d97bf35bcafff0d9512fef2dad477f0`.

Blot, Solomon's Boneyard, Space Monkey and Spoing returned cleanly through
emulator Home. The recorder separated the remaining seven failures into
different causes rather than treating their terminal panics as one family.

| App | First decisive Dev 2.02 evidence | Dev 2.03 decision |
|---|---|---|
| 8bitninja 1.2 | Bundle lookup for `sounds/swoosh.wav` exhausts exact and localized candidates, returns nil, and the guest passes that null URL to file-system conversion after 17 presented frames. | No compatibility change. The package supplies a differently named candidate; do not fabricate a resource or make null memory valid. |
| DoDoFly | A null-callback CF dictionary intentionally does not retain its values. It receives small integer object `0x30236f30`; the guest releases that object, and the dictionary later returns the deallocated pointer to `intValue`. | Implement Apple's process-lifetime CFNumber/NSNumber cache for integer values -1 through 12. This preserves correct non-retaining dictionary semantics while correcting the value's real lifetime. |
| LegendaryWar | A nested immutable dictionary returned from property-list state later receives `setValue:forKey:` for `height`. | No compatibility change. Mutating immutable `NSDictionary` is not a valid fix; retain the mutability-origin evidence for later investigation. |
| Peggle | An Audio Unit receives `kAudioUnitProperty_ClassInfo` on input scope and the emulator panics. | Return Apple's `kAudioUnitErr_InvalidScope`; ClassInfo is a global-scope property. Do not fabricate a successful audio configuration. |
| Tiki Totems 2 | Gameplay presents 107 frames, then `NSInvocation` is invoked with its ordinary argument at index 2 unset. | No compatibility change. The historical contract requires arguments to be set before invocation; do not silently manufacture nil arguments without evidence of an emulator ordering defect. |
| TrenchesVS | Gameplay presents 736 frames, then the guest requests array index 3 when the current array length is 2 after earlier removals. | No compatibility change. Keep the bounds check strict while preserving the mutation/caller history. |
| ZA! 2ndWave | Startup reaches `_glCurrentPaletteMatrixOES`, which iDared advertises through `GL_OES_matrix_palette` but did not export. | Implement the complete four-function OES matrix-palette entry-point surface on native GLES 1.1 and the GL 2.1 compatibility backend. |

## Implementation boundary

Dev 2.03 makes three independent general corrections:

- canonical process-lifetime integer objects for the Apple Core Foundation
  range -1 through 12, shared by toll-free bridged `CFNumber`/`NSNumber` paths;
- the defined Audio Unit invalid-scope result for ClassInfo rather than a host
  panic;
- `GL_OES_matrix_palette` current-palette, palette-load, matrix-index and
  weight-pointer entry points, including the corresponding client-array state
  on the desktop compatibility backend.

The OpenGL change closes an internal contract mismatch: the guest extension
string already advertised matrix palettes. The GL 2.1 backend maps the OES
surface to `ARB_matrix_palette`/`ARB_vertex_blend` and preserves the caller's
matrix mode while copying the model-view matrix into the selected palette.

No change mutates immutable collections, accepts out-of-range arrays, invents
unset invocation arguments, fabricates resources, retains values for null
Core Foundation callbacks, or weakens null/invalid-object checks. The universal
recorder is unchanged and remains enabled for every eligible app.

## Validation and device predictions

Focused automated coverage checks the integer cache's identity, range and
static lifetime, plus the Audio Unit invalid-scope result. Hosted compilation
also verifies both matrix-palette backends and the exported guest symbols.
Compilation and automated tests run only on GitHub-hosted runners; local native
preflight remains disabled after the November 2 Windows resource-saturation
incident.

The focused device prediction is that DoDoFly will pass the recorded
deallocated-small-number boundary, Peggle will receive a genuine Audio Unit
error instead of terminating, and ZA! 2ndWave will pass its first matrix-palette
entry point. Blot, Solomon's Boneyard, Space Monkey and Spoing are the relevant
regressions. The other four failing titles are retained in the return queue but
are not expected to change in this build.
