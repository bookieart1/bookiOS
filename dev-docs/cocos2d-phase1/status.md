# Cocos2d project resume point

**Dev 2.26 implementation in progress (2026-11-04):** all thirteen Dev 2.25
titles remain in one interpretable batch. Eight later exact contracts receive
general corrections (gradient creation, progress-view state, visible table-cell
lookup, mutable-data/plist class-cluster behavior, standard-descriptor `fcntl`,
XML-parser cancellation, and failable image decoding). Cordy, Guerrilla Bob and
Muffin Knight receive bounded diagnostics at their precise unsafe-to-guess
boundaries; Current validates Cordy's shared URL-loading surface and Spoing is
the general control. No HTTP success, network service, image format, touch
bypass, or invalid object is fabricated. See
[the exact Dev 2.26 scope](dev-2.26-later-contracts-and-causal-probes.md).

**Dev 2.25 physical result (2026-11-04):** both supplied logs embed the exact
delivered implementation `18719b57629070ee53c8775402f1851eeb346d6c` and IPA
workflow `41808379942`. All ten changed contracts crossed their prior first
boundary, and Spoing remains a working control, but no affected title is yet
promoted to fully compatible. The new exact boundaries are Core Graphics
gradient creation (Cannon Cadets), `UIProgressView` (Clowning Around), image
format/path identification before `UIImage` decode (Kick the Buddy), table
cell lookup (Labyrinth 2), mutable-data property-list output (Mini Militia),
vacant-descriptor `F_GETFL` handling (Sonic Dash), mutable-data
`initWithBytes:length:` dispatch (Trainyard), and `NSXMLParser abortParsing`
(ZombieSmash). Cordy again reaches the later local-file response typing problem
(`statusCode` sent to `NSURLResponse`). Current returned cleanly after 3,634
loops and 416 presentations, but no visual/gameplay observation was supplied,
so its functional status remains unknown.

Guerrilla Bob and Muffin Knight are now causal investigations rather than
generic visual symptoms. Guerrilla Bob presents 1,507 frames, but a visible,
interactive, full-screen `UIView` at alpha 0.4 receives every recorded touch;
the next safe step is bounded creation/attachment/dismissal provenance for that
overlay, not globally disabling hit testing. Muffin Knight stops at 82 main
loops and four presentations, then remains in a stable guest loop around
`0x15b3b50..0x15b3bdc` after the offline connection failure; queued Home events
cannot be handled. Its next step is offline symbolization plus bounded socket
retry/return history, not fabricated network success. See
[the complete per-title result](dev-2.25-exact-system-contracts.md).

**Dev 2.24 physical result (2026-11-04):** the complete requested device pass
tested implementation `fb810996` from IPA workflow `41796486938`. The user
reports Addicus and Kiko working in the tested scope; Dirt and Tiki Totems
Premium remain healthy controls. Super Stick Golf now reaches its menus and
fails only when loading actual gameplay. The remaining compatibility attempts
crossed their intended first boundaries but exposed later blockers: AbstractWar
now reaches a later null-page read; Continuity 2 sends `boolValue` to an
`RTGRoomBorder`; Flow Free consumes an object whose `isa` is nil; Infinity
Field reaches an NSString assertion; Mini Militia now requires
`NSMutableSet member:`; and Stick Stunt Biker reaches a non-ASCII NSString
conversion assertion. Super Turbo Action Pig and Wispin still report sticky GL
error `0x506`; every sampled framebuffer completeness probe was complete, so
the incomplete-current-FBO hypothesis is weakened rather than confirmed. No
title is claimed fully compatible beyond the user's stated test scope.

**Dev 2.24 update (2026-11-04):** expanded implementation built and downloaded at `fb810996`; all eight new tests pass on Windows and macOS. Both full suites are 292/293 because of the unchanged XML plist fixture failure. Eleven affected games and two controls await physical testing. See [implementation and predictions](dev-2.24-implementation.md) and [exact validation](dev-2.24-validation.json). Baseline results below are preserved; no new device pass is claimed.

## Preserved phase-1 checkpoint

The remainder below records the completed investigation before Dev 2.24. Statements that implementation had not begun refer to that earlier checkpoint; the current implementation and validation are recorded above and in the linked Dev 2.24 record.

## Exact baseline and scope

