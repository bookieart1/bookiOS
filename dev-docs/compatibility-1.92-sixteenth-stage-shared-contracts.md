# Dev 1.92: sixteenth-stage shared compatibility contracts

Dev 1.91 device evidence is archived outside this checkout as
`outputs/device-logs/iDared-Dev-1.91-device-results.txt` (598,319 bytes,
SHA-256 `0ed369e06ecc0312dd8e3488ba57e85f303db8c6f5fd28209898fb64d6214a70`).
It embeds implementation commit `34adad5822892507e593680040ebd14193eed4f5`
and IPA workflow `41362430320`.

## Evidence and contracts

### Cube: permissive but bounded integer-PCM RIFF decoding

Cube reaches playable first-person gameplay, accepts shooting input and can
return cleanly through emulator Home. Its bundled shotgun sample is an ordinary
RIFF/WAVE PCM file (format tag 1, mono, 22050 Hz, 8-bit, one-byte block align),
but the strict `hound` reader and the fallback decoder both reject it. The game
then visibly reports `failed to load sample`.

Dev 1.92 adds a bounded RIFF chunk parser for uncompressed 8- and 16-bit integer
PCM files rejected by `hound`. It validates channels, sample rate, bit depth and
block alignment, ignores bounded unknown chunks, tolerates legal chunk ordering
and stale data lengths, and converts unsigned 8-bit PCM to signed little-endian
16-bit samples. It does not guess compressed codecs or fabricate successful
audio. A focused test covers a data-before-format file and exact sample
conversion.

The separate Cube touch assertion seen on one run is not attributed to audio
and receives no speculative change in this build.

### Junny and Rolando: `CFNumberGetType`

Junny and Rolando independently reach valid `NSNumber` objects and stop before
their first main loop on the same missing Core Foundation function,
`CFNumberGetType`.

Dev 1.92 exports the function and reports the emulator's canonical internal
storage category: 8-, 16-, 32- or 64-bit signed storage, or 32-/64-bit floating
storage. Unsigned values are promoted to a signed category wide enough for the
represented width where possible. The public historical type constants through
`kCFNumberCGFloatType` are also defined. A focused test checks integer,
unsigned-integer, float and double objects. This does not claim a complete
implementation of every `CFNumber` conversion API.

### Toy Bot 2: Core Animation transition subtype constants

Toy Bot 1 is now a verified functional checkpoint in the tested scope: the
user played through an area, advanced to another, used menus/resume, preserved
progress, exited and reopened successfully. Its first run returned through Home
after 205.306 seconds, 18,162 main loops and 6,117 EAGL returns; its relaunch
also returned cleanly.

Toy Bot 2 passes the earlier persistence blocker and reaches its comic panel.
Pressing Next enters `-[ComicView transitionNext]`, obtains panel count 13 and
panel index 1, then dereferences a null non-lazy symbol. The launch diagnostics
identify the missing standard symbols `_kCATransitionFromTop` and
`_kCATransitionFromBottom` (also used by Toy Bot 1); the faulting path needs one
as the direction argument to `transitionTo:direction:`.

Dev 1.92 exports all four historical Core Animation transition subtype strings:
from-right, from-left, from-top and from-bottom. This supplies API constants;
it does not claim to add native transition rendering or animation semantics.
A focused test checks the exported symbol names and string values.

### Fragger: corrected regression history, no speculative patch

Fragger's whole-host termination did not begin with Dev 1.90. Dev 1.89 first
ran Fragger for 44.427 seconds, 2,604 main loops and 2,604 presented frames,
then stopped on the property-list error-output assertion while saving score
state. In the same Dev 1.89 log and same embedded build, the immediately
following Fragger launch terminated iDared after its second EAGL context was
created. Dev 1.90 and later reproduce that same terminal point.

Therefore the Dev 1.89-to-1.90 source changes cannot by themselves explain the
onset. The strongest current hypothesis is a progress/defaults-dependent
relaunch path established by the successful session. A native graphics,
audio or thread failure remains possible. Because iOS terminates the process
without an emulator panic, Home event or recorder-end marker, an iOS Analytics
`.ips` report is needed to distinguish a native memory fault, abort, watchdog or
system kill. Dev 1.92 does not reset saves, weaken safety checks or add a
game-specific bypass.

## Device predictions

- Cube should load the shotgun sample without its visible failure message; the
  gun should produce audio if the remaining audio path is correct.
- Junny and Rolando should pass `_CFNumberGetType`; a later blocker is progress,
  not proof of compatibility.
- Toy Bot must retain its verified gameplay/save/relaunch behavior.
- Toy Bot 2's comic Next button should advance beyond the missing direction
  constant; a later blocker is progress.
- Spoing must retain normal launch and gameplay behavior.
- Fragger may still terminate iDared. Preserve its current save and obtain the
  matching iOS Analytics report rather than repeatedly changing unrelated APIs.

The permanent universal flight recorder remains enabled and unchanged.
