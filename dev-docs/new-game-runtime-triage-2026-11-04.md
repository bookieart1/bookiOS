# New-game runtime triage and cross-library priorities — November 4, 2026

This is an evidence-preservation and planning record. It does not authorize or
claim an emulator source change, commit, push, workflow, or IPA build.

## Evidence and scope

- Combined device log: `1A3F8B76-4D87-421B-9391-99165477E7AA/1-Log.txt`
- SHA-256: `4E98390805B43C0333EADE7A504A4477F5A857112F70FB39CDCA1754CDA05409`
- Embedded implementation: `eeb7775480cb8bd7b76089fdcdf64d9f16db1b3f`
- Workflow run: `41816961881`
- The earlier seven-log encryption inventory remains separate evidence. This
  pass analyzes runtime failures and the newly supplied replacement packages.
- Nyan Cat is deliberately excluded from the repair queue: its replacement
  package ran for 31.6 seconds, presented 747 frames, and returned through Home.
- Previously recorded titles were retained only where this pass establishes a
  newer boundary. Mega Jump, Paper Pilot, and Paper Toss repeat their existing
  known stages and are not promoted as new discoveries.

## Replacement packages that materially change the evidence

| Title | Package relationship | New verified reach | First decisive boundary | Decision |
|---|---|---|---|---|
| JellyCar | Decrypted JellyCar 1.5.4 (`com.walaber.jellycar`), distinct from JellyCar 3 | Guest code runs for 277 ms | A previously deallocated/reused `NSNumber` address is released again while draining an autorelease pool | High-value lifetime canary. Add pool-entry generation/provenance and test historical stack-order draining before changing semantics. |
| Sky Burger | Decrypted v1.2.2 replaces the encrypted v2.0 evidence | Framework initialization reaches UIKit | `UIActionSheet` is absent at `+alloc` | Exact legacy UIKit contract. Implement presentation, button indices, delegate callbacks, cancellation, dismissal, ownership, and teardown—not an allocation-only stub. |
| TowerBloxx | Decrypted full-version v2.0.26 (`com.digitalchocolate.iHouse`), a sibling rather than the same encrypted Lite bundle (`com.digitalchocolate.iHouseLite`) | Nib loading and one frame | `CustomPickerView` cannot receive the `dataSource` outlet because `UIPickerView` lacks that property | Exact legacy UIKit contract. Implement stable data-source state and the row/component query/reload behavior it drives. |

### JellyCar uncertainty boundary

The recorder proves use-after-deallocation at a reused address. It does not yet
prove whether the producer is duplicate autorelease scheduling, a stale entry
from an older allocation generation, premature pool drain, or drain ordering.
The current emulator stores raw object identifiers and drains them in insertion
order. Historical Objective-C pools behave as stacks, but changing order alone
is not sufficient evidence that JellyCar will be fixed. The next experiment
must associate every pool entry with the object's allocation generation and
record scheduling site, pool identity, drain site, and prior removal.

## Newly decisive runtime contracts

The following titles expose concrete general boundaries and are suitable for
focused implementation batches. A title appearing here is not claimed fixed.

| Contract family | Primary games | Boundary |
|---|---|---|
| Foundation paths and files | BikeRacePro, Legends of Ooo | Missing `-[NSFileManager URLsForDirectory:inDomains:]`. |
| Correct short reads | Carnival; Wire Way as an existing related canary | `-[NSFileHandle readDataOfLength:]` incorrectly asserts that every read returns the requested length instead of returning available bytes at EOF. |
| Parsing and time | Fieldrunners, MX Mayhem | `scanHexInt:` rejects real hexadecimal input; `_difftime` is absent. |
| Invocation and scheduling | 100 Chambers, Dismount, NinJump | Missing `NSInvocation.selector`, run-loop ordered selector scheduling, and `NSInvocationOperation initWithInvocation:`. |
| Data/string utilities | ParaPanicHD, The Sims 3, Scribblenauts | Missing `subdataWithRange:`, `_CFStringFindWithOptions`, and daylight-saving query behavior. |
| Pasteboard | Super Hexagon | Missing `containsPasteboardTypes:`. |
| Objective-C runtime | R.Racing GTI, Real Racing | `class_getProperty` is still an explicit runtime TODO. |
| Legacy UIKit | Sky Burger, TowerBloxx, 1112 ep.01, Highway Rider | `UIActionSheet`, `UIPickerView.dataSource`, `UIWindow.contentMode`, and notification launch-image state. |

## Other causal clusters

These clusters require a narrower experiment or historical research before a
broad compatibility correction is safe.

### Audio conversion and decoding

