# Cocos2d compatibility project: implementation roadmap

**Dev 2.26 result and next decision (2026-11-04):** prioritize four direct
general contracts: linear-gradient drawing for Cannon Cadets, mutable-set copy
factory semantics for Kick the Buddy, invocation-backed scheduled timers for
Labyrinth 2, and correction of the mutable-data method ABI/registration exposed
by Trainyard. Research historical zero-row-byte bitmap semantics before adding
ZombieSmash. Add only bounded failed-receiver provenance for Clowning Around
and guest touch-callback/coordinate evidence for Current. Symbolize Muffin
Knight offline before another device pass; treat Sonic Dash as a separate,
high-cost OpenGL ES 2/fallback investigation. Mini Militia is dropped by user
request. Full evidence, predictions and exclusions are in
[the Dev 2.26 device result](dev-2.26-device-result.md).

**Dev 2.26 active milestone (2026-11-04):** implement the eight evidence-backed
later contracts exposed by Dev 2.25 and pair them with three bounded causal
probes. Cordy remains standards-correctly non-HTTP for local files; Guerrilla
Bob keeps normal hit testing; Muffin Knight stays honestly offline. Current and
Spoing are the smallest sufficient controls. The batch is intentionally broad
but attribution remains per contract. Full predictions and retirement criteria
are in [the Dev 2.26 record](dev-2.26-later-contracts-and-causal-probes.md).

**Dev 2.25 completed milestone (2026-11-04):** the exact delivered build was
physically tested. Every changed contract crossed its former boundary and
Spoing passed, but the pass exposed later work rather than a newly verified
fully compatible title. The highest-confidence next batch is the small general
contracts now named exactly: `CGGradientCreateWithColorComponents`,
`UIProgressView`, `UITableView cellForRowAtIndexPath:`, mutable-data property
list/output initialization, vacant-descriptor `F_GETFL`, and
`NSXMLParser abortParsing`. Before including Kick the Buddy, identify the exact
resource/header rejected by `UIImage`; do not add a decoder based only on
"unknown image type." Cordy's HTTP-response typing remains a separate known
contract, not a Dev 2.25 regression. Current needs a short visual/gameplay
observation before it is treated as either fixed or still blocked.

Two bounded investigations should remain separate from that contract batch:
record the creation/attachment/dismissal producer of Guerrilla Bob's
full-screen alpha-0.4 interactive overlay, and symbolize Muffin Knight's fixed
guest loop around `0x15b3b50..0x15b3bdc` while recording only the adjacent
socket retry results. Neither result supports a global input change, network
success fabrication, or watchdog-style escape.

**Dev 2.24 update (2026-11-04):** expanded implementation built and downloaded at `fb810996`; all eight new tests pass on Windows and macOS. Both full suites are 292/293 because of the unchanged XML plist fixture failure. Eleven affected games and two controls await physical testing. See [implementation and predictions](dev-2.24-implementation.md) and [exact validation](dev-2.24-validation.json). Baseline results below are preserved; no new device pass is claimed.

## Preserved phase-1 proposal

Phase 1 completed 2026-11-04. The ranking and proposal below are the historical audit baseline. The Dev 2.24 update above records which proposals were subsequently implemented. The objective is correct shared Apple behavior, not replacing cocos2d or suppressing strict failures.

## Ranked opportunities

The rank balances evidence, generality, regression exposure and implementation cost. Counts refer to present corpus canaries, not promised newly playable games. Each row specifies a concrete discriminator; per-game details are in [cases.md](cases.md).

