# Dev 2.27 physical-device result

Date: 2026-11-04

All three supplied logs identify implementation `4b4b3bd8abd91aa1bd65153624ed8071bd40be11`
and unsigned IPA workflow `41837856742`. The universal recorder was active for
every eligible launch. This is therefore verified Dev 2.27 evidence rather than
an inference from a filename or picker label.

## Evidence

| File | SHA-256 | Scope |
|---|---|---|
| `5D0107E2-FBA5-4B6A-9132-595893A71CBA/1-Log.txt` | `32cb308396aee0cfadb6ccfe0bfe6fe473b5b6f9a30300c39f37f7b817a9100c` | 100 Chambers through Fieldrunners 2 |
| `5D0107E2-FBA5-4B6A-9132-595893A71CBA/2-Log.txt` | `2d3d79a30dbbdd3dbbf585c864aea7b50aceb65805dbf513632dedeaea4d7ebe` | Legends of Ooo through Wire Way |
| `9739453D-A784-4AD0-9BD8-86C63055B3EF/1-Log.txt` | `9552b6f9bd01d3c14066e806c39d0d9bfaf1f83969341b540db949dfdfb0ef2a` | Super Hexagon follow-up |

## Results

Every Dev 2.27 contract crossed its former first blocker. That fact does not
promote any title to fully compatible by itself. Wire Way is the one large
functional advance: gameplay, input and presentation were observed for about
124 seconds, but pause and post-level screens remain invisible.

| App | Verified result | Current first boundary / next decision |
|---|---|---|
| 100 Chambers | Crosses `NSInvocation.selector`; stops before the application main loop. | Missing `NSOperationQueue addOperationWithBlock:`. This is a general Foundation contract, but it requires correct copied-block lifetime and queue execution rather than an unconditional synchronous call. |
| BikeRacePro | Receives the requested directory URL. | Missing `NSURL URLByAppendingPathComponent:`. This is shared with Legends of Ooo and is a strong small general-contract candidate. |
| Carnival | File reads cross the former short-read assertion. | Missing `_CFNotificationCenterGetDarwinNotifyCenter`. Implement only a historically bounded notification center and actual observer/post semantics used by the caller; do not turn it into a generic no-op success. |
| Dismount | Ordered run-loop delivery crosses and three frames present. | `printf` does not support dynamic `*` width/precision. This is a general libc format-contract candidate with focused parsing/ABI tests. |
| Fieldrunners | Hex scanning crosses. | A null value is consumed as a mutable NSString while loading bootstrap form/XML state. The null producer is not yet established; record the exact call/return producer before changing string semantics. |
| Fieldrunners 2 | Multi-domain search no longer asserts. | Startup never enters the application main loop or presents a frame. Six guest threads continue mutex/time/sleep activity; progress checkpoints show `main-loops=0`, `EAGL-returns=0`, and queued Home events cannot be handled. This is a guest-side initialization wait/loop, not a frozen native picker button. Symbolize the recurring guest PCs and take a bounded all-thread snapshot before any semantic patch. |
| Legends of Ooo | Receives the requested directory URL. | Same missing `NSURL URLByAppendingPathComponent:` contract as BikeRacePro. |
| MX Mayhem | `difftime` links and execution reaches 101 presentations. | Missing `NSFileManager subpathsOfDirectoryAtPath:error:`. A checked recursive, relative-path enumeration is a safe general candidate. |
| NinJump | Invocation-operation construction crosses. | Null-page read at `0x0` before the application loop. The log proves the endpoint but not the nil producer; add focused invocation target/selector/argument and execution-result provenance rather than relaxing memory checks. |
| ParaPanic HD | `subdataWithRange:` crosses. | Missing CommonCrypto `_CC_SHA224`. A real SHA-224 implementation is a narrow, testable general contract; do not alias it to SHA-256 or fabricate a digest. |
| Scribblenauts | Fixed-zone DST query crosses. | Null-page read at `0x0` before the application loop. The current recorder does not identify the origin sufficiently; preserve strict null-page protection and add producer/caller provenance. |
| Super Hexagon | `containsPasteboardTypes:` executes and returns normally. | Startup later reads through a null value in `SuperHexagon` at PC `0x003fe12c`, before any main loop or frame. Static guest-code analysis/provenance is required. The valid IPA has a blank display name, which also exposed a separate picker-pinning bug. |
| The Sims 3 | `CFStringFindWithOptions` links and executes. | Missing `_CGContextBeginPath`. This is a general Core Graphics path-state contract, but implementation must preserve the user's existing uncommitted `cg_context.rs` changes. |
| Wire Way | **Major advance:** smooth gameplay and input observed; 10,070 main loops and 3,679 presentations over 123.9 seconds; clean Home return. | Pause and level-complete states render black while touches continue to reach the EAGL view. The final visual history shows the guest actively drawing, including a black full-screen primitive, so this is not a UIKit hit-test failure or application freeze. Capture the first transition into the black frame with bounded texture/blend/matrix/draw state before changing graphics semantics. |

## Super Hexagon picker-placement defect

The user correctly reported that only thirteen requested titles were pinned to
the front. The valid package is
`com.distractionware.superhexagon_1.0.0_ios-4.0_hexagon_63300f7e.ipa`; its
`CFBundleDisplayName` is empty. A second package named `Super_Hexagon_1.0.ipa`
is an invalid ZIP and is skipped. Dev 2.27's title-based pin matched neither the
launchable blank-name entry nor its bundle identifier, so the valid title stayed
in its normal picker position. Future requested-title pinning should prefer the
stable bundle identifier and fall back to normalized filename/internal name,
not require a non-empty display name.

## Recommended next engineering batch

Safe direct contracts can be combined without obscuring attribution:

1. `NSURL URLByAppendingPathComponent:` (BikeRacePro, Legends of Ooo).
2. `NSFileManager subpathsOfDirectoryAtPath:error:` (MX Mayhem).
3. Real CommonCrypto SHA-224 (ParaPanic HD).
4. Dynamic `printf` width/precision (Dismount).
5. `CGContextBeginPath` if the existing user-owned Core Graphics edit can be
   preserved without conflict (The Sims 3).

The remaining titles need bounded causal experiments rather than speculative
compatibility behavior: all-thread startup capture for Fieldrunners 2; first
black-frame graphics capture for Wire Way; and nil/value provenance for
Fieldrunners, NinJump, Scribblenauts and Super Hexagon. `addOperationWithBlock:`
and Darwin notification-center support are legitimate general work, but should
be included only after their ownership/delivery model is implemented and tested.

No source implementation, push or IPA build is authorized by this device-result
review alone.
