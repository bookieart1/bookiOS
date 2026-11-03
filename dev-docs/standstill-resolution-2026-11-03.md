# Standstill investigation and causal diagnostics — 2026-11-03

## Scope and evidence

This review covers all 54 entries in the November 3 near-standstill inventory:
34 runtime/provenance cases, two reported native terminations, six historical
semantics/interaction cases, one resource question, and 11 encrypted packages.
These categories are investigative dispositions, not 54 confirmed emulator bugs.
No game is declared fixed by this investigation or the diagnostic implementation.

Source baseline: `ci/unsigned-ipa-baseline`, commit
`f725b961ba1917ea926a42b74cd5bf7df8cf0f3b`. The two latest device logs embed
`f725b96`, workflow `41581222689` (Dev 2.10). Build labels are retained alongside
embedded commits; they are not used as proof of source identity. The accompanying
`standstill-cases-2026-11-03.csv` has all 54 individual decisions, source line
ranges, exact recorded boundary, preceding events, counterevidence, and the next
discriminating action. Older observations remain dated evidence, not retests on
current HEAD. In particular, many titles have not been observed since 1.71/1.93.

Workspace evidence is preserved in `outputs/`: the compatibility ledger, dated
November 2 library workbook, two narrative audits and associated CSVs, universal
1.71 survey, later archived device logs, and this review's evidence-index JSON,
case CSV and shared-instruction proof JSON. The index excludes later launches
that failed solely because the host's guest-memory reservation was exhausted;
those are not new game-specific compatibility results. Original logs are intact.
The old 1.42 page-seven duplicate remains excluded from independent counts.

No native `.ips` report was found in the accessible project evidence. Visual
observations are explicitly user reports: frame presentations cannot prove correct
pixels, responsive gameplay, sound or saved progress. The November 2 workbook is
left intact as a dated snapshot. Its stale entries are superseded by the case CSV.

## Why progress stalled

The fork is a high-level reimplementation of historical iOS frameworks around a
guest ARM CPU, guest memory/allocator, Objective-C host-object table, guest/native
call bridge, framework state and cooperative scheduling. iDared uses the mGBA-
derived interpreter and SDL-backed OpenAL path instead of upstream's usual CPU/
audio configuration. Native iOS presentation, keyboard, lifecycle and audio also
remain capable of terminating the host outside the guest panic path.

Missing selectors were comparatively easy to turn into explicit contracts. The
remaining failures increasingly consume values produced much earlier, exercise
restored state, or occur outside the guest exception handler. Repeating the last
visible boundary as a hypothesis has reached its limit. A nonzero API return can
precede an unrelated null global load; a valid pointer can be freed before its
last use; 300 presentations can all contain corrupted pixels.

Existing diagnostics already provide image-relative callers, raw instruction
windows, register/stack context, API entry/return histories, resource failures,
run-loop/presentation counters and limited lifetime records. GDB and linking-info
dumps also exist. They should be used before designing more tracing. However,
the existing lifetime watch is seeded by non-retaining CFDictionary callbacks
and limited to 16 addresses. It does not explain arbitrary object origins. The
ordinary history may filter allocator/retain traffic; the final API is therefore
not necessarily the producer of a bad value.

There is already a frozen slow-presentation history, including slow-from-start,
and current HEAD also retains the worst single present gap. Do not add another
generic slow-frame recorder. ZA's 2.07 capture freezes at 3.740 seconds (a
1.247-second early gap), before its later intermittent presentation collapse.
Between 35.043 and 70.076 seconds it advances 2,055 main loops but only 30 presents
(about 0.86 presents/s). The frozen early window mostly contains TLS operations
and host UIKit calls around present; it does not explain that later period.
The useful next change would be a scene marker/rearm of existing bounded capture
and timing attribution, not another log buffer or an unqualified FPS average.