| Rank / work | Primary and secondary evidence | Expected value; confidence | Cost/risk | Prediction and disproof condition | Minimal control |
|---|---|---|---|---|---|
| 1. Foundation value correctness | Continuity 2; Infinity Field; Super Stick Golf; Stick Stunt Biker | Four independent, general value-representation errors; high evidence from source and recorder | Low–medium; KVC dispatch and archive ownership require guest tests | Cross their exact KVC/box/Boolean/date boundary. Same boundary with the same valid input falsifies the implementation; a later blocker is advancement, not compatibility | Tiki Totems Premium for scene/transition/save smoke test; focused API fixtures are the main regressions |
| 2. Honest offline stream lifecycle | Addicus; Kiko (0.99.5-rc1); later Cordy/Current outside corpus | Strong identical analytics producer in two games, broader shared CF contract | Medium–high; callback ownership, reentrancy, run-loop scheduling and retry behavior | Both leave analytics initialization through a real error path without crash/retry storm. Continued creation/scheduling failure disproves lifecycle completeness | Moonlights for older loop/input/audio; synthetic create/open/fail/close fixtures |
| 3. Alpha-only Core Graphics bitmaps | AbstractWar 2.0; second failing corpus member not established | Direct cocos CCTexture2D path; high-confidence legal-input defect with plausible broad texture/font benefit | Medium; pixel formats affect image drawing and queries | Alpha texture conversion completes with correct mask bytes. Correct object creation but wrong alpha/pixels fails the contract | Dirt for existing color textures; dedicated 1-channel/RGBA comparison fixtures |
| 4. Unavailable iAd view lifecycle | Flow Free; Mini Militia (0.99.5-beta3); Donguri historical candidate | Two exact allocation blockers across different codebases | Medium; union of actual selectors, layout, delegates, callback ordering | Both pass banner creation and handle unavailable ads. Missing subsequent lifecycle/callback state means the partial contract is inadequate | Tiki Totems Premium for UIKit/main-loop recovery; no need for every working game |
| 5. Three small independent UI/crypto surfaces | ZombieSmash SHA-1; Cannon Cadets animation flag; Clowning Around dismissal | Clear boundaries; real digest and proper alert lifecycle are generic | Low–medium SHA; medium animation/alert reentrancy | Pass exact API with correct digest/state/order. Mere absence of panic with fake success is failure | Tiki Totems Premium for UI transitions; standard digest and delegate fixtures |
| 6. Table-controller creation | iStunt Reloaded Pro | Wrong root-view producer supported by explicit fallback evidence | Medium; default versus nib/subclass lifecycle | A genuine UITableView is returned and level table works. Adding a method to plain UIView does not satisfy the test | Synthetic UIViewController/UITableViewController nib+lazy-load fixtures; Tiki brief transition |
| 7. More complex Foundation contracts | Junk Jack Retro; Doodle Grub; Lumi | Missing enumeration/coding/aggregate layout confirmed, but not one bug | Medium–high ABI/archive risk | Correct block arguments/stop flag; historical color round-trip; signature layout and invocation both valid | Tiki as short scene control; guest ABI tests mandatory |
| 8. First GL-error attribution | Wispin; Super Turbo Action Pig (older unprefixed engine) | High-value cross-generation diagnostic, shared origin not yet established | Low if existing history suffices; medium if intercepting GL error state | Identify first producing operation with the bound context/FBO/attachments. Different producers split the cluster. Never consume/clear guest errors invisibly | Dirt graphics/transition control |
| 9. Orientation and interaction separation | PZS; Tap the Frog; FastBall 2 as a separate native-resize case; Hook Champ field case | High generality if coordinate/lifecycle error confirmed; presently mixed causes | Medium/high for global transform edits; low for bounded observations | One known tap maps to the drawn control and host resize succeeds. Delivery already reaching the correct guest control refutes a coordinate-only explanation | Dirt rotation/movement control |
| 10. Producer and historical-contract investigations | Blockoban, Mega Jump, Robokill, Silent Swords, Thief Lupin, Legendary Wars, Trenches | Broad benefit from finding true producer; current final symptoms do not justify a common patch | Bounded offline analysis first; native reference where indicated | Exact error-out value, profile creation, address generation, or legal mutation behavior distinguishes the alternatives in cases.md | Only after a subsystem is identified; not all controls now |
| 11. Other specific missing boundaries | Contre Jour HD dlopen; Trainyard source0 lifecycle | Potentially broad but request/path/context must be decoded first | Medium | Correct absent-library error or correct source lifecycle; dummy handles/timers disallowed | Moonlights loop control when relevant |
| External | League of Evil; StickWars | Cannot exercise these encrypted copies | No compatibility change justified | Matching legitimate runnable packages required | None |

Love Catcher's white/audio presentation needs a screenshot correlated with existing GL/guest sleep state before a rendering patch. It is explicitly retained in the investigation queue, not silently omitted or labelled fixed.

## Recommended first implementation milestone

Implement the **four Foundation value corrections** in rank 1 as a small batch. They are separate safe contracts sharing a testable value-handling surface, not one speculative four-game patch. They offer better near-term attribution and lower callback/graphics risk than starting with a large stream or renderer rewrite. Keep each change separately reviewable even if they share one eventual IPA.