- Zombie Crisis 3D reaches an `ExtAudioFile` conversion assertion because the
  requested client format differs from the source format.
- Sky Burger also reports unsupported AIFC audio, but that warning is not its
  immediate crash.
- Required work: real format conversion/decoding and deterministic tests; do
  not return fabricated audio success.

### Graphics, context, and presentation

- Forlorn first exposes OpenGL error `0x506`.
- Pocket Tanks enters OpenGL without a current EAGL context.
- Trigger Fist presents frames but is cropped/displaced and not usefully
  interactive. BlockBlast and Granny Smith become black and unresponsive.
- Existing related standstills include Flip's Escape, Slingshot Racing,
  Decoloro, Bungee Ball, Sonic Dash, and ZA Second Wave.
- Required work: bounded capture at the first consequential GL error or visible
  corruption, including context, framebuffer/renderbuffer attachments,
  viewport/scissor, presentation, and guest caller. A shared visual symptom is
  not proof of a shared producer.

### Invalid object and nil producers

- CataGugl and PREDATORS both eventually compare an `NSString` against nil.
  That is a shared terminal family, not evidence that nil should be accepted.
- Ski Safari reaches a mutable-dictionary/class-semantics assertion.
- JellyCar now provides a specific deallocation/reuse history and is the best
  lifetime canary in this survey.

### Null-page consumers

Crash!, Dark Meadow, Earn to Die, FaceFighter, M.HUNTER, Mega Jump,
ShadowGun, SpongeBob Diner Dash, Vector, and Zuma HD end in null-page accesses.
They are not one root-cause cluster. The shared next capability is guest-PC
symbolization plus producer tracing for the value that became null.

### CPU, host termination, and services

- PES2012 and Worms hit undefined guest instructions; Lara Croft hits an
  unexpected SVC. These require instruction/call-site analysis, not exception
  suppression.
- Boss and Dead Trigger exit without a conclusive in-app terminal event. Native
  iOS `.ips` and Jetsam evidence are required before attributing them to the
  emulator.
- Sandstorm2 and Modern Combat 2 reach absent `GKSession`. This belongs to a
  truthful offline GameKit state/callback milestone, never fabricated network
  success.

## Package or externally blocked evidence

- Burrito Bison is reported as badly decrypted and is not a normal emulator
  compatibility target.
- Back to the Future Episode 1 has a bundle/resource checksum problem that must
  be separated from runtime emulation.
- The remaining explicitly encrypted packages stay outside the runtime queue
  unless a legitimate runnable replacement is supplied.

## Prioritized cross-library engineering queue

1. **Exact Foundation/POSIX contracts.** Implement the independently testable
   short-read, path, scanner, time, invocation, data, string, timezone, and
   pasteboard contracts. These have explicit failing calls, strong historical
   semantics, low-to-moderate regression exposure, and the broadest immediate
   validation set.
2. **Legacy UIKit controls and nib/KVC behavior.** Use Sky Burger and
   TowerBloxx as primary canaries; validate UIActionSheet behavior and picker
   data-source ownership/querying rather than only making selectors exist.
3. **Autorelease/lifetime provenance, then semantics.** Use JellyCar 1.5.4 as
   primary, with SUPERFALL, Peggle, Tiki Totems 2, Tesla Wars, Anomaly HD,
   Thief Lupin, and Silent Swords as secondary evidence. First distinguish stale
   generations, duplicate scheduling, premature drain, and ordering.
4. **Audio conversion.** Use Zombie Crisis 3D as the deterministic primary
   canary. Implement actual conversion rather than relaxing the assertion.
5. **Graphics-state causal snapshots.** Correlate the first bad frame/error for
   Forlorn, Pocket Tanks, Trigger Fist, and existing visual standstills.
6. **Null-value producer tracing and guest symbolization.** Apply one bounded
   diagnostic method across the null-page inventory, but keep the resulting
   root causes separate.
7. **CPU/host/service investigations.** Obtain native crash evidence for host
   exits, decode unsupported guest instructions, and model offline GameKit only
   after historical callback behavior is established.

## Recommended next milestone

The most productive next build is a scoped **exact-contract batch**, not a
large speculative graphics/lifetime combination. Primary new device tests
should be BikeRacePro, Carnival, Fieldrunners, Legends of Ooo, MX Mayhem,
ParaPanicHD, and Super Hexagon, plus only subsystem-relevant working controls.
Sky Burger and TowerBloxx should follow in a dedicated UIKit milestone unless
their complete contracts and tests are ready without obscuring attribution.
JellyCar should receive diagnostic provenance first; it should not be presented
as a compatibility fix until the producer has been discriminated.