Upstream HEAD was verified remotely as
`ca1de89c578e79746f33c1332e28efd106c334ba`. Relevant upstream object, array and
NSThread files were compared with the fork. Upstream also has the thread-start
ownership pattern discussed below; being upstream behavior is not proof of its
historical correctness. No unverified wholesale upstream merge is proposed.

## Consequential findings and competing theories

| Finding | Evidence and implication | What is not established |
|---|---|---|
| Crash Drive, Jack Lumber, Slingshot, Stick Skater, Wind-up Knight form a real instruction-level cluster | Their 96-byte fault windows match after masking ARM BL displacements only. `objectAtIndex:0` returns a valid NSNumber; the later PC-relative indirect load produces zero. | The five imported/global symbols have not yet been identified. Same instructions need not mean the same missing symbol. |
| Peggle is an allocator/object-table disagreement | Dev 2.10: 414 loops, 137 presentations, then an allocation returns an address still registered as an Objective-C object during CALayer display. | Zone/VM free, missing object-table removal, allocator overlap and corruption remain competing causes. Ordinary free() already removes registered objects through an existing accommodation; zone-free does not. The actual path is unproved. |
| Topple 2 has a concrete reuse clue | Dev 1.90: NSThread helper receives an address whose isa is `0x4d555458`, exactly this fork's pthread mutex `MUTX` marker. | Premature thread-object destruction is plausible, but a wrong pointer or raw overwrite could produce the same marker. `NSThread.start` lacks an explicit self-retain; this is a candidate, not a patch authorization by itself. |
| Anomaly HD, SUPERFALL and Tiki Totems 2 justify lifetime investigation | Search-path string teardown/extra release, a stale address after prolonged play, and an invalid retain respectively. | They are not proven to share an owner, producer or defective retain contract. |
| Fragger's chronology contradicts a simple 1.90 regression claim | Gameplay followed by later launch/host-exit trouble was already reported on 1.89. Latest identifiable archived launch is 1.91, with later user reports of continuing host closure. | A code regression remains possible, but saved-state/launch-order effects must be separated. Log truncation does not identify the native fault. |
| DeepBlue's old failure is no longer its latest endpoint | Later Dev 1.90 record exits via Home after 1.214 seconds, 67 loops and no EAGL returns. | This proves neither gameplay nor a persistent black screen. Move to observation-needed; retain the old null as history. |
| 8-bit Ninja's resource diagnosis was overstated | `sounds/swoosh.wav` fails, but the captured directory listing is only an eight-entry sample. | Absence from the whole package has not been established. Audit the ZIP directory and case/path mapping before requesting replacement. |
| Notification/class-lookup clusters were too broad | A Monster, Azada, Gesundheit!, GravBot and Rolando reach nonzero notification-center results; MiniGame Paradise and Silly Owls reach nonzero class results. | The last API does not prove the source of their later null. Keep separate fault operands/imports until demonstrated otherwise. |
| Error-domain failures differ materially | Blockoban has an NSError, Parachute Ninja treats UIApplication as an NSError, Scoops uses `0x1`. | These are not one generic NSError/nil problem. Decode callback signatures and register moves, including void returns, before changing error creation. |
| Collection/defaults failures are different contracts | Hook World/Mage Gauntlet mutate restored preference dictionaries; Legendary War mutates an immutable property-list result; Trenches removes array elements during indexed selector dispatch. | A global mutable-copy policy or skipping array bounds would conceal these distinctions. `UIView.subviews` already returns a copied array. |
| Active presentation does not imply correct rendering or speed | Flip's Escape: 288 presentations/9.764s with user-confirmed corruption. DizzyWizards: music/input and 1,461 presentations/38.434s but a tiny central image. ZA: 4,523 loops versus 384 presentations/75.111s. | Counters alone cannot distinguish malformed transforms/texture state, intended throttling, or expensive guest work. Main loops are not simulation FPS. |