- Active checkout: `repo/`, branch `ci/unsigned-ipa-baseline`, HEAD `51a633428ec2b461761434036b6968d729c15594`; origin `https://github.com/bookieart1/iDared32bit.git`.
- All three original logs embed `51a6334` and workflow `41778866142`. The existing Dev 2.23 artifact-verification record agrees with that commit/run and records unsigned arm64 development identity. Package SHA-256: `bf0d0923674d28dedb393e2e1052f69a920071b575d6b0ac02833a08fc83f0a6`. No new build verification or hosted tests were run in this phase.
- 37 launch records, 37 matching IPAs, 37 successful ZIP CRC checks. Two encrypted pre-emulation failures; all 35 eligible launches enable the recorder. Fresh Clowning Around and newly supplied ZombieSmash are included. Zombie Shock was a naming error corrected by the user.
- Input IPA directory: `C:\Users\booki\Documents\Codex\iDared-IPAs\cocos2d-iphone IPAs`. No original IPA was edited, executed, redistributed or acquired. Metadata was read from each archive in memory. No full decompilation was attempted.
- The input logs are the two files in attachment `50FCDE2D-AF2B-4773-8BFB-94A03DADF83E` and Clowning Around's `127582EE-3D3B-4F73-8609-B5A44C8125B4/1-Log.txt`. Exact absolute paths, SHA-256 hashes and original line spans are in `log-index.json`.
- Existing user changes in `Cargo.lock`, `src/frameworks/core_graphics/cg_context.rs`, and untracked `artifacts/` remain untouched. In particular, future alpha-context work must reconcile the pre-existing graphics edit rather than overwrite it.

## Architecture and available evidence

This fork retains touchHLE's Rust high-level Apple-framework implementations, guest Objective-C runtime and guest ABI. Its mGBA-derived ARM interpreter replaces the upstream Dynarmic CPU path, and SDL-backed OpenAL replaces OpenAL Soft. Cocos scene/physics/game code is embedded in the guest executables; identifying cocos does not imply the emulator contains a single engine switch to fix it.

Existing tools already provide bounded API/Objective-C history, argument/return context, image-relative locations, object allocation/lifetime generations, raw allocations, collection mutations, GL state milestones, progress reports and panic/Home dumps. The audit used these instead of adding instrumentation. The first-12 progress-message cap is not proof that the bounded crash recorder stops. Existing GDB/debugger and guest TestApp facilities remain available for later controlled reproductions; no debugger or game was run in this phase.

Source and surrounding consumers were checked for the principal boundaries, including Foundation values, Core Foundation streams/time, Core Graphics bitmaps, UIKit table/alert/controller behavior, GL presentation, arrays and Objective-C dispatch. Commit history distinguishes the Dev 2.22 KVC token-parser change from the still-missing object setter path, and Dev 2.17 epoch exports from the still-incorrect negative Gregorian conversion. Current imported/read-stream gaps persist despite later stream work. These findings reuse existing implementations rather than proposing duplicate frameworks.

## Durable files

| File | Purpose |
|---|---|
| [manifest.md](manifest.md) | Human-readable 37-game status and exact filename/bundle mapping |
| [manifest.json](manifest.json) | Complete metadata, dependencies, resources, confidence, original log references, selected role and next action for every game |
| [cases.md](cases.md) | Individual evidence, causal interpretation, predictions and competing explanations |
| [failure-clusters.md](failure-clusters.md) | Whole-corpus counts, supported relationships, misleading similarities and historical corrections |
| [plan.md](plan.md) | Prioritized milestones, canaries/controls, falsifiers, validation and primary sources |
| [validation.json](validation.json) | Machine-checked completeness and counts |
| `ipa-index.json.gz` | Compressed inspection metadata for all slices; no embedded executable/resource payloads |
| `log-index.json`, `logs/` | Hash-indexed original logs and per-launch text extracts |
| `inspect_corpus.py`, `curate_audit.py` | Repeatable standard-library inspection and deterministic generation of manifest/cases/counts |

The original logs are byte-preserved; extracts normalize line endings. Some raw segments extend through the following picker warnings, so only the active launch before `No app specified` is used for runtime conclusions. The aggregate inspector's broad marker matches are candidate evidence only; curated engine attribution requires meaningful classes/source paths. Original text encoding artifacts are preserved, not treated as semantic differences.

## Decisions and next action

Working controls are Dirt, Moonlights and Tiki Totems Premium. Do not promote the five partial titles or games with a large presentation count to working. Do not classify the two encrypted packages as emulator runtime failures.

Recommended next authorized milestone: four independent Foundation value corrections for Continuity 2, Infinity Field, Super Stick Golf and Stick Stunt Biker, with focused host/guest fixtures and Tiki Totems Premium as the one physical healthy control. Follow with the properly specified failed-stream lifecycle (Addicus/Kiko) and alpha-only bitmap path (AbstractWar). The reason for this order and all remaining games' next discriminating actions are preserved in the linked records.

Do not build or push until the user requests the implementation phase. Keep the user's paused Dev 2.23 non-cocos batch separate; its ledger evidence is not discarded. No new physical test is required merely to restate these same failures. Native references are still needed for ambiguous mutation/archive/ad contracts, and corrected runnable packages for League of Evil/StickWars. Exact cocos versions and causes of several invalid-value/render/input faults remain unresolved.
