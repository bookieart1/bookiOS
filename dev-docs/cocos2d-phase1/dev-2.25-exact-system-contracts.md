# Dev 2.25 — exact missing system contracts

Prepared 2026-11-04 from the complete Dev 2.24 phone result and the broader
compatibility ledger. This is one batched build of independent historical
contracts. None of the implementations branches on an app identifier, weakens
invalid-memory handling, or turns an unavailable service into success.

## Included titles and exact change

| Title(s) | Evidence before this build | Dev 2.25 implementation | Prediction and boundary |
|---|---|---|---|
| Mini Militia | Dev 2.24 crossed iAd startup and next failed on `NSMutableSet member:` | Implement `member:` on both concrete set classes, returning the identity of the stored equal object rather than the probe object | The missing-selector failure disappears. A later failure is advancement, not proof that gameplay works. |
| Cannon Cadets | Corpus log stops at missing `+[UIView setAnimationBeginsFromCurrentState:]` | Store the flag in the active animation transaction and use the current presentation value for supported implicit CALayer properties | The exact selector is accepted and replacement animations begin at their visible value. The compositor is not otherwise redesigned. |
| Clowning Around | Corpus log stops at missing `-[UIAlertView dismissWithClickedButtonIndex:animated:]` | Add bounded, idempotent programmatic dismissal with asynchronous delegate ordering and presentation ownership | The alert can dismiss without a selector panic. The native host message box remains synchronous while it is onscreen. |
| Cordy; Current | Dev 2.23 stops at `_CFReadStreamScheduleWithRunLoop` | Validate the owned offline stream scheduling/error lifecycle implemented in Dev 2.24; no duplicate stream model was added | Both should receive a real scheduled POSIX `ENETUNREACH` path instead of the old missing export. Network success is not fabricated. |
| Guerrilla Bob; Muffin Knight | Dev 2.23 reaches `strerror(51)` after a failed UDP path | Add stable Darwin strings for `ENETUNREACH`, `ECONNREFUSED`, and `EHOSTUNREACH` | The missing error-text assertion disappears; the genuine network error remains. |
| Kick the Buddy | Dev 2.23 reaches missing `_glGetTexParameteriv` | Export the GLES 1.1 query through native and GL2-backed contexts with a one-scalar guest write | Texture state can be queried. No texture value or successful rendering is fabricated. |
| Labyrinth 2 | Dev 2.23 reaches `setShouldIndentWhileEditing:` | Add the historical UITableViewCell property, defaulting to `YES` with getter/setter state | The exact cell-configuration selector succeeds. Table layout beyond existing support is unchanged. |
| Sonic Dash | Dev 2.23 passes a nonnegative closed descriptor to `fcntl(F_GETFL)` and reaches an unwrap | Treat both out-of-range and vacant descriptor slots as `EBADF` | The call returns `-1`/`EBADF` rather than panicking. Valid descriptors are unchanged. |
| Trainyard | Corpus log stops at `_CFRunLoopSourceCreate` | Add version-0 source lifecycle: retained context info, ordered scheduling, membership, signal/delivery, cancellation, invalidation and teardown | A source-0 callback can be scheduled and delivered. Observers, source-1/port signaling, and full mode separation are not claimed. |
| ZombieSmash | Corpus log stops at `_CC_SHA1_Init` | Implement the 96-byte 32-bit CommonCrypto SHA-1 context entirely in guest memory, including Init/Update/Final and copyable state | Incremental SHA-1 produces the standard digest and copied contexts work. This does not imply the game's next boundary is compatible. |

## Focused validation added

- Concrete immutable/mutable set stored-member identity.
- Transaction scoping for the begins-from-current-state flag.
- UIAlertView programmatic dismissal scheduling and idempotence.
- UITableViewCell editing-indent default and round trip.
- Darwin network error text and stable storage.
- Nonnegative vacant `fcntl` descriptor recognition.
- CFRunLoop source membership, signaling, invalidation and order preservation.
- Incremental SHA-1 ABI size, chunking, copied context and known `abc` digest.

The existing Dev 2.24 offline-stream tests remain the contract validation for
Cordy and Current. OpenGL is additionally validated by hosted compilation on
both GLES backends; the real-device Kick the Buddy run is the required guest
driver test.

## Physical-device scope

Alphabetical affected tests: Cannon Cadets, Clowning Around, Cordy, Current,
Guerrilla Bob, Kick the Buddy, Labyrinth 2, Mini Militia, Muffin Knight, Sonic
Dash, Trainyard and ZombieSmash. Spoing is the one general control. These are
prioritized at the front of the picker and the header is labelled Dev 2.25.

For every affected title, stop at its first reproducible failure and return the
combined log plus the furthest visible screen. A crossed boundary is recorded
as a blocker removal; working/menu/gameplay/save status still requires the
corresponding physical observation.

## Deliberate exclusions

This build does not mix in invalid-object origin work, rendering corruption,
input/orientation investigation, encrypted packages, or speculative network
success. Those remain separate ranked clusters so this device result can be
attributed to the exact contracts above.

## Hosted validation and delivered artifact