The full CSV also covers OldFBSession initialization (Amateur Surgeon), CFRelease-
adjacent failures (Galaxy/Mystery Agency 2), nil dictionary origin (Gravity Guy),
thread-start similarities (Little Acorns/Tongue Tied), Pudding Panic's distinct
MOVW/MOVT-derived global, SPYmouse's nil string and Paper Toss's resource-bundle
Info.plist path. Each has a falsifiable next action, not a replacement API guess.

## Research conclusions

Historical non-ARC ownership rules require ownership provenance, not retaining
everything indefinitely. Apple's CFPreferences source documents immutable
returned property-list values; the exact old NSUserDefaults behavior/caller copy
expectation still needs a period-correct comparison. The existing modern macOS
copy/defaults fixture is useful but cannot settle iPhone OS 2–6 behavior.
Earlier preserved research already found immutable deep copies in Apple's
contemporary CF-550 preferences implementation. The outstanding issue is the
games' direct mutation expectation, not whether current documentation promises
mutable defaults. The fork's separate retain-instead-of-snapshot storage gap
could expose these invalid mutations earlier if corrected; it is not their fix.

NSArray selector enumeration forbids mutating the receiving array. Trenches
therefore requires identifying why that array is aliased/mutated and whether the
guest makes an independent copy. A snapshot workaround is not yet a verified
historical contract. Likewise null pthread user_data is legal, and successful
pthread_create's zero return is not evidence of a bug.

CFStream creation, opening, failure, error retrieval and scheduled callback
delivery are separate states. Addicus and ORBITAL expose an actionable incomplete
offline stream model: returning null from construction is not equivalent to an
allocated stream failing to open. This deserves a focused implementation with
context ownership and callback ordering tests, rather than inventing connectivity.
Addicus also reaches a concrete missing CFWriteStreamSetClient function.

Primary references used:

