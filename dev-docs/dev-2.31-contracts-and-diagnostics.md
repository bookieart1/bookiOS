# Dev 2.31 contracts and diagnostics

Date: 2026-11-04

Dev 2.31 is the first compatibility candidate using the hosted Fast Preflight
gate. It follows the verified Dev 2.29 device result; Dev 2.30 was a separate
picker-performance instrumentation build and did not change these contracts.

No title is called fixed merely because code was implemented or a former
exception can be crossed. The physical-device pass must establish the next
visible and recorded boundary.

## General compatibility contracts

| App | Implementation boundary | Classification | Device prediction |
|---|---|---|---|
| Bike Race Pro | `NSDictionary -writeToURL:atomically:` delegates file URLs to the existing property-list writer and rejects nil/non-file URLs. | Partial historical Foundation contract | Cross the missing selector and either create the property list or expose the next honest failure. |
| Carnival | Decode packetized AIFC `ima4` audio into interleaved PCM16 using the existing Apple IMA4 decoder. | General audio correction | IMA4 decode failures disappear; music/effects may recover. Slowdown is separately unproven. |
| Labyrinth 2 | Implement `UIActivityIndicatorView.hidesWhenStopped`, its historical default, and visibility changes on start/stop. | General UIKit contract | Cross the missing getter without changing the intermittent physics defect. |
| Legends of Ooo | Export a real `NSManagedObjectModel` Objective-C class identity, without fabricating Core Data persistence behavior. | Strict partial Core Data contract | Cross `+alloc`; later model/persistence requirements remain visible. |
| ParaPanic HD | Return a stable positive guest-local `NSProcessInfo.processIdentifier`. | General Foundation contract | Cross the missing selector without exposing the host PID. |
| The Sims 3 | Implement Porter-Duff Clear blending for straight and premultiplied bitmap contexts. | General graphics correction | Cross blend mode 16 with transparent output in the covered region. |
| Trism | Export `CGContextClosePath` and close the context's existing mutable path. | General Core Graphics contract | Cross the missing symbol and preserve real close-subpath semantics. |

## Bounded investigations and controls

- **100 Chambers:** retain category-isolated KVC boundary history so later
  traffic cannot erase the producer/consumer context.
- **Cannon Cadets:** keep the strict bitmap-layout boundary; the existing
  diagnostics remain the evidence source rather than accepting arbitrary
  layouts.
- **Current:** retain category-isolated input/callback history to identify the
  first guest callback after touches reach its EAGL view.
- **Dismount:** repeat actual gameplay to determine whether the approximately
  15 presentations/second result persists after its former contract was fixed.
- **Fieldrunners 2:** add the instruction bytes at bounded guest thread
  snapshots so recurring wait PCs can be symbolized without an instruction
  trace.
- **Kick the Buddy:** preserve presentation/input evidence around the active
  black-rendering state; invalid graphics or objects remain strict.
- **MX Mayhem:** continue bounded allocation/lifetime evidence for the varying
  invalid receiver rather than special-casing an address.
- **Wire Way:** keep category-isolated transition history so the decisive
  completion/fade/audio-stop boundary survives later layout/presentation
  traffic.
- **Spoing:** healthy regression control for input, graphics, audio and return
  to the picker.

## Validation boundary

The implementation adds focused host tests for the new Foundation, UIKit,
Core Data, graphics and IMA4 behavior. The hosted Fast macOS Preflight must pass
before one unsigned iPhone archive is eligible to start. Windows and Android
remain concurrent non-gating information. Package verification and physical
device results remain separate evidence categories.