Final implementation commit: `18719b57629070ee53c8775402f1851eeb346d6c`.
The first hosted passes exposed only compilation/test-fixture defects in the
new tests; those were corrected before delivery. In final test workflow
`41808379950`, Windows and macOS each ran 302 library tests: all 301 current
tests, including every Dev 2.25 contract test, passed. The sole failure on both
hosts is the unchanged XML property-list serialization fixture at
`ns_property_list_serialization::xml_serialization_emits_xml_and_reports_no_error`.
The macOS native Foundation copy/defaults reference passed. Formatting and the
post-library integration/sanity steps were skipped by workflow configuration
or because the known library fixture leaves the job failed; Android failed in
its separate setup/lint lane, and the dedicated native-reference-only job was
skipped. These are reported separately rather than treating the overall red
badge as a Dev 2.25 contract failure.

Unsigned IPA workflow `41808379942` completed successfully. The downloaded
artifact is arm64, contains no code-signature load command or signature
directory, contains no provisioning profile, uses
`com.bookieart1.idared32bit.dev`, embeds the final commit and workflow run, and
retains the universal flight-recorder default. IPA SHA-256:
`e07f8f4631b217533f3d4b27abea3e4e7ae8f8555bacc92584c337111e40bad8`.
## Physical-device result

The two returned logs both embed implementation `18719b5` and workflow
`41808379942`, so this result belongs to the verified artifact above. Every
former first blocker addressed by Dev 2.25 is absent. That establishes blocker
removal, not complete compatibility.

Exact evidence is preserved outside the source checkout at
`outputs/device-logs/iDared-Dev-2.25-exact-contracts-part1-2026-11-04.txt`
(SHA-256 `a9bce207e2c1d8967ebb28d64b6bbde841e201807111c5485473a446212269cc`)
and `outputs/device-logs/iDared-Dev-2.25-exact-contracts-part2-2026-11-04.txt`
(SHA-256 `8893021ffb7c825005d47992bc79a72bf077e4428b81be418a0ef623736a3568`).

| Title | Dev 2.25 result | Classification and next discriminating action |
|---|---|---|
| Cannon Cadets | Reaches `_CGGradientCreateWithColorComponents`. | Later exact Core Graphics contract; implement historical gradient creation and focused component/location tests. |
| Clowning Around | Presents seven frames, then requests `UIProgressView`. | Later exact UIKit class; implement bounded historical progress/style/value state rather than a title stub. |
| Cordy | Runs 12.9 seconds, 732 loops and 283 presentations, then sends `statusCode` to a local-file `NSURLResponse`. | Offline stream scheduling passed; correct response typing/failure delivery remains the known next contract. |
| Current | Returns cleanly after 3,634 loops and 416 presentations. | No terminal emulator error, but no visual/gameplay report; obtain a short functional observation before promotion. |
| Guerrilla Bob | Runs 31.9 seconds and presents 1,507 frames; user sees a dark overlay and cannot use the menu. | Recorder confirms a full-screen interactive `UIView` at alpha 0.4 receives every touch. Capture its creation, controller ownership, attachment and expected dismissal. Do not disable hit testing globally. |
| Kick the Buddy | Crosses texture-query startup, then `UIImage` rejects an unknown byte format after missing `Dust_03b.png` and `particleTexture.png`. | Identify the exact resource path and magic bytes at the failing `initWithData:` before implementing any image-format support. |
| Labyrinth 2 | Reaches `UITableView cellForRowAtIndexPath:` after 145 loops and 74 presentations. | Later exact UIKit lookup contract with focused table lifecycle tests. |
| Mini Militia | Reaches property-list serialization with `NSMutableData` after 62 loops and 31 presentations. | Later exact Foundation output/class-cluster contract; retain strict property-list validation. |
| Muffin Knight | Black screen; emulator Home cannot exit. Four frames are presented, then main-loop count freezes at 82 while one runnable guest thread repeatedly executes around `0x15b3b50..0x15b3bdc` after an offline connection failure. | Deterministic guest loop, not a crash. Symbolize that image-relative region and add bounded adjacent socket-return/retry history; do not fake connectivity. The unimplemented intro movie remains a secondary visual limitation. |
| Sonic Dash | Reaches a later `fcntl(F_GETFL)` vacant-descriptor unwrap. | Dev 2.25 `F_GETFD` fix passed; apply the same historical `EBADF` discipline to all descriptor-dependent `fcntl` branches with tests. |
| Trainyard | Allocates `NSMutableData`, then hits a typed-dispatch mismatch for `initWithBytes:length:`. | Source lifecycle passed; repair the real mutable-data class-cluster initializer/dispatch contract, not typed-message enforcement. |
| ZombieSmash | Reaches `NSXMLParser abortParsing`. | SHA-1 passed; implement parser cancellation state and callback termination semantics. |
| Spoing | User reports normal operation; clean Home return after 1,176 loops and 480 presentations. | Relevant healthy control passed. |

No title is marked fully fixed solely because the previous panic disappeared.
The result creates eight direct contract candidates, preserves Cordy's known
response issue, leaves Current awaiting visual confirmation, and converts the
two visual stalls into specific bounded investigations.