- [touchHLE debugging facilities](https://github.com/touchHLE/touchHLE/blob/trunk/dev-docs/debugging.md).
- [Historical Apple memory-management rules](https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/MemoryMgmt/Articles/mmRules.html).
- [Apple CFPreferences source](https://github.com/apple-oss-distributions/CF/blob/main/CFPreferences.c).
- [Apple CFStream tasks and error handling](https://developer.apple.com/library/archive/documentation/Networking/Conceptual/CFNetwork/CFStreamTasks/CFStreamTasks.html).
- [NSArray selector enumeration](https://developer.apple.com/documentation/foundation/nsarray/makeobjectsperformselector%3Awithobject%3A?language=objc).
- [Historical key-value accessor conventions](https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/KeyValueCoding/AccessorConventions.html).
- [Apple OpenGL ES application design](https://developer.apple.com/library/archive/documentation/3DDrawing/Conceptual/OpenGLES_ProgrammingGuide/OpenGLESApplicationDesign/OpenGLESApplicationDesign.html).
- [Apple native crash analysis](https://developer.apple.com/documentation/xcode/diagnosing-issues-using-crash-reports-and-device-logs) and [memory-access crash investigation](https://developer.apple.com/documentation/xcode/investigating-memory-access-crashes).

These establish contracts and investigative methods, not the unobserved cause of
a particular game's failure. Exact guest binaries are still needed where captured
instruction/literal windows cannot recover an import or field producer.

Alternatives considered:

- Existing GDB register/memory access and software breakpoints are immediately
  useful once the exact executable and reproducible state are available. The
  current `src/gdb.rs` does not implement the `Z` packet family and does not
  advertise reverse execution; do not assume hardware watchpoints or rewind work.
  [GDB's watchpoint documentation](https://sourceware.org/gdb/current/onlinedocs/gdb.html/Set-Watchpoints.html)
  explains why falling back to instruction stepping can be expensive. This is an
  engineer-controlled follow-up for one known address, not a universal iPhone mode.
- [QEMU record/replay](https://www.qemu.org/docs/master/system/replay.html) records
  nondeterministic inputs and uses instruction counting/checkpoints. Applying that
  model here would also require control of host timers, audio, scheduling, resource
  and graphics interactions. It is a substantial architecture project, not a flag
  that turns the existing flight recorder into reverse debugging. Bounded causal
  boundary records have a better first-step cost/risk ratio.
- A native sanitizer build can investigate a classified native memory fault, but
  cannot by itself explain a guest pointer that is invalid within the emulated
  address space. Start with the matching .ips and build identity, then target the
  implicated native component. No sanitizer or local build was run here.
- Full GL call tracing/readback would produce much more data and could distort
  timing. A marked state transition using tracked state is preferable; correct
  framebuffer completeness still would not prove correct texture/vertex content.

## Ranked engineering queue

Rank reflects expected information gain and defensible corrections, not a promise
of easiest games. Controls are selected by subsystem.

| Rank | Opportunity / primary | Secondary validation / controls | Prediction; falsifier; residual limit | Cost and risk |
|---|---|---|---|---|
| 1 | Allocation, lifetime and storage-reuse provenance: Peggle | Anomaly HD, SUPERFALL, Tiki Totems 2, Topple 2; Spoing control | Distinguish free-before-registration-removal from normal deallocation/reuse or no observed free. No retained origin is inconclusive, not proof against lifetime failure. Even a proven extra release needs owner/caller interpretation. | Medium implementation, bounded storage, timing risk to measure; no semantic correction. |
| 2 | Exact indirect-import reconstruction: Crash Drive | Jack Lumber first; then Slingshot, Stick Skater, Wind-up Knight; Spoing control | Identify actual slot/symbol/value. Different symbols split the cluster; a populated slot/register mismatch rejects the simple missing-import theory. A zero slot alone does not prove why it is zero. | Low, terminal-only reads, high cross-game evidence. |
| 3 | Offline CFStream lifecycle contract: Addicus | ORBITAL, other CFNetwork users; Spoing offline-network control | Stream creation succeeds only as an object; opening fails truthfully with queryable error and correctly ordered callbacks. A failure before stream use rejects this explanation. Later networking/gameplay is unproven. | Medium/high implementation; callback ownership/run-loop risks. Separate future batch. |
| 4 | Typed offline callback provenance: Parachute Ninja | Scoops and Blockoban as comparisons; Dark Nebula 2 error/UI control | Determine the first boundary where an expected NSError becomes UIApplication/1. Correct argument followed by guest overwrite falsifies callback construction theory. Shared SDK not sufficient proof. | Medium, narrow bounded capture; avoid ABI changes without signature evidence. |
| 5 | Native crash classification: Fragger | Oddy Smog; Peggle if it again closes host | Correct .ips/UUID identifies native exception, signal, memory pressure or watchdog. Guest panic with no native incident is a different path. Symbolized frame still may be a victim. | Low code cost, external report dependency; new symbols retained now. |
| 6 | First-corrupt-state / slow-transition experiment: Flip's Escape | DizzyWizards and Roboto/Oddy if visually matched; ZA timing comparison; Cube/Space Monkey graphics controls | Compare valid and marked-bad GL state under the same path; distinguish viewport/FBO/texture/transform defects from no draw or expensive guest work. Normal state with corrupt pixels shifts to data/shader/driver path. | Medium/high, not included in first implementation; avoid readback and state-changing queries. |
| 7 | Period-correct copy/defaults and collection references | Hook World/Mage Gauntlet, Legendary War, Trenches; Toy Bot 1/Crazy Golf controls | Reproduce actual caller and ownership/copy contract. Matching modern macOS behavior does not settle historical guest behavior. | Requires exact caller/reference; global mutability changes high risk. |
| 8 | Interaction and package evidence | Toy Bot 2; 8-bit Ninja/Paper Toss; encrypted titles | Show intended intro swipe/drag behavior or exact ZIP resource absence. A failed resource lookup alone is not package corruption. | External evidence first; no emulator patch for encrypted code. |

Other concrete missing-contract work remains available outside this standstill
inventory: Junny navigation bar, ADVENA/ZENONIA2 run-loop timer validity,
BouncyMouse gesture recognition, Bungee Ball navigation-controller access,
Unmechanical host_statistics size semantics and LOCV numeric conversion. Do not
silently classify these as insoluble pointer failures or mix them into this IPA.

## Selected Dev 2.11 implementation — diagnostic-only

1. `src/mem/object_history.rs`: 512 sets x four ways (2,048 guest addresses), eight
   recent events per retained address, allocation origin, previous generation,
   last removal/free stamps, eviction count. Under 2 MiB on supported 64-bit hosts.
   No guest allocation is quarantined and no guest object is retained by tracing.
2. `src/mem.rs` / `src/objc/objects.rs`: observe object creation, host-table removal,
   completed heap free, same-address heap allocation and overlapping VM free.
   Check an existing object-table collision before writing a replacement isa,
   and print evidence while preserving the strict failure. This is not recovery.
3. `src/diagnostics.rs`: reuse retain/release/autorelease hooks and update a compact
   last-observed API/ObjC boundary (thread, PC, LR, receiver, selector). At terminal
   panic inspect bounded recent arguments/registers and emit at most eight cached
   addresses. Existing universal recorder stays enabled and otherwise unchanged.
4. `src/environment.rs`: reconstruct only the verified straight-line A32 import
   load idiom from current memory, then resolve the slot through existing Mach-O
   relocation/indirect-symbol metadata. No instruction-by-instruction tracing.
5. Hosted IPA workflow: preserve archive dSYMs plus executable/dSYM UUIDs, commit
   and workflow identity in a separate artifact. These do not change the IPA.
6. App picker: put this diagnostic batch first, remaining library alphabetical.

`--diagnostics=off` disables the object cache and new terminal decoder as well as
the universal recorder. This initial cache observes host boundaries globally but
does not globally log every object/message. Per-event work is bounded; actual
device performance still requires measurement. The previous universal recorder's
acceptable performance does not prove this addition is free.

Limitations: cache eviction is explicit; long-lived objects can lose their origin.
Raw guest stores, memcpy corruption, direct isa writes, every allocator path and
native-host allocations are not traced. Guest-defined retain/release overrides
may bypass host hooks. Boundary context is the most recent observed boundary,
possibly nested or inherited, not a reconstructed allocation stack. A register's
coincidental address match is only a candidate, not a proven fault operand.
Native termination can occur before the in-memory history is dumped. The import
decoder is not Thumb, arbitrary control-flow analysis or Pudding Panic coverage.
Preserving Xcode dSYMs does not enable missing Rust release DWARF line information;
the existing Cargo release profile is unchanged. UUID identity and available native
symbols remain useful, but complete Rust source-level symbolization is not claimed.

No save reset, compatibility accommodation, networking success, error suppression,
global mutability change or diagnostic redesign is included. Existing provisional
fork accommodations remain separate; this review does not certify them.

## Validation and acceptance

Focused tests exercise bounded eviction/generations/free/removal/VM overlap, the
exact A32 literal/import chain plus rejection of intervening writes/branches, and
a real guest-call ABI boundary: `_malloc_zone_free` through the SVC dispatcher followed by
an ObjC allocation collision. The latter must fail with diagnostics both off/on,
preserve the conflicting isa, and expose `_malloc_zone_free` with caller LR when on. It is a
synthetic invariant reproducer, not a claim to reproduce Peggle's actual cause.
The initial fixture incorrectly assumed ordinary `_free` left registration
behind; hosted testing disproved that assumption. Source inspection confirms
ordinary free already removes the host object, while the default-zone free path
does not. No allocator or free semantics were changed to make the test pass.

Only source inspection, lightweight Python evidence extraction and `git diff
--check` run locally. No local Rust, Cargo, CMake or preflight helper is allowed.
Hosted library tests and iOS archive compilation are separate gates. The current
development workflow skips the full compiled guest suite and formatting steps;
lint is allowed to fail. Report their actual results independently of green badges.
Actual hosted results and artifact identity are recorded below; no physical-device
result is inferred from compilation or the synthetic reproducer.

Device acceptance requires (a) recognizable origins/free/reuse evidence in at
least one failing title, or an identified import slot/symbol in the five-game
cluster, and (b) no observed Spoing regression. An unchanged game screen is expected
in this diagnostic build. If histories are absent/evicted or point nowhere useful,
revise selection/pinning or inspect the exact executable; do not automatically
increase logging capacity. No current title moves to 'working' on a diagnostic
pass. DeepBlue moves only to observation-needed; CFStream moves to actionable
contract work and the five-game group to a specific import investigation.

## Simple device batch and external evidence

Without deleting saves, test the first eight picker entries: Anomaly HD (launch),
Crash Drive (launch), Jack Lumber (launch), Peggle (launch), SUPERFALL (play until
the usual failure, or about a minute), Tiki Totems 2 (select a level), Topple 2
(launch/play), Spoing (brief play and Home). Stop after each first failure. Send
the combined log; if the entire host closes, save that log before relaunching and
send the next segment separately. Report any changed symptoms or slowdown.

For Fragger/Oddy Smog, when separately collecting native evidence:

1. Note the installed Dev label and local time, launch only that title and
   reproduce one host closure. Preserve the current log before another launch.
2. Open iPhone Settings > Privacy & Security > Analytics & Improvements >
   Analytics Data. Look for an iDared/iDared 32bit report at that time; also check
   JetsamEvent at that time if no app report is present. A report may appear later.
3. Open the report, Share > Save to Files, then attach the full `.ips` file with
   the time/game/Dev label. Do not send only a screenshot of the report.
4. Engineering matches process, bundle `com.bookieart1.idared32bit.dev`, timestamp,
   termination fields and loaded-image UUID to the archived executable/dSYM and
   embedded commit/run. A Dev label or similar filename alone is insufficient.
   New dSYMs cannot symbolize an older, different UUID. Preserve existing saves;
   compare restored versus clean state only after backup and explicit permission.

For Flip's Escape, a future bounded graphics experiment must associate an explicit
capture marker with a monotonic presentation ID and a short pre/post-state ring.
Capture viewport/scissor, FBO/RBO attachments/dimensions/completeness, transform,
active texture/binding/format and draw dimensions from tracked wrapper state.
Use read-only completeness checks only at a defined diagnostic checkpoint; do not
drain glGetError or routinely read back pixels. Correlate a screenshot's timestamp
and marker with the nearest presented frame. A screenshot alone cannot identify
the first corrupted frame; to claim that precision, an explicit user mark/freeze
or a defined machine-detectable invariant must capture that exact presentation.
ZA should reuse and rearm the existing slow/worst-gap capture at an explicit scene
marker or after a new healthy baseline, retaining the startup sample separately.
Add only bounded thread-wait/callback/draw duration attribution that distinguishes
where the later frame gap is spent. The old 3.740-second capture must not be
mistaken for the subsequent gameplay transition. This experiment is designed,
not implemented in Dev 2.11.

Encrypted-package cases remain external: aqua baller, Burrito Bison, FallingBalls,
Geared, iStunt 2, Potty 2, Santa Engineer, Solomon's Keep, Spell Sword, Subrats HD,
The Slayers. Require a legitimately obtained, executable unencrypted build with
matching title/version/architecture and intact resources. Do not infer absent
APIs from encrypted execution. For 8-bit Ninja, first compare full archive resource
names, integrity and exact failed path; replacement is warranted only if the
required resource is actually absent/corrupt. A native run or legitimate alternate
version is useful for Toy Bot 2's intro interaction and ambiguous legacy semantics.

## Completed hosted validation and delivered artifact

Final implementation/package commit:
`d13d547c1427a62e11975ceac0313485fc6d7b36`. The subsequent documentation-only
record commit does not identify a different executable. No local Rust, Cargo,
CMake, preflight, or build helper was run.

- Hosted test run [41586511224](https://github.com/bookieart1/iDared32bit/actions/runs/41586511224):
  **217/217 library tests passed on both macOS and Windows**. macOS also passed
  the native Foundation reference and all three focused diagnostic reproducer
  invocations. The universal-recorder-default/explicit-override test passed.
- The workflow as a whole was **not green**: Android NDK r28c extraction failed
  before lint because the runner's 7-Zip rejected nested symlinks. macOS/Windows
  lint was allowed to fail on two pre-existing overlong comments in `dlfcn.rs`
  and `ns_dictionary.rs`. Formatting, the complete guest-executable suite, and
  native sanity were branch-gated/skipped. These checks are not reported as passes.
- The actual zone-free/SVC reproducer retained generation 2, class and allocation
  origin for address `0xa8d0`, recorded `_malloc_zone_free` on thread 0 with LR
  `0x1235`, heap reuse, and final object-table collision, with no intervening
  host-object removal. It preserved the strict failure with tracing off and on.
  This demonstrates additional causal evidence; it does **not** prove Peggle
  takes that path. The initial ordinary-free fixture failed because ordinary
  free already removes registration; the test was corrected, not the semantics.
- Hosted IPA run [41586511172](https://github.com/bookieart1/iDared32bit/actions/runs/41586511172)
  successfully built and packaged the arm64 iOS executable. The downloaded
  `outputs/iDared-Dev-2.11.ipa` is **20,209,581 bytes**, SHA-256
  `929e3d8a3a081a98b9b4ea4c6a2dea5b113f870152c1811b5ac59cdb19bffa09`.
  Verified ZIP integrity, embedded commit/run, development identity
  `com.bookieart1.idared32bit.dev`, minimum iOS 16.0, no provisioning profile,
  no signature directory/load command, and binary markers for the universal
  recorder, object history and A32 decoder. Downloaded artifact digest matches
  GitHub's recorded digest.
- Separate native-symbol artifact was downloaded and checked against the actual
  IPA executable: both Mach-O UUIDs are
  `4DF512A1-78B6-3CC3-A475-D1AAEA867485`. Preserve these with this exact package;
  they cannot symbolize older builds. Full Rust DWARF source coverage is not
  claimed. Verification details and job logs are archived in `outputs/`.

Device acceptance is still pending. No standstill has been promoted to working.
The next result must identify a useful origin/import or falsify the selected
theory; merely producing a longer log is not acceptance. Spoing is the one
working control for this diagnostic-only implementation.

## Dev 2.11 device result and Dev 2.12 resolution pass

The physical Dev 2.11 pass observed no slowdown and validated the bounded
diagnostic infrastructure as usable, but it covered eight selected standstills,
not the entire portfolio. It produced three actionable general corrections:

- Crash Drive and Jack Lumber identify `_NSBuddhistCalendar` as their shared
  zero data import.
- Peggle identifies a 3D-mixer input-bus stream-format query that the emulator
  incorrectly forced through bus-zero-only state.
- Topple 2 proves that a started `NSThread` receiver can be deallocated and its
  storage reused before the thread entry helper runs.

SUPERFALL and Tiki Totems 2 remained inconclusive because their address records
were evicted before the final invalid consumption. Dev 2.12 adds only bounded
correlation from a final consumed value to the most recent retained boundary
that returned it; it does not enlarge the recorder or trace every store.
Anomaly HD remains unresolved and receives no speculative ownership change.

The selected contracts, predictions, picker filter, and restored Dev 2.10
follow-up collection are recorded in
`dev-docs/compatibility-2.12-calendar-thread-audio-diagnostics-and-picker.md`.
Hosted validation and physical-device acceptance are pending; none of these
titles is promoted to working by implementation alone.
