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
Physical-device status remains pending; no affected title is promoted to
working by compilation or package verification.
