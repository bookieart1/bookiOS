# Dev 2.18 later compatibility contracts — 2026-11-03

## Evidence and scope

Dev 2.17 (`eb27d7f5`, workflow `41694954194`) was tested across both supplied
device logs. All 22 titles remained visibly at their prior failure stage, but
eight earlier blockers were removed and exposed exact later framework or libc
boundaries. Dev 2.18 addresses those later boundaries as shared contracts. It
does not modify the existing universal recorder or weaken invalid-object,
collection, or memory checks.

## Implemented contracts

| Contract | Primary apps | Classification and boundary |
|---|---|---|
| BSD `bcopy` | ADVENA, ZENONIA 2 | Verified overlap-safe memory-copy contract with Darwin's source-first argument order. |
| `+[AVPlayerItem playerItemWithAsset:]` and `-initWithAsset:` | EPOCH, Unmechanical | Partial AVFoundation construction contract. The item retains and returns the same asset; no decoding, tracks, duration, or playback success is fabricated. |
| `-[UITableView dequeueReusableCellWithIdentifier:]` | Bungee Ball | General UIKit reuse contract. A table retains removed reusable cells, returns a matching cell through autorelease ownership, calls `prepareForReuse`, and returns nil when none exists. |
| `hw.cachelinesize` | Ragdoll 3 | Historical original-iPhone sysctl value: a 32-bit integer reporting the ARM1176's 32-byte cache line. |
| `CGPathAddCurveToPoint` | Rolando | Partial Core Graphics drawing contract. Cubic curves are transformed and flattened into a fixed 16-line representation consumed by the existing renderer. |
| `-[UILabel drawTextInRect:]` | Junny | Historical UILabel drawing entry point; `drawRect:` now delegates through it so subclasses inherit or override the expected selector. |
| Historical six-argument `UIAlertView` initializer | Topple 2 | Partial private UIKit contract: copied title/message, weak delegate, default/cancel title roles, and an NSArray of other titles. It reuses the existing strict alert ownership and callback path. |
| `NSXMLParser` namespace option state | Wind-up Knight | Partial Foundation contract. Namespace configuration is stored and namespace-enabled element names use local names. Namespace-URI resolution and prefix mapping callbacks remain unsupported and are not claimed. |

Focused tests cover asset identity/ownership, overlapping `bcopy`, cache-line
ABI, cubic endpoint and bounded segment count, table reuse identity/ownership,
historical alert button roles, and XML parser option state.

## Physical-device batch

Only titles with changed compatibility code are requested, plus Spoing as the
single healthy general control. The picker places them first in alphabetical
order: ADVENA, Bungee Ball, EPOCH, Junny, Ragdoll 3, Rolando, Spoing, Topple 2,
Unmechanical, Wind-up Knight, and ZENONIA 2.

The other Dev 2.17 titles are deliberately omitted. B'Blast, Decoloro, Flip's
Escape, JellyCar 3, Parachute Ninja, Peggle, Splode, SUPERFALL, Tiki Totems 2,
Toy Bot 2, and Trenches need a new producer-level diagnostic or external crash
evidence; replaying them unchanged would not generate a distinct experiment.

## Predictions and limits

Each primary title must pass its named selector/sysctl/import. Reaching any
later blocker is useful contract progress but not a functional pass. The same
exact missing boundary falsifies linkage or ABI correctness. A new crash does
not by itself validate the entire contract. Existing saves remain untouched.
No app-identifier branch, network success, media playback success, nil-object
tolerance, out-of-bounds collection access, stale-object retention, or package
substitution is included.