1. **Continuity 2 — KVC object identity.** Inspect the destination setter type before unboxing NSValue/NSNumber. An `@` setter receives the original object. Exercise object, scalar, nil-object, signed/unsigned, offset-bearing encodings and existing scalar behavior. Do not widen support for unsupported aggregate invocation accidentally.
2. **Infinity Field — unsigned-byte NSNumber.** Reuse existing numeric representation and conversions. Test 0, 127, 128 and 255 through guest message ABI; verify sign-extension does not occur.
3. **Super Stick Golf — primitive Boolean keyed-archive object.** Use existing Boolean/numeric identity semantics. Test nested objects, repeated UIDs, true/false and retained graph teardown. Preserve the user's existing saves; do not replace them with a fresh-state-only test.
4. **Stick Stunt Biker — signed date conversion.** Handle dates before Apple's 2001 epoch without unsigned Duration conversion. Check 1970, 2000, 2001, negative fractional intervals and supported pre-Unix dates. Do not clamp invalid values or silently change the time zone.

Before that future implementation, inspect existing guest fixtures and reuse them. Required validation: focused Rust/API fixtures and ARM guest ABI tests through hosted workflows, then iOS compilation/package verification and the following physical test. These are distinct proof categories. Dev 2.23's recorded hosted test failures (including the plist XML test) must remain visible; a green packaging job is not a clean test matrix.

Proposed physical test, alphabetical: **Continuity 2** launch/start once; **Infinity Field** launch/start once; **Stick Stunt Biker** launch through its initial news/menu sequence and start once; **Super Stick Golf** open existing progress and try a hole; **Tiki Totems Premium** brief gameplay, one transition and Home/reopen as the single healthy control. Stop at first reproducible blocker and return the full log plus visual observations. No retest is requested now and no successful gameplay is promised from these corrections.

## How later rounds avoid the old one-error cycle

- Before implementing an absent API, inspect the caller's immediate next operations and the union of selectors used by its secondary game. A missing export by itself is not a complete stream/ad/alert contract.
- Reuse the universal recorder and existing object-generation, return, GL and allocation histories. Do offline symbolization against the hashed executable first. Do not request a new identical crash merely to collect the same tail.
- Instrument only an unresolved discriminator: one error-out slot, one object generation, one first GL error, one orientation transition or one first-responder attempt. Bound both storage and dump frequency; keep errors and return values unchanged.
- Do not group immutable-dictionary mutation, collection shrinking, empty profiles, and bad receivers under one generic Foundation fix. Likewise continuing EAGL returns do not prove meaningful rendering or correct input.
- After each device pass update the existing ledger and add a dated result record; preserve this baseline and raw files. Change hypotheses when predictions fail, not merely log volume.
- Use only hosted compilation for future authorized builds; no local build/preflight helpers. Do not submit/requeue a workflow in this investigation phase.

## Historical contracts and primary sources

Reviewed 2026-11-04. These sources support the narrow contracts below; they do not validate full emulation or every historical edge case.

- Apple's [KVC non-object values guide](https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/KeyValueCoding/DataTypes.html) distinguishes object setters from scalar/structure unboxing and documents unsigned-byte boxing. Apply the object/nonobject rule; do not import modern host BOOL ABI assumptions into ARMv6/v7 guests.
- Apple's [Quartz graphics-context format table](https://developer.apple.com/library/archive/documentation/GraphicsImaging/Conceptual/drawingwithquartz2d/dq_context/dq_context.html) includes 8-bit alpha-only bitmaps with NULL color space on iOS. This supports AbstractWar's request; every image/query consumer still needs tests.
- Apple's archived [CFNetwork stream guide](https://developer.apple.com/library/archive/documentation/Networking/Conceptual/CFNetwork/CFStreamTasks/CFStreamTasks.html) separates creation, opening, event scheduling, error reporting and teardown. Review [CFStream source](https://github.com/apple-oss-distributions/CF/blob/main/CFStream.c) with version-appropriate headers before designing socket failure delivery; a nil scheduling stub is not that model.
- [Upstream keyed-unarchiver source](https://github.com/touchHLE/touchHLE/blob/trunk/src/frameworks/foundation/ns_keyed_unarchiver.rs) was consulted as a comparison; the primitive match shown still lacks Boolean support. This is not a full current-upstream audit or proof no branch/PR addresses it. Local `upstream/trunk` is `ca1de89c` dated 2026-10-13 and may be stale.
- Historical iAd callback order, UIColor archive encoding and mutation during `makeObjectsPerformSelector:` remain explicit research/reference-test dependencies. No modern documentation assumption is substituted for those missing answers.
