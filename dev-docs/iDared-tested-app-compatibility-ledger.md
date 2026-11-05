# iDared 32-bit tested-app compatibility ledger

Last reconciled: November 5, 2026
Latest device-tested checkpoint: **iDared Dev 2.32**, implementation `9d568fc7526acc08f4ce6ee9185ca8fbff5e2cea`, IPA workflow `41990584956`; two supplied logs cover the requested pass.
Latest built checkpoint: **iDared Dev 2.32**, implementation `9d568fc7526acc08f4ce6ee9185ca8fbff5e2cea`, unsigned IPA workflow `41990584956`; package/build evidence and the physical-device result remain separately qualified below.
Current activity: **Dev 2.32 physical-device evidence is reconciled. Highway Rider reaches responsive gameplay but has major missing scene graphics; iStunt Reloaded Pro reaches its level-pack table but lacks level buttons and later stalls input; Sky Burger reaches name entry but tapping it terminates the host and now requires a matching native `.ips`; TowerBloxx reaches gameplay but fails during Home/guest teardown; Tiki Totems Premium remains a healthy control. The logs also expose exact later boundaries for 1112 ep.01, Junk Jack Retro, both Real Racing packages and Stick Stunt Biker. No subsequent implementation or IPA is authorized.**

The maintainer review at the end of this ledger qualifies earlier implementation claims. In particular, Dev 1.47's font fallback is provisional, its arc implementation is approximate, and its date formatter is not locale/time-zone complete. Earlier sections are historical checkpoints, not a claim that all listed work is currently complete.

This is the durable resume point for hands-on compatibility testing. It records what the user actually observed, not merely whether a launch avoided a panic. “Working” therefore means usable gameplay; a title that opens but freezes, cannot be controlled, loses progress, or has severe audiovisual faults remains partial.

## Status meanings

- **Working:** usable gameplay and no known blocking fault in the tested portion.
- **Working with minor issues:** usable, but has a documented non-blocking defect.
- **Partial:** launches or plays, but a serious defect prevents a normal session.
- **Blocked:** does not reach usable gameplay.
- **Skipped:** deliberately removed from the current batch.
- **Needs new dump:** the executable is explicitly encrypted or the package is invalid; emulator work cannot meaningfully test that copy.

## Historical hands-on checkpoint: Dev 1.42 / planned 1.43

| App | Status at 1.42 | Exact resume point / known issues | Next intended check |
|---|---|---|---|
| A Monster Ate My Homework | Blocked | Still fails during early startup. The original seven-page baseline ended in a null-page access. No visible improvement in the current batch. | Pause with this group; re-evaluate after broad memory/runtime fixes. |
| Addicus | Blocked | 1.42 advanced past the prior missing `NSException -raise`, then raised “The capture argument is not valid” twice and hit a null-page access. Its executable also references missing Blocks runtime symbols. | 1.43 adds minimal shared Blocks ABI support. Test once; do not call it fixed merely for reaching a later blocker. |
| Air Penguin | Working | Earlier revisions advanced it through the publisher logos, menu, world map, and first-level launch. Later user report said it appeared to work normally. | Regression-check after broad changes, not every narrow build. |
| aqua baller | Needs new dump | Log evidence explicitly says its executable is FairPlay-encrypted. The user intentionally stopped clicking it in the current batch. | Obtain a fresh legitimate decrypted dump. |
| Azada | Blocked | Still failed early in the current batch; baseline decisive error was a null-page access. | Pause; re-test after broader memory/filesystem work. |
| B'Blast | Blocked | 1.42 advanced beyond the ASL logging blocker, then failed on process query `sysctl(1,14,1,1)`. | 1.43 supplies a safe general `kern.proc` fallback. |
| Backdraft | Blocked | Still failed early; baseline decisive error was a null-page access. | Pause; revisit with the next null-memory cluster. |
| Blockoban | Blocked | Still failed early; baseline decisive error was a null-page access, with possible service/resource complications. | Pause; first separate local runtime failure from unavailable service behavior. |
| Blot | Partial | Full image is now visible and the game can be played, but it is letterboxed. It can slow progressively until unplayable. Dying calls missing `___vsnprintf_chk` and crashes. | 1.43 implements the checked formatter. Re-test death once and separately note whether slowdown changes; they are distinct defects. |
| Bounce | Partial | Gameplay became smoother and progress was observed saving, but intermittent lag remains. Some secondary/menu buttons have crashed it. In 1.42 the new offline-render success path produced severely distorted/laggy audio because it returned artificial silence. | 1.43 replaces artificial silence with decoding of queued audio. Test normal gameplay audio, one progress save/reopen, and the known crashing secondary button. |

## Previously established reference apps

| App | Current status | Details worth preserving |
|---|---|---|
| Spoing | Working in tested portion | Original target. Early levels became smooth and playable; level transitions advanced; saved progress eventually worked. Keep as a core regression title. |
| Diversion | Working with minor issues | Playable and saving progress. Publisher logo can flash briefly before a black interval; loading transitions sometimes have glitched or “crusty” audio. |
| Sticky | Working with minor issues | At least three levels played and progress persisted. One early run reached an animated but unresponsive menu; later attempts worked. Intro can show a one-frame visual glitch and occasional crusty audio. |
| AB Buster | Skipped | Emulator reached its publisher logo before `_CFHostCreateWithName` failed, but the same copy also stalls at its logo on the user’s original iPod while music plays. Test validity is therefore uncertain, and the user chose to skip it. |

## Historical group handoff: Dev 1.43

After 1.43, this group can be paused without losing continuity. The most valuable short retest is:

1. Addicus: note the first decisive error only.
2. B'Blast: note whether it passes the process query and its next blocker.
3. Blot: die once, then spend long enough in one level/menu to assess the separate slowdown.
4. Bounce: listen through gameplay and a transition, verify save/reopen, then try the known secondary button.

If time is limited, prioritize B'Blast, Blot death, and Bounce audio. A Monster Ate My Homework, Azada, Backdraft, and Blockoban do not need another run for this narrow build.

## Complete wider-library baseline

The exhaustive seven-page audit remains the authoritative baseline for every other app tested. These companion files must stay with this ledger:

- `iDared-compatibility-audit-7-pages-1.0.md` — full narrative audit, error-family landscape, encryption/corruption classifications, roadmap, and unresolved launches.
- `iDared-launch-inventory-7-pages-1.0.csv` — all 127 recorded launch attempts and their boundaries.
- `iDared-per-game-compatibility-7-pages-1.0.csv` — complete 123-game compatibility table.

Baseline scope: **127 launch attempts, 123 unique loadable IPA files, 26 additional packages that could not be opened as ZIP archives, and 149 distinct IPA filenames in total.** Do not overwrite this baseline with later behavior. The next seven-page pass should be recorded as a new snapshot and compared launch-by-launch and error-family-by-error-family.

## Completed Dev 1.00 to Dev 1.42 seven-page comparison

The exhaustive longitudinal audit is complete. Its companion files are:

- `iDared-longitudinal-compatibility-audit-1.00-vs-1.42.md` — complete source inventory, per-app comparison, recurrence analysis, attribution, roadmap, safety plan, and final lists.
- `iDared-exact-launch-inventory-1.00-vs-1.42.csv` — all 275 raw launch segments, including the 20 Page 7 duplicates explicitly marked against Page 6.
- `iDared-per-app-compatibility-delta-1.00-vs-1.42.csv` — the full 127-identity union with both outcomes, exact line references, and comparison classification.
- `iDared-error-family-delta-1.00-vs-1.42.csv` — launch and unique-app counts for every first-blocker family in both snapshots.

Verified comparison scope: **15 files, 77,376 physical lines, 275 raw launch segments, and 255 segments after removing the exact 20-launch Page 6/Page 7 duplication.** Dev 1.42 covers 122 of the baseline's 123 launched IPAs (99.2%). Crazy Pinball Galaxy is the only baseline omission; Crazy Pinball Jungle is a different title, not a replacement.

Longitudinal result:

- 5 shared apps became newly partial/no-panic sessions: Blot, Bounce, Bulba the Cat, Flick Buddies, and Paper Monsters.
- 9 advanced to a later blocker: Addicus, B'Blast, Bungee Ball, Catacombs, Gesundheit, junny, Riddim, Silverfish, and Splode.
- 7 reached user-ended runtime without a panic in both snapshots: Air Penguin, Daredevil Dav, Diversion, Floe, Kosmo Spin, Mini Beasts HD, and Sprinkle. This does not prove complete gameplay.
- 100 union identities retained the same logged classification.
- No definitive shared-app regression was proven. Rogue Sky is the principal unresolved stability concern: its former selector blocker is gone, but Dev 1.42 records 382 repeated reachability-callback registrations before the enclosing log terminates.
- The explicit encrypted set remains 8 files. The corrupt/invalid set remains the same 26 files.

The comparison supports continued general emulator work: the clearest gains come from shared AudioQueue, Foundation, cryptography, threading, locale, predicate, and reachability implementations. The results also show rising diminishing returns because most successful fixes expose another blocker instead of completing compatibility.

## Regression set for broad fixes

Every broad runtime, graphics, audio, filesystem, or persistence change should be checked against:

- Spoing: launch, gameplay, level completion, return to map, saved progress.
- Diversion: launch, transition audio, gameplay, saved progress.
- Sticky: responsive menu, gameplay, saved progress.
- Air Penguin: publisher logos, menu/world map, first-level launch.
- Blot: complete framing, normal speed, death path.
- Bounce: gameplay smoothness, audio, secondary-button behavior, saved progress.
- The separately installed App Store iDared app must remain untouched; development builds keep the `com.bookieart1.idared32bit.dev` identity.

## Maintenance rule

Update this ledger only from a concrete test report or decisive log evidence. Preserve prior observations when a later run is ambiguous. “No crash notification” is not proof of compatibility, and “advanced to a new blocker” is progress but not a working result.

## Dev 1.44 engineering checkpoint

Dev 1.44 is a bounded legacy Foundation/Objective-C runtime batch selected from the completed Dev 1.00 versus Dev 1.42 audit. It implements five repeated decisive blockers without changing graphics, audio, app-picker identity, or save-container layout:

| Contract | Known apps directly unblocked | Expected result |
|---|---|---|
| `CFArraySetValueAtIndex` | CrazyGolf; Mystery Agency 2 | Advance to the next blocker. |
| `+[NSBundle bundleForClass:]` | DizzyWizards; Marblenauts | Advance to the next blocker. |
| `method_getTypeEncoding` | Fly Kiwi Fly; Ski Jumping | Advance to the next blocker. |
| `+[NSThread mainThread]` | Riddim; Tesla Wars | Advance to the next blocker. |
| `-[NSKeyedArchiver encodeFloat:forKey:]` | HookWorld; QuickHook | Advance; these are the best candidates in this batch to reach playable UI because both already reached application/UI startup. |

The implementation covers non-retaining Core Foundation arrays, guest and host Objective-C method encodings, stable main-thread identity, bundle lookup for a guest class, and keyed float archive values. No title is marked working until device testing confirms it.

Required regression matrix for this batch:

- Spoing: launch, gameplay, level completion, map return, save/reopen.
- Diversion: launch, transition audio, gameplay, save/reopen.
- Sticky: responsive menu, gameplay, save/reopen.
- Air Penguin: logos, menu/world map, first-level launch.
- Blot: complete framing, normal speed, death path.
- Bounce: gameplay smoothness, audio, secondary button, save/reopen.
- Packaging: verify development identity `com.bookieart1.idared32bit.dev`, unsigned packaging, and no effect on the separate App Store installation.

### Dev 1.44 verification and artifact

- Baseline was clean at `b6126818`; no pre-existing user changes were present or overwritten.
- Final repository commit: `00178c3bcf54adc84decabf40ed3644c464a3e7d`.
- GitHub build run 73 completed successfully and validated the separate `com.bookieart1.idared32bit.dev` / `iDared Dev` identity, unsigned packaging, and absence of an embedded provisioning profile.
- The final lint/test run completed successfully on macOS, Windows, and Android. Development runs execute the current library tests; the repository's obsolete iPhone OS 3 guest linker and native Objective-C sanity compile remain release-only because current hosted toolchains cannot link/compile that unchanged legacy harness. Release-branch strict checks were not relaxed.
- Final IPA SHA-256: `7AE313172D7901ADDCA9C4B9CCDF3E2B7EE7E9FC999F4944B7F4130BA3EADD9D`.
- Device testing covered all ten target apps. The original target blocker was removed in every case; one title became usable in the tested portion and nine exposed later blockers.

### Dev 1.44 device results (`Log11.txt`)

| App | Dev 1.44 result | Decisive evidence / next blocker |
|---|---|---|
| Crazy Golf | **Working in tested portion** | Opened and accepted gameplay input with no panic (lines 32–146; user-confirmed functional). |
| DizzyWizards | Advanced to later blocker | Both attempts reached a one-byte null-page access (`0x3365e`, lines 184–186 and 273–275). |
| Fly Kiwi Fly | Advanced to later blocker | Ignored “capture argument is not valid” exceptions, then a one-byte null-page access (lines 500–502). |
| HookWorld | Advanced to later blocker | Reached UI startup, then attempted to insert `nil` into `NSMutableDictionary` (lines 777–779). |
| Marblenauts | Advanced to later blocker | One-byte null-page access (line 867). |
| Mystery Agency 2 | Advanced to later blocker | Four-byte null-page access and `MemoryError` (lines 966–968). |
| QuickHook | Advanced to later blocker | Reached UI startup, then attempted to insert `nil` into `NSMutableDictionary` (lines 1215–1217). |
| Riddim | Advanced to later blocker | Missing `NSTimer initWithFireDate:interval:target:selector:userInfo:repeats:` (line 1415). |
| Ski Jumping | Advanced to later blocker | Missing `_class_addMethod` (line 1552). |
| Tesla Wars | Advanced to later blocker | Missing `+[NSTimeZone timeZoneWithAbbreviation:]` (line 1848). |

This is a meaningful but bounded result: **10/10 intended blockers were removed, 1/10 titles became functional in the tested portion, and 9/10 advanced to a later blocker.** It does not establish full-game compatibility for Crazy Golf.

## Dev 1.45 engineering checkpoint

Dev 1.45 deliberately addresses only the three exact, standard runtime contracts exposed by Dev 1.44:

| Contract | Target app | Expected result |
|---|---|---|
| `-[NSTimer initWithFireDate:interval:target:selector:userInfo:repeats:]` | Riddim | Advance beyond timer construction. |
| `_class_addMethod` | Ski Jumping | Advance beyond dynamic Objective-C method installation. |
| `+[NSTimeZone timeZoneWithAbbreviation:]` | Tesla Wars | Advance beyond legacy time-zone lookup; best chance of reaching usable UI in this batch. |

The batch leaves the raw null-page cluster and `NSMutableDictionary setObject:nil` failures unchanged because either requires deeper causal evidence; globally accepting `nil` dictionary values would violate Foundation behavior and could conceal the actual missing-data bug.

Required device test order for Dev 1.45:

1. Riddim
2. Ski Jumping
3. Tesla Wars

No other title needs testing for this narrow build unless a concerning system-wide symptom appears. The standing broad regression matrix remains Spoing, Diversion, Sticky, Air Penguin, Blot, Bounce, persistence, audio, rendering, and the separate App Store iDared installation.

### Dev 1.45 verification and artifact

- Baseline was clean at `00178c3b`; no pre-existing user changes were present or overwritten.
- Objective-C/Foundation implementation commit: `2ae4e709e04061ac6d528240f344fa6d02809e6f`.
- Final repository commit: `0a2680c63ff2da78753c80cb07b46c77c3edf1ac`. The follow-up extends the general host-method bridge from five to six Objective-C parameters, which is required by the legacy `NSTimer` initializer.
- GitHub lint/test run 10 completed successfully on macOS, Windows, and Android.
- GitHub unsigned-IPA run 75 completed successfully and produced one verified artifact.
- The downloaded artifact ZIP matched GitHub's SHA-256 digest: `25A770C1B26ECB00CD2365661456FCE2FA8AC72DBCBB1FADF63D069B1D90D1CE`.
- Final IPA SHA-256: `1A8BAF0EC8F042F9233FBD982DB6CCD9F1EDE795DBFBD7A28549DEBC1F72B541`.
- Package inspection found one `Payload/*.app` bundle and no embedded provisioning profile. The binary plist contains the separate development identity `com.bookieart1.idared32bit.dev`, display name `iDared Dev`, and executable `iDared 32bit`.
- Device testing confirmed that all three target contracts were exercised and removed. Riddim and Tesla Wars still failed before presenting usable UI, while Ski Jumping newly reached its title/menu screen before failing.

### Dev 1.45 device results (`Log12.txt`)

| App | Dev 1.45 result | Decisive evidence / next blocker |
|---|---|---|
| Riddim | Advanced to later blocker | Passed timer construction and continued through network/audio startup, then the emulator panicked while treating binary-plist bytes as UTF-8 through a failable `NSString` initializer (lines 180–200). |
| Ski Jumping | Newly reached title/menu, then crashed | Passed dynamic method installation and visibly reached the title/menu. After numerous unsupported audio files and one missing `start4.jpg`, an opaque host-object lookup panicked without reporting the requested object type (lines 422–465). |
| Tesla Wars | Advanced to later blocker | Passed time-zone abbreviation lookup and reached substantially later UI/audio/analytics startup, then called the missing standard property `-[NSDateFormatter setLocale:]`; `setTimeZone:` immediately beforehand was only a no-op (lines 736–764). |

The Dev 1.45 result is **3/3 prior blockers removed**, with three later blockers exposed and no title yet confirmed playable. Ski Jumping made the only visible UI advance.

## Dev 1.46 engineering checkpoint

Dev 1.46 is a small Foundation-correctness and diagnostic batch:

| Contract | Target app | Expected result |
|---|---|---|
| Failable `NSString` UTF-8 initialization | Riddim | Invalid binary data now returns `nil` instead of crashing the emulator. This is standard behavior and may let the app use its own fallback; it may instead expose a later missing-data assumption. |
| `NSDateFormatter` locale/time-zone properties and ownership | Tesla Wars | Advance through `setLocale:` and preserve both configured objects correctly. This is a general Foundation implementation, not an app-specific bypass. |
| Typed missing-host-object panic diagnostics | Ski Jumping | The next failure will identify both the guest object address and exact host object type, making the opaque lifecycle/image crash actionable. This is diagnostic progress and is not expected by itself to make the game playable. |

Apple's published XNU contract says hardware frequency sysctls are 64-bit values. Dev 1.46 therefore deliberately does **not** force Ski Jumping's four-byte `hw.cpufrequency`/`hw.busfrequency` buffers to succeed; that would be an inaccurate global compatibility hack.

Required device test order for Dev 1.46 remains alphabetical:

1. Riddim
2. Ski Jumping
3. Tesla Wars

Capture the first decisive error for each. For Ski Jumping, the new panic text containing “while borrowing” or “while mutably borrowing” is the essential evidence. No other title needs testing for this narrow build unless a broad regression is noticed.

### Dev 1.46 verification and artifact

- Baseline was clean at `0a2680c6`; no pre-existing user changes were present or overwritten.
- Final repository commit: `26f0a4a053e4e7c26488acac60289c4ae1ad1ec6`.
- GitHub lint/test run 11 completed successfully on macOS, Windows, and Android.
- GitHub unsigned-IPA run 76 completed successfully and produced exactly one artifact.
- The downloaded artifact ZIP matched GitHub's SHA-256 digest: `21C7834E46ED0502ECFADAC9178F036969A1E601D0CDF5FEB3348E9EA7AD4B79`.
- Final IPA SHA-256: `6B2A65B5971D3A3909215EBB37A4C43F30309204F554E5446828A51CFDE4F126`.
- Package inspection found one `Payload/*.app` bundle, one `Info.plist`, and no embedded provisioning profile.
- Device testing was pending at artifact delivery; the results are recorded immediately below.

### Dev 1.46 device results (pasted report, build `26f0a4a`)

| App | Dev 1.46 result | Decisive evidence / next blocker |
|---|---|---|
| Riddim | **Newly reached its menu/start interface, then crashed** | The failable UTF-8 change removed the binary-plist panic. The app passed network/audio/UI startup and then called missing `_CGPathAddArcToPoint` (lines 32–253). |
| Ski Jumping | Still reached title/menu, then crashed | The typed diagnostic identified an unregistered pointer requested as a `UIFontHostObject` during text rendering (line 527). It does **not** establish that this was once a valid font or why the pointer is invalid. Numerous WAV assets also fail to decode (lines 434–512), and `files_ipod/start4.jpg` is missing (line 522), but neither is the recorded panic. |
| Tesla Wars | Advanced to later blocker before visible startup | Locale and time-zone property handling succeeded. The next blocker is the standard Unicode date-format pattern `ccc` in `NSDateFormatter` (line 825). |

Dev 1.46 therefore removed **2/2 intended behavioral blockers** and converted Ski Jumping's diagnostic target into actionable evidence. Riddim made the clearest visible advance; no title is yet confirmed playable.

## Dev 1.47 engineering checkpoint

Dev 1.47 attempts to advance the three newly decisive blockers while keeping audio, persistence, display/orientation, app-picker identity, and save-container code unchanged. This is a historical description of the delivered patch, not semantic or device validation; see the maintainer review below:

| Contract | Target app | Expected result |
|---|---|---|
| `CGPathAddArcToPoint` tangent-arc geometry | Riddim | Advance past menu/start-interface path construction. The implementation stores a smooth polyline so both the existing renderer and path-animation consumer can use it. |
| Provisional `UIFont` substitution | Ski Jumping | Substitute a 17-point system font when selected text helpers receive an unregistered or non-font object. This may hide a lifetime/dispatch defect and is not an established root-cause fix; unsupported WAV assets are a separate issue. |
| Unicode date patterns `c`, `cc`, `ccc`, and `cccc` | Tesla Wars | Advance through stand-alone weekday formatting; the exact `ccc` call should now produce the abbreviated weekday. |

Required Dev 1.47 device test order remains alphabetical:

1. Riddim
2. Ski Jumping
3. Tesla Wars

For Riddim, attempt to start gameplay if the menu remains responsive. For Ski Jumping, note whether it passes the title/menu crash and whether missing audio is audible. For Tesla Wars, record the first visible screen and first decisive error. No other app needs testing for this narrow build unless a broad regression is noticed.

### Dev 1.47 implementation status

- Clean baseline: `26f0a4a053e4e7c26488acac60289c4ae1ad1ec6`.
- Implementation commit: `d7f34ba107ba0c28aad0c9812e29697423a04f78`.
- GitHub lint/test run 12 completed successfully on Android, Windows, and macOS.
- GitHub unsigned-IPA run 77 completed successfully and produced exactly one artifact.
- Downloaded artifact ZIP SHA-256: `7ADD11897FB6C5A57B643573FB138BBF6D6E8A3BD932C7CE1DDD6A91C58BFFDB`, matching GitHub's recorded digest.
- Final IPA SHA-256: `21699C4FCE4C0AD801737B67B11478694C6DD2F86182860856C9D83912D1A062`.
- Package inspection found one `Payload/*.app`, one `Info.plist`, and no embedded provisioning profile. The binary plist contains the separate development identity `com.bookieart1.idared32bit.dev`, display name `iDared Dev`, and executable `iDared 32bit`.
- Direct device-testing IPA: `artifacts/1.47-d7f34ba1/iDared-Dev-1.47.ipa`.

## Maintainer reconstruction and operating checkpoint — October 31, 2026

### Scope and provenance

This review reconstructs the working model from the conversation, the saved audit and ledger, local commit history, the current source, relevant upstream differences, CI configuration, and decisive contexts in the latest Dev 1.46 report. It is **not** a fresh exhaustive reread of every historical raw log or a full correctness audit of the fork. Unverified attribution stays unverified.

- Active checkout: `repo/`; origin `bookieart1/iDared32bit`; branch `ci/unsigned-ipa-baseline`; HEAD `d7f34ba107ba0c28aad0c9812e29697423a04f78`. Working tree was clean at review start.
- Local upstream ref: `touchHLE/touchHLE`, `upstream/trunk` at `ca1de89c578e79746f33c1332e28efd106c334ba` (October 13, 2026). It is an ancestor of HEAD, **not a verified latest remote snapshot**. The fork also contains work predating this chat; do not attribute every upstream difference to these compatibility batches.
- Latest device evidence: `C:/Users/booki/.codex/attachments/69eb9ac7-b077-4b8d-9832-699b24ee507d/Pasted text.txt`, embedded `26f0a4a`. Riddim reaches menu/start then crashes; Ski Jumping still reaches title/menu then crashes; Tesla Wars has not visibly opened.
- Dev 1.47 remains a preserved, untested artifact. This review changes documentation only; no source behavior, commit, push, workflow run or IPA is changed.
- Persistent workflow rules are in the workspace-root `AGENTS.md`; future work must consult this ledger before interpreting a new crash in isolation.

### Recovered technical model and evidence strength

| Area | Existing work and observed result | Limits / retained investigation |
|---|---|---|
| Foundation, coding and persistence | The Spoing sequence crossed missing calendar/date components, cookie storage, numeric encoding, `NSNumber objCType`, string coding, no-copy string initialization, decimal numbers and process-info behavior. Local commits preserve these implementations. User eventually confirmed smooth early levels, progress surviving reopen/crash, and advancement past the former level-5 freeze. | These are observed aggregate improvements, not proof that every individual implementation is complete or that a particular commit alone fixed a particular symptom. Keep serialization/readback tests and old saves. |
| Guest memory / POSIX I/O | History includes `vm_allocate` conflicts, `MAP_FIXED` replacement, partial deallocation, invalid descriptors, memory protection and output handling. Earlier Spoing failures included `AddressUnavailable` and `posix_io.rs` unwraps. | Null-page faults in other games are not automatically the same bug, a shortage of phone storage, or solved by these patches. Trace the origin of nil/invalid values and guest callers. |
| Offline services and dynamic lookup | `dlsym` missing-symbol behavior and local `NSUbiquitousKeyValueStore` support preceded user-confirmed Diversion play/saving. Fake legacy SDK classes and unsupported network requests remain in current logs. | Local storage is not iCloud synchronization. SDK substitutions and offline responses are accommodations, not evidence that remote services work. Riddim can encounter a service-dependent blocker after graphics is repaired. |
| Rendering and layout | Blot moved from a cropped/offset corner through repeated unsuccessful viewport/layout attempts to full visible but smaller framing. Current source also has Blot-scoped layout diagnostics. | Full framing is not proof of correct scaling, input coordinates or orientation. Preserve the successful visibility result; do not repeat earlier zoom/offset guesses without measuring view bounds, backing size, viewport and touch mapping together. |
| Performance and audio | Blot's `MAGICK_THREAD_LIMIT=1` is an explicit app-scoped accommodation (`c0b81b0b`, `src/environment.rs`), not a proven general slowdown fix. Dev 1.42's AudioQueue offline path returned artificial silence; 1.43 changed it to queued-data decoding (`b6126818`). | User reports still show Blot severe slowdown and Bounce distracting audio/performance faults. No supplied later device result establishes that 1.43 eliminated them. Separate render/guest CPU time, callback timing, decoding and log overhead. Do not count silent/fabricated audio success as fidelity. |
| Shared runtime batch | Dev 1.44 added CF array replacement, bundle-for-class lookup, method encodings, main-thread identity and keyed float encoding. All ten prior blockers disappeared; Crazy Golf alone was user-confirmed usable. | Nine games exposed later failures. HookWorld/QuickHook's nil dictionary values and the null-fault cluster require provenance, not global acceptance of invalid state. |
| ABI and call machinery | Dev 1.45 added a timer initializer, dynamic method addition and time-zone abbreviation lookup; a follow-up extended the Objective-C host bridge to six parameters. Ski reached its menu; all three exposed later errors. | Inspect argument/return layouts, struct returns and dynamic signatures when a typed object is wrong. Compiler success and pure Rust tests do not exercise all guest ABI paths. |
| Diagnostic progress | Dev 1.46 replaced an opaque host-object failure with a pointer and requested host type. It also corrected failable UTF-8 initialization and added retained formatter properties. | The font diagnostic narrows the investigation but does not prove use-after-free; retained properties do not imply their behavioral effects are implemented. |

### Current three-game decision record

| Problem / apps | Subsystem and strongest evidence | Current code / implementation status | Competing hypotheses or missing proof | Next useful discriminating evidence |
|---|---|---|---|---|
| Riddim: missing `_CGPathAddArcToPoint` | Core Graphics, Dev 1.46 line 253; missing symbol is definite. Register dump has a null transform at this call. | `src/frameworks/core_graphics/cg_path.rs`, `CGPathAddArcToPoint`, `arc_to_points`; introduced by `d7f34ba1`. **Partial, approximate contract; device-unverified.** Existing consumers use move/line elements, so the new code flattens arcs. | It transforms control points then scales radius by the mean basis length. That does not implement a general affine transform for nonuniform scale/shear. Fixed angular sampling is not a pixel-error guarantee. Only quarter-circle endpoint and collinear fallback unit tests exist. Null-transform support may advance this call but cannot justify calling the whole primitive complete. | Validate both turn directions, degenerate/empty/subpath cases and nonuniform/reflected/sheared transforms; compare to a native reference where available. Check drawing and path-animation consumers together. Observe menu geometry and actual gameplay separately. |
| Ski Jumping: unregistered pointer used as font | UIKit/text plus possible object lifetime, dispatch or ABI. Dev 1.46 line 527 requests `UIFontHostObject` for `0x300154f0`; stack includes unnamed host functions. | `src/frameworks/uikit/ui_font.rs`, `font_properties`; introduced by `d7f34ba1`. **Provisional accommodation, not root-cause correction.** Four text helpers accept missing/wrong-type objects by substituting 17-point regular font. Instance font getters still use strict borrowing. | Was it allocated as a font and freed? Was the caller argument decoded incorrectly? Was an earlier object corrupted or reused? Was a UIKit owner missing a retain? An app-side bug is possible but not demonstrated. The comment asserting old-app dependence on cached-font lifetime is an unsupported hypothesis. Current `UILabel::setFont:` already retains, so adding a second retention scheme without tracing is unjustified. | Bounded allocation-generation and retain/release/deallocation records for fonts; text selector, decoded arguments, guest caller, thread and pool drain; safe object-table presence/type. Compare call ABI including struct returns before blaming the app. Never dereference an invalid pointer just to log its class. Record known font creation metadata so a fallback does not erase evidence. |
| Tesla Wars: unsupported `ccc` | Foundation formatting, Dev 1.46 line 825. Token parsing is definitely incomplete; caller expectations beyond this token are not yet established. | `src/frameworks/foundation/ns_date_formatter.rs`, `format_date` and `stringFromDate:`; properties added by `26f0a4a0`, token handling by `d7f34ba1`. **Partial contract, not full locale/time-zone support.** `stringFromDate:` ignores the stored properties and calculates UTC/en_US output. | Correctness needs locale, time zone, weekday boundaries and pattern-width behavior, not merely accepting the token. Longer weekday widths collapse to full names and `cc` is zero-padded by the current implementation. Historical target behavior still needs checking before a comprehensive correction. | Capture requested format, locale/time-zone identifiers and input timestamp once; test zone-crossing midnight, pre-reference dates, style changes and weekday widths using existing Foundation time/locale machinery. Confirm game startup after the narrow token call; do not infer complete formatting from that result. |

Pattern-width reference: the [Unicode date-field table](https://unicode.org/reports/tr35/tr35-dates.html#Date_Field_Symbol_Table) distinguishes numeric, abbreviated, wide, narrow and short stand-alone weekdays. It supports treating the current formatter as partial; the guest-era Apple/Unicode contract must still be checked rather than applying modern changes indiscriminately.

### Negative knowledge and guardrails

- **Font fallback:** do not expand this into global invalid-object tolerance. A later crash or visible menu is not evidence of correct ownership. Instrument first if lifetime/ABI remains ambiguous.
- **Blot:** repeated coordinate adjustments did not cure the original issue until later full framing; preserve that history. A thread-limit workaround did not establish the cause of later slowdown. Measure before adding another special case.
- **Bounce:** smoother gameplay and saved progress coexist with broken audio and secondary-menu crashes. Keep these independent test dimensions. Do not revert to success-with-silence merely to suppress a failure.
- **Null faults / nil dictionary values:** do not suppress the fault or accept nil insertions globally. Trace the upstream missing resource/API result and return-value handling. The shared symptom does not prove a shared cause.
- **Frequency sysctls:** the prior review deliberately retained the 64-bit contract instead of making a four-byte guest buffer appear successful. Do not revive that proposed shortcut absent contrary guest-era evidence.
- **AB Buster:** deliberately skipped; the same app copy stalls on original hardware. Do not interpret it as a clean emulator-only test case.
- **Inputs:** retain the explicit eight-encrypted and 26-unreadable archive sets separately. Do not reclassify other failures as encryption, or infer undamaged assets from successful ZIP opening.
- **Rogue Sky / giant logs:** callback repetition and log termination indicate a serious stability investigation, not proof of a watchdog cause or deadlock. Log throttling must retain event counts, and is not a substitute for repairing a loop.
- **No-panic sessions:** some wider-audit apps freeze or behave incorrectly despite no panic. Historical no-terminal-error counts are not a playable-games count.

### Verification limits that change future handoffs

The current `.github/workflows/iDared32bit_lint-test.yml` runs library unit tests on Windows/macOS for this development branch. Formatting checks, the guest integration harness and native Objective-C sanity checks are release-branch-only. Development lint is allowed to fail, and Android is lint-only. Thus a green workflow must not be summarized as strict lint plus full multi-platform/guest/device validation. Preserve recorded artifact/build success, but describe its actual coverage. Improving the guest-ABI test path is a separate engineering target; do not silently weaken checks to get a green badge.

No new runtime tests were run in this documentation review, and Dev 1.47 is not promoted to a device-validated checkpoint.

### Focused regression plan and immediate next decision

Keep the last delivered 1.47 artifact unchanged as a comparison checkpoint. For further changes, start with the evidence above, not another automatic three-error batch. Ski Jumping's invalid font is the strongest instrumentation-first candidate. Riddim's geometry needs contract-level tests; Tesla's formatter needs explicit supported semantics and relevant state wired through existing time/locale machinery. Do not assume the three must ship together.

| Change area | Minimum relevant checks beyond the target app |
|---|---|
| Object ownership / dispatch / ABI | Spoing, Diversion and Sticky menus and level transitions; Air Penguin world-to-level transition; Crazy Golf gameplay; repeated launch/exit. Add focused guest fixtures for affected signatures, retain/autorelease and teardown. |
| Text / path drawing | Riddim menu/input and geometry; Ski title text; Blot complete framing and touch mapping; compare screenshots before/after. Expand to all reference-app menus for shared text changes. |
| Date / serialization / persistence | Deterministic format and ownership tests; Spoing, Diversion, Sticky and Bounce existing-save read, new progress write, normal relaunch and crash recovery. Test relevant locale/time-zone boundary behavior without overwriting original saves. |
| Audio / scheduling / performance | Bounce gameplay and transitions, Diversion/Sticky transition audio, Blot prolonged session and death path, Spoing level-five completion; bounded timing/callback/log-volume measurements, not just subjective launch success. |
| Packaging / isolation | Same development bundle ID `com.bookieart1.idared32bit.dev`; stable development save location; separate App Store app and its data untouched. Verify artifact commit and report direct IPA download on requested build deliveries. |

For each subsequent fix append: problem, affected apps, subsystem, root-cause hypothesis and confidence, changed files/commit, provisional versus verified scope, predicted result, actual test result, regression exposure and remaining uncertainty. Reconsider the hypothesis when the result contradicts it. Do not overwrite older failed attempts; mark them superseded with the evidence that changed the decision.

## Dev 1.47 device results and Dev 1.48 decision — October 31, 2026

The user ran two complete Riddim → Ski Jumping → Tesla Wars sequences. The log embeds build `d7f34ba` and GitHub Actions run `41174826973`, so it is confirmed Dev 1.47 evidence. Both sequences reproduced the same decisive blockers.

| App | Dev 1.47 result | Evidence and interpretation |
|---|---|---|
| Riddim | Advanced to a later blocker; still not playable | It passed `_CGPathAddArcToPoint` and twice stopped at missing `_CGPathCloseSubpath` (new log lines 253 and 1102). This validates only that the prior call no longer aborts. It does not validate arc geometry or menu behavior. |
| Ski Jumping | Advanced through one helper, then exposed the same invalid object | The provisional fallback ran once (line 527), immediately followed by `orig_class != nil` in Objective-C dispatch (line 528). The receiver was `0x30015500`; the second launch reproduced the assertion with the same receiver address and registers (line 1376). This shows that substituting rendering properties did not repair the receiver stored in guest state. The unsupported WAV set and missing `start4.jpg` remain separate defects. |
| Tesla Wars | Advanced to a later formatter token; still no visible startup | It passed `ccc` and twice stopped at `LLL` (lines 829 and 1677). `LLL` is the standard stand-alone abbreviated month pattern. This is another incomplete formatter surface, not evidence that locale/time-zone behavior is now complete. |

### Dev 1.48 implementation boundary

- **Core Graphics, general implementation:** add `_CGPathCloseSubpath` with an actual close element (no points), current-point restoration to the subpath start, closing-segment consumption by the existing renderer, and closed-loop points for path animations. This follows the documented Core Graphics close semantics within touchHLE's existing flattened line renderer. It does not make Dev 1.47's approximate arc transforms complete.
- **Foundation formatting, partial but general implementation:** add `L` stand-alone month widths to the formatter's explicitly documented English/UTC model and correct the existing `E`/`c` abbreviated, wide, narrow and short widths. Locale-specific month forms and configured time-zone effects remain unimplemented and must not be claimed as solved.
- **UIKit/Objective-C, instrumentation-first:** remove the permissive invalid-font substitution. Record a bounded set of recently deallocated fonts and produce a precise tombstone or wrong-type diagnostic at the first invalid text operation. Improve nil-`isa` dispatch diagnostics with the receiver and selector. This intentionally seeks root-cause evidence; it is not predicted to make Ski Jumping playable in this build.

Predicted focused test:

1. Riddim: start gameplay if possible; capture geometry problems and first decisive blocker.
2. Ski Jumping: capture the exact font tombstone/unregistered/wrong-type message. No extended play is expected yet.
3. Tesla Wars: record whether visible startup begins and the next decisive blocker.

Regression exposure is concentrated in path drawing/animation and date strings. If target testing advances, run Riddim menu geometry, Blot framing/touch mapping, reference-app menus, and deterministic date-format unit tests. The Ski diagnostics do not alter valid-font behavior. No audio, persistence, save-container, orientation or packaging code is intentionally changed.

### Dev 1.48 build and verification record

- Primary implementation commit: `af42aeba01fa24a3416d9f936f0cf6c12e9c87eb` (`Implement Dev 1.48 path and formatter batch`).
- First verification attempt failed before compilation because the `objc_classes!` macro requires class methods before instance methods; the new `UIFont -dealloc` diagnostic had been placed before class methods. Failed runs: lint/test 13 (`41176741168`) and unsigned IPA 78 (`41176741192`). This was a source-order issue, not device evidence or a change in the implementation decision.
- Ordering correction and final build commit: `5300db7dda7a16439d50c3fda0b76e03daaf3595` (`Fix UIFont macro method ordering`). The diagnostic implementation was moved after the class methods without changing its behavior.
- GitHub lint/test run 14 (`41177206187`) completed successfully. Under the current development-branch workflow this covers Windows/macOS library tests and Android lint; it does not constitute physical-device, guest-harness or strict-formatting validation.
- GitHub unsigned-IPA run 79 (`41177206191`) completed successfully and produced exactly one artifact.
- Downloaded artifact ZIP SHA-256: `439E9D44F8A3D77FFDF99E0D37BF564AE73CAE0DB09EB67F3FC08C2A2E833A8C`, matching GitHub's recorded digest.
- Final IPA SHA-256: `A88FE1ECC0D540A48BC2842A8EF1C2885405FF4D830B34A6098521B4377CD397`.
- Package inspection found one `Payload/*.app`, one `Info.plist`, and no embedded provisioning profile. The binary plist contains the separate development identity `com.bookieart1.idared32bit.dev`, display name `iDared Dev`, and executable `iDared 32bit`.
- Direct device-testing IPA: `artifacts/1.48-5300db7d/iDared-Dev-1.48.ipa`.

## Dev 1.48 device results and Dev 1.49 decision — October 31, 2026

The user tested Riddim, Ski Jumping, and Tesla Wars on the delivered Dev 1.48
artifact. The supplied log embeds build `5300db7`, so it is confirmed Dev 1.48
evidence rather than an inferred version match.

| App | Dev 1.48 result | Evidence and interpretation |
|---|---|---|
| Riddim | Advanced to another Core Graphics blocker; still not playable | It passed the prior `_CGPathAddArcToPoint` and `_CGPathCloseSubpath` calls, then stopped at missing `_CGContextAddPath` (log line 253). This is evidence that execution moved later, not that the prior flattened arc geometry is correct. |
| Ski Jumping | Reached the same title/menu region and exposed a definite font-lifetime failure | The bounded tombstone diagnostic reports that receiver `0x300154f0` had actually been deallocated and was a 15-point `SansRegular` `UIFont` (line 527). This resolves the earlier wrong-type/ABI ambiguity in favor of a lifetime problem for this object. Unsupported WAV assets, missing `files_ipod/start4.jpg`, and four-byte buffers for 64-bit frequency sysctls remain separate warnings/defects. |
| Tesla Wars | Advanced to another standard formatter token before visible startup | It passed `ccc` and `LLL`, then stopped at Unicode date-format pattern `ZZ` (line 825). This is another partial `NSDateFormatter` surface, not evidence that locale or time-zone behavior is complete. |

The repeated corrupt-archive warnings in this log are the known app-picker
inputs and are not new failures in these three launches.

### Dev 1.49 implementation boundary

- **Core Graphics, general implementation:** `_CGContextAddPath` now copies the
  source path elements into the context's mutable current path without drawing
  or aliasing the source. Coordinates remain in user space for the existing
  context rendering path. This should remove Riddim's exact missing-symbol
  blocker, but it does not validate Dev 1.47's approximate arc flattening or
  imply that the menu/gameplay geometry is correct.
- **Foundation formatting, partial general implementation:**
  `NSDateFormatter` now applies its configured/default time-zone offset when
  deriving the local Gregorian date and supports Unicode `Z`, `ZZ`, `ZZZ`,
  `ZZZZ`, and `ZZZZZ` offset forms. Unit coverage includes UTC and a synthetic
  negative half-hour offset. `NSTimeZone` itself remains a fixed-offset GMT
  model, so regional zones and daylight-saving behavior remain outside this
  implementation's verified boundary.
- **UIKit lifetime, general compatibility cache with a bounded scope:** UIFont
  factory methods now return immutable flyweights held by a framework-owned
  retain while callers still receive normal autoreleased references. This
  repairs the demonstrated pool-drain lifetime gap instead of fabricating font
  properties after deallocation. It can help other older apps that retain
  system-font pointers the same way, but historical UIKit object-identity
  behavior has not been native-reference tested and the result remains
  device-unverified. The tombstone diagnostic remains active for genuinely
  deallocated fonts.

No audio decoding, persistence, orientation, viewport, input, save-container,
or app-picker identity code was intentionally changed. The focused device test
remains:

1. Riddim — reach the menu, try to start gameplay, and note geometry/input as
   well as the next decisive blocker.
2. Ski Jumping — confirm whether it remains beyond the title/menu transition;
   separately note missing/distorted audio rather than conflating it with a
   crash.
3. Tesla Wars — record the first visible screen and first decisive blocker.

If these targeted calls advance, the relevant regression set is Riddim menu
geometry and input, Blot framing/touch mapping, Ski and reference-app text,
deterministic date strings, and existing saved-progress relaunches. The separate
App Store iDared installation remains outside the development bundle and must
not be altered.

### Dev 1.49 build and verification record

- Clean source baseline: `5300db7dda7a16439d50c3fda0b76e03daaf3595`.
- Final implementation commit: `9cb9939513c727b7a8adc0b86083bcff04ec2d79`
  (`Implement Dev 1.49 path, date, and font lifetime batch`).
- GitHub lint/test run 15 (`41178310650`) completed successfully: macOS and
  Windows library-test jobs passed and the Android job completed successfully.
  Development-branch lint remains configured as allowed-to-fail, and the guest
  integration harness, release-only formatting checks, native Objective-C
  sanity test, and physical-device behavior are not covered by this result.
- GitHub unsigned-IPA run 80 (`41178310643`) completed successfully and
  produced exactly one artifact.
- Downloaded artifact ZIP SHA-256:
  `14BA4C89FAA4945D3C5C3B7801D565431784BDA684768DB6EDE55F3C4DB25BDF`,
  matching GitHub's recorded digest.
- Final IPA SHA-256:
  `4353DF2F6D19C3F446291FEFEA42511E111865234757F3747AC9D704E02EC084`.
- Package inspection found one `Payload/*.app` bundle, one `Info.plist`, and no
  embedded provisioning profile. The plist contains bundle identifier
  `com.bookieart1.idared32bit.dev`, display name `iDared Dev`, and executable
  `iDared 32bit`.
- Direct device-testing IPA:
  `artifacts/1.49-9cb99395/iDared-Dev-1.49.ipa`.

## Dev 1.49 device results and Dev 1.50 decision — October 31, 2026

The supplied device log identifies build `9cb9939`, matching the delivered
Dev 1.49 artifact. The user's accompanying Tesla Wars screenshot confirms that
the host alert displayed its title and buttons but omitted the guest text
field that should have appeared between them.

| App | Dev 1.49 result | Evidence and interpretation |
|---|---|---|
| Riddim | Advanced to a later Core Graphics blocker; still not playable | It passed `_CGContextAddPath` and then called missing `_CGContextClip` (line 253). The current path and context registers are non-null. This requires persistent clipping state and nonzero-winding behavior, not a no-op symbol stub. |
| Ski Jumping | Advanced past the font lifetime failure to a standard OpenGL ES extension query | It reached `_glGetBufferPointervOES` with `GL_ARRAY_BUFFER`, `GL_BUFFER_MAP_POINTER_OES`, and a valid output address (lines 529–532). The existing map-buffer shadowing also contained a real binding-query bug: it queried vertex-array client state and wrote through a temporary, effectively keying mappings as buffer zero. Unsupported WAV files, the missing `start4.jpg`, and undersized frequency-sysctl buffers remain secondary. |
| Tesla Wars | Newly partially functional; menu reached quickly, but new-profile entry is unusable | The game repeatedly creates a `UITextField`, makes it active, and shows `UIAlertView` titled “Enter Your Name” (lines 840–854). The modern keyboard appears, but the SDL message-box replacement discards the alert's guest subviews and blocks the guest event loop, so there is no visible field and keystrokes cannot update it. On exit the SDL input responder remains active and the keyboard covers the native app picker. There is no game panic in this launch. |

### Dev 1.50 implementation boundary

- **Core Graphics, general clipping implementation:** represent every clip
  operation as transformed device-space subpaths, apply the nonzero winding
  rule, intersect successive clips, include clipping state in save/restore,
  implicitly close open subpaths, and reset the current path after
  `CGContextClip`. Apply the same state to bitmap fills, images and glyphs.
  This follows Core Graphics' documented contract within touchHLE's existing
  flattened-line renderer; it does not repair the earlier approximate
  tangent-arc transform model.
- **OpenGL ES, general OES map-buffer correction:** implement
  `glGetBufferPointervOES` using the guest-visible shadow allocation, return
  null when the current buffer is not mapped, and correct the shared binding
  lookup to query `GL_ARRAY_BUFFER_BINDING`/`GL_ELEMENT_ARRAY_BUFFER_BINDING`
  into actual mutable storage. This changes Map, Get and Unmap together so
  their buffer identity is coherent.
- **UIKit/iOS host integration, general legacy alert text-entry bridge:** when
  an old `UIAlertView` contains guest `UITextField` subviews, present a native
  text-capable alert on iOS, transfer initial text/placeholder/security state,
  write the submitted text back to the guest fields, end guest editing, then
  deliver the original button callbacks. Ordinary alerts continue to use the
  existing message box. The native app picker also forcibly resigns any stale
  host first responder so a game keyboard cannot leak into it.

Predicted focused device test for Dev 1.50:

1. Riddim — note menu geometry, whether controls render correctly, and the
   first decisive blocker after clipping.
2. Ski Jumping — note title/menu stability, then attempt to enter gameplay;
   record visual behavior separately from known missing audio/assets.
3. Tesla Wars — choose New, type a visible name, submit it, and attempt to
   start the game; also exit once while the keyboard is visible and confirm
   that the app picker is unobstructed.

Regression exposure is concentrated in bitmap rendering/clipping, mapped VBO
state, and legacy text alerts. If the targets advance, retest Riddim and Blot
rendering/touch alignment, Ski Jumping and one known OpenGL title, Tesla's
cancel/submit/exit paths, one ordinary non-text alert, and app-picker keyboard
cleanup. Audio and persistence code, the development bundle ID, save-container
isolation, and the separate App Store iDared installation are not intentionally
changed.

### Dev 1.50 build and verification record

- Clean source baseline: `9cb9939513c727b7a8adc0b86083bcff04ec2d79`.
- Final implementation commit: `0781c5c6b52d9da28570bad9969be85718e545b9`
  (`Implement Dev 1.50 graphics, GL, and text input batch`).
- GitHub lint/test run 16 (`41179847603`) completed with all three platform
  jobs green. The macOS and Windows library-test steps passed; development-
  branch lint returned nonzero but remains explicitly allowed to fail, so this
  is not evidence of a clean strict-lint pass. Physical-device behavior and
  the native alert's end-to-end text transfer remain device-test requirements.
- GitHub unsigned-IPA run 81 (`41179847647`) completed successfully and
  produced exactly one artifact.
- Downloaded artifact ZIP SHA-256:
  `7998B9E95140008833B555942805F033AE2F93A5C60E4B085285BEF7FC5E0E5A`,
  matching GitHub's recorded artifact digest.
- Final IPA SHA-256:
  `8745CCB2F1EE87249E819A0C9E2DF1F655ABFAAC1E7423D059FD40B912CFC332`.
- Local package inspection found 36 archive entries, exactly one guest app
  bundle and `Info.plist` (`Payload/iDared 32bit.app/Info.plist`), and no
  embedded provisioning profile. The successful workflow's identity check
  verified bundle identifier `com.bookieart1.idared32bit.dev`, display name
  `iDared Dev`, and the unsigned development package boundary.
- Direct device-testing IPA:
  `artifacts/1.50-0781c5c6/iDared-Dev-1.50.ipa`.

## Dev 1.50 device results and Dev 1.51 decision — October 31, 2026

All three supplied device logs identify build `0781c5c`, matching Dev 1.50.
The repeated Tesla Wars launches are not contradictory: they expose three
different timing-dependent failure paths around the same native text-entry
transition.

| App | Dev 1.50 result | Evidence and interpretation |
|---|---|---|
| Riddim | Advanced through the clipping change to its menu, then crashed on Play | The first decisive blocker is Objective-C class lookup for `UITableViewController`, specifically its inherited `+initialize` path. The game now requires a functional table-controller class rather than another graphics bypass. |
| Ski Jumping | Advanced through `glGetBufferPointervOES` to the title/menu, then crashed on Play | The first decisive blocker is `UITextField` not responding to `font`. This is a missing UIKit property-forwarding behavior. Repeated WAV-open failures and missing `files_ipod/start4.jpg` remain secondary and may still affect audio or later rendering after this crash moves. |
| Tesla Wars, launch 1 | Native name alert appeared, then the entire iDared host disappeared without a guest panic | The log ends immediately after presenting the alert. This is consistent with the Swift bridge's process-fatal `precondition(Thread.isMainThread)` being reached from a guest worker thread. It is a host-integration failure, not proof of a Tesla guest crash. |
| Tesla Wars, launch 2 | Name field appeared and accepted typing; submitting it crashed the guest | The decisive panic is `Attempt to increment refcount on static-lifetime object`. The runtime was treating an Objective-C class/constant object like an ordinary heap object. |
| Tesla Wars, launch 3 | Failed before the name alert | The decisive blocker is missing `-[NSDate dateByAddingTimeInterval:]`, a standard Foundation alias absent from the current implementation. |

### Dev 1.51 implementation boundary

- **UIKit, real table-controller support:** register
  `UITableViewController : UIViewController`; support `allocWithZone:`,
  `initWithStyle:`, a real `UITableView` root view, controller data-source and
  delegate wiring, `tableView` replacement, and
  `clearsSelectionOnViewWillAppear`. This is general UIKit behavior intended
  for Riddim and any other legacy app using table controllers, not a Riddim
  name check.
- **UIKit, text-field font behavior:** expose `-[UITextField font]` through
  the existing backing label, matching the already implemented setter. This
  addresses Ski Jumping's exact selector failure without changing audio or
  game assets.
- **Foundation, date API completeness:** implement
  `-[NSDate dateByAddingTimeInterval:]` as the modern spelling of the existing
  date-offset behavior.
- **Objective-C lifetime correctness:** make retain, release and autorelease
  no-ops for static-lifetime objects, with an effectively immortal retain
  count. Classes and constant/static objects must not enter ordinary heap
  reference-count accounting; this fixes Tesla's exact panic at the runtime
  boundary rather than bypassing a Tesla call site.
- **Native alert thread safety:** remove the process-killing main-thread
  precondition. Copy the guest JSON before crossing threads and synchronously
  present the native alert on the main queue when invoked off-main. The helper
  retains a main-queue dispatch precondition internally so actual UIKit work
  remains constrained to the correct thread.

Focused Dev 1.51 device test:

1. **Riddim** — choose Play from the menu and report table/menu geometry,
   input, and the next decisive blocker.
2. **Ski Jumping** — choose Play and report whether gameplay begins; record
   missing or distorted audio and missing imagery separately from a crash.
3. **Tesla Wars** — launch at least three times because Dev 1.50 proved the
   transition is timing-sensitive. Test name submit once, Cancel once, and
   exit once while the alert or keyboard is active. Record whether the entire
   host still disappears and whether the app picker is left with a keyboard.

The regression boundary is table-backed UIKit screens, text-field appearance,
date offset callers, Objective-C static objects, and native alert threading.
No graphics, OpenGL buffer, audio, save-container, game-persistence, app
identity, or separate App Store iDared code was intentionally changed.

### Dev 1.51 build and verification record

- Clean source baseline: `0781c5c6b52d9da28570bad9969be85718e545b9`.
- Final implementation commit: `c62a893d24c72697edb97dfa7ae8141d44705263`
  (`Implement Dev 1.51 UIKit and lifetime batch`).
- GitHub lint/test run 17 (`41181083600`) completed successfully in 3m35s.
  The macOS, Windows and Android jobs are green. As in prior development
  builds, their lint steps returned exit code 101 but are explicitly
  allowed-to-fail; this is not a clean strict-lint result. The library-test
  jobs completed, but physical-device UIKit, guest-thread alert timing, and
  end-to-end gameplay remain device-test requirements.
- GitHub unsigned-IPA run 82 (`41181083585`) completed successfully and
  produced exactly one artifact,
  `iDared32bit-dev-unsigned-c62a893d24c72697edb97dfa7ae8141d44705263`.
- Downloaded artifact ZIP SHA-256:
  `272086873B1EF7E8F3ABFEB01857C2E6E739691DC2FAADFEF791BA3FB64D7F4B`,
  matching GitHub's recorded artifact digest.
- Final IPA SHA-256:
  `39EE0822E02F25B3D84371C8622C9A8F110D2B4AC0C3DCB0AD29EC4C0224138A`.
- Local package inspection found 36 archive entries, exactly one guest app
  bundle and `Info.plist` (`Payload/iDared 32bit.app/Info.plist`), and no
  embedded provisioning profile. The successful workflow verified the
  development bundle identity (`com.bookieart1.idared32bit.dev`, display name
  `iDared Dev`) and preserves separation from the App Store installation.
- Direct device-testing IPA:
  `artifacts/1.51-c62a893d/iDared-Dev-1.51.ipa`.

## Dev 1.51 device results and Dev 1.52 decision — October 31, 2026

The supplied log identifies build `c62a893` and contains five launches:
Riddim, Ski Jumping, and three repeat launches of Tesla Wars. All three apps
advanced beyond their Dev 1.50 blockers, but none is yet confirmed compatible.

| App | Dev 1.51 result | Evidence and interpretation |
|---|---|---|
| Riddim | Advanced from Play to track/store selection; Back then crashed | The game displayed its track-choice screen and its own “Connection Required” message. No bundled tracks were visible, so downloadable content may be an unavailable external-service dependency. The immediate emulator blocker on Back is missing `-[UIWebView stringByEvaluatingJavaScriptFromString:]`, not the connection warning itself. |
| Ski Jumping | Advanced to character/outfit selection; Next → Tutorial crashed | The decisive blocker is missing `-[NSString getBytes:maxLength:usedLength:encoding:options:range:remainingRange:]`. Repeated WAV-open failures and missing `files_ipod/start4.jpg` remain secondary and may become visible after the string-conversion blocker moves. |
| Tesla Wars, three launches | Consistently skipped name entry, reached a new still/loading screen, then crashed | Every launch fails on thread 1 by sending `initWithTimeIntervalSinceNow:` to an object whose `isa` has become nil. Combined with Dev 1.50's refcount panic immediately after submitting the native alert, this strongly indicates delayed-callback object-lifetime corruption rather than a missing NSDate selector. |

### Dev 1.52 implementation boundary

- **Objective-C runtime, lifetime-state correctness:** store an object's
  static/immortal status independently from its current reference count. A
  normal object whose final release has set its refcount to zero is no longer
  misclassified as a static object during its deallocation chain.
- **UIKit, alert presentation ownership:** retain `UIAlertView` before
  entering synchronous host UI, then release it after the delayed delegate
  callback (or immediately when dismissal produces no callback). This models
  UIKit's presentation ownership and prevents the guest autorelease pool from
  destroying the alert while the native prompt is blocking/yielding.
- **Foundation, range-aware string conversion:** implement
  `getBytes:maxLength:usedLength:encoding:options:range:remainingRange:` with
  range validation, partial writes, remaining-range reporting, lossless/lossy
  handling, UTF-8/16/32 and legacy encodings, and surrogate-pair-safe buffer
  boundaries. This is a general NSString capability, not a Ski Jumping
  accommodation.
- **UIKit, no-document UIWebView JavaScript behavior:** implement
  `stringByEvaluatingJavaScriptFromString:` and return `nil` when there is no
  loaded web document. touchHLE still has no embedded web engine and cannot
  restore Riddim's remote track store; the change supplies the historical
  failure result rather than fabricating JavaScript output.

Focused Dev 1.52 device test:

1. **Riddim** — choose Play, dismiss the connection/store message, then use
   Back. Report whether Back returns safely. Absence of downloadable tracks
   remains an external-content limitation even if the crash is gone.
2. **Ski Jumping** — proceed through customization, Next, and Tutorial. Record
   the next blocker and separately note missing images or broken/missing audio.
3. **Tesla Wars** — launch three times because the prior failure was
   timing-sensitive. Report whether the loading screen advances; if the name
   prompt reappears, test one submit and one Cancel path and note any host-app
   disappearance.

The highest regression exposure is Objective-C object teardown and legacy
alert callbacks. Retest one ordinary non-text alert if any target behaves
unexpectedly. Graphics, OpenGL buffer handling, audio, save containers,
development bundle identity, and the separate App Store installation were not
intentionally changed.

### Dev 1.52 build and verification record

- Clean source baseline: `c62a893d24c72697edb97dfa7ae8141d44705263`.
- Primary implementation commit: `34d323fe3675e6827c2aeaf2e9f0bb658a32b4d8`
  (`Implement Dev 1.52 text, web view, and alert lifetime batch`). Its first
  CI attempt correctly rejected two compile-time integration details: the
  host Objective-C method bridge generated signatures only through six
  explicit parameters, and a host `usize` needed conversion to guest `u32`.
- Final build commit: `d77df52f6b6d8e63b633ae68fb4e88569ebe709c`
  (`Fix Dev 1.52 host method ABI compilation`). The correction extends the
  general host-method bridge to seven parameters and uses checked guest-size
  conversion; it does not reduce the NSString implementation to a stub.
- GitHub lint/test run 19 (`41182634306`) completed successfully. macOS,
  Windows and Android jobs are green; all three lint steps and both library-
  test jobs completed successfully.
- GitHub unsigned-IPA run 84 (`41182634276`) completed successfully and
  produced exactly one artifact,
  `iDared32bit-dev-unsigned-d77df52f6b6d8e63b633ae68fb4e88569ebe709c`.
- Downloaded artifact ZIP SHA-256:
  `25A3A1A145A5BFCFA1D71AAB5E4D4435BBCA7B52468F0661C7D3176E0E25866C`,
  exactly matching GitHub's recorded artifact digest.
- Final IPA SHA-256:
  `5182BBAF98F99D7E35CE0E5CCBD05B456F9A487898957A511AB5051BEA13108A`.
- Local package inspection found 36 archive entries, exactly one guest app
  bundle and `Info.plist` (`Payload/iDared 32bit.app/Info.plist`), no embedded
  provisioning profile and no code-signature directory. The successful
  workflow also verified the separate development bundle identity and
  unsigned packaging boundary.
- Direct device-testing IPA:
  `artifacts/1.52-d77df52f/iDared-Dev-1.52.ipa`.

## Dev 1.52 device results and Dev 1.53 decision — October 31, 2026

The supplied log identifies build `d77df52` and contains one Riddim launch,
one Ski Jumping launch, seven Tesla Wars launches, and one incidental Ramps
launch. Ski Jumping is the first title in this focused batch to reach an
apparently playable state. Riddim and Tesla Wars now expose two different
UIKit/runtime problems.

| App | Dev 1.52 result | Evidence and interpretation |
|---|---|---|
| Riddim | Reached its track/store screen; Back still crashed | The no-document `UIWebView` query now returns safely. Execution proceeds to `-[TracksViewController isViewLoaded]`, which fails because touchHLE lacks the standard inherited `UIViewController` query. The external track-store limitation remains separate. |
| Ski Jumping | Smooth gameplay, good frame rate, extensive interaction, no crash | The launch ended normally through iDared's on-screen Home button rather than a guest panic. This is **apparently playable**, not yet proof of complete compatibility. The log still reports multiple undecoded WAV files and missing optional/localized image paths, although the tester did not report an audible or visible defect in this run. |
| Tesla Wars | Seven launches reached the main menu; Play and Back from several submenus crashed | All seven paths converge on `Attempt to increment refcount on zero-refcount object` on guest thread 1. The repeatable object address changes with allocation layout, and the current pointer-only panic does not identify its class. This strongly supports one shared use-during-deallocation defect rather than separate menu implementations. The missing name prompt may simply reflect already-saved profile state, so this run does not independently validate or falsify alert presentation. |
| Ramps (incidental) | Did not start | The unchanged blocker is unsupported startup orientation `LandscapeRight`; it was not part of the requested focused test. |

### Dev 1.53 implementation boundary

- **UIKit, view-controller lifecycle completeness:** implement
  `-[UIViewController isViewLoaded]` as a side-effect-free query of the
  controller's current root-view state. It must not call `-view`, load a nib,
  or invoke `viewDidLoad`. This is standard UIKit behavior and is not keyed to
  Riddim.
- **Objective-C runtime, actionable lifetime diagnostics:** retain the strict
  rejection of attempts to resurrect a dynamic zero-refcount object, but add
  the object's Objective-C class and host-object type to the panic. Allowing
  the retain would hide a use-during-deallocation error and risk corrupting
  arbitrary games. The next Tesla log should identify the ownership subsystem
  that needs a semantic correction.
- **No Ski Jumping change:** preserve the newly working gameplay path. Its
  audio-decoding and missing-path warnings remain ledgered as secondary until
  a user-visible defect is reproduced.

Focused Dev 1.53 device test:

1. **Riddim** — use Back from the track/store screen. Report whether it
   returns to the preceding screen and, if it advances, exercise Play once.
2. **Tesla Wars** — one Play attempt and one submenu → Back attempt are enough.
   The new panic should name the zero-refcount object's class; repeated launches
   are unnecessary unless the two actions produce different named classes.
3. **Ski Jumping** — a brief launch and one jump is a regression check only;
   another long play session is not required for this batch.

The separate App Store iDared identity, save containers, graphics pipeline,
audio implementation, alert UI, and persistence code are unchanged in Dev
1.53.

### Dev 1.53 build and verification record

- Clean source baseline: `d77df52f6b6d8e63b633ae68fb4e88569ebe709c`.
- Primary implementation commit: `6237c55104ea15913150ac1df8156e97dcf70fea`
  (`Implement Dev 1.53 view lifecycle and lifetime diagnostics`). Its first CI
  attempt exposed a format-string syntax error confined to the new diagnostic.
- Final build commit: `36ed596ef7b0bbd53da3eda82b6173d5649ebafd`
  (`Fix Dev 1.53 diagnostic formatting`).
- GitHub lint/test run 21 (`41184344003`) completed successfully: macOS,
  Windows and Android jobs are green.
- GitHub unsigned-IPA run 86 (`41184344045`) completed successfully and
  produced exactly one artifact,
  `iDared32bit-dev-unsigned-36ed596ef7b0bbd53da3eda82b6173d5649ebafd`.
- Downloaded artifact ZIP SHA-256:
  `68CAEE656AF4A87D7DD0670E7CD92F4C7A26328BD707910A7AC0C857819B9590`,
  exactly matching GitHub's recorded artifact digest.
- Final IPA SHA-256:
  `3A115280D8F0CFAD26CB429DE5AB802BEDE6689B3F75BC7BF440C790752503BA`.
- Local package inspection found 36 archive entries, exactly one guest app
  bundle and `Info.plist` (`Payload/iDared 32bit.app/Info.plist`), no embedded
  provisioning profile, and no code-signature directory. The successful
  workflow verified the separate development bundle identity and unsigned
  packaging boundary.
- Direct device-testing IPA:
  `artifacts/1.53-36ed596e/iDared-Dev-1.53.ipa`.

## Dev 1.53 device results and Dev 1.54 decision — October 31, 2026

The supplied 1,893-line device report identifies build `36ed596` and contains
one Riddim launch, four Tesla Wars launches, and one Ski Jumping launch. The
Dev 1.53 diagnostics disproved the earlier idea that Tesla's crash involved one
particular menu object: the same lifetime failure follows whichever screen is
active on the worker thread.

| App | Dev 1.53 result | Evidence and interpretation |
|---|---|---|
| Riddim | Back navigation now works smoothly; Options reaches a later UIKit blocker | The launch advances to `-[UIScrollView setCanCancelContentTouches:]` (device log line 303). This is a standard `UIScrollView` Boolean property, not an app-specific selector. Internet-required/store paths remain an external-content limitation. |
| Tesla Wars | Name entry works, but Play and submenu Back paths still crash | Four independent launches fail while retaining the active `MainMenu`, `Options`, `Help`, or `Profiles` object (lines 660, 987, 1306, and 1632). Every failure occurs on guest thread 1 at the same retain-on-zero site. The common pattern strongly supports a cross-thread ownership/order problem rather than four broken menu implementations. The current log does not reveal which thread performed the final release or whether the screen object crossed a raw pthread boundary without ownership. |
| Ski Jumping | Remains smooth and working in the tested scope | There is no guest panic. The launch ends through the ordinary app-resign path (lines 1892–1893). Existing WAV decode and missing optional/localized asset warnings remain secondary because no matching user-visible defect was reported. |

### Dev 1.54 implementation boundary

- **UIKit, scroll-view property state (partial contract):** implement the
  historical Boolean state and defaults for `delaysContentTouches`,
  `canCancelContentTouches`, and `bounces`, including correctly typed getters
  and setters. This removes Riddim's decisive Options blocker without
  pretending that touch-delay, cancellation, or rubber-band animation are
  fully modeled by the current rudimentary scroll implementation.
- **Objective-C lifetime instrumentation (diagnostic-only):** retain the strict
  prohibition on resurrecting a dynamic object after its final release.
  Record the guest thread plus PC/LR at the transition to zero and report them
  if another retain races the deallocation chain. This distinguishes same-thread
  reentrancy from a main-thread-to-worker lifetime race.
- **pthread ownership instrumentation (diagnostic-only):** when a raw pthread
  is created with a registered Objective-C object as its untyped context,
  record the new thread, entry point, object and class once. This is bounded at
  thread creation and will determine whether Tesla relies on an unretained
  Objective-C context crossing. It does not retain the pointer or alter
  scheduling, either of which would be an unsafe guess before provenance is
  known.
- **No Ski Jumping change:** preserve the working gameplay path and use it only
  as a brief regression check.

Focused Dev 1.54 device test:

1. **Riddim** — open Options, interact with one control, then use Back. If it
   advances, report the first later failure or non-crash defect.
2. **Tesla Wars** — one Play attempt plus one submenu → Back attempt. Two
   launches are sufficient if they produce the expanded ownership diagnostic;
   repeated launches across every menu are unnecessary.
3. **Ski Jumping** — launch and perform one jump as a UIKit/runtime regression
   check only.

The separate App Store iDared identity, development save container, graphics
pipeline, audio implementation, alert input and persistence code are unchanged
in this batch.

### Planned pause and next scouting batch

After one focused Dev 1.54 check, the user elected to pause the Riddim/Tesla
Wars blocker chain and explore a fresh cross-application batch.

| Paused app | Durable resume status |
|---|---|
| Riddim | Partial. Menu and Back navigation are smooth on Dev 1.53, but several track/store paths depend on unavailable remote content. Options currently stops at `setCanCancelContentTouches:`; Dev 1.54 implements that standard property and is awaiting device verification. Resume only if local/bundled gameplay can be distinguished from the discontinued service path. |
| Tesla Wars | Partial. Menu, legacy name alert and typing work, but Play and submenu Back paths fail in the same worker-thread retain-on-zero pattern for the active screen object. Dev 1.54 adds final-release and raw-pthread provenance without altering ownership. Resume from one Play and one submenu/Back trace rather than repeating every menu. |

The recommended next **scouting** batch is alphabetical and intentionally uses
apps whose Dev 1.42 first blockers are already implemented by later general
work, so it can reveal the current landscape without writing speculative code:

1. **Sandwiches** (IPA `Bring_Me_Sandwiches_v2_1.ipa`, display name
   `Sandwiches!!`) — retest its old `NSDictionary allValues` blocker.
2. **SimplePhysics** — retest its old `NSString initWithContentsOfURL:`
   blocker.
3. **Spirits** — its old inherited `UIViewController isViewLoaded` blocker is
   implemented in Dev 1.53.
4. **Splode** — its old `NSTimeZone timeZoneWithAbbreviation:` blocker and
   several adjacent date-format tokens were implemented during the current
   Foundation work.

This is a discovery pass, not a claim that any of the four is now playable.
Capture the first decisive blocker and any visible rendering, input, audio,
network or persistence behavior for each. The resulting blockers should decide
the next coherent engineering batch; known-unimplemented apps need not be
retested merely to reproduce the same selector.

### Dev 1.54 build and verification record

- Clean source baseline: `36ed596ef7b0bbd53da3eda82b6173d5649ebafd`.
- Final repository commit: `fd10c9de5b54e76c881b4d8cd8a15f8c18f477c0`
  (`Implement Dev 1.54 scroll state and ownership tracing`). The working tree
  is clean after the push.
- GitHub unsigned-IPA run 87 (`41185462025`) completed successfully and
  produced exactly one artifact,
  `iDared32bit-dev-unsigned-fd10c9de5b54e76c881b4d8cd8a15f8c18f477c0`.
- Downloaded artifact ZIP SHA-256:
  `A55F44F8FAC8715D316094810213BB19DF97D58BAEF0280F88233FD010D499F`,
  exactly matching GitHub's recorded artifact digest.
- Final IPA SHA-256:
  `6676040FB67DD8C23EB749A98558ACE522820C0B613B713F5108CC7C0A4A84FD`.
- Local package inspection found 36 entries, exactly one guest app bundle and
  `Info.plist` (`Payload/iDared 32bit.app/Info.plist`), no embedded provisioning
  profile and no code-signature directory. The local plist contains the
  separate development bundle identity `com.bookieart1.idared32bit.dev`,
  display name `iDared Dev`, and executable `iDared 32bit`; the packaged
  executable contains commit `fd10c9d`.
- GitHub lint/test run 22 (`41185462063`) completed successfully. The macOS and
  Windows library-test jobs passed. As at earlier checkpoints, development
  clippy on macOS, Windows and Android emitted historical diagnostics and
  exited 101 under the workflow's explicit `continue-on-error`; this is not a
  strict lint pass, and Android remains lint-only.
- Direct device-testing IPA:
  `artifacts/1.54-fd10c9de/iDared-Dev-1.54.ipa`.

## Dev 1.54 scouting result and Dev 1.55 Foundation/media contract batch

The four-app scouting log is from the expected Dev 1.54 executable
`fd10c9d` (workflow run `41185462025`) and contains four launch segments.
The corrected user-facing name of the first app is **Sandwiches**; retain its
IPA filename and bundle ID as identity evidence, but do not label it “Bring Me
Sandwiches” in future test instructions.

| App | Dev 1.54 result | First decisive blocker / evidence |
|---|---|---|
| Sandwiches | Crashes during startup before usable UI | Immutable `_touchHLE_NSDictionary` does not respond to `allValues` (scouting log lines 32–181; panic line 142). The existing implementation was mistakenly located only on `_touchHLE_NSMutableDictionary`, so the prior ledger claim that this immutable path was implemented was incorrect. |
| SimplePhysics | Crashes during startup after its main NIB and settings probe | `_touchHLE_NSString` does not respond to the deprecated one-argument `initWithContentsOfURL:` (lines 182–411; panic line 341, worker thread 1). The current code had only the encoding/error overload, so the prior ledger claim that this exact selector was implemented was incorrect. |
| Spirits | Opens; selecting Play reaches the intro-movie path, then crashes | It advances beyond the old `isViewLoaded` blocker and constructs an `MPMoviePlayerController` for `intro_iphone.m4v` (line 512), then guest code dereferences nil (lines 514–516). The strongest current hypothesis is the emulator's explicitly nil `MPMoviePlayerController.view`; the current log cannot prove which selector returned nil. |
| Splode | Opens and loads substantially farther, but crashes before its menu | `NSFileHandle` does not respond to `truncateFileAtOffset:` (lines 561–1422; panic line 1321). This is standard file-handle behavior and maps directly to the emulator's existing POSIX `ftruncate` and guest filesystem machinery. |

### Dev 1.55 implementation boundary

- **NSDictionary contract (general, semantically complete for this method):**
  provide a generic `allValues` implementation on the abstract `NSDictionary`
  class using `keyEnumerator` and `objectForKey:`. This covers immutable
  touchHLE dictionaries and genuine guest subclasses without assuming the
  private host-map representation; the mutable concrete class may retain its
  optimized override.
- **NSString historical URL loading (general, bounded):** implement both the
  deprecated class convenience method and one-argument initializer. Loading is
  delegated to `NSData`; BOM/default-encoding behavior matches the existing
  file initializer. Unsupported remote schemes still fail with `nil` rather
  than being falsely reported as working network I/O.
- **NSFileHandle truncation (general, semantically complete within current file
  error handling):** implement `truncateFileAtOffset:` through the existing
  guest `ftruncate` machinery, preserving the current file position and adding
  invalid-descriptor/negative-length error handling to the shared POSIX path.
- **MPMoviePlayerController view lifecycle (general partial contract):** replace
  the incorrect always-`nil` `view` property with a lazily created,
  screen-framed, retained `UIView`. Video decoding remains unimplemented; this
  only establishes the UIKit object/lifetime contract used while configuring
  playback.
- **Memory-fault diagnostics (diagnostic-only):** for a fatal guest memory
  fault, report registered Objective-C objects in R0–R3 and decode R1 when it
  is a selector string. The log is emitted only at the terminal fault and does
  not weaken memory checks or change execution.

Expected immediate outcomes are **advancement**, not proven playability:
Sandwiches, SimplePhysics and Splode should pass their exact recorded selectors;
Spirits may pass the intro setup if the nil movie view is causal, and otherwise
the next log should identify the relevant selector/object register. These APIs
are shared Foundation/MediaPlayer contracts and are not bundle-ID special
cases, but this dataset only directly proves one known app at each decisive
blocker.

Focused Dev 1.55 device test:

1. **Sandwiches** — launch through the first screen and report the next blocker
   or any visible menu.
2. **SimplePhysics** — launch and report whether a menu/gameplay screen appears.
3. **Spirits** — select Play once; if it still faults, preserve the new
   Objective-C candidate and selector-candidate lines.
4. **Splode** — wait through loading to its first interactive screen.
5. **Regression checks:** briefly launch Spoing and one intro/movie-heavy title
   (Diversion or Sticky); verify one existing saved-progress relaunch. A full
   replay of Air Penguin, Blot and Bounce is unnecessary unless a shared
   Foundation, movie, filesystem or persistence regression appears.

The separate App Store iDared bundle identity and container, graphics and audio
pipelines, and existing development saves are not changed by this batch.

### Dev 1.55 build and verification record

- Final repository commit: `fe72e52ba6eb9d0a71c3fab26522152a0886add8`
  (`Implement Dev 1.55 Foundation and movie contracts`). The branch was clean
  at the commit and pushed to `origin/ci/unsigned-ipa-baseline`.
- GitHub unsigned-IPA run 88 (`41187026689`) completed successfully and
  produced exactly one artifact,
  `iDared32bit-dev-unsigned-fe72e52ba6eb9d0a71c3fab26522152a0886add8`.
- Downloaded artifact ZIP SHA-256:
  `6E5B81F227B4A1FEF03F3570A31A545E345CCCA121902BD2624C5D2C9CB1F904`,
  exactly matching GitHub's recorded artifact digest.
- Final IPA SHA-256:
  `69DFADA623FDD48C88D75FCB101C5923F977A6995D2EBE0B7D37744D8C40D77C`.
- Local package inspection found 36 entries, exactly one app bundle and
  `Info.plist`, no embedded provisioning profile and no code-signature
  directory. The plist contains the separate development identity
  `com.bookieart1.idared32bit.dev` and display name `iDared Dev`; the packaged
  executable contains commit prefix `fe72e52`.
- GitHub lint/test run 23 (`41187026633`) completed successfully. macOS and
  Windows library tests passed. macOS, Windows and Android lint steps also
  passed; full integration tests and the native macOS Objective-C sanity step
  remain intentionally skipped on the development branch by workflow design.
- Direct device-testing IPA:
  `artifacts/1.55-fe72e52b/iDared-Dev-1.55.ipa`.

## Dev 1.55 device result and Dev 1.56 request/linker/archive batch

Both returned logs embed the expected Dev 1.55 commit `fe72e52` and workflow
run `41187026689`. The Sandwiches log contains one launch; the combined log
contains exactly one launch each for SimplePhysics, Spirits and Splode.

| App | Dev 1.55 device result | Current first decisive blocker / interpretation |
|---|---|---|
| Sandwiches | Advances past immutable-dictionary startup, then remains indefinitely black and silent; taps have no target window | `UIWebView loadRequest:` receives `nil`, followed by repeated “Couldn't find a window for touch” messages (Sandwiches report lines 142–155). `NSURLRequest` currently returns `nil` solely because emulator network access is disabled, which is incorrect construction behavior and can also suppress local-file requests. When the user exits, state serialization separately reaches missing `-[NSKeyedArchiver initForWritingWithMutableData:]` (lines 156–158). This is **advanced but not functional**. Even after request construction is fixed, a web-content dependency may remain because touchHLE has no embedded web engine. |
| SimplePhysics | User reports smooth, working gameplay; the original app is also silent on the physical iPod touch | This is **apparently functional within the tested scope**. The physical-device comparison confirms that silence is native app behavior, not an emulator audio regression, and its emulator log contains no audio API calls. Remote notification/score URLs fail offline without affecting local gameplay (combined report lines 177–196). |
| Spirits | Menu opens; selecting Play still crashes during intro setup | Dev 1.55 disproves the claim that only an always-nil movie view explained the crash. The new terminal diagnostic shows guest code dereferencing nil immediately after constructing the intro `MPMoviePlayerController` (combined report lines 301–330). The binary also has an unresolved standard `_MPMoviePlayerDidExitFullscreenNotification` constant. Add that real constant and make the one-time movie-view creation diagnostic visible; do not ignore a nil observer or weaken memory checks. This remains **partial/menu only**. |
| Splode | Reaches its menu, starts live gameplay and runs well until a later arbitrary crash | This is a clear advancement from pre-menu failure, but not playable. Immediately before the crash, Unity/Mono asks `dlsym(RTLD_DEFAULT, ...)` for native functions exported by its own executable; touchHLE's `dlsym` only searches host shims. The terminal state is a nil receiver followed by `componentsSeparatedByString:` (combined report lines 1129–1177), consistent with the missing locale bridge. This is a general dynamic-linker scope defect, not justification to stub Unity locale strings. |

### Dev 1.56 implementation boundary

- **Process-wide dynamic lookup (general, semantic correction):** make
  `dlsym(RTLD_DEFAULT, name)` search defined symbols in the main guest
  executable and loaded guest dylibs before touchHLE's host exports. Preserve
  the Mach-O slide and Thumb bit. Specific-library handles remain scoped to the
  corresponding host library until real guest `dlopen` handles are modeled.
  Splode is the directly observed beneficiary; other Unity and middleware apps
  using `DllImport("__Internal")` may also advance.
- **NSURLRequest construction (general, semantic correction):** construct a
  request whenever its URL is non-nil, regardless of current connectivity.
  Offline behavior belongs to connection/socket/reachability layers, not the
  request value object. This should turn Sandwiches' suppressed request into a
  diagnosable URL and may advance local-file web requests without enabling
  external network access.
- **NSKeyedArchiver mutable-data initializer (general, bounded complete
  contract):** retain the caller's `NSMutableData`, serialize into it on
  `finishEncoding`, and release it at teardown. This removes Sandwiches' exit
  serialization failure and also addresses the audit's unchanged first blocker
  for GarbageSprite (file labeled Mech Guardian).
- **MediaPlayer constants and bounded diagnostics (general/diagnostic):** export
  the observed standard load-state and did-exit-fullscreen notification names,
  and emit the movie view's one-time creation at normal log level. This may
  repair Spirits if the missing notification pointer is causal; otherwise the
  next run will establish whether the real view was requested before the nil
  dereference. Video decoding remains unimplemented.

Focused Dev 1.56 device test:

1. **Sandwiches** — launch once, wait through the black-screen interval, and
   report the URL now shown by `UIWebView loadRequest:` plus any visible screen.
   Exit once so the keyed-archive path is exercised.
2. **Spirits** — select Play once and retain the movie-view line and terminal
   fault context if it still fails.
3. **Splode** — play until the previous arbitrary gameplay crash or for several
   minutes; preserve any remaining unresolved `dlsym` messages.
4. **SimplePhysics** — no full replay required. Check its sound setting and one
   short launch only as a Foundation regression test.
5. **Regression checks:** one saved-progress relaunch (Spoing or Bounce) and one
   short Diversion/Sticky launch cover serialization, request, and MediaPlayer
   exposure. The separate App Store iDared installation remains untouched.

### Dev 1.56 build and verification record

- Implementation commit: `a22c7e18ffa0ad8fa8939df888a85ceb4d546dfd`
  (`Implement Dev 1.56 request and linker contracts`). The first IPA and test
  runs exposed only an Objective-C macro declaration-order error; no device
  behavior was involved.
- Final repository commit: `e18ef60d9b4d75c2813dc03f17fd18456c5dc782`
  (`Fix NSKeyedArchiver method declaration order`). The correction only moves
  the initializer below all class methods as required by `objc_classes!`; it
  does not change the initializer's behavior. The working tree is clean.
- GitHub unsigned-IPA run 90 (`41189314263`) completed successfully and
  produced exactly one artifact,
  `iDared32bit-dev-unsigned-e18ef60d9b4d75c2813dc03f17fd18456c5dc782`.
- Downloaded artifact ZIP SHA-256:
  `35DB8830FF464A7852935464CCBEAAEC30C55C6DD33EF03D9B8D0AF224C18011`,
  exactly matching GitHub's recorded artifact digest.
- Final IPA SHA-256:
  `CD525CAE0E2029AA1E029B20057DDB2477709606067BA683F5996F591EA39BD8`.
- Local package inspection found 36 entries, a valid `Payload` app bundle, no
  embedded provisioning profile and no code-signature directory. The binary
  plist contains the separate development identity
  `com.bookieart1.idared32bit.dev`, display name `iDared Dev`, and executable
  `iDared 32bit`.
- GitHub lint/test run 25 (`41189314238`) completed successfully. This confirms
  the Rust build and the development branch's configured library checks; the
  native Objective-C CLI test added for the mutable-data archiver path remains
  outside this branch's intentionally skipped native integration stage, so the
  physical device exit/relaunch test remains meaningful.
- Direct device-testing IPA:
  `artifacts/1.56-e18ef60d/iDared-Dev-1.56.ipa`.

## Dev 1.56 device result and Dev 1.57 Foundation/export-trie batch

The returned log embeds the expected Dev 1.56 commit `e18ef60` and workflow
run `41189314263`. It contains one Sandwiches launch, six Spirits launches and
two Splode launches. The repeated launches are useful here: Splode reaches the
same terminal guest address after its level transition twice, while the
different Spirits menu paths expose both standard missing APIs and a repeated
unidentified guest-code fault.

| App | Dev 1.56 device result | Current first decisive blocker / interpretation |
|---|---|---|
| Sandwiches | Remains fully black; it eventually returns to the iDared picker without a recorded panic | Dev 1.56 repaired request construction: `UIWebView loadRequest:` now receives the actual `http://www.bango.com` URL rather than `nil`. The remaining launch depends on real web content, but touchHLE's UIWebView has no web engine or renderable window. The repeated touch warnings and hidden exit control are consequences of that architectural gap. This is **advanced to an external web/UIWebView blocker**, not functional compatibility. |
| Spirits | Main menu and music work. Play reaches its tutorial/intro path, but gameplay-area taps do not advance; How To, Options, Credits, Select Level and tutorial Next lead to crashes on the tested paths | Select Level reaches a real table reload and then missing `+[NSIndexPath indexPathForRow:inSection:]`. Options creates its slider and then reaches missing `-[NSString canBeConvertedToEncoding:]`. Several other paths terminate at the same guest PC `0x62fac` after a null access, with different callers; the current exported-symbol map cannot name that address. This is **partial/menu only**. |
| Splode | Live gameplay works through a level, then crashes reproducibly while transitioning after completion | Both launches end at guest PC `0x46d0c4` after the executable's native bridge functions fail `dlsym(RTLD_DEFAULT, ...)` lookup. Dev 1.56 corrected lookup scope, but the Mach-O loader only records classic `LC_SYMTAB` definitions. Stripped iOS executables commonly carry runtime exports in the dyld export trie, so the process-wide search has no symbols to find. This is a **general loader-model gap**, not evidence that the individual Unity bridge results should be stubbed. |

### Dev 1.57 implementation boundary

- **Mach-O export-trie ingestion (general, semantic loader correction):** parse
  exports from both the export fields of `LC_DYLD_INFO` and the newer
  `LC_DYLD_EXPORTS_TRIE` command, map image-relative addresses through the
  loaded `__TEXT` base and merge them with classic symbol-table definitions.
  Prefer an existing symbol-table entry on duplicates so its explicit ARM
  Thumb metadata is preserved. Re-exports remain the responsibility of the
  referenced dylib. This completes the data-source half of Dev 1.56's
  process-wide `dlsym` correction and is broadly relevant to stripped games,
  Unity `DllImport("__Internal")` bridges and other native middleware.
- **NSIndexPath (general Foundation/UIKit contract):** implement immutable
  index storage, construction, comparison, hashing, copying and the standard
  table/collection `section`, `row` and `item` accessors. The implementation
  supports genuine index-path behavior rather than returning a synthetic row
  solely for Spirits.
- **NSString conversion capability (general Foundation contract):** implement
  `canBeConvertedToEncoding:` as a lossless conversion check using the abstract
  NSString primitives and the emulator's existing encoding implementation.
  This does not claim support for encodings the existing converter cannot
  represent.
- **Guest memory-fault symbol context (bounded diagnostic):** when a terminal
  guest memory fault occurs inside a loaded image, report the nearest preceding
  exported guest symbol and offset. This runs only at the terminal fault and
  changes no guest execution. It is intended to identify Spirits' repeated
  `0x62fac` path and to make future stripped-game failures substantially more
  actionable.
- **No Sandwiches workaround:** do not fake a successful webpage load or
  fabricate obsolete Bango content. Its standard request/value-object blocker
  is fixed; implementing a web engine or a justified app-content replacement
  is a separate, much larger decision.

Predicted immediate outcomes remain **advancement**, not verified playability.
Spirits should pass the exact Select Level and Options blockers; its Play,
tutorial input and other menu faults may remain, but the next terminal log
should name their guest-code neighborhood. Splode should expose its executable
bridge exports to `dlsym`; if those bridge functions themselves rely on missing
APIs, it will advance to those real later blockers.

Focused Dev 1.57 device test:

1. **Spirits** — try Select Level and Options first. Then make one Play/tutorial
   attempt, including one gameplay-area tap and tutorial Next. Preserve any
   `CPU memory-fault nearest symbol` line.
2. **Splode** — complete one level and observe the transition. Preserve any
   unresolved `dlsym` names and the nearest-symbol line if it still faults.
3. **Sandwiches** — no retest is needed in this batch; its remaining blocker is
   outside the changed subsystems.
4. **Regression checks:** briefly launch Spoing or Bounce and verify one saved
   progress relaunch. Launch Diversion or Sticky through its intro once. These
   cover loader lookup, Foundation object behavior, transitions and persistence
   without replaying unrelated titles.

The separate App Store iDared bundle identity/container, save files, graphics,
audio and web-view implementations are unchanged by this batch.

### Dev 1.57 build and verification record

- Final repository commit: `22c51128f7b6c6d2eec18c2ff96155be3ff7e370`
  (`Implement Dev 1.57 Foundation and export trie contracts`). The commit is
  pushed to `origin/ci/unsigned-ipa-baseline`, and the repository working tree
  is clean.
- GitHub unsigned-IPA run 91 (`41190801054`) completed successfully and
  produced exactly one artifact,
  `iDared32bit-dev-unsigned-22c51128f7b6c6d2eec18c2ff96155be3ff7e370`.
- Downloaded artifact ZIP SHA-256:
  `29A821631B7DF420449FDE521B134F200D23D81C60531DB2C34E27125629CA9A`,
  exactly matching GitHub's recorded artifact digest.
- Final IPA SHA-256:
  `B7CF9C5829B692E0DAF297BDC796BBAD4E0962EF6B9A3A7F36729B3E1C8347AC`.
- Local package inspection found 36 entries, exactly one app bundle and one
  binary `Info.plist`, no embedded provisioning profile and no code-signature
  directory. The plist contains the separate development identity
  `com.bookieart1.idared32bit.dev`, display name `iDared Dev`, and executable
  `iDared 32bit`; the packaged executable contains commit prefix `22c5112`.
- GitHub lint/test run 26 (`41190801112`) completed successfully. macOS and
  Windows development library tests passed; macOS, Windows and Android lint
  passed. Full integration tests, the native Objective-C sanity test and the
  workflow formatting step remain intentionally skipped on this development
  branch, so they are not claimed as passes. The separate iOS build, identity
  verification, packaging and artifact upload steps all passed.
- Direct device-testing IPA:
  `artifacts/1.57-22c51128/iDared-Dev-1.57.ipa`.

## Dev 1.57 device result and Dev 1.58 table/request/diagnostic batch

The returned log embeds the expected Dev 1.57 commit `22c5112` and workflow
run `41190801054`. The user's spoken report initially called the extra failing
title “Balance,” then clarified that this was only a naming slip. The tested
title was **Bounce 1.1** (`com.backflipstudios.bouncethebunny`, `bounce.app`);
no separate Balance test occurred.

| App | Dev 1.57 device result | Current first decisive blocker / interpretation |
|---|---|---|
| Spirits | Main menu remains partial. Select Level, Options, How To/Credits-style paths and tutorial Next still crash; Play reaches the tutorial, but touches in the gameplay area do not advance it | Select Level **did advance** past the new NSIndexPath constructor and now stops on missing `-[UITableView scrollToRowAtIndexPath:atScrollPosition:animated:]`. Options also passes `-[NSString canBeConvertedToEncoding:]`, creates two still-unrendered UISlider objects, then reaches a host-side null-page access. The other paths likewise panic in the host memory accessor at common PC `0x62fac` but different guest callers. Dev 1.57's CPU-fault diagnostic did not run because these are host-side memory-access panics, so the selector/register context remains unknown. The tutorial input symptom may be an overlay/state/input issue; the log does not yet justify changing touch coordinates or swallowing the fault. |
| Splode | Gameplay still reaches level completion and then crashes during the transition | All seven executable bridge lookups remain unresolved, including `_GetCurrentNSLocaleCode` and `_GetNSLocalizedString`; the export-trie ingestion therefore disproves the hypothesis that the functions were present only in a dyld export trie. The nearest exported symbol is merely `start`, far from the fault, and does not identify a bridge implementation. The bridge functions may be stripped/static, generated into a different native-plugin path, or absent from this package. Returning fabricated locale/achievement/stat values remains unjustified. Capture the guest caller for each failed `dlsym` before deciding whether any shared implementation is possible. |
| Bounce | The logged launch now crashes before the game loads | Startup proceeds into Burstly advertising and then sends `mutableCopyWithZone:` to `NSMutableURLRequest`, which lacks the standard NSMutableCopying contract. This is a concrete Foundation blocker. It is not established as a Dev 1.57 loader regression: Dev 1.56 deliberately made offline request construction return a real request instead of `nil`, so the ad path can now expose this later missing method. Dev 1.42's audit only proves a no-panic/user-exit session, not that this particular startup branch ran. |
| Spoing | User reports behavior unchanged and working; the log reaches gameplay/level transitions and returns normally through the Home control | This is a focused pass for launch, Unity/Mono startup, rendering/input and normal exit in the observed scope. It does not by itself verify every save/audio path, but it argues against a general loader or UIKit startup regression. |

The repeated invalid-ZIP warnings for the same 26 files are unchanged picker
inventory noise and are not part of this batch.

### Dev 1.58 implementation boundary

- **UITableView layout/scrolling (general partial contract):** make
  `reloadData` obtain section and row counts from the data source, establish
  uniform-row content geometry, and implement all four historical
  `UITableViewScrollPosition` behaviors with clamped content offsets. Cell
  creation, drawing, variable row heights and selection remain unimplemented;
  this removes Spirits' exact selector blocker without claiming a complete
  table view.
- **NSURLRequest copying (general semantic correction):** implement independent
  immutable and mutable snapshots which preserve URL, cache policy, timeout,
  method, body and headers. Add the matching property accessors and real
  cache-policy/timeout setters, and allow `setValue:nil` to remove a header as
  Foundation specifies. This directly addresses Bounce's Burstly failure and
  remains independent of whether network transport is enabled.
- **Host-side null-fault diagnostics (diagnostic-only):** extend the bounded
  terminal diagnostic to null-page panics raised while a guest-to-host API is
  reading guest memory. Report PC, LR/caller, image/symbol neighborhood,
  Objective-C argument candidates and a selector-shaped R1. Do not weaken the
  null page or continue execution. This is aimed at Spirits' repeated
  `0x62fac` family, which bypassed the CPU-fault-only Dev 1.57 logging.
- **Unresolved `dlsym` provenance (diagnostic-only):** add the guest LR and its
  loaded-image/symbol neighborhood to each failed lookup. Splode performs a
  bounded set of seven observed lookups during the transition. No
  app-specific bridge stub or fabricated locale/stat/achievement result is
  added.

Predicted immediate outcomes are advancement rather than verified
compatibility. Spirits should pass its exact table-scroll exception and either
remain on a usable level screen or reveal the next table limitation; its other
paths should finally identify the host API/selector neighborhood. Bounce
should pass mutable request copying, after which offline connection behavior
or a later SDK contract may still appear. Splode is expected to keep failing
in this diagnostic build, but its next log should distinguish which guest
callers own the absent bridge lookups.

Focused Dev 1.58 device test:

1. **Spirits** — open Select Level once and report whether rows/controls are
   visible and interactive. Then try Options and one Play/tutorial Next path;
   preserve every `Host memory-fault` line.
2. **Bounce** — launch with the existing development save/container intact. If
   it passes startup, enter gameplay briefly and verify the existing progress
   still appears; do not reset or delete data.
3. **Splode** — complete one level once and preserve the unresolved `dlsym`
   lines with their new guest-caller contexts plus the terminal fault.
4. **Spoing** — one short existing-save launch is enough for the focused
   regression check. Add either Diversion or Sticky through its intro if
   practical because the batch touches Foundation objects and UI transitions.

The separate App Store installation, development bundle identity/container,
saved data, graphics/audio pipelines and network-transport policy are not
changed by this batch.

### Dev 1.58 build and verification record

- Implementation commit: `230f62ee23b6b9549d2675d849c8131b3683b065`
  (`Implement Dev 1.58 request and table contracts`). Its first IPA and
  lint/test workflows failed at compile time because the new `dlsym`
  diagnostic returned a local variable named `context` where the computed
  variable was named `nearest`; no package was produced and no device behavior
  is associated with that failed run.
- Final repository commit: `e8b4d9f7b6ff15f5f56049e6d21d3b358c9d6d5a`
  (`Fix dlsym diagnostic symbol context`). The one-line correction changes
  only that diagnostic value name. It is pushed to
  `origin/ci/unsigned-ipa-baseline`, the local and remote-tracking heads agree,
  and the working tree is clean.
- GitHub unsigned-IPA run 93 (`41192503247`) completed successfully and
  produced exactly one artifact,
  `iDared32bit-dev-unsigned-e8b4d9f7b6ff15f5f56049e6d21d3b358c9d6d5a`.
  The separate iOS compile, development identity/package verification and
  artifact upload steps all passed.
- Downloaded artifact ZIP SHA-256:
  `1DD057DCD7F1DE5153881DCE71581FB4598927BE906907890924C3CADF8D6BDE`,
  exactly matching GitHub's recorded artifact digest.
- Final IPA SHA-256:
  `96D9BF74DC13929E13F93EBEB7C6DFCE8485C86248F7F7CFC538EDC7A6E03E1D`.
- Local package inspection found 36 entries, exactly one app bundle and one
  binary `Info.plist`, no embedded provisioning profile and no code-signature
  directory. The plist contains the separate development identity
  `com.bookieart1.idared32bit.dev`, display name `iDared Dev`, and executable
  `iDared 32bit`; the packaged executable contains commit prefix `e8b4d9f` and
  workflow run `41192503247`.
- GitHub lint/test run 28 (`41192503256`) completed successfully. macOS and
  Windows development library tests passed; macOS, Windows and Android lint
  passed. The workflow formatting checks, full integration tests and macOS
  native sanity checks remain intentionally skipped on this development
  branch, so they are not claimed as passes.
- Direct device-testing IPA:
  `artifacts/1.58-e8b4d9f7/iDared-Dev-1.58.ipa`.

## Dev 1.58 device result and Dev 1.59 animation/request diagnostic batch

The returned log embeds the expected Dev 1.58 commit `e8b4d9f` and workflow
run `41192503247`. It contains four Spirits launches, one Bounce launch, two
Splode launches and one Spoing launch. The two Splode tests cover different
game paths but converge at the same terminal function, which is stronger
evidence than another repeat of one level-completion route.

| App | Dev 1.58 device result | Current first decisive blocker / interpretation |
|---|---|---|
| Spirits | Select Level no longer crashes, but the resulting screen has no visible choices and some text appears oversized. How To, Options and Credits still crash. Play reaches the tutorial image; pressing Next still crashes. | The Select Level result matches the explicit Dev 1.58 boundary: table geometry/scrolling exists, but cell creation, drawing and selection do not. It is **blocker removed, not a usable level selector**. All four logged crashes now identify one concrete contract family. Three send `+[UIView setAnimationDidStopSelector:]` with a null selector and tutorial Next sends `+[UIView setAnimationWillStartSelector:]` with a null selector. The implementation incorrectly dereferences every selector as a C string. Options also constructs two UISlider objects that remain unrendered, so removing its animation crash will not make the controls complete. |
| Bounce | Still terminates during startup before gameplay | Dev 1.58's mutable request snapshot works: Burstly advances past `mutableCopyWithZone:` and now sends the standard `setHTTPShouldHandleCookies:` property to `NSMutableURLRequest`. This is another concrete Foundation value-object blocker, not evidence of a save failure or a general loader regression. |
| Splode | Quitting from a different game mode on the return-to-menu path crashes; completing the previously tested level also still crashes | Both routes converge on guest PC `0x46d0c4`, dereferencing address `0x8` from a null R0, with the same impending `componentsSeparatedByString:` selector. Their managed call stacks differ in the route-specific middle but share the later tail. The new `dlsym` caller is the same stripped Mono/Unity resolver for every missing bridge name, so it does not identify a bridge consumer. A common return/menu string or locale path is plausible, but the log still cannot prove which object became nil or justify fabricating locale, analytics or achievement results. |
| Spoing | Behavior remains unchanged and working; the log reaches its Unity scenes and returns normally through the Home control | This is a focused pass for launch, Unity/Mono startup, rendering/input, scene transitions and normal exit in the observed scope. It argues against a general regression in the shared loader, request or Unity paths. |

The tutorial's apparent gameplay area is an instructional image, correcting
the earlier input interpretation; no touch-coordinate or event-dispatch defect
is inferred from that image. The repeated invalid-ZIP picker warnings remain
unchanged inventory noise.

### Dev 1.59 implementation boundary

- **Nullable UIView animation selectors (general semantic correction):**
  accept `NULL` for both legacy animation-selector setters. Preserve the fork's
  existing default-callback compatibility only when the app never called the
  corresponding setter; an explicit `NULL` is stored distinctly and disables
  that callback. This addresses the exact four Spirits terminal faults without
  swallowing a memory error or inventing a delegate call.
- **NSURLRequest cookie-handling state (general Foundation contract):** add
  `HTTPShouldHandleCookies` to immutable and mutable requests, default it to
  true, preserve it across copy/mutable-copy snapshots, expose the immutable
  getter and implement the mutable setter. Network transport and cookie-store
  behavior remain outside this narrow value-object change.
- **Stripped-code terminal context (diagnostic-only):** retain the existing
  terminal memory fault and add a fixed 96-byte guest-code window plus a fixed
  64-byte stack window. This does not continue execution or alter the nil
  object. The next Splode result should make the common stripped function
  disassemblable and show how the null receiver is obtained.
- **No table-cell, slider or Splode bridge accommodation in this batch:** the
  malformed Spirits level screen and invisible Options sliders are confirmed
  partial UIKit implementations, but completing those controls is materially
  larger than the two proven crash contracts. No app-specific locale,
  localization, analytics or achievement result is fabricated for Splode.

Predicted immediate outcomes remain advancement rather than verified
compatibility. Spirits should pass the four null-selector crashes and expose
the next real limitation on each path; its Select Level and slider rendering
are still expected to be incomplete. Bounce should pass the cookie-property
message and may reveal another request/advertising contract. Splode is expected
to retain its current failure in this diagnostic build, with more actionable
terminal code and stack bytes.

Focused Dev 1.59 device test:

1. **Spirits** — try How To, Options and Credits once each, then Play and
   tutorial Next. Report the first screen reached after each action. Open Select
   Level only once to confirm it remains stable; its missing choices and large
   text are already known and are not claimed fixed.
2. **Bounce** — launch with the existing development save/container intact. If
   it reaches gameplay, verify that existing progress remains; do not reset or
   delete data.
3. **Splode** — repeat only one return-to-menu route. Preserve the terminal
   `nearby guest code` and `guest stack` lines; both prior routes already prove
   the common crash.
4. **Spoing** — one short launch and normal Home exit is enough for the focused
   regression check. No unrelated full replay is required.

The separate App Store installation, development bundle identity/container,
saved data, graphics/audio pipelines, table-cell/slider rendering and network
transport policy are unchanged by this batch.

### Dev 1.59 build and verification record

- Final repository commit: `0d712ecec7e888844c9f9465398ffb41eb3fba09`
  (`Implement Dev 1.59 animation and request contracts`). It is pushed to
  `origin/ci/unsigned-ipa-baseline`.
- GitHub unsigned-IPA run `41193921362` completed successfully for that exact
  commit. The separate iOS compile, development identity/package verification
  and artifact upload steps all passed. The produced artifact is
  `iDared32bit-dev-unsigned-0d712ecec7e888844c9f9465398ffb41eb3fba09`.
- Downloaded artifact ZIP SHA-256:
  `2A52DFF897A7091C13D1664A09C0222B72B7AB04E472FD7BE2D50F6F0E6DC78B`,
  exactly matching GitHub's recorded artifact digest.
- Final IPA SHA-256:
  `FF41DE5068FBDF75BC4DE6B398BC927CFE557C52BD962E26580704EFC05566B6`.
- Local package inspection found 36 entries and exactly one app bundle, no
  embedded provisioning profile and no code-signature directory. The plist
  contains the separate development identity
  `com.bookieart1.idared32bit.dev`, display name `iDared Dev`, and executable
  `iDared 32bit`; the packaged executable contains commit prefix `0d712ec` and
  workflow run `41193921362`.
- GitHub lint/test run `41193921322` completed successfully. macOS and Windows
  development library tests passed; macOS, Windows and Android lint passed.
  The workflow formatting checks, full host/integration `Test` steps and macOS
  native sanity checks were skipped, so they are not claimed as passes.
- Direct device-testing IPA:
  `artifacts/1.59-0d712ece/iDared-Dev-1.59.ipa`.

## Dev 1.59 device result and Dev 1.60 string/URL/diagnostic batch

The returned log embeds the expected Dev 1.59 commit `0d712ec` and workflow
run `41193921362`. The reported improvement is supported by the transition
points in the log rather than inferred from the build label.

| App | Dev 1.59 device result | Current first decisive blocker / interpretation |
|---|---|---|
| Spirits | How To, Options, Credits and Select Level now open; the level selector appears more normal. Play advances through tutorial Next. Both logged attempts crash only when leaving the tutorial to start gameplay. | Both attempts converge on missing `-[NSString rangeOfCharacterFromSet:options:range:]`. The previous four nullable-animation-selector crashes are absent. This is verified menu/tutorial advancement, not yet verified gameplay. The requested overload has a documented general contract supporting range restriction plus backwards/anchored search. |
| Bounce | Still terminates during startup | Mutable request copying and `setHTTPShouldHandleCookies:` now pass. Burstly starts its network gateway thread, which sends the standard `host` accessor to a valid `NSURL`. `NSURL` currently stores the URL spelling but exposes no host component. This is the next concrete Foundation blocker and is unrelated to saved progress. |
| Splode | In-game Menu and return to the main menu now work cleanly. Level completion still crashes. | The menu-return route is now an observed pass in this scope, narrowing the remaining failure to the level-completion path. The terminal site remains guest PC `0x46d0c4`, null R0, address `0x8`, immediately before `componentsSeparatedByString:`. The new code window shows a direct ARM call at `0x46d0b0` returning null before the dereference; its target is `0x47cff4`. The current log does not contain that target's code or establish which missing locale/service bridge owns the value. |
| Spoing | Continues to work and returns normally through the Home control | Focused regression pass for launch, Unity/Mono startup, scene transitions and normal exit in the observed scope. |

The log also contains an unreported Cannon Cadets 2.3.6 launch which stops on
missing `+[UIView setAnimationBeginsFromCurrentState:]`. That API has real
presentation-state semantics for replacing an in-flight animation. It is
recorded as an incidental blocker, but this batch does not add a misleading
no-op or partial implementation merely to advance that launch.

### Dev 1.60 implementation boundary

- **NSString character-set search (general partial contract):** implement the
  two missing `rangeOfCharacterFromSet:options:` variants, including the
  range-limited form used by Spirits. For a valid non-nil character set and an
  in-bounds UTF-16 range, support forward, backwards and anchored searches and
  return `{NSNotFound, 0}` when no character matches. Unsupported option bits
  are diagnosed rather than silently changing the search. Raising the exact
  historical Objective-C exception for invalid arguments remains outside this
  boundary.
- **NSURL host component (general contract within parsed non-file URLs):**
  expose the authority host, excluding user information and numeric port,
  decode valid UTF-8 percent escapes, handle bracketed IP literals, and return
  nil for relative, missing-host or malformed spellings. File-URL host/base
  semantics and the other NSURL component accessors are not expanded here.
- **Splode direct-producer context (diagnostic-only):** on terminal ARM memory
  faults, decode the nearest preceding direct `BL` and record a fixed 64-byte
  window at its target. This remains bounded to termination and does not alter
  the null value or continue execution.
- **No Splode bridge fabrication and no Cannon Cadets animation no-op:** the
  evidence still does not justify invented locale/localization/analytics or
  achievement results, and the animation API needs real in-flight presentation
  behavior rather than unconditional success.

Predicted immediate outcomes remain advancement rather than full
compatibility. Spirits should pass the exact tutorial-to-game string search
and either enter gameplay or expose its next dependency. Bounce should pass
the URL host query and may proceed to a later network-request contract. Splode
is expected to retain the level-completion failure in this diagnostic build,
but its next log should include the direct null-producing function bytes.

Focused Dev 1.60 device test:

1. **Spirits** — use Play and complete the tutorial once, then report whether
   gameplay starts. The menu pages do not need another exhaustive pass.
2. **Bounce** — launch once with the existing development save/container; do
   not reset or delete data.
3. **Splode** — complete one level once and preserve the new `nearest preceding
   ARM BL` line. The already-passing in-game menu-return path need not be
   repeated.
4. **Spoing** — one short launch and normal Home exit is sufficient.

The separate App Store installation, development bundle identity/container,
saved data, graphics/audio pipelines, network transport policy and Cannon
Cadets animation behavior are unchanged by this batch.

### Dev 1.60 build and verification record

- Final repository commit: `93193016374e36092af8e2e68b768c59f42c0136`
  (`Implement Dev 1.60 string and URL contracts`). It is pushed to
  `origin/ci/unsigned-ipa-baseline`.
- GitHub unsigned-IPA run `41194912134` completed successfully for that exact
  commit. The separate iOS compile, development identity/package verification
  and artifact upload steps all passed. The produced artifact is
  `iDared32bit-dev-unsigned-93193016374e36092af8e2e68b768c59f42c0136`.
- Downloaded artifact ZIP SHA-256:
  `D5DBBCD373CF5268AA6C877F563DDBF33C0C9CFAA5A6C2EE4FFC790C2531489A`,
  exactly matching GitHub's recorded artifact digest.
- Final IPA SHA-256:
  `5E6A9AAAC375922B070696289F220D8C4078B46527753A76CE7E99993F549F22`.
- Local package inspection found 36 entries and exactly one app bundle, no
  embedded provisioning profile and no code-signature directory. The plist
  contains the separate development identity
  `com.bookieart1.idared32bit.dev`, display name `iDared Dev`, and executable
  `iDared 32bit`; the packaged executable contains commit prefix `9319301` and
  workflow run `41194912134`.
- GitHub lint/test run `41194912130` completed successfully. macOS and Windows
  development library tests passed; macOS, Windows and Android lint passed.
  The workflow formatting checks, full host/integration `Test` steps and macOS
  native sanity checks were skipped, so they are not claimed as passes.
- Direct device-testing IPA:
  `artifacts/1.60-93193016/iDared-Dev-1.60.ipa`.

## Dev 1.60 device result and Dev 1.61 dictionary/URL diagnostic batch

The returned log embeds the expected Dev 1.60 commit `9319301` and workflow
run `41194912134`. The user's visible results appear unchanged, but the log
shows that both Dev 1.60 contract changes executed and advanced to later,
independent blockers.

| App | Dev 1.60 device result | Current first decisive blocker / interpretation |
|---|---|---|
| Spirits | Still crashes when leaving the tutorial to start gameplay | The new character-set search completes. The next blocker is the missing general C API `CFDictionaryGetValueIfPresent`, called with a valid dictionary, key and non-null output pointer. This is a later Core Foundation dependency, not a repeat of the Dev 1.59 string failure. |
| Bounce | Still terminates during startup before gameplay | The new `NSURL host` accessor completes and Burstly's network gateway thread starts. That thread then asks the same valid full URL for `path`; the current implementation asserts that every non-file URL is already only a path. Dev 1.42 genuinely reached gameplay, ran more smoothly and preserved progress. The present observed-usability regression traces to Dev 1.56's semantically correct change from returning `nil` for offline `NSURLRequest` construction to constructing a real request: Bounce's advertising path now runs and exposes incomplete URL component support that the older incorrect `nil` behavior skipped. The repair target is therefore the earlier gameplay state without reverting the general request contract or deleting the existing save. |
| Splode | Still crashes at level completion | The direct target at `0x47cff4` is now captured. Its first three ARM instructions load and return the pointer stored in global slot `0x00acf87c`; that slot is null at the terminal dereference. This establishes an uninitialized/cleared shared value before the impending `componentsSeparatedByString:` call, but not yet which initialization path owns it. It still does not justify fabricating locale, localization, analytics or achievement results. |
| Spoing | Continues to work as before and returns normally through the Home control | Focused regression pass for launch, Unity/Mono startup, scene transitions and normal exit in the observed scope. |

The repeated invalid-ZIP picker warnings remain unchanged inventory noise.

### Dev 1.61 implementation boundary

- **CFDictionaryGetValueIfPresent (general Core Foundation contract):** return
  false when a key is absent; return true for a present key and write its value
  when the output pointer is non-null. Callers must not rely on output storage
  when the key is absent because the documented contents are undefined.
  Presence is tracked separately from value so a CF dictionary created with
  NULL callbacks can correctly report a stored NULL value. Tests cover present,
  absent, stored-NULL and NULL-output cases.
- **NSURL path component (general Foundation contract):** split non-file URLs
  into components, exclude query and fragment text, decode valid UTF-8 percent
  escapes, support full and relative URL spellings and return nil for malformed
  escapes. File URL behavior and unrelated URL component accessors are not
  expanded by this change.
- **Splode global-slot context (diagnostic-only):** recognize the small ARM
  direct-global-getter form at a terminal fault, report the slot and current
  value, then report at most 12 aligned literal references from at most 16 MiB
  of the owning image's `__text` section with fixed 48-byte contexts. This is
  bounded, runs only at termination and neither changes the slot nor continues
  execution.
- **No app-specific Bounce bypass or Splode value fabrication:** the Burstly
  path is not disabled, network success is not faked and the older
  request-construction bug is not restored merely to skip the current startup
  path.

Predicted immediate outcomes remain advancement rather than verified
compatibility. Spirits should pass its exact missing dictionary API and either
enter gameplay or expose the next dependency. Bounce should pass the asserted
full-URL `path` query; reaching the prior Dev 1.42 gameplay state is the
regression target, but another advertising/network contract may still appear.
Splode is expected to retain the level-completion failure in this diagnostic
build; its next terminal log should identify other code paths that reference
the null global slot.

Focused Dev 1.61 device test:

1. **Spirits** — use Play and finish the tutorial once; report whether gameplay
   starts and preserve the first terminal error if it does not.
2. **Bounce** — launch once with the existing development save/container
   intact. If it reaches the menu or gameplay, confirm whether the old progress
   remains; do not reset or delete data.
3. **Splode** — complete one level once and preserve the new `direct getter for
   global slot` and `global-slot literal reference` lines. No other mode needs
   replaying in this batch.
4. **Spoing** — one short launch and normal Home exit is sufficient.

The separate App Store installation, development bundle identity/container,
saved data, graphics/audio pipelines and network transport policy are unchanged
by this batch.

### Dev 1.61 build and verification record

- Final repository commit: `acb1dbc21a803fc683851451e173fb10c16ed3df`
  (`Implement Dev 1.61 dictionary and URL contracts`). It is pushed to
  `origin/ci/unsigned-ipa-baseline`.
- GitHub unsigned-IPA run `41196030682` completed successfully for that exact
  commit. The separate iOS compile, development identity/package verification
  and artifact upload steps all passed. The produced artifact is
  `iDared32bit-dev-unsigned-acb1dbc21a803fc683851451e173fb10c16ed3df`.
- Downloaded artifact ZIP SHA-256:
  `8E58C95E6417282F6D1B4F53D38E22886B122EDDD34416FED6F82EC3C759A231`,
  exactly matching GitHub's recorded artifact digest.
- Final IPA SHA-256:
  `30D374D09F044389BA466C3C7AA324D9F5B408EF2A64E2607854AC78B108A81F`.
- Local package inspection found 36 entries and exactly one app bundle, no
  embedded provisioning profile and no code-signature entries. The plist
  contains the separate development identity
  `com.bookieart1.idared32bit.dev`, display name `iDared Dev`, and executable
  `iDared 32bit`; the packaged executable contains commit prefix `acb1dbc` and
  workflow run `41196030682`.
- GitHub lint/test run `41196030699` completed successfully. macOS and Windows
  development library tests passed; macOS, Windows and Android lint passed.
  The workflow formatting checks, full host/integration `Test` steps and macOS
  native sanity checks were skipped, so they are not claimed as passes.
- Direct device-testing IPA:
  `artifacts/1.61-acb1dbc2/iDared-Dev-1.61.ipa`.

## Dev 1.61 device result and Dev 1.62 URL/producer diagnostic batch

The returned log embeds the expected Dev 1.61 commit `acb1dbc` and workflow
run `41196030682`.

| App | Dev 1.61 device result | Current status / first decisive blocker |
|---|---|---|
| Spirits | The tutorial completed, a full level and the start of the next level were played, menu navigation and return worked, normal exit/relaunch worked, and the existing progress was restored | Functionally compatible within this tested gameplay, menu, transition and save/relaunch scope. This is stronger than merely reaching gameplay, but it is not a claim that every level or optional service has been exhaustively tested. The general `CFDictionaryGetValueIfPresent` implementation removed the observed tutorial-to-game blocker. |
| Bounce | Still terminates during startup before its menu or gameplay | The Dev 1.61 `path` implementation executes successfully. Burstly's gateway then requests the `scheme` of the same valid `NSURL`, which is the next missing general Foundation component accessor. |
| Splode | Still terminates after completing a level | The global-slot diagnostic confirms that direct getter `0x0047cff4` returns the null value in slot `0x00acf87c`. It found five literal occurrences. In particular, the code immediately before the literal at `0x0047d17c` loads the slot address, passes zero to a helper and returns, which is consistent with a clearing/release path. The current evidence still cannot distinguish never initialized from initialized and cleared too early, nor identify the value's documented owner. |
| Spoing | Continues through its established working path and returns normally through Home | Focused regression pass in the observed scope. |

The repeated invalid-ZIP picker warnings remain unchanged inventory noise.

### Dev 1.62 implementation boundary

- **NSURL scheme (general Foundation contract):** expose `file` for represented
  file URLs, return the stored valid scheme without percent-decoding for other
  absolute/custom URLs, and return nil for relative or invalid scheme
  spellings. Scheme validation follows the RFC grammar: an ASCII letter first,
  followed by ASCII letters, digits, `+`, `-` or `.`. Tests cover HTTP, custom,
  file, relative and invalid forms.
- **Splode literal-use context (diagnostic-only):** after a terminal fault has
  identified a direct global getter, decode ARM PC-relative literal-load
  instructions that target the already bounded set of matching slot literals.
  Report at most 24 uses, each with a fixed 192-byte context. This runs only at
  termination and does not write the slot, invent a string or resume execution.
- **No app-specific Burstly bypass or Splode value fabrication:** network
  success remains unfaked and no guessed locale/localization/analytics value is
  substituted for the null receiver.

Predicted immediate outcomes remain advancement rather than a promise of full
compatibility. Bounce should pass the exact missing `scheme` selector and may
reach its prior Dev 1.42 menu/gameplay state or expose another URL/network
contract. Splode is expected to retain its level-completion failure in this
diagnostic build, but the new literal-load contexts should identify which
routines read, initialize or clear the shared slot.

Focused Dev 1.62 device test:

1. **Bounce** — launch once with the existing development save/container. If
   it reaches the menu or gameplay, confirm whether the old progress remains;
   do not reset or delete data.
2. **Splode** — complete one level once and preserve the new
   `global-slot literal loaded by ARM instruction` lines.
3. **Spoing** — one short launch and normal Home exit is sufficient as the
   URL/Unity regression check.

Spirits does not need to be replayed for this focused batch. Its Dev 1.61
result remains recorded, and neither change modifies the dictionary or save
implementation that removed its observed blocker.

### Dev 1.62 build and verification record

- The implementation commit was
  `4537306b20a606b0acdb2416f843cc59267c1dee`. Its first automatic build run
  `41197003776` and lint/test run `41197003738` failed compilation with Rust
  error E0382 because the diagnostic's first loop consumed the collected
  reference vector before the second loop borrowed it. No IPA was produced by
  that run. The one-line ownership correction is commit
  `cda3daedd7e699238cdfb262caaf23cf348d4ca5`; both commits are pushed to
  `origin/ci/unsigned-ipa-baseline` and the latter is the final packaged source.
- Replacement unsigned-IPA run `41197279597` completed successfully for exact
  commit `cda3daedd7e699238cdfb262caaf23cf348d4ca5`. The separate iOS compile,
  development identity/package verification and artifact upload steps all
  passed. The artifact is
  `iDared32bit-dev-unsigned-cda3daedd7e699238cdfb262caaf23cf348d4ca5`
  (artifact ID `13041510704`).
- Downloaded artifact ZIP SHA-256:
  `984245379F204CA205A0C120F9F46C2DE3A594F4381D28C57A026B598F741AD6`,
  exactly matching GitHub's recorded artifact digest.
- Final IPA SHA-256:
  `7221735076CB68783333EE8E27D9C84D9C1D0F4ABA778F00A7AB2C9C150C0AD8`.
- Local package inspection found 36 entries and exactly one app bundle,
  `iDared 32bit.app`, with no embedded provisioning profile and no
  `_CodeSignature` entries. The binary plist contains development identifier
  `com.bookieart1.idared32bit.dev` and display name `iDared Dev`; the packaged
  executable contains commit prefix `cda3dae` and workflow run
  `41197279597`.
- Replacement lint/test run `41197279599` completed successfully. macOS and
  Windows development library tests passed; macOS, Windows and Android lint
  passed. Formatting checks, full host/integration `Test` steps and the macOS
  native sanity checks were skipped by this branch workflow, so they are not
  claimed as passes.
- Direct device-testing IPA:
  `artifacts/1.62-cda3daed/iDared-Dev-1.62.ipa`.

## Dev 1.62 device result and current rotation decision

The returned log embeds the expected Dev 1.62 commit prefix `cda3dae` and
workflow run `41197279597`. It contains separate Bounce, Splode and Spoing
sessions.

| App | Dev 1.62 device result | Current interpretation / boundary |
|---|---|---|
| Bounce | Reached its earlier usable state again, according to the user. The log contains no Bounce panic, loads several Unity scenes/assets, and returns normally to the picker through Home. | The general `NSURL scheme` implementation repaired the Dev 1.56-era startup regression without reverting request construction or adding a Burstly/app-specific bypass. This restores the previously observed partial/usable baseline; it does not establish that the historical intermittent lag, audio defects or secondary-button crashes are all gone. |
| Splode | Still crashes immediately after the first level completes. | The terminal fault is unchanged in behavior: a direct getter returns null from global slot `0x00acf87c`, after which the game tries to use the result as the receiver of `componentsSeparatedByString:`. The new context does identify a routine that stores zero into the slot, but its surrounding argument setup is the standard shape of a static initializer registering a destructor. It therefore explains the slot's initial null state rather than proving premature teardown. No direct non-null producer was identified. |
| Spoing | Continues through its established working path and returns normally through Home. | Focused regression pass for the shared Unity/Mono, input, scene-transition and normal-exit path in the observed scope. |

### Splode durable resume point

- The first decisive blocker is still a missing shared string/object value, not
  the later null-page exception itself.
- The five direct references found by the bounded diagnostic include the
  getter, consumers/destruction-related code and the static zero initializer;
  they do not reveal the legitimate non-null initialization path. The producer
  may be indirect.
- Immediately before the failure, the game also looks up absent native bridge
  names including `_GetCurrentNSLocaleCode`, `_GetNSLocalizedString`, analytics
  calls and an OpenFeint achievement call. Their proximity is useful but does
  not prove which bridge owns this slot or what its historical return contract
  is.
- Returning a guessed locale/localized string, swallowing the fault, or
  inventing analytics/achievement success would be an app-shaped workaround
  without contract evidence and is not authorized by this result.
- Runtime write provenance would require materially more intrusive memory
  watching in the current CPU path and could change timing/performance. Another
  Splode-only build is therefore not justified now.

Splode is **parked, not abandoned**. Resume it when either (a) another title or
general bridge work identifies and implements the historical locale/
localization provider contract, or (b) a bounded, low-intrusion way to capture
the slot's actual non-null writer becomes available. Preserve this level-end
save/test path as the reproducer.

### Batch outcome and next scouting pass

- Spirits remains functionally compatible within its Dev 1.61 tested gameplay,
  menu, transition and save/relaunch scope.
- Bounce moves back to the regression backlog at its recovered prior usable
  baseline.
- Spoing remains the passing control.
- Splode rotates out at the evidence boundary above. No source patch, new
  commit, build or save reset follows from this report.

The next high-information scouting pass should use the existing Dev 1.62 IPA,
in two related pairs: **HookWorld and QuickHook**, then **Bulba the Cat and
Paper Monsters**. The first pair shares a developer/engine lineage and can show
whether one remaining dictionary/serialization contract generalizes across
both. The second pair had previously advanced to partial/no-panic behavior
after shared AudioQueue work and may expose smaller, broadly useful next
blockers. Record the first decisive failure and visible behavior for each; do
not reset any development saves.

## Dev 1.62 four-app scouting result

The returned log again embeds exact Dev 1.62 commit prefix `cda3dae` and
workflow run `41197279597`.

| App | User-observed result | Log evidence and current classification |
|---|---|---|
| HookWorld | Crashes before presenting usable UI | The app reaches OpenGL and settings initialization, reports `Loaded Version: (null)`, then calls `NSMutableDictionary setObject:forKey:` with a nil object. Foundation correctly rejects that invalid insertion. **Blocked.** |
| QuickHook | Same visible result as HookWorld | It follows the same engine path: OpenGL and settings initialization, `Loaded Version: (null)`, then the same nil-object dictionary insertion. **Blocked.** |
| Bulba the Cat | Opens without crashing, but appears to show a zoomed fragment/image and does not respond usefully to taps | The session has no terminal panic. It constructs a request for bundled resource `Data/Raw/ENLanguage.unity3d`, but the current `NSURLConnection` stub returns nil. Four later touches are ignored because the target view already has an active touch while multiple touch is disabled. **Partial/unusable:** the frozen resource-loading path is currently stronger evidence than a proven viewport transform error. |
| Paper Monsters | Smooth playable gameplay in the tested portion; saving works; no defect observed | The log loads several Unity scenes, contains no panic and returns normally through Home. **Functionally compatible within the observed gameplay, performance and local-save scope.** Missing iCloud, Game Center and iCade dynamic hooks mean those optional services are not claimed to work. |

### Hook pair: shared contract and next discriminator

The matching failures establish a shared engine/settings problem, but not yet
the source of the nil value. Both apps also attempt to open bundled `data.db3`
for writing and receive `AccessDenied`, then continue from a readable database
descriptor. Competing hypotheses include an absent first-launch/defaults value,
missing bundle-version metadata, and a failed writable database initialization
or migration. The next diagnostic should remain terminal and bounded: when
`setObject:forKey:` receives nil, report the key's class/string when safely
decodable and the guest caller context, then preserve the exception/failure.
Globally accepting nil dictionary objects is explicitly rejected.

### Bulba: general implementation candidate

Dev 1.42 reached the same no-panic runtime with a nil request; later general
`NSURLRequest` work now constructs the local resource request and exposes the
next missing contract. A file-URL/file-path `NSURLConnection` must return a
real connection, load the bundled bytes or report a real file error, and
deliver delegate callbacks with correct ordering/lifetime. It must not enable
arbitrary network success or synchronously fake completion. Before changing
viewport or touch transforms, test whether completing this resource load
advances Bulba past the apparent frozen image.

No source change, new commit, push, build or save reset was initiated from
this scouting report. A focused future build should combine the general local-
file connection implementation with the bounded Hook diagnostic. The relevant
regression set is Bulba, HookWorld, QuickHook, Paper Monsters and Bounce; Paper
Monsters and Bounce ensure remote/offline request behavior and existing Unity
gameplay do not regress.

## Dev 1.63 local-file connection and Hook provenance batch

### Implementation boundary

- **Local-file URL connections (general partial Foundation contract):**
  `NSURLConnection` now recognizes represented local-file requests, snapshots
  the request, retains its delegate through completion/cancellation, and
  schedules asynchronous work on the initiating/current run loop. Successful
  asynchronous callbacks are delivered in response, data, finish order; a
  missing file reports an actual file-not-found error. The synchronous API
  returns the real bundled bytes plus an `NSURLResponse`. `start`, `cancel` and
  explicit run-loop scheduling are bounded by a one-shot connection state.
- **Remote behavior deliberately unchanged:** non-file requests continue down
  the existing offline/nil path. This is not arbitrary network success and does
  not pretend that unavailable services answered.
- **Hook pair diagnostic-only:** `NSMutableDictionary setObject:forKey:` still
  rejects a nil object. At that terminal boundary it now reports the key class,
  a safely decoded string key when applicable, and guest thread/PC/LR. It does
  not accept invalid dictionary contents or invent the missing value.
- **No per-app branch:** there is no Bulba, HookWorld or QuickHook identifier,
  executable-name check or hard-coded resource/value bypass in the source
  change.
- **Verification status:** the implementation is a partial contract, not yet a
  device-verified compatibility result. A guest contract fixture covers
  synchronous bytes/response and asynchronous callback order, but this branch's
  workflow skips the full integration-test step, so the fixture is recorded as
  added rather than executed.

Predicted immediate outcomes are intentionally narrow. Bulba may advance past
the frozen language-resource screen or expose a later blocker. HookWorld and
QuickHook may still terminate at the same invalid dictionary insertion, but the
new key/caller details should discriminate the remaining producer hypotheses.
Paper Monsters and Bounce should retain their previously observed behavior.

### Dev 1.63 build and verification record

- Final repository commit:
  `cf293e45bba066b22cc6077bae6dbe3cbbf2ec01`
  (`Implement local-file URL connections`). It is pushed to
  `origin/ci/unsigned-ipa-baseline`; the checkout was clean at packaging time.
- GitHub unsigned-IPA run `41199950938` completed successfully for that exact
  commit. The separate iOS archive compile, development identity/package
  verification and artifact upload steps passed. Artifact ID `13042600533` is
  `iDared32bit-dev-unsigned-cf293e45bba066b22cc6077bae6dbe3cbbf2ec01`.
- Downloaded artifact ZIP SHA-256:
  `C93E755544AA8DA16A79E3F9DDF3745FBF380D79F2E5FF753CA1A0743870FA09`,
  exactly matching GitHub's recorded artifact digest.
- Final IPA SHA-256:
  `5564F8F23727F3197CAB655EA1AFCDF782B6C56F6BAB31C81A9CC8AAC7FB892B`.
- Local package inspection found 36 entries and exactly one app bundle,
  `Payload/iDared 32bit.app`, with no embedded provisioning profile and no
  `_CodeSignature` entries. Its binary plist contains development identifier
  `com.bookieart1.idared32bit.dev`, display name `iDared Dev`, and executable
  `iDared 32bit`; the packaged executable contains commit prefix `cf293e4` and
  workflow run `41199950938`.
- GitHub lint/test run `41199951063` completed successfully. macOS and Windows
  development library tests passed; macOS, Windows and Android lint passed.
  Formatting checks, full host/integration `Test` steps and the macOS native
  Objective-C sanity check were skipped by the development-branch workflow, so
  they are not claimed as passes. The local Windows checkout has no Rust/iPhone
  guest toolchain, so no additional local compile or guest test is claimed.
- Direct device-testing IPA:
  `artifacts/1.63-cf293e45/iDared-Dev-1.63.ipa`.

### Focused Dev 1.63 device test

Preserve the existing development containers and saves; do not reset or delete
them.

1. **Bulba the Cat** — launch it first and wait briefly at the screen that was
   previously frozen/zoomed. Report whether the screen advances or becomes
   normally interactive. If gameplay becomes available, play briefly, return
   normally, and reopen once to check that the prior resource load and save
   path remain stable.
2. **HookWorld** — launch once. If it terminates, that is sufficient; preserve
   the new nil-dictionary line with its key and guest PC/LR.
3. **QuickHook** — launch once under the same rule. Do not keep tapping after a
   terminal failure.
4. **Paper Monsters** — play briefly, then return and reopen once to confirm its
   prior gameplay and save behavior did not regress under the shared Unity/file
   path.
5. **Bounce** — launch to its previously recovered menu/gameplay state and exit
   normally through Home. This checks that remote/offline request behavior was
   not accidentally broadened.

One combined log is sufficient. Spoing and Spirits do not need replaying for
this narrowly scoped batch.

## Dev 1.63 device result and Hook identity boundary

The returned log embeds expected commit prefix `cf293e4` and workflow run
`41199950938`. It contains two Bulba launches, one HookWorld launch, one
QuickHook launch, two Paper Monsters launches and one Bounce launch.

| App | Dev 1.63 device result | Current interpretation / boundary |
|---|---|---|
| Bulba the Cat | The formerly frozen screen advanced into clean, responsive gameplay. Buttons and play worked without an observed glitch. Progress was made, the app exited normally, and relaunch restored that progress. | **Functionally compatible within the observed menu/input/gameplay/save-relaunch scope.** The log shows real local-file `NSURLConnection` instances repeatedly loading the bundled `ENLanguage.unity3d`, no panic in either session, and normal Home exits. This verifies that the general local-file transport removed the observed blocker; it does not claim exhaustive coverage of every level or optional service. |
| HookWorld | Still terminates before visible UI. | The terminal diagnostic identifies the rejected dictionary key as constant string `qh/uID`, guest thread 0, with caller LR `0x000722a0`. The app logs `Loaded Version: (null)` and equips initial items immediately beforehand. **Blocked.** |
| QuickHook | Same visible result as HookWorld. | It rejects nil for the same constant key `qh/uID` at the corresponding startup stage, with caller LR `0x00052138`. The shared key and parallel flow establish one engine/settings producer problem rather than two independent crashes. **Blocked.** |
| Paper Monsters | A new level was completed, the app exited normally, and relaunch restored the new progress. | **Functionally compatible within the observed gameplay/level-transition/save-relaunch scope.** Both sessions have normal Home exits and no panic. Optional iCloud hooks remain unavailable and are outside this result. |
| Bounce | Continues to work as it did in the recovered Dev 1.62 state. | Focused regression pass for existing Unity gameplay and remote/offline URL behavior. Historical intermittent lag, audio and secondary-button issues remain outside this short check unless separately retested. |

### Hook pair: what the new evidence proves

- The decisive invalid insertion is specifically a missing value for
  `qh/uID`; globally accepting nil dictionary values remains incorrect.
- Both apps reach the same settings phase and fail on corresponding call sites,
  so a shared Rocketcat/QuickHook settings or persistent-identity producer is
  now the primary subsystem boundary.
- The existing general `UIDevice uniqueIdentifier` path already returns a
  non-nil 40-character value and matches the locally available upstream
  implementation. That makes a wholly missing `UIDevice` selector less likely,
  but the current log does not prove which API the game used to obtain
  `qh/uID`.
- Both apps contain Security/keychain-related imports and both attempt a
  read/write open of bundled `data.db3` before continuing from readable state.
  Either is nearby evidence, but neither is proven as the nil producer. A
  guessed identifier, global keychain success stub or writable-app-bundle
  accommodation would therefore be premature.

The next discriminating step is one bounded general diagnostic at the existing
nil-insertion boundary: retain a small per-thread history of recent Objective-C
message returns and report only the immediately preceding nil-returning calls,
plus a short fixed guest-code context around the recorded caller. That should
separate a missing first-run/defaults value, optional-service identity, keychain
lookup and other producer paths without accepting invalid state or inventing an
ID. If that diagnostic still does not reveal the producer, rotate to another
app group rather than repeatedly elaborating the same Hook hypothesis.

No source patch, commit, push, build or save reset was initiated from this
device-result report.

## Dev 1.63 Blot, B'Blast and Addicus rotation result

The returned log embeds the expected Dev 1.63 commit prefix `cf293e4` and
workflow run `41199950938`. It contains two Blot launches, two matching
B'Blast launches and one Addicus launch.

| App | Dev 1.63 device result | Current interpretation / boundary |
|---|---|---|
| Blot | Dying no longer reproduced the historical crash. Entering Classic directly and returning to the home screen worked. Entering Challenges and then using Back triggered severe progressive slowdown affecting both video and audio; once triggered, it persisted, with other buttons producing varying severity. | **Partial.** The old missing `___vsnprintf_chk` death blocker appears removed within this test, but the order-sensitive slowdown is independently unresolved. Both sessions end normally through Home with no panic. The log shows repeated ImageMagick environment probes and continued heap growth while the existing Blot-only `MAGICK_THREAD_LIMIT=1` accommodation is active. That does not yet distinguish a guest workload/resource leak, duplicated scheduling/callbacks, or another state-transition defect. Do not call death universally fixed from one action order, and do not add another performance workaround without bounded measurements. |
| B'Blast | Both launches show a fraction of opening audio, remain black, and terminate. | **Blocked, but advanced to a precise shared contract.** Each run reaches the same four-element `kern.proc` query `(KERN_PROC_PID, 1)`. The current emulator deliberately returns `-1/ENOENT`; guest code immediately calls `___assert_rtn` at source line 43. This matches the historical current-process-information pattern used to inspect `kinfo_proc`, and is not evidence that the assert should be swallowed. A bounded implementation should return the emulated current process record with no traced/debugger flag, while preserving failure for unsupported process queries. |
| Addicus | Launching terminates the entire iDared application rather than returning to the picker. | **Blocked; two faults are now separated.** The first fault is a guest null-page access at PC `0x001d7adc`, called from `-[PHInvocation initWithTarget:selector:arguments:]` (`0x001d33dc + 0xa8`) after initial views are configured. While reporting that fault, the terminal diagnostic itself triggers a second null-page panic before register/stack output; the log then ends. The nested diagnostic panic explains the whole-iDared termination, but fixing the reporter will not fix Addicus's original nil producer. Harden only the bounded diagnostic first, then use the completed register/selector context to classify the app fault. |

### Next engineering boundary

The highest-confidence general correction is the historical 32-bit
`KERN_PROC_PID` read used by B'Blast. Apple's XNU 1504 interface identifies
subtype `1` as `KERN_PROC_PID` and returns a 32-bit `kinfo_proc`; the emulated
process PID is already `1`. Support should be limited to that current PID,
respect caller buffer/length and write semantics, and report an untraced
process. This replaces the current blanket `ENOENT` for a valid query; it does
not fabricate arbitrary process-list results.

Independently, the memory-fault diagnostic's Objective-C candidate inspection
must use fallible guest-memory access. It must never turn the original guest
fault into a second host panic. That change is diagnostic-only: Addicus is
predicted to return to the picker with a complete terminal context, not to
become playable.

Blot should not receive another speculative performance change in the same
batch. Its next useful investigation is bounded, low-frequency measurement
around the Challenges/Back transition to distinguish callback/thread growth,
guest CPU saturation and heap/resource accumulation. The existing app-scoped
ImageMagick thread-limit accommodation remains provisional and did not prevent
the observed slowdown.

No source patch, commit, push, build or save reset was initiated from this
device-result report.

## Dev 1.64 current-process sysctl and fault-diagnostic batch

### Implementation boundary

- **`KERN_PROC_PID` (general partial Darwin contract):** four-element
  `sysctl` queries for the emulator's current PID `1` now return a zero-filled
  32-bit `kinfo_proc` record of 492 bytes. The implemented stable fields are an
  untraced `p_flag`, running `p_stat`, and PID `1`; the sizing query reports the
  historical six-record capacity, an absent PID succeeds with output length
  zero, and a short caller buffer reports `ENOMEM` with consumed length zero.
  Unsupported `kern.proc` subtypes retain the prior failure behavior. This is
  limited to the historical current-process query and does not fabricate a
  process list or globally suppress assertions.
- **Fault reporting (diagnostic-only general hardening):** terminal memory-
  fault inspection now reads a possible Objective-C `isa` and selector string
  through bounded fallible memory access. Invalid diagnostic candidates are
  omitted instead of causing a second emulator panic. This is predicted to
  preserve Addicus's original guest failure context and return control to the
  picker; it does not repair Addicus's original nil producer.
- **No per-app branch:** the change contains no B'Blast or Addicus identifier,
  executable-name check, or app-specific bypass. Any historical app making the
  same supported current-process query receives the same record.
- **Blot deliberately unchanged:** its order-sensitive Challenges/Back
  slowdown still needs bounded scheduling/CPU/resource measurements. Dev 1.64
  makes no performance claim for it and adds no second workaround.
- **Verification status:** the sysctl work is a partial contract within the
  explicit current-PID boundary; the diagnostic work is diagnostic-only. Unit
  coverage checks successful record contents and short-buffer `ENOMEM`
  behavior, but neither app outcome is device-verified yet.

### Dev 1.64 build and verification record

- Final repository commit:
  `759dfcb06c2ca1173dcea0299283b23db6859fa2`
  (`Implement current-process sysctl and harden fault diagnostics`). It is
  pushed to `origin/ci/unsigned-ipa-baseline`; the source checkout is clean and
  matches that remote branch.
- GitHub unsigned-IPA run `41210179965` completed successfully for that exact
  commit. The separate iOS archive compile, development identity/package
  verification, and artifact upload steps passed. Artifact ID `13045309197` is
  `iDared32bit-dev-unsigned-759dfcb06c2ca1173dcea0299283b23db6859fa2`.
- Downloaded artifact ZIP SHA-256:
  `53AFC1EC45BA7411F0CDC4B3BF4A597F6DC29F31C37E8631604ECC972FA91569`,
  exactly matching GitHub's recorded artifact digest.
- Final IPA SHA-256:
  `9842307AAA6ECB7839D9B43B62391B0D71ACC1DC71BC0E9734DDE90B1D277B99`,
  exactly matching the workflow's package-verification output.
- Local package inspection found 36 entries and exactly one app bundle,
  `Payload/iDared 32bit.app`, with no embedded provisioning profile and no
  `_CodeSignature` entries. Its binary plist contains development identifier
  `com.bookieart1.idared32bit.dev`, display name `iDared Dev`, and executable
  `iDared 32bit`; the packaged executable contains commit prefix `759dfcb`,
  workflow run `41210179965`, and branch `ci/unsigned-ipa-baseline`.
- GitHub lint/test run `41210180000` completed with green job conclusions.
  macOS and Windows `cargo test --lib` development-branch steps passed,
  exercising the new sysctl unit tests. The macOS, Windows and Android lint
  commands each returned exit code 1 but are allowed to fail on this branch;
  they are not recorded as passes. Formatting, full host/integration `Test`
  steps and the macOS native Objective-C sanity check were skipped. The local
  Windows checkout has no Rust/iPhone guest toolchain, so no additional local
  compile or guest test is claimed.
- Direct device-testing IPA:
  `artifacts/1.64-759dfcb0/iDared-Dev-1.64.ipa`.

### Focused Dev 1.64 device test

Preserve the existing development containers and saves; do not reset or delete
them.

1. **B'Blast** — launch once. Report whether it gets beyond the prior brief
   audio/black-screen assertion and how far it reaches. If a menu or gameplay
   becomes usable, interact briefly. If it still terminates, preserve the exact
   terminal lines after the `kern.proc` query rather than repeatedly relaunching.
2. **Addicus** — launch once. Report both the original app result and whether
   iDared itself remains alive and returns to the picker. If it still faults,
   preserve the completed register/stack and Objective-C/selector diagnostic;
   do not interpret a clean return to the picker as Addicus compatibility.

Blot does not need replaying for this narrowly scoped build because its
performance path was untouched. Spoing, Bounce and the other working titles
also do not need an unrelated full regression pass. One combined log for the
two launches is sufficient.

## Dev 1.64 B'Blast and Addicus device result

The returned log embeds expected commit prefix `759dfcb` and workflow run
`41210179965`. It contains one B'Blast launch and one Addicus launch.

| App | Dev 1.64 device result | Current interpretation / boundary |
|---|---|---|
| B'Blast | The visible result is exactly unchanged: the same fraction of opening audio, black screen and in-app termination. | **Still blocked, but the internal first blocker changed.** The former `KERN_PROC_PID` assertion is absent, proving that the bounded current-process record was consumed. Startup proceeds through resources, Localytics/SQLite, locale and URL setup before terminating on the newly reached unimplemented libc function `_if_nametoindex`. This is progress to a later general networking-interface contract, not a visible compatibility improvement. |
| Addicus | The screen remains black and the game terminates with an error, but iDared itself stays alive and returns to the app picker. | **Still blocked; diagnostic objective achieved.** The original null-page access remains at PC `0x001d7adc`, with caller `-[PHInvocation initWithTarget:selector:arguments:] + 0xa8`. Dev 1.64 now emits the complete context without a nested reporter panic. R0, R1 and R2 are all zero at the failing host-call trampoline, but the current diagnostic cannot yet name that linked host API. |

### Next engineering boundary

Implement the general `if_nametoindex` name-to-index contract against a small,
stable guest-visible interface table rather than leaking host interface
indexes. Unknown names must return zero with Darwin `ENXIO`; no network request
should be fabricated. This is a partial interface-discovery contract and does
not imply that remote networking is online.

For Addicus, add one bounded terminal diagnostic that decodes the SVC at the
faulting trampoline and reports the already-linked host function name. This is
diagnostic-only and should not swallow the null access or guess an object,
selector or invocation argument. The next device log should identify the exact
API boundary needed to classify the nil producer.

## Dev 1.65 virtual-interface lookup and host-fault naming batch

### Implementation boundary

- **`if_nametoindex` (general partial Darwin contract):** interface names now
  resolve through a stable guest-visible table: `lo0` is index `1` and `en0`
  is index `2`. Unknown names and invalid UTF-8 return zero and set Darwin
  `ENXIO` (`Device not configured`). The implementation logs the decoded name
  and result once per call so the next B'Blast launch will establish which
  historical name it requested. It neither exposes host interface indexes nor
  treats interface existence as proof of online reachability.
- **Host-call fault naming (diagnostic-only):** when a terminal memory fault
  occurs at an A32 linked-host trampoline, the reporter decodes the SVC at
  `PC - 4`, removes the lazy-return flag and maps it through the already-linked
  host-function table. Addicus's next terminal report should therefore name
  the active API at the preserved null fault. Dispatch behavior and the fault
  itself are unchanged.
- **No per-app branch:** neither implementation checks B'Blast or Addicus
  identifiers, executable names or caller addresses. Other apps using the same
  supported contract or diagnostic path receive the same behavior.
- **Status:** `if_nametoindex` is a partial contract within the two-interface
  virtual table; `if_nameindex` and `if_indextoname` are not claimed. The
  Addicus change is diagnostic-only. Neither app is considered fixed until the
  focused device result demonstrates usable behavior.

### Dev 1.65 build and verification record

- The initial implementation commit `9198e93b4d49dab1d30932dce34038167a43cce0`
  failed both iOS and host-test compilation because a borrowed interface-name
  string remained live while `errno` was mutated. No artifact from that failed
  run is used. Final commit `c9c3f7652a18de022ab5e4c29ce61f48492ad2e6`
  owns the decoded name before setting `errno`; the checkout is clean, pushed
  to `origin/ci/unsigned-ipa-baseline`, and matches that remote-tracking ref.
- GitHub unsigned-IPA run `41212045030` completed successfully for the final
  commit. The separate iOS archive compile, development identity/package
  verification and artifact upload passed. Artifact ID `13046364195` is
  `iDared32bit-dev-unsigned-c9c3f7652a18de022ab5e4c29ce61f48492ad2e6`.
- Downloaded artifact ZIP SHA-256:
  `912213A9B84C0C5A289AED79579372DEA4CF3103A975FF74F7D71DE22B99BD9D`,
  exactly matching GitHub's artifact digest. Final IPA SHA-256:
  `7DD2A601EE2B8E63505D68A443634E92757BDEFA524F92AA7BEF41C8D31D7A31`,
  exactly matching the package-verification step.
- Local inspection found exactly one app bundle, `Payload/iDared 32bit.app`,
  with no provisioning profile or `_CodeSignature` entry. `Info.plist` contains
  identifier `com.bookieart1.idared32bit.dev`, display name `iDared Dev` and
  executable `iDared 32bit`; the executable contains commit prefix `c9c3f76`,
  workflow run `41212045030`, branch `ci/unsigned-ipa-baseline` and repository
  `bookieart1/iDared32bit`.
- GitHub lint/test run `41212044973` completed successfully. The inspected
  macOS and Windows development-branch library-test steps passed, including the
  new interface and host-trampoline unit tests. The macOS, Windows and Android
  lint commands still return exit code `1` under allowed-to-fail steps and are
  not recorded as lint passes. Formatting, full host/integration `Test` steps
  and the macOS native Objective-C sanity check were skipped. No local Rust,
  guest-ABI or physical-device test is claimed.
- Direct device-testing IPA:
  `artifacts/1.65-c9c3f765/iDared-Dev-1.65.ipa`.

### Focused Dev 1.65 device test

Preserve the current development app container and saves; do not reset or
delete them. One combined log is sufficient.

1. **B'Blast** — launch once. Report whether the visible brief-audio/black-
   screen result changes and preserve the line
   `if_nametoindex("...") -> ...` plus the next terminal error. If it reaches a
   menu or gameplay, interact briefly before exiting normally.
2. **Addicus** — launch once. Report the visible result and preserve the new
   `active host function` diagnostic at the null-page fault. Returning to the
   picker remains diagnostic containment, not app compatibility.

No Blot or working-title regression replay is required for this narrow batch;
it changes neither Blot's performance path nor normal input/render/audio/save
behavior.

## Dev 1.65 B'Blast and Addicus device result (corrections paused)

The user tested Dev 1.65 from commit
`c9c3f7652a18de022ab5e4c29ce61f48492ad2e6`, workflow run
`41212045030`. Both titles had the same visible result as before. The focused
instrumentation nevertheless established two new internal boundaries:

| App | Visible result | Decisive Dev 1.65 evidence | Status / boundary |
|---|---|---|---|
| B'Blast | Unchanged: a fraction of a second of audio, black screen, then an in-app error/return. | `if_nametoindex("en0") -> 2` proves that the new stable guest interface lookup was consumed successfully. Execution then reached `sysctl()` with a six-integer MIB and stopped at `not implemented: sysctl() for name length 6 is unimplemented!`. The terminal registers show the expected sizing-query shape (`oldp` null and `oldlenp` supplied), but the six MIB integers have not been decoded. | **Blocked.** The interface-name blocker is removed; the first decisive blocker is now the unsupported six-component `sysctl` query. Decode the MIB and its historical contract before designing an implementation. |
| Addicus | Unchanged: black screen, then an in-app error/return; iDared remains alive. | The preserved null-page access at PC `0x001d7adc` now reports `environment: Host memory-fault active host function: _objc_msgSend`. R0, R1 and R2 are zero at the host trampoline, and the guest caller remains `Addicus!-[PHInvocation initWithTarget:selector:arguments:] + 0xa8`. | **Blocked; diagnostic-only objective achieved.** The crash boundary is Objective-C message dispatch from the PHInvocation path. The log does not yet prove which earlier operation produced the zero target/selector state. Do not globally tolerate invalid pointers or fabricate an invocation. |

No correction, source commit, push or new build was started from this result.
At the user's request, both investigations are deliberately paused while the
project changes direction. If resumed, the next discriminating observations
are the decoded B'Blast MIB and a bounded Addicus target/selector creation and
lifetime trace immediately preceding the PHInvocation call.

## Development-method audit and Dev 1.66 diagnostic build — October 31, 2026

The user explicitly requested a broad historical/architectural audit, a small
implemented improvement, and a new IPA. This supersedes the earlier pause only
for diagnostic infrastructure; B'Blast/Addicus compatibility fixes remain
paused. Active checkout and remote were verified at `c9c3f765`, clean, branch
`ci/unsigned-ipa-baseline`. Remote upstream trunk was independently checked at
`ca1de89c578e79746f33c1332e28efd106c334ba`, also the merge base. Local history is
shallow, so raw upstream-range commit counts must not be interpreted as a
complete attribution of inherited work.

The repository record is `dev-docs/compatibility-development-audit.md`. It
compares six opportunities, identifies existing GDB/linking-info/fault-context
facilities, records provisional implementations and exception-suppression risk,
and chooses bounded API/Objective-C history with scheduler progress samples.
The next larger investment is a repaired strict guest ABI fixture pipeline.
Earlier audits/CSVs and all existing compatibility evidence remain unchanged.

### Implementation and predictions

- **Classification:** diagnostic-only infrastructure, not a compatibility fix.
- **Commits:** `766978fd` implementation; `677e505c` corrects a borrowing error in
  the new Objective-C unit-test setup and wraps three new comment lines. The
  first CI library compilation caught the error before delivery; superseded
  queued/build jobs were cancelled. Dev label remains 1.66, not yet delivered.
- **Files:** new `src/diagnostics.rs`; dispatch hooks in `environment.rs`,
  `objc/messages.rs`, `objc/methods.rs`; source counts in `ns_run_loop.rs`;
  options/help/default presets; one CI step prints the synthetic failure proof.
- **Bounds:** 96 recent call boundaries, 24 candidate nil/resource events,
  24 progress samples, under 80 KiB recorder storage per enabled app, at most
  12 live five-second lines, one final report on panic or return to picker.
  Recording does no per-call allocation/formatting/log I/O/clock read. Routine
  GL/memory/ownership calls are counted rather than individually retained.
- **Identity:** capture defaults off; the Dev 1.66 cohort opts in B'Blast,
  Addicus, Blot and Spoing. These app options affect only recording. No saves,
  package identity, API responses, memory protections or exception behavior
  were changed. An options override can disable capture.
- **Expected new evidence:** B'Blast's actual MIB; preceding Addicus dispatch
  and host object-return values; Blot rates/source-count trends during the
  Challenges→Back slowdown; Spoing as a working capture/control session.
  Spirits is already confirmed working and is not treated as a current crash.
- **Limits:** candidate nils may be normal; raw registers are not typed returns;
  host-internal LR can be inherited; no memory-write provenance or host-blocking
  watchdog; no final-report guarantee after an OS kill; normal nil floods can
  overwrite candidates. Capture overhead on physical hardware remains to test.

### Validation in progress

Local changed-file Rust formatting and diff whitespace checks passed using an
isolated workspace Rust toolchain. Native host compilation is unavailable here
without C++ build prerequisites; library and iOS builds use hosted CI. The first
Windows test compilation found a test-only borrowing error, fixed in `677e505c`.
The Android lint job's green badge masked an allowed failure in the comment-line
check; it did not reach clippy. No Android compile/lint pass is claimed.

Tests added: circular history order/wrapping; invalid diagnostic pointers and
interleaved thread identities with unchanged registers; real Objective-C nil
object versus numeric zero; bounded progress/storage/escaped text; a synthetic
six-word sysctl query through actual dyld SVC dispatch with capture off and on,
requiring the same original panic and the decoded MIB only when enabled. The
fixture is the same failing API shape as B'Blast, not a reproduction of its
unknown exact MIB or an execution of the game binary.

Device results, final hosted test results and verified downloaded IPA identity
will be appended below. Do not promote this build to device-verified status.

### Dev 1.66 final host verification

Final build source is `288a5e40966d5a322ab660344d01e8c4dacb2531`. Two additional
test-fixture commits, `c8385215` and `288a5e40`, establish null-page protection
before dyld allocates its stubs. This preserves the original unprotected fake
environment for existing tests and gives the invalid-pointer test realistic
loader setup. The interim second run had 89 passing tests and one fixture
failure (zero was readable because no binary had established the null segment).
No guest memory contract or production code was relaxed to make it pass.

- Test workflow `41216006091`: **90/90 library tests passed on macOS and Windows**.
- Separate macOS reproducer evidence step: **passed**. It printed the original
  unsupported six-component sysctl panic plus the new entry event:
  `_sysctl`, `args=[00009150, 00000006, 00000000, 00000000]`, four stack words,
  `mib=[4, 17, 0, 0, 3, 2]`, thread 0, PC `0x9184`, call sequence 1. These are
  synthetic fixture addresses/values, not B'Blast's recovered device query.
- All three lint steps stopped at the pre-existing comment-length gate and
  were allowed failures. **No clippy, rustdoc or Android compile pass claimed.**
- Full guest integration, native Objective-C sanity and workflow formatting
  remain skipped. Changed-source formatting was checked locally.
- Final test logs: `outputs/dev166-final-macos-job.txt`,
  `outputs/dev166-final-windows-job.txt`, `outputs/dev166-final-android-job.txt`.
- Device testing is pending. No new compatibility improvement is claimed.

### Dev 1.66 verified IPA and focused device test

- IPA workflow `41216006158` completed successfully for
  `288a5e40966d5a322ab660344d01e8c4dacb2531`.
- Artifact `13048746075`, name
  `iDared32bit-dev-unsigned-288a5e40966d5a322ab660344d01e8c4dacb2531`.
- Downloaded IPA: `artifacts/1.66-288a5e40/iDared-Dev-1.66.ipa`;
  19,867,195 bytes; SHA-256
  `7b02d9631af24a7e789cdb1001027b2aec35002e168a0a8b6210ce794e7599c1`,
  matching the workflow's printed checksum. Outer artifact ZIP SHA-256
  `0aadf7dc07d1094673a4a7ad435badf14df6487a93c83fc67b36ef850854dcc9`
  matches GitHub's artifact digest.
- Independent package inspection confirmed ZIP integrity, arm64 executable,
  embedded commit/run, recorder marker, all four preset flags, separate
  `com.bookieart1.idared32bit.dev` / `iDared Dev`, no provisioning profile and
  no Mach-O code signature. Metadata saved beside the IPA as
  `iDared-Dev-1.66.verification.json`; iOS log is
  `outputs/dev166-final-ios-job.txt`.
- A follow-up documentation-only commit records final validation without
  rebuilding. It does not change the delivered IPA's source identity.

Install over the existing development app, retaining its data. One combined
log is enough:

1. B'Blast: launch once and allow the existing error to occur.
2. Addicus: launch once and allow the existing error to occur.
3. Blot: play Classic about 15 seconds, return to its menu, open Challenges,
   press Back, wait about 20 seconds after slowdown, then use emulator Home.
4. Spoing: play briefly/finish a level, use emulator Home, reopen and check
   existing progress. Report any new slowdown or instability.

On receipt, first verify embedded build identity and recorder activation.
Extract the actual MIB, the sequence before PHInvocation, and main-loop/
presentation/source-count deltas around slowdown. Compare Spoing's functioning
session. Judge whether each narrows a competing hypothesis; do not count extra
lines alone as success. If Addicus's origin remains outside the retained
history, select a precise producer/lifecycle probe from the recovered caller
instead of automatically enlarging the ring. Device overhead remains an open
question until this test.

### Dev 1.66 device result: flight recorder validated

The combined device log identifies source `288a5e4` and workflow
`41216006158`, matching the verified Dev 1.66 IPA. Recording was enabled for
all four intended apps. The user followed the requested order and reported no
new instability in the working control.

| App | User-visible result | New bounded evidence | Conclusion / next boundary |
|---|---|---|---|
| B'Blast | Same short audio fragment, black screen and error. | Immediately after `if_nametoindex("en0") -> 2`, `TFDeviceInfo -macAddressSha1` issues `_sysctl` with the actual MIB `[4, 17, 0, 18, 3, 2]`, `oldp == NULL` and an `oldlenp` sizing pointer. This decodes as `CTL_NET`, `PF_ROUTE`, protocol 0, `AF_LINK`, `NET_RT_IFLIST`, interface index 2. | **Recorder success; app still blocked.** This is a link-layer interface-record request for `en0`, used on the MAC-address hash path. A future correction must implement the historical two-call sizing/data contract and correct 32-bit route/interface structures; returning arbitrary success is not justified. |
| Addicus | Same black screen and in-app error. | `PHWebView` receives `loadHTMLString:baseURL:` and asks `PHInvocation` to perform that selector on the underlying `UIWebView`. `+[UIWebView instanceMethodSignatureForSelector:loadHTMLString:baseURL:]` returns `nil`; that exact `nil` is immediately passed to `+[NSInvocation invocationWithMethodSignature:]`, followed by the preserved null-page access. | **Recorder success; producer recovered.** The first missing compatibility contract is `UIWebView -loadHTMLString:baseURL:`, which the current class does not export. The later `NSInvocation` nil dereference is also non-ideal failure handling, but globally accepting a nil signature would mask the real missing API. A future implementation must label the no-web-engine behavior boundary and cannot claim functional HTML compatibility merely for passing startup. |
| Blot | Classic and menu return were smooth. Challenges→Back triggered the severe audiovisual slowdown, which persisted. | Presentation rate rises to roughly 35–57/s during the smooth interval, drops to 27/s over 45–50 s, then remains about 3.2–4.4/s from 50–85 s. Main-loop rate falls with it. From 15 s onward, source counts remain `[2 timers, 0 selectors, 1 audio queue, 0 audio units]`; the process retains one thread slot rather than accumulating threads/sources. | **Useful phase classification, not root cause.** The observed slowdown is real and sustained, but the sampled evidence rejects simple run-loop-source proliferation. It is more consistent with increased work or blocking on the main guest/emulation path than with a deadlock or stopped presentation. The next diagnostic should freeze one bounded boundary/PC snapshot at the first slow-loop/present threshold, not enlarge the general ring or add unbounded frame logging. |
| Spoing | Level completion, emulator Home, reopen and saved progress all worked normally. | Recorder remained enabled for a roughly 130 s session and a 12 s reopen. It produced bounded reports rather than per-call log flooding despite very high internal boundary counts. No panic occurred. | **Working control passed within this scope.** This does not quantify recorder overhead against an instrument-off baseline, but it shows no visible gameplay or persistence regression in this test. |

This result meets the first-version success rule: it produced causal evidence,
not merely larger logs. B'Blast and Addicus each advanced from a terminal
symptom to a specific missing shared contract; Blot eliminated one competing
hypothesis and established a measurable transition; Spoing remained a passing
control. Keep v1 bounded and opt-in. If Blot is investigated next, add a
single-shot slow-transition capture rather than enabling global instruction
tracing. No code change, push, IPA build or compatibility claim follows from
this log-analysis entry.

## Dev 1.67 recovered-contract batch — October 31, 2026

This batch acts on the specific contracts recovered by Dev 1.66. It contains
two general compatibility implementations and one bounded Blot investigation;
it does not claim that all three apps are fixed before physical-device testing.

### Implementation boundary and rationale

- **B'Blast / networking-interface contract (partial verified-contract
  candidate):** `sysctl` now recognizes only the decoded six-component
  `CTL_NET/PF_ROUTE/AF_LINK/NET_RT_IFLIST` query for the emulator's existing
  virtual `en0` interface at index 2. It implements both the sizing call and
  the data call and returns one 132-byte 32-bit Darwin route message containing
  an `if_msghdr`, 96-byte `if_data`, and `sockaddr_dl`. The record describes a
  deterministic virtual Ethernet link with a locally administered MAC address;
  it does not expose or hash the host device's real hardware identity. Writes,
  malformed arguments, short buffers and unrelated six-component MIBs remain
  strict failures. Layouts and constants were checked against Apple's
  `xnu-1504.15.3` headers (`route.h`, `if.h`, `if_dl.h`, `socket.h` and
  `if_types.h`, tag commit `99c11f3f90a18f35d3692ea77444b51fd972a4f4`).
- **Addicus / UIWebView method surface (partial contract):** `UIWebView` now
  exports the historical `loadHTMLString:baseURL:` selector. This gives
  `instanceMethodSignatureForSelector:` and `NSInvocation` the real Objective-C
  signature instead of a fabricated `NSInvocation` response. The method
  accepts the call and records only bounded length/base-URL context. touchHLE
  still has no WebKit renderer, DOM or JavaScript engine, so this implementation
  deliberately sends no false load-finished callback and makes no claim that
  HTML is displayed. The device test will establish whether Addicus used this
  surface only as startup plumbing or immediately requires a web engine.
- **Blot / first sustained slowdown capture (diagnostic-only):** the opt-in
  flight recorder now recognizes a transition only after at least 30 healthy
  presentation intervals followed by three consecutive intervals of at least
  180 ms. It then freezes the 32 call boundaries that preceded the first slow
  interval, the first gap, boundary delta and presentation count. Requiring a
  short sustained collapse prevents a normal one-frame scene-loading hitch
  from consuming the one-time capture. It does not alter scheduling, timers,
  graphics, audio or the existing provisional `MAGICK_THREAD_LIMIT=1`
  accommodation. No speculative Blot performance fix is included because Dev
  1.66 disproved source/thread proliferation but did not identify the expensive
  or blocking operation.
- **Generality:** neither compatibility implementation checks an app bundle
  identifier. The sysctl behavior applies to callers of the exact historical
  virtual-link query; the UIWebView method applies to every caller. Bundle
  presets only keep bounded recording enabled for B'Blast, Addicus, Blot and
  the Spoing control during this focused retest.

### Predictions, risks and validation plan

- B'Blast should consume both interface-list calls and either advance visibly
  or expose a later independent blocker. Success means a valid record was
  consumed, not merely that the old panic text disappeared.
- Addicus should no longer create a nil method signature at the recorded
  `PHInvocation` path. It may still remain black or reach a later web-rendering
  limitation; that outcome would define the honest implementation boundary.
- Blot is not predicted to run faster in this build. Reproducing the exact
  Classic→menu→Challenges→Back sequence and then using emulator Home should
  print one frozen `first sustained slow-present transition` section. If it
  reports `not observed`, thresholds or trigger placement must be reconsidered
  rather than adding unbounded logging.
- Regression exposure is narrow: callers parsing route-interface records,
  code introspecting or invoking `UIWebView`, and opt-in diagnostic sessions.
  Spoing remains the focused recorder/control check. Saves and app-container
  identity are unchanged.

Unit tests cover the virtual route-record sizing/data layout, short-buffer
failure, the exported UIWebView signature/argument count, and the synthetic
healthy-to-three-slow-interval transition. Local changed-file formatting and
whitespace checks pass. Native Windows linking is unavailable in this
workspace because the MSVC linker is absent, so no local compiled-test pass is
claimed; hosted macOS/Windows tests and the iOS package build remain required.
Build commit, workflow results, downloaded IPA identity and device evidence
will be appended after they exist.

The first hosted test run (`41219121194`) compiled the new implementations and
passed both route-record tests plus the slow-transition test, but exposed two
test-only assumptions. The retained unsupported-sysctl reproducer still
expected the obsolete generic panic wording, and the UIWebView test tried to
construct an autoreleased `NSMethodSignature` in the deliberately threadless
fake environment. The follow-up keeps the unsupported synthetic MIB strict and
checks the exported host method's exact `v@:@@` Objective-C encoding directly;
no production behavior or Dev label changes.

### Dev 1.67 final host verification and IPA

Final source is `9159a0a6bb17c9deea3290db8c30c594c2bf16cb`. The final
commit only wraps two new comments to the project's 80-character convention;
it does not change the tested runtime behavior from `a51774f4`.

- Test workflow `41219859506` completed successfully. The library suite passed
  **94/94 on macOS and 94/94 on Windows**. The macOS diagnostic reproducer also
  passed and retained the original unsupported synthetic MIB panic plus its
  decoded flight-recorder entry.
- The macOS, Windows and Android lint steps remain allowed failures at the two
  pre-existing long comments in `src/libc/dlfcn.rs:61` and
  `src/frameworks/foundation/ns_dictionary.rs:531`. The two Dev 1.67 comments
  initially reported by that gate were corrected. The gate still prevents
  clippy, rustdoc and Android compilation, so none of those passes is claimed.
  Workflow `Test`, `Sanity check tests`, and the corresponding full guest/native
  integration checks remain skipped by their existing conditions.
- IPA workflow `41219859529` (run 109, job `124591983798`) completed
  successfully. Artifact `13049791223`, name
  `iDared32bit-dev-unsigned-9159a0a6bb17c9deea3290db8c30c594c2bf16cb`,
  has GitHub artifact ZIP SHA-256
  `befae638658c7edf11054f544e00b8919585a2bf763167d85a5826b29013227c`.
- Downloaded IPA: `artifacts/1.67-9159a0a6/iDared-Dev-1.67.ipa`;
  19,888,553 bytes; SHA-256
  `cd349f76e72fc2f0284e1302f9deb2f729c3576c33a1abbf8ec2688a8f8c06fc`,
  matching the workflow's printed checksum. The outer downloaded artifact ZIP
  is 19,842,232 bytes and matches GitHub's displayed digest.
- Independent inspection passed the ZIP CRC check and confirmed the arm64
  executable, embedded commit abbreviation `9159a0a`, run `41219859529`,
  branch/repository identity, slow-transition marker, all four focused default
  option entries, separate `com.bookieart1.idared32bit.dev` / `iDared Dev`
  identity, no provisioning profile and no code-signature directory. Details
  are saved beside the IPA in `iDared-Dev-1.67.verification.json`.

Physical-device status remains **untested**. The route-message and UIWebView
contracts are host-tested general implementations, not yet proof that B'Blast
or Addicus reaches gameplay. Blot receives diagnostic evidence only and is not
predicted to run faster. Use one retained-data device pass: B'Blast launch and
brief interaction if it advances; Addicus launch and brief interaction if it
advances; Blot Classic about 15 seconds, menu, Challenges, Back, wait about 20
seconds after the slowdown and use emulator Home; Spoing brief play, emulator
Home, reopen and verify progress. Return one combined log and visible results.

### Dev 1.67 device result: all three intended boundaries advanced

The combined device log identifies source `9159a0a6` and workflow
`41219859529`, matching the independently verified Dev 1.67 IPA. The requested
B'Blast, Addicus and Blot sequences were run. Spoing was omitted in this pass;
its earlier Dev 1.66 control result remains historical evidence, not a new
regression pass.

| App | User-visible result | Decisive evidence | Status / next contract |
|---|---|---|---|
| B'Blast | Unchanged brief audio fragment, black screen, then in-app termination. | Both launches consume `if_nametoindex("en0")` and the new route-interface query. They next stop at `_NXGetArchInfoFromCpuType`; terminal arguments are CPU type 12 and subtype 9, the historical ARM/ARMv7 pair. | **Advanced internally, still blocked.** The virtual-link record was accepted. The next missing contract is the shared Mach-O architecture lookup, not another networking guess. |
| Addicus | A publisher logo appeared very briefly for the first time, then the app terminated. | `UIWebView -loadHTMLString:baseURL:` is accepted and the old nil method-signature path is absent. Startup proceeds through audio setup and stops at `_UIImagePNGRepresentation` with a non-nil `UIImage`. | **Visible and internal advance, still blocked.** The partial UIWebView surface did what was predicted without pretending to render HTML. The next missing contract is general UIKit PNG serialization. |
| Blot | The requested Classic→menu→Challenges→Back sequence still triggers the severe persistent audiovisual slowdown. | The one-shot capture froze at 56.332 s after a 275 ms first slow presentation gap. It records `qsort` over 1,356 four-byte elements with guest comparator `0xef61`; later five-second progress samples repeatedly sit inside that comparator. Source inspection shows the emulator's current recursive quicksort always selects the first element, giving quadratic guest callbacks on sorted, reverse-sorted or equal-heavy input. | **High-confidence general emulator bottleneck; device correction still unverified.** Replace the pathological algorithm, not Blot timing or rendering. The capture does not prove that no later Blot workload exists. |

The diagnostic infrastructure again improved causal resolution: each old
terminal point disappeared, the next two missing functions are named with
arguments, and Blot's transition is tied to a specific emulator algorithm.
No memory fault was suppressed and no app-specific response is justified by
this result.

## Dev 1.68 shared contracts and `qsort` correction — built, device test pending

This batch is intentionally limited to the three boundaries established by
the verified Dev 1.67 device log.

- **B'Blast / Mach-O architecture lookup (verified-contract candidate):**
  implement `NXGetArchInfoFromCpuType` for the legacy 32-bit ARM family and
  known ARMv4T, ARMv5, ARMv6, XScale, ARMv7, ARMv7F, ARMv7S and ARMv7K
  subtypes. Returned guest
  `NXArchInfo` structures and strings have stable process lifetime, include
  the historical names/descriptions and little-endian value, and are cached.
  `CPU_SUBTYPE_MULTIPLE` selects the ARM family; unknown CPU types/subtypes
  still return null as Apple's contract specifies. The implementation is not
  keyed to B'Blast and does not claim the other `mach-o/arch.h` APIs.
- **Addicus / PNG representation (verified-contract candidate):** implement
  `UIImagePNGRepresentation` for the decoded bitmap-backed `UIImage` objects
  touchHLE already supports. It returns autoreleased `NSData`, returns nil for
  nil or absent image data, and uses the existing PNG encoder. A shared image
  helper converts touchHLE's premultiplied RGBA storage to PNG straight alpha;
  the existing saved-photo path now reuses the same encoding rather than
  maintaining a second implementation. This does not add unsupported image
  sources, JPEG encoding or a WebKit renderer.
- **Blot / standard-library performance and stack safety:** replace the
  recursive first-pivot quicksort with iterative heapsort. C `qsort` does not
  promise stability or comparator order, while heapsort preserves the required
  ordering contract and guarantees O(n log n) comparisons for the ordered and
  duplicate-heavy inputs that made the former implementation quadratic. The
  change is general and removes unbounded recursion; it contains no Blot check,
  frame-rate override or scheduler accommodation.

Unit coverage checks legacy ARM lookup selection/rejection, PNG signature and
premultiplied-pixel round-trip, ordinary `qsort` ordering, and fewer than
50,000 comparisons for Blot-sized sorted, reversed and equal inputs. The prior
algorithm requires roughly 918,000 comparisons for its sorted worst case.
Local whitespace checks pass. The local workspace lacks MSVC `link.exe`, so
compilation was validated in the hosted development-branch library tests
instead. Predictions are narrow: B'Blast and Addicus should
either advance to their next independent boundary or become usable; Blot's
Challenges→Back collapse should be eliminated or materially reduced if this
captured sort is causal. A different result must revise the model rather than
add per-app timing hacks.

### Dev 1.68 source, hosted validation and package verification

- Source commit: `2836802443e634b3ab0f77428648f3fd612a374b`.
- Lint/test workflow `41221870862` completed successfully. Android lint passed;
  the macOS and Windows development-branch library-test steps passed. The
  workflow's release-only full-app test and legacy guest/sanity steps were
  skipped, as configured for this development branch; formatting was also a
  configured skip rather than an independently executed pass.
- Unsigned-iPhone workflow `41221870864` completed successfully, including
  iOS compilation, separate development identity verification, packaging and
  artifact upload.
- Artifact `13049823496`,
  `iDared32bit-dev-unsigned-2836802443e634b3ab0f77428648f3fd612a374b`, is
  19,832,427 bytes. Its downloaded ZIP SHA-256 exactly matches GitHub's
  recorded artifact digest:
  `72341B3AF005F12FAD901C27E7809726FDF4C3799E0E951CEDECCE158CCE2FAE`.
- The extracted IPA is 19,880,488 bytes with SHA-256
  `D6AF7754DD30AB7CB193F454A93C18EF21DFD337386EE599E5B31FC282A6F7E3`.
  It contains 36 archive entries, identifies as
  `com.bookieart1.idared32bit.dev` / `iDared Dev`, embeds source prefix
  `28368024`, and contains neither `_CodeSignature` nor
  `embedded.mobileprovision`.
- Local handoff: `outputs/iDared-Dev-1.68.ipa`.

Focused device test: install over the development app without clearing its
container; launch B'Blast and interact briefly if it advances; launch Addicus
and interact briefly if it advances; in Blot run Classic for about 15 seconds,
return to the menu, open Challenges, press Back and wait about 30 seconds; then
use Spoing as the control by completing one level, exiting with emulator Home,
reopening and confirming progress. Return the combined log and visible results.

### Dev 1.68 device result: Blot fixed; two later shared contracts recovered

The combined log identifies source `2836802` and workflow `41221870864`, which
matches the independently verified Dev 1.68 IPA. B'Blast and Addicus each ran
twice and stopped at the same new boundary on both runs. Blot and Spoing were
also exercised without save deletion.

| App | User-visible result | Decisive evidence | Status / next contract |
|---|---|---|---|
| B'Blast | Same tiny audio fragment, black screen and termination as before. | The new ARM architecture lookup is consumed. Both launches next send `localizedName:locale:` to a valid `NSTimeZone`, with style `1` (short standard) and a non-nil locale, then stop because the selector is absent. | **Advanced internally, still blocked.** Within touchHLE's existing fixed-GMT time-zone model, the next shared Foundation contract is deterministic. The unchanged visible symptom is not evidence that Dev 1.68 had no effect. |
| Addicus | Publisher logo and, for the first time, the main menu appeared briefly before termination. | The new PNG representation is consumed. Rendering reaches the main loop and EAGL presentation. Google Analytics then creates `GANTCPSocket` and calls `connect:port:` with port 80 immediately before `_CFHostCreateWithName` is requested. | **Visible and internal advance, still blocked.** The first new blocker is an optional analytics/DNS path, not evidence that gameplay initialization itself failed. A correction must model an unavailable resolver honestly rather than claim a successful network connection. |
| Blot | Classic, menu return, Challenges and Back remained smooth despite repeated menu interaction; the old persistent audiovisual slowdown could not be triggered. | The session ran about 95 seconds, returned through emulator Home, and recorded no sustained slow-present transition. | **Functionally fixed in the tested scope.** The general iterative `qsort` replacement is now device-verified for the captured Blot regression. This does not prove every Blot mode or every C comparator workload. |
| Spoing | Brief play remained normal; no full level completion or save/relaunch check was performed in this pass. | The recorder ran for roughly 25 seconds and the app returned normally through emulator Home without a panic. | **Partial regression observation only.** Earlier complete level/save checks remain historical evidence; this pass does not replace them. |

The result supports the Dev 1.68 model: all three intended old boundaries
disappeared, Blot's captured `qsort` bottleneck was causal, and the two still
failing apps exposed later independent APIs. No game-specific response or
weakened crash handling is justified by this log.

## Dev 1.69 time-zone naming and offline CFHost surface

This batch is restricted to the two contracts established by the Dev 1.68
device log. It contains no new Blot change and no diagnostic-system change.

- **B'Blast / `NSTimeZone -localizedName:locale:` (partial contract within an
  existing model):** implement all six historical `NSTimeZoneNameStyle`
  values. touchHLE already deliberately represents every zone as fixed GMT,
  so each valid style returns `GMT`; an unknown style returns nil. The locale
  does not change the spelling of GMT in this limited model. This is general
  Foundation behavior and contains no B'Blast check. It does not add a time
  zone database, daylight-saving rules, or host-device time-zone leakage.
- **Addicus / CFHost lifecycle and explicit offline resolution (partial
  contract):** add an owned CFHost object for `CFHostCreateWithName` and
  `CFHostCreateCopy`, including correct name retention and CFRelease-compatible
  teardown. The tightly connected client/scheduling/start/get/cancel surface is
  present so callers can handle unavailability without falling into another
  missing-symbol panic. Non-null asynchronous client registration returns
  false, resolution start returns false with POSIX `ENETUNREACH`, and result
  getters return nil with `hasBeenResolved == false`. No address, successful
  callback, DNS lookup, socket or network access is fabricated. Run-loop
  schedule calls have no work after client registration has been rejected.
- **Implementation status:** the time-zone result is exact only within the
  emulator's pre-existing GMT-only boundary. CFHost is an intentionally
  offline partial contract, not a resolver implementation. It should let
  optional service code disable itself; software requiring working DNS remains
  unsupported. Neither implementation checks a bundle identifier.

Focused unit tests cover all six time-zone styles plus invalid-style rejection,
and CFHost creation, retained name, explicit resolution failure/error, and
unresolved addressing state. Changed-file formatting and whitespace checks
pass. The local Windows environment still lacks MSVC `link.exe`; its dependency
cache was refreshed successfully, but no local compiled-test pass is claimed.

Predictions: B'Blast should leave the recorded time-zone call and expose its
next independent boundary or render further. Addicus should reach and handle a
legitimate offline analytics result; if the SDK requires asynchronous resolver
callbacks even for failure, that limitation should become explicit rather than
being hidden by a fake success. Blot should remain smooth and Spoing remains a
focused working control.

### Dev 1.69 hosted validation and package verification

- Implementation commit `8d5a2af4a3551b58f06d85d61381a5ea652ede5a`
  added the two contracts. Its first hosted test run (`41246604591`) caught a
  test-only `&str`/`String` compile mismatch. Commit `0d464376` corrected that;
  the next test run (`41246902706`) compiled and ran 101 tests, exposing a
  signed/unsigned ABI mismatch only in the new time-zone test loop. Final source
  commit `acf10ae95011950b1028caa4a082bf7f5e0bfcad` corrected the fixture to
  use the method's `NSUInteger` ABI. No production behavior changed in either
  follow-up.
- Final lint/test workflow `41247250362` (run 48) completed successfully. The
  macOS and Windows development-branch library suites each passed all 101
  tests; the macOS diagnostic reproducer step also passed. The release-only
  full-app test, legacy sanity-test and formatting steps were configured skips.
  The development branch's allowed-to-fail lint script still stops at two
  pre-existing long comment lines in `dlfcn.rs` and `ns_dictionary.rs`, so this
  run does not claim clippy, rustdoc or Android compilation.
- Unsigned-iPhone workflow `41247250422` (run 113) completed successfully,
  including iOS compilation, separate development identity verification,
  unsigned packaging and artifact upload. Earlier artifacts from the two
  superseded test-fixture commits were not handed off.
- Artifact `13060195873`,
  `iDared32bit-dev-unsigned-acf10ae95011950b1028caa4a082bf7f5e0bfcad`, is
  19,830,601 bytes. Its downloaded ZIP SHA-256 exactly matches GitHub's digest:
  `726089DA14E2BC737ED7DCE349930E8302638E48E90831735B6A0E3A651E4A6F`.
- The extracted IPA is 19,877,048 bytes with SHA-256
  `6A0FF9E5B60ADC7009CA484F1EC94449ECD5DEA8E1731C28E7AFC6C3BA7011E2`.
  All 36 ZIP entries read successfully. It identifies as
  `com.bookieart1.idared32bit.dev` / `iDared Dev`, contains an arm64 Mach-O,
  embeds source prefix `acf10ae`, workflow `41247250422`, the expected branch
  and repository, and contains neither `_CodeSignature` nor
  `embedded.mobileprovision`.
- Local handoff: `outputs/iDared-Dev-1.69.ipa`; complete package evidence is in
  `outputs/iDared-Dev-1.69.verification.json`.

Focused device test: install over the development app without deleting its
container. Launch B'Blast once and report whether it advances beyond the black
screen. Launch Addicus, leave its menu untouched for about 20 seconds, then
start a game and return to the menu if it remains open. Repeat Blot's
Classic→menu→Challenges→Back path once as the regression check. Brief Spoing
play is sufficient as the working control; a full level is not required in
this pass. Return the combined log and visible results.

### Additional Dev 1.68 Blot result: smooth long session, death-path formatting crash

This later observation came from the verified Dev 1.68 build (`2836802`,
workflow `41221870864`), not Dev 1.69. Blot ran smoothly for approximately
151 seconds. The recorder counted 7,905 main-loop iterations and 7,714 EAGL
returns and explicitly reported no sustained slow-presentation transition.
That reinforces the earlier conclusion that the general `qsort` replacement
removed the captured Challenges→Back performance collapse.

When the player died after the longer session, the emulator panicked at
`src/libc/stdio/printf.rs:418` on `assertion failed: !prepend_sign`. The last
recorded boundary is `_vsnprintf_l`, and that assertion is in the floating-point
`f` conversion path after the parser has accepted the standard `+` flag. This
is evidence of a general missing `printf` formatting contract, not a Blot-only
fault or a recurrence of the sorting problem. The visible association with the
death transition is observed once and is not yet independently reproduced.

Blot is therefore **partial overall**: gameplay and the previously failing menu
path are smooth, but a sufficiently long run can terminate on death. If this is
prioritized later, the next implementation candidate is general signed
floating-point formatting with focused libc tests; no app-specific bypass is
justified. No source fix or new IPA was made for this note.

### Dev 1.69 device result: intended blockers removed, later blockers repeated

The combined log identifies source `acf10ae` and workflow `41247250422`, which
matches the independently verified Dev 1.69 package. B'Blast and Addicus were
each launched three times. Every launch of a given app reached the same later
boundary, so the results are reproducible within this pass. The user's sense
that Addicus might have remained visible for a fraction longer is explicitly
uncertain and is not counted as a visible improvement.

| App | User-visible result | Decisive evidence | Status / next contract |
|---|---|---|---|
| B'Blast | Unchanged tiny audio fragment, black screen and termination. | The Dev 1.69 `NSTimeZone -localizedName:locale:` blocker is absent. All three launches instead retrieve an `NSNull` from a dictionary, reject it as both `NSString` and `NSNumber`, and send it `description`. `NSNull` lacks that selector and panics after about 394–435 ms, before any main-loop iteration or EAGL return. | **Advanced internally, still blocked.** `NSNull -description` is a normal shared Foundation contract, not evidence of an invalid object. Its exact historical textual result should be verified before implementation. |
| Addicus | Still reaches the menu only momentarily and terminates; no reliable visible advance is claimed. | Dev 1.69's `_CFHostCreateWithName` returns a retained host object on all three launches. Google Analytics proceeds to its TCP-socket path and then stops at the unimplemented `_CFStreamCreatePairWithSocketToCFHost`. Two warm launches reach 53–54 main-loop iterations and 52–53 EAGL returns before the same panic; the first launch also reaches the same API after a longer initialization. | **Advanced internally, still blocked.** This is a shared Core Foundation stream API, but it is a more consequential contract than offline host creation. A correction must represent unavailable networking honestly and must not fabricate connected streams. |

These results verify only that the two Dev 1.69 blockers were removed and the
new implementation paths were exercised. They do not establish functional
compatibility or a user-visible improvement. Blot was not rerun in this log;
its verified Dev 1.68 long-session death result remains the separate formatting
issue above. No source change, new package, or push was made from this status
report.

## Dev 1.70 three recovered contracts — built and awaiting device test

This batch follows the three independently decisive boundaries above. It does
not change the diagnostic recorder, memory-fault policy, rendering, saves or
game-specific options, and it contains no bundle-identifier checks.

- **B'Blast / `NSNull -description` (general Foundation contract):** the
  singleton now returns the historical Foundation representation `<null>`.
  The object remains static-lifetime and singleton-identical. This is the
  normal behavior required when collections or diagnostic serializers ask an
  explicit null placeholder for a description; it does not tolerate an
  invalid pointer or convert `nil` into an object.
- **Addicus / `CFStreamCreatePairWithSocketToCFHost` (explicitly offline
  partial contract):** CFNetwork now exports the historical five-argument ABI,
  validates the allocator and CFHost object, and writes null only to the output
  pointers the caller supplied. Apple's contract permits null stream outputs
  when creation fails. Because this emulator still has no socket-backed
  CFStream implementation, returning that documented creation failure is
  honest and consistent with its existing
  `CFStreamCreatePairWithSocketToHost` boundary. It does not resolve the name,
  open a socket, claim a connection, or fabricate callbacks. Software that
  requires network streams remains unsupported.
- **Blot / floating `printf` sign flag (general libc contract):** `%+f`, `%+e`
  and `%+g` now prefix nonnegative values with `+`, preserve negative signs,
  and place space or zero padding around the sign correctly. The existing
  precision, left-justification and general-format selection paths remain in
  use. This removes the exact assertion reached through `_vsnprintf_l` without
  swallowing the formatting request or special-casing Blot.

Four focused unit tests cover the NSNull singleton/description, CFHost stream
failure output pointers (including omitted outputs), and signed fixed and
exponential formatting with space and zero widths. The changed files pass the
available Rust formatter and `git diff --check`. A local library-test attempt
reached dependency compilation but cannot link because this Windows
environment has no MSVC `link.exe`; no local compiled-test pass is claimed.
The repository's macOS and Windows hosted suites must compile and run the full
library tests before packaging is accepted.

Predictions: B'Blast should leave the repeated NSNull selector panic and either
advance initialization or reveal an independent blocker. Addicus should receive
a normal offline stream-creation failure and let its optional Google Analytics
client handle it; a caller that incorrectly assumes non-null streams may expose
a later fault, which must not be hidden with fake stream objects. Blot should
survive the recorded death formatting request; a long Classic session ending
in death is required because the menu-slowdown fix and this formatter fix are
separate. The relevant regression checks are a brief Spoing session, Blot's
smooth Classic/menu/Challenges path, and repeated launch/return behavior.

### Dev 1.70 hosted validation and package verification

- Implementation commit `3341bbf00af1e1f7a367c48743a8c6e255fa2caa`
  added the three shared contracts. Its first unsigned-iPhone workflow
  (`41248565043`) correctly rejected a `Cow<str>`/`String` type mismatch in the
  new CFHost diagnostic text. Final source commit
  `6ca62e0eb576cc80b91e0ca83fb5d5237ba0bba7` uses the owned string expected by
  that helper; this follow-up does not change the API result or stream behavior.
- Final lint/test workflow `41248909683` (run 50) completed successfully. The
  macOS and Windows development-branch library suites each passed all 105
  tests, including the four new focused tests; the macOS diagnostic reproducer
  also passed. The release-only full-app test, legacy sanity-test and formatting
  steps were configured skips. The allowed-to-fail lint script still stops at
  the two pre-existing long comment lines in `dlfcn.rs` and
  `ns_dictionary.rs`, so this run does not claim clippy, rustdoc or Android
  compilation.
- Unsigned-iPhone workflow `41248909612` (run 115) completed successfully,
  including iOS compilation, separate development identity verification,
  unsigned packaging and artifact upload.
- Artifact `13060895244`,
  `iDared32bit-dev-unsigned-6ca62e0eb576cc80b91e0ca83fb5d5237ba0bba7`,
  is 19,834,916 bytes. Its downloaded ZIP SHA-256 exactly matches GitHub's
  digest:
  `0A59CB27EA5AB4EB2431ABC093A995DD3DD533A63EBCB10F1FCD65F3CA5F33B2`.
- The extracted IPA is 19,881,346 bytes with SHA-256
  `4B08A0FB6C2E655D30173472CEABD716CD40ECB7BBF61BB64DB5B1EFC6CB2872`.
  All 36 ZIP entries read successfully. It identifies as
  `com.bookieart1.idared32bit.dev` / `iDared Dev`, contains an unsigned arm64
  Mach-O, embeds source prefix `6ca62e0`, workflow `41248909612`, the expected
  branch and repository, the bounded diagnostic recorder and cohort options,
  and contains neither `_CodeSignature` nor `embedded.mobileprovision`.
- Local handoff: `outputs/iDared-Dev-1.70.ipa`; complete package evidence is in
  `outputs/iDared-Dev-1.70.verification.json`.

Focused device test: install over the development app without deleting its
container. Launch B'Blast once and allow its current result to occur. Launch
Addicus; if the menu remains, wait about 20 seconds and try starting once. In
Blot, play Classic long enough to die; if it remains open, return to the menu,
open Challenges, press Back and wait about 20 seconds. Brief Spoing play is the
working regression control; a full level is not required. Return the combined
log and visible results.

### Dev 1.70 device result: later blockers recovered; batch paused

The supplied log identifies workflow `41248909612`, the independently verified
Dev 1.70 package built from source `6ca62e0eb576cc80b91e0ca83fb5d5237ba0bba7`.
The original 258,918-byte log is preserved verbatim at
`outputs/device-logs/iDared-Dev-1.70-bblast-addicus-blot-device-log.txt` with
SHA-256
`2A05D95347EA0DDA710F7C48593A292C471708EBD8B6E0A1EF131EBBD38A76D1`.

| App | User-visible result | Decisive evidence | Status / preserved next boundary |
|---|---|---|---|
| B'Blast | Both launches looked unchanged: the brief audio fragment, black screen and termination. | The Dev 1.69 `NSNull -description` panic is absent. Both launches instead stop after roughly 407–415 ms at `CFNumber` creation with `not implemented: type 11`, before a main-loop iteration or EAGL return. | **Advanced internally, still blocked.** Dev 1.70's Foundation contract was consumed. Historical `CFNumber` type-11 semantics are the next candidate to verify when this batch resumes; the unchanged screen is not evidence that the implementation was unused. |
| Addicus | The menu still appeared only momentarily before termination, with no reliable visible improvement. | The log explicitly records `CFStreamCreatePairWithSocketToCFHost(www.google-analytics.com, 80)` returning null streams, then immediately stops on missing `_CFWriteStreamSetClient` after 54 main-loop iterations and 53 EAGL returns. | **Advanced internally, still blocked.** The explicit offline stream-creation path was exercised, but this analytics client does not stop there. The log does not yet decide whether the next correct boundary is null-input rejection or a non-connected stream object with later error delivery; do not fabricate a connected stream. |
| Blot | A comparably long play session remained smooth and did not crash. | The session returned normally through emulator Home after 195.3 seconds, with 9,387 main-loop iterations, 9,287 EAGL returns, stable source counts and no sustained slow-presentation transition or panic. | **Passing long-session observation; death fix not yet promoted to universal verification.** This is strong non-regression evidence for the prior `qsort` fix and one non-recurrence of the formatting crash. The log has no explicit death marker, so it does not by itself prove that the exact death transition was exercised. |
| Spoing | Not tested in this pass. | No Spoing launch is present in the log. | No status change. Its earlier working-control result remains the current evidence. |

Per the user's direction, stop this compatibility batch here. Do not implement
the recovered B'Blast or Addicus boundary yet. The next separate task is a
dedicated survey IPA with bounded flight diagnostics enabled for every app,
followed by a full page-by-page catalog pass. Performance- or timing-only
anomalies from that survey must be repeated on the ordinary diagnostics-off
baseline before being classified as game defects. Resume this three-app batch
after the catalog evidence has been collected. No source change, push or new
IPA was made for this result.

## Dev 1.71 dedicated universal flight-recorder survey — source preparation

The user explicitly authorized one survey IPA for a comprehensive page-by-page
catalog pass. This is a **diagnostic-only** build, not a compatibility release.
It contains no B'Blast, Addicus, Blot or other app-specific accommodation and
does not suppress panics, make the null page accessible, fabricate API success,
or expand instruction tracing.

The only functional change is that `Options::default().diagnostics` follows the
temporary `UNIVERSAL_FLIGHT_RECORDER_SURVEY` switch, which is enabled for the
survey source commit. Therefore every app launched from the picker receives the
same recorder that was previously enabled only for selected cohorts. Each
instrumented app launch also prints
`flight recorder v1 survey build: universal app coverage enabled`, making
coverage auditable in a combined log. The picker itself remains outside the
recorder.

The recorder remains bounded at 96 recent framework/API events, 32 entries in a
slow-presentation snapshot, 24 diagnostic signals and 24 progress samples. It
still filters high-frequency Objective-C retain/release traffic, OpenGL calls
and common memory/string primitives, emits only the first twelve five-second
progress reports, and dumps on panic or emulator Home. No guest instruction
trace was added. This limits log growth and overhead, but it cannot make the
survey timing-neutral: a performance- or timing-only anomaly observed in this
IPA must be repeated on the ordinary diagnostics-off build before it is called
a compatibility regression.

The survey predicts that every launched game log will contain both the survey
marker and the ordinary bounded-recorder banner. Crashes should include the
recent causal boundary and thread/progress state; normal Home exits should dump
the same bounded record. A short calibration on known-working Spoing and Paper
Monsters, plus previously repaired Spirits, Bounce and Blot, should precede the
full catalog pass. The source switch will be restored to `false` in a separate
`[skip ci]` commit immediately after the survey artifact is downloaded and
verified, so later ordinary builds do not inherit universal diagnostics.

### Dev 1.71 hosted validation, package verification and source restoration

- Survey source commit `50ece578ceb83d940cf7fc7da34949fc15d7f41a`
  triggered unsigned-iPhone workflow `41250258745` (run 116). Its macOS job
  `124682106527` completed successfully in 4m 28s after iOS compilation,
  separate development-identity verification, unsigned packaging and artifact
  upload.
- Lint/test workflow `41250258575` (run 51) completed successfully. The macOS
  and Windows development-branch library suites each ran 105 tests and passed
  105/105 with zero failures. The bounded diagnostic reproducer also passed on
  macOS. The allowed lint step still stops only at the two pre-existing long
  comment lines in `src/libc/dlfcn.rs` and
  `src/frameworks/foundation/ns_dictionary.rs`; clippy, rustdoc and Android
  compilation therefore are not claimed. The release-only full-app test,
  legacy sanity test and formatting steps were configured skips.
- GitHub artifact `13060214082`,
  `iDared32bit-dev-unsigned-50ece578ceb83d940cf7fc7da34949fc15d7f41a`,
  downloaded as a 19,835,746-byte ZIP. Its SHA-256
  `856B6570EF268F115D9D4BB9644565DED374C012C645589BF05F73D401E3EA17`
  exactly matches GitHub's published digest.
- The extracted `iDared-Dev-1.71-Survey.ipa` is 19,882,327 bytes with SHA-256
  `5084F594ADBFE4D2665DB7A0FF44F70CD9CD1E682C09BC854B202499D3EE7D59`.
  All 36 ZIP entries read successfully. The package identifies as
  `com.bookieart1.idared32bit.dev` / `iDared Dev`, contains an unsigned arm64
  executable, embeds source prefix `50ece57`, workflow `41250258745`, the
  expected branch and repository, and contains neither `_CodeSignature` nor
  `embedded.mobileprovision`. The executable contains the exact
  `flight recorder v1 survey build: universal app coverage enabled` marker,
  independently distinguishing this universal survey package from an ordinary
  diagnostics-off build.
- After the artifact was downloaded and independently verified, commit
  `1b0328f09fde9cc7e646f39c9cc38d20779264b5` restored the ordinary source
  switch to `false` with `[skip ci]`. That restoration does not alter the
  already-built survey IPA, whose embedded source remains `50ece578`.

Local handoff: `outputs/iDared-Dev-1.71-Survey.ipa`; complete package evidence
is in `outputs/iDared-Dev-1.71-Survey.verification.json`. Physical-device
behavior remains unverified. Calibrate first with Spoing, Paper Monsters,
Spirits, Bounce and Blot. For the catalog pass, export a separate combined log
after each app-picker page so the iOS logging buffer cannot erase early apps;
normal exits should use emulator Home so the recorder emits its bounded dump.

## Dev 1.71 complete catalog survey — superseding maintainer review

**Evidence checkpoint, October 31, 2026. Audit only; no compatibility or
diagnostic implementation, commit, push, build, or save reset.** This review
supersedes older broad compatibility assumptions where the new evidence
contradicts them; the original observations and audits remain preserved.

The complete eight-file ZIP (Page 5 is split) embeds `50ece57` and workflow
`41250258745` in every file, matching the verified Dev 1.71 Survey package.
The preserved ZIP is `outputs/device-logs/iDared-Dev-1.71-survey.zip`,
SHA-256 `11AC9469B1ABAE6A206EFD666379543BC77E723322797C106ADD27E3F027E849`.
All original historical ZIP entries were also checked byte-for-byte against
the preserved extracted files used in comparison; all three ZIP CRC checks
passed.

### Coverage and what the survey does not prove

- 131 attempts, 126 distinct selected IPA filenames, 53,967 physical lines.
  All 120 attempts entering CPU emulation (115 unique apps) enable both the
  universal and normal recorder banners. 119 complete bounded dumps follow.
- Rogue Sky is the only eligible session without a final dump. The log ends
  during repeated reachability registrations. This does not establish a host
  crash, watchdog kill, or deadlock.
- Eleven pre-runtime rejections are separate: eight encrypted executables,
  NOM 5 constant-string decoding, Wimp Mach-O debug-symbol handling, and Ramps
  startup orientation. Another 26 invalid ZIP filenames are picker skips.
- Unique-title outcomes: 97 panic/fatal, 18 Home without fatal, eight encrypted,
  one guest exit (Pig Rush, twice calling exit(1)), one Home shutdown timeout
  (LoveCatch), one uncertain terminal (Rogue Sky). These are outcomes of tested
  paths, not an overall percentage of fully working games.
- 46 attempts have successful EAGL returns, including 28 that later fail.
  Pixels, guest controls, audible correctness and game progress are not proved
  by that counter. Zero EAGL does not exclude UIKit rendering.
- Current shutdown writes/defaults synchronization provide useful API evidence,
  but no log-only proof of saved game contents or successful reload.
- No slow-transition snapshot was captured. Its threshold cannot detect every
  constant slowdown, no-frame condition or sudden stall; no diagnostics-on/off
  performance benchmark was performed.

### Historical reconciliation

The original mixed-commit survey remains 127 attempts/123 identities.
Dev 1.42 is 128 canonical attempts/126 identities, after excluding its known
20 duplicated Page 6 segments at the start of Page 7. Dev 1.71 covers all
123 original identities and 125 of 126 from 1.42; Jungle is absent and Galaxy
returns. The combined per-app delta has 127 identities.

Four earlier fatal 1.42 records now return Home without fatal: Spirits,
SimplePhysics, Ski Jumping and AB Buster. Only the first three also have
earlier user-confirmed usable gameplay. AB Buster remains skipped/test-validity
uncertain because the same copy stalled on the original iPod. Relative to the
original survey, nine fatal-to-Home changes include Blot, Bounce, Bulba,
Flick Buddies and Paper Monsters in addition to those four.

The unchanged-terminal bucket must not erase qualitative improvements: Bulba
previously displayed a frozen picture and Blot previously slowed on
Challenges/Back despite nonfatal Home records. The earlier user-confirmed
repairs remain valid within their stated scope. Spirits/Paper Monsters/Bulba/
Spoing earlier save-relaunch evidence remains historical; this survey does not
repeat every functional assertion. SimplePhysics silence was also observed on
the original iPod and is not an established emulator audio fault.

No matched 1.42 Home-without-fatal title becomes a panic title here. That is
not proof of regression freedom because test paths and saved state differ.
CrazyGolf now fails on two separate paths (UISlider setContinuous and encoding
0x500), so its earlier working-in-tested-portion status must remain partial.
Sprinkle's Home return with 1,786 loops and zero EAGL returns plus API2 rejection
does not certify working graphics. Sandwiches now decodes saved state before
missing finishDecoding; its earlier black-screen path is a separate unresolved
observation. Never delete saves to hide this distinction.

The survey itself adds no compatibility fix over 1.70. Attribute actual
improvements to intervening compatibility changes and user tests, not to the
recorder. “Same screen” can conceal a changed internal blocker; “same panic
site” can conceal a different causal path.

### Corrected causal model and prioritized next work

| Problem / apps | Evidence and hypothesis | Implementation status / next verification |
|---|---|---|
| Defaults nil removal: MageGauntlet, HookWorld, QuickHook | Recorder shows guest setObject:nil -> NSUserDefaults -> strict dictionary. Current defaults setter unconditionally forwards nil. Modern Foundation removes the preference for nil. Previous assumption that the game necessarily supplied an invalid identity is no longer justified. | **Not implemented. Strong contract candidate.** Verify legacy behavior, registration fallback/KVO, persistence and dictionary rejection separately. MageGauntlet primary; Hook games optional because user priority is low. |
| Mutable-set copy: Infinight, Sharpshooter | Real mutable sets call mutableCopyWithZone; missing in the emulator. Sharpshooter repeats three times; both titles present first. | **Missing contract.** Independent membership, correct shallow element retention, immutable/mutable variants and release-order tests; then both games. |
| B'Blast CFNumber conversion | The decisive operation is **CFNumberGetValue type 11**, not creation. Earlier Dev 1.70 wording was wrong. CFNumberCreate already handles long long. | **Partial contract.** Extend only after 64-bit output width, exact/lossy conversions and canary tests. Still blocked; no visible compatibility gain yet. |
| Glowfish / Glowfish HD anonymous mmap | Both pass valid-looking Darwin VM tag bits 0xf0000000; source asserts fd == -1. Historical Apple mmap documentation permits tags on MAP_ANON. | **Partial platform contract.** Verify supported flags/tag behavior; test zeroing, map/unmap, failure and file-backed mapping. VM changes need Unity regression controls. |
| Hamlet / HookChamp descriptions | NSLog requests instance descriptions on valid UIImage / DBH objects; NSObject only has class description. | **Missing inherited contract.** Review semantic overrides before implementing fallback; no game-specific selectors. |
| Rogue Sky callback recursion | 119 reachability registrations repeat the callback/target while context addresses descend. SetCallback resets delivered state; ScheduleWithRunLoop invokes synchronously. | **Strong hypothesis, not a proven host crash.** Re-register/reschedule fixture and cancellation/context ownership first. High risk to Spoing/Diversion/Sticky networking if guessed. |
| Null URL / early Unity cohorts | Three pinball titles plus Mystery Agency share URL construction/release; four Unity titles share collection initialization. Crucial preceding results are nonnil. | **Unresolved causes.** Resolve guest fault instruction/imported operand; compare a second game before implementation. Do not globally change URL, array or null behavior. |
| Cut Buttons / Soul / Fly Kiwi / Tesla | Stack-block copy / audio NSError consumer / returned UIImage with isa zero / MainMenu dealloc scheduling at refcount zero. | **Separate runtime/ownership investigations.** Current evidence rejects one generic dispatch fix. No object resurrection or invalid-pointer tolerance. |
| Blot | Earlier Challenges slowdown repair remains supported. Long-session death crash did not recur in 1.70; current survey has no explicit death marker. | **Working in sampled scope, exact death path not universally verified.** Preserve earlier formatter prediction and test death when relevant. |
| Splode | 593 EAGL returns then achievement/offline-service path and null 0x8. | **Partial gameplay, unresolved completion fault.** Inspect guest service initialization/dereference; no fake OpenFeint success. |
| Addicus | 54 presentations then null analytics streams and missing CFWriteStreamSetClient. | **Still blocked.** Determine legacy stream creation/error lifecycle before adding client handling. |
| Saved-state coding: Sandwiches / Piyo Blocks | Missing finishDecoding / encodeBool on distinct coder paths. | **Partial contracts.** Preserve old saves and test round trips/lifetime, not only new-game startup. |

Recommended next iteration: two small, separately reviewable Foundation
changes for defaults nil removal and set copying. Primary device targets:
**MageGauntlet, Infinight and Sharpshooter**. Spoing and Spirits are the
focused input/transition/existing-save regression controls. The low-priority
Hook games can corroborate the defaults fix without becoming the main batch.
B'Blast is the next isolated conversion follow-up. Proposed work is not yet
implemented or native/guest-contract verified.

No broader recorder is required before these candidates. For opaque null
faults, first inspect existing guest addresses against executable instructions.
Only if that is insufficient, consider a bounded fault-time instruction/
operand snapshot and targeted allocation-generation history for implicated
classes. More universal event volume is not the objective.

### Durable audit artifacts and validation boundary

Full rationale: `outputs/iDared-holistic-survey-audit-1.71.md`.
Companions in `outputs/`:
`iDared-per-app-review-survey-1.71.csv` (126 apps),
`iDared-launch-inventory-survey-1.71.csv` (131 attempts),
`iDared-per-app-delta-1.00-1.42-1.71.csv` (127 identities),
`iDared-cause-groups-survey-1.71.csv`,
`iDared-error-family-delta-survey-1.71.csv`,
`iDared-invalid-packages-survey-1.71.csv` (26),
`iDared-recent-boundaries-survey-1.71.csv` (11,283 events),
`iDared-survey-source-manifest-1.71.csv`,
and `iDared-engineering-queue-survey-1.71.csv` (14 ranked proposals).

Reproducible analysis is in `work/audit-survey-1.71/analyze.py` and
`review.py`. Archive CRC/hash checks, historical-byte comparisons, CSV/schema
counts, per-launch commit/banner checks and dump coverage checks passed.
No emulator compilation or automated compatibility test was run during this
audit; earlier build/test records remain separate. The two historical narrative
audits and their CSVs were not overwritten.


## Dev 1.72 — survey-led Foundation batch (October 31, 2026)

User authorized the first two shared survey fixes and the next IPA. They report
**no noticeable overall slowdown** during universal 1.71 testing and requested
universal recording permanently. This is a device observation, not a controlled
zero-overhead benchmark for every game.

- Baseline: tested survey `50ece578`; previous normal source `784ef91a`;
  full survey ledger preserved in `b6856aae`. Fetched upstream `ca1de89c` still
  lacks the two contracts.
- MageGauntlet: nil for `trpg/gcID` now uses existing NSUserDefaults removal,
  preserving KVO and application-domain-only behavior. Lower-priority defaults
  and unrelated values remain; dictionary nil rejection is unchanged.
- Infinight / Sharpshooter: implement independent shallow mutable copies of sets
  and immutable copies of mutable sets. Correct directly involved allObjects
  snapshot ownership so temporary arrays retain members and balance on pool drain.
- All eligible apps: flight is the permanent default. Explicit
  `--diagnostics=off` remains available. No recorder capacity/filter changes.
- Status: general semantic corrections, not app-specific hacks. Build/tests
  pending; not physically validated and not a claim that these games are fixed.
- Four added host tests cover defaults fallback/unrelated settings, dictionary
  strictness, set copying/mutability/membership/identity/ownership/release order,
  and recorder defaults/overrides. Independent native macOS Foundation fixture
  checks the reference contracts including nil removal, one KVO event and
  persisted-domain exclusion. It is not a historical iPhone OS runtime oracle.
- Local formatting/whitespace reviewed. Offline compilation stopped because the
  cached SDL dependency submodule is incomplete; hosted results pending.
- Device plan: launch/play MageGauntlet, Infinight and Sharpshooter, return to menu
  and reopen saved progress if available. Brief Spoing and Spirits regressions
  include existing saves. HookWorld/QuickHook remain optional low priority.
- Limits: existing broader NSSet and defaults domain/KVO TODOs remain. Host tests
  do not verify ARM ABI, actual game binaries or emulator disk persistence;
  downstream blockers may appear. B'Blast/Addicus are outside this batch.

Details and primary references: `dev-docs/compatibility-1.72-foundation.md`.
Build identity and actual validation will be appended when CI completes.


### Dev 1.72 build verification — October 31, 2026

Delivered source `05732a45f5f666d1dda508629aeabfddf908abf4` (implementation
`9bf75cd3`; test return-type annotations corrected in `05732a45`). Superseded
initial CI runs were cancelled; the delivered source is the fully tested revision.

- [IPA build 41258098358](https://github.com/bookieart1/iDared32bit/actions/runs/41258098358): iOS arm64 archive/package succeeded.
- [Tests 41258098543](https://github.com/bookieart1/iDared32bit/actions/runs/41258098543): **109 library tests passed on macOS and 109 on Windows**. All four new tests passed on both hosts. Native Foundation reference fixture and separate diagnostic reproducer also passed on macOS.
- Lint still exits on pre-existing long comments in dlfcn.rs:61 and
  ns_dictionary.rs:531 (allowed failure). Clippy/rustdoc and Android compilation
  are not reached. Full guest integration and global format jobs are configured
  skips; only changed Rust files were locally format-checked.
- Artifact `13063809817`; artifact ZIP SHA256
  `b3018202acf7b17cba00d878c41c088439b9ca1d9ba023893fd509395b1a3607` matches GitHub.
- Local `outputs/iDared-Dev-1.72.ipa`, **19,884,758 bytes**, SHA256
  `378ecd242137d6c83c7d90efe9dd23e010bae2ccbaca7d722dd4f0118a0761a2`
  matches the packaging job. ZIP CRC, arm64 Mach-O, embedded source/run,
  `com.bookieart1.idared32bit.dev`, `iDared Dev`, no profile/signature verified.
- Universal default is permanent: source and both compiled option tests agree;
  packaged help says default flight, and the obsolete temporary survey marker
  is absent. The recorder implementation is byte-equivalent to the 1.71 survey
  after line-ending normalization. No post-build default-off restoration.
- Physical-device status: **pending**. Neither gameplay nor old-save/relaunch
  success for this batch has been inferred from compilation or host tests.

Package verification record: `outputs/iDared-Dev-1.72.verification.json`.
Detailed job logs are retained under `work/verify-1.72/`.


## Dev 1.72 device result and Dev 1.73 follow-up — October 31, 2026

The new log embeds `05732a4`, workflow `41258098358`; SHA256
`12f537252f911e82c4a2f6cc281c4f38a8868acd6a72199d96d9377a246bc0ec`.
Unmodified local copy: `outputs/device-logs/iDared-Dev-1.72-device-results.txt`.
All five CPU launches have enabled recording and complete terminal dumps.

| App | Device result / evidence | Revised status and next work |
|---|---|---|
| Infinight 1.05 | User completed a level, returned to menu, exited/reopened and saw saved completion. Two Home exits; 4,238 and 577 EAGL returns. | **Working in the stated test scope**, including progress persistence. The mutable-set blocker was removed. Do not infer all levels/audio paths tested. |
| MageGauntlet 1.30 | Two presentations; GDataXML children sends NSMutableArray arrayWithObject, receives _touchHLE_NSArray, then addObject panics. | Prior defaults blocker removed; **wrong factory mutability confirmed**. Implement a mutable singleton factory using existing constructors/mutation ownership. |
| Sharpshooter 1.8.1 | 1,103 presentations; reaches just before gameplay. Existing AVAudioPlayer receives url after stop during track selection. | Prior set-copy blocker removed; **missing getter confirmed**. Expose retained source URL; no playback/callback changes. |
| HookWorld 1.10 | 4,307 presentations; avalanche death, score 552, local-best decoding, then FeintScore gets the _touchHLE_NSString class as PlayerEquipment and sends playerChar. | Defaults startup blocker removed; **playable until death, wrong argument origin unresolved**. No fabricated playerChar response or score bypass. Capture recent guest caller code at panic to distinguish a field/global load from earlier return corruption. |

No new Spoing or Spirits regression test was supplied. QuickHook was not played.
The changed failure stages are progress, not established regressions or complete
compatibility. Retain all older survey findings as dated evidence.

## Dev 1.73 — singleton-array and audio URL corrections

- Scope: general NSMutableArray singleton factory with strict nil rejection;
  AVAudioPlayer url getter backed by existing owned state. No app-specific hacks.
- HookWorld: diagnostic-only this build. At panic, inspect at most eight distinct
  observed caller LR locations from the existing ring on the failing thread,
  up to 112 bytes each. Loaded-image bounds and fallible reads; no larger ring,
  extra normal-gameplay events, guest calls/writes, or repeated dump output.
- Fresh upstream remains `ca1de89c`, with the same two missing contracts.
- Seven new host tests cover array mutability/ownership/nil rejection, audio URL
  lifetime and data initialization, and bounded caller evidence/failure
  preservation. Native Foundation singleton reference checks added. Compile,
  test and physical-device results are **pending**, not inferred.
- Changed-file formatting/whitespace checks pass locally. Hosted compilation is
  used because local SDL/native build dependencies remain incomplete.
- Risk: shared Foundation factory behavior and audio URL consumers; retain
  Infinight as current save/relaunch control, with Spoing in the wider backlog.
- HookWorld still needs producer evidence. Separate coder UID-zero/ownership
  concerns found during inspection are **unproven causes** and left unchanged.
- Details: `dev-docs/compatibility-1.73-followup.md`.


### Dev 1.73 build verification — October 31, 2026 (local time)

Delivered source **`452a8571f86846713672f2c803f99d5d913d0635`**. Implementation
commit `a0cece8b`; follow-up `452a8571` gives the new WAV/NSData test a guest
stack and wraps a documentation comment. Initial tests were 115/116 on both
hosts: the fixture's SP=0 underflowed on a five-argument initializer. This was
a test-harness error, corrected without changing emulation. Superseded IPA
run 41260323710 was cancelled, not delivered. Initial logs remain preserved.

- [iOS build 41260610289](https://github.com/bookieart1/iDared32bit/actions/runs/41260610289): archive/package succeeded.
- [Tests 41260610296](https://github.com/bookieart1/iDared32bit/actions/runs/41260610296): **116 library tests passed on macOS and 116 on Windows**, including all seven added tests. Native Foundation singleton/defaults/set-copy fixture passed. Both the existing sysctl SVC reproducer and new missing-selector reproducer passed on macOS.
- The new diagnostic test emitted one 112-byte synthetic caller window despite
  repeated dump requests. Its recovered bytes contain mov r3,r6 before BL and
  bx lr after it; registers/CPSR/guest memory and the original panic remained
  unchanged. This is verified synthetic evidence, not a HookWorld root cause.
  Applying the eight-caller selection to HookWorld's actual history includes
  both `0x972dc` (score caller) and `0x709c0` (failed playerChar caller).
- Lint's allowed failure remains the old dlfcn.rs:61 and ns_dictionary.rs:531
  long comments. Clippy/rustdoc and Android compilation are not reached.
  Full guest integration/global formatting are configured skips. The new
  comment-length issue in the initial revision was corrected before delivery.
- Artifact **13065650145**; artifact ZIP SHA256
  `30e2fafee38748d917124d48b6af01f85e3a54e85366dcc64b067774dd75092a`
  matches GitHub. Local `outputs/iDared-Dev-1.73.ipa`, **19,893,866 bytes**,
  SHA256 `9d03ead7fc1c7225cfb0fd168eb669c64fc3162474ba0d4148b898bc23fa2f11`
  matches the package job. CRC, arm64 Mach-O, embedded source/run, unsigned
  package and `com.bookieart1.idared32bit.dev` / `iDared Dev` verified.
- Packaged help still says default flight for every app; compiled default/
  override tests pass. New crash-caller marker exists in the binary. Universal
  recording remains enabled. No save deletion or app identity change.
- **Physical-device validation of 1.73 remains pending.** MageGauntlet and
  Sharpshooter have contract repairs for the observed blockers. HookWorld has
  diagnostic-only work and an unresolved death/score fault. Infinight's 1.72
  success remains confirmed within the user's reported scope.

Focused retest: MageGauntlet start/play; Sharpshooter start/play/menu; HookWorld
death/retry; Infinight old saved progress and relaunch. If HookWorld still fails,
decode the captured score-caller instructions before considering serialization,
lifetime, dispatch/ABI or service-related changes. No blanket runtime workaround.

Verification: `outputs/iDared-Dev-1.73.verification.json`. Job logs, initial
failure logs and machine-checked diagnostic evidence are in `work/verify-1.73/`.


## Dev 1.73 device result and Dev 1.74 Foundation follow-up — October 31, 2026

The supplied log embeds source `452a8571` and workflow `41260610289`; SHA-256
`9076db46ad3291f05a4753b9cd45fe322ccdfd2e404f9d323948272aa7746ba`.
The unmodified file is preserved as
`outputs/device-logs/iDared-Dev-1.73-device-results.txt`. All five CPU launches
have universal recorder coverage and terminal dumps.

| App | Dev 1.73 device result | Interpretation and next work |
|---|---|---|
| Mage Gauntlet 1.30 | Two launches fail at 174/126 ms before a main loop: immutable `_touchHLE_NSDictionary` receives `setObject:forKey:` at guest caller `0x1504d`. | **Blocked on restored settings/player state.** Receiver identity is proven; older construction is outside the bounded history. Mutable singleton factory is a credible, unproven origin. |
| Sharpshooter 1.8.1 | User reaches initial gameplay. At 18.138 s, 1,746 loops and 968 EAGL returns, `_touchHLE_NSMutableSet` receives missing `addObjectsFromArray:`. | **Partial; decisive standard API blocker.** Implement the method with existing set ownership/uniqueness semantics. |
| HookWorld 1.10 | Two launches fail near 140 ms after one EAGL return while Settings adds an item to an immutable dictionary. | **Blocked on this saved-state path.** Dev 1.72 was playable until death before saving, so the changed stage is consistent with reload-dependent behavior, not proof of a general 1.73 regression. Preserve the save. |

Infinight was not repeated in this test and retains its Dev 1.72
working-in-stated-scope result, including completed-level persistence. The log
does not justify making immutable dictionaries mutable, deep-mutating property
list children, deleting saves, or treating all three failures as one cause.

### Dev 1.74 implementation and build verification

- `-[NSMutableSet addObjectsFromArray:]` is implemented by enumerating the
  supplied array through existing `addObject:` behavior. This is the exact
  Sharpshooter blocker and a general Foundation correction.
- `+[NSMutableDictionary dictionaryWithObject:forKey:]` now constructs the
  requested mutable concrete class. The inherited implementation incorrectly
  always returned an immutable dictionary. NSDictionary remains immutable and
  nil input remains rejected. This is a real contract repair, but device
  confirmation is still needed to establish that either Rocketcat dictionary
  originated through this factory.
- Added emulator tests cover mutable factory identity, mutation, keyed archive
  round-trip mutability, bulk set insertion, duplicates and membership. The
  independent macOS Foundation fixture checks the same reference contracts.
- Implementation commit `c3639a95751488ba58d3902f15e99b7861f72dff`;
  delivered source `27825333da0232607623fb116902913925f5d3c6`. An initial
  Windows run passed 117/118: the new archive fixture lacked synthetic guest
  stack space for nested ABI calls. The fixture was corrected without changing
  shipped behavior, and the full final matrix was rerun.
- [Tests 41261997512](https://github.com/bookieart1/iDared32bit/actions/runs/41261997512):
  **118/118 library tests passed on macOS and 118/118 on Windows**. Native
  Foundation reference and both bounded-diagnostic reproducers passed. Lint's
  allowed failure remains the two pre-existing long comments; subsequent
  clippy/rustdoc and Android compilation are not reached. Guest integration is
  a configured skip.
- [IPA build 41261997478](https://github.com/bookieart1/iDared32bit/actions/runs/41261997478):
  arm64 archive/package succeeded. Artifact `13065223507`; artifact ZIP SHA-256
  `1db3d2148842c9c8510976ff458c17b1a5302d35a3b01c6ec76c831973992de8`
  matches GitHub.
- Local `outputs/iDared-Dev-1.74.ipa`, **19,895,490 bytes**, SHA-256
  `14ac52044f2539c10b29d34e753ca18c9a22917f3a4c809e3d516582578e2863`
  matches the package job. ZIP CRC, arm64 Mach-O, embedded source/workflow,
  `com.bookieart1.idared32bit.dev` / `iDared Dev`, no signature/profile and
  permanent default-flight help were verified.
- No game-specific branch, save migration, audio/graphics change, memory-fault
  suppression or diagnostic expansion is included. **Physical device testing
  is pending.**

Focused Dev 1.74 order: Mage Gauntlet launch/start; Sharpshooter reach gameplay
and repeat the crashing interaction; HookWorld launch, play to death if it
opens, then relaunch with the existing save; brief Infinight old-progress/
relaunch control. A still-immutable Rocketcat receiver falsifies the narrow
factory-origin hypothesis and calls for targeted archive/property-list origin
tracing, not broader mutation tolerance.

Verification: `outputs/iDared-Dev-1.74.verification.json`; exact job logs are
retained in `work/verify-1.74/`. Engineering detail:
`dev-docs/compatibility-1.74-foundation.md`.


## Dev 1.74 device result and Dev 1.75 provenance checkpoint — October 31, 2026

The supplied log embeds source `27825333` and workflow `41261997478`; its
SHA-256 is `ee94bad4c631c3898d98e197397f5341718e8ef8a73b47d41bfe8fadaa245849`.
The unmodified 228,943-byte file is preserved as
`outputs/device-logs/iDared-Dev-1.74-device-results.txt`. All four launches
have universal recorder coverage and terminal dumps.

| App | Dev 1.74 device result | Status and interpretation |
|---|---|---|
| Sharpshooter 1.8.1 | User reports that the previously failing zoom interaction now works. Run exits normally after 56.367 s, 4,991 loops and 3,243 presentations. | **Blocker removed; partial compatibility.** `NSMutableSet addObjectsFromArray:` is device-validated for this path. Later gameplay remains outside this test. |
| Infinight 1.05 | Existing completed-level progress is present; run exits normally after 9.623 s, 561 loops and 560 presentations. | **Working in the stated scope; regression pass**, including old-save persistence. |
| Mage Gauntlet 1.30 | Saved version 1.30/player state loads, then immutable `_touchHLE_NSDictionary` receives `setObject:forKey:` at caller `0x1504d`; 163 ms, no loop/presentation. | **Still blocked during restored-state startup.** The 1.74 mutable singleton-factory origin hypothesis is falsified. |
| Hook World 1.10 | Saved version 1.10 loads, then Settings mutates immutable `_touchHLE_NSDictionary` at caller `0x74460`; 144 ms, one presentation. | **Still blocked during restored-state startup.** Earlier playable 1.72 launch loaded version `(null)`, so current evidence is a first-run/save-relaunch stage change, not proof of a source regression. |

Apple's documented preferences/property-list contract does not justify making
all restored dictionaries mutable. Competing origins remain direct immutable
defaults retrieval, an incorrect copy/mutable-copy path, and decoder/unarchiver
mutability. The current event ring loses the producing boundary before failure.

Dev 1.75 is therefore **diagnostic-only**: retain at most 128 live dictionary
pointers and each pointer's latest selected preferences/copy/decoder/factory
boundary. Concrete non-dictionary results are excluded; deallocation clears
the pointer; only one match for the terminal receiver is printed at dump time.
No guest calls, ownership changes, fabricated returns, memory-fault tolerance,
save migration, app-specific behavior or per-frame logging is added. Tests cover
ring-wrap survival, deallocation, filtering and newest-transfer replacement.

Local changed-file formatting/whitespace checks pass. Local compilation is
blocked by the host's missing MSVC linker; hosted tests, iOS compilation,
package verification and physical-device behavior remain separate pending
stages. Focused device test after packaging: launch existing-save Hook World
and Mage Gauntlet in that order and preserve both terminal dumps. Engineering
detail: `dev-docs/compatibility-1.75-collection-provenance.md`.

### Dev 1.75 verified delivery

- Source and implementation commit:
  `cdcaf3d189cd5bc556ab1565d421afe8172810b5`.
- [Tests 41263772973](https://github.com/bookieart1/iDared32bit/actions/runs/41263772973):
  **120/120 library tests passed on macOS and 120/120 on Windows**. Both new
  provenance tests, both existing bounded-diagnostic reproducers and the native
  Foundation reference fixture passed. Guest integration is skipped by
  configuration. Lint's allowed failure remains two pre-existing long comments;
  later clippy/rustdoc and Android compilation were not reached.
- [IPA build 41263773139](https://github.com/bookieart1/iDared32bit/actions/runs/41263773139):
  arm64 packaging succeeded. Artifact `13066091948`; downloaded artifact ZIP
  SHA-256
  `10cbec05825cbe836395032e1a56e7c8cdd65b58574a3a14f4100755e6ac1f86`
  matches GitHub.
- Local `outputs/iDared-Dev-1.75.ipa`, **19,888,352 bytes**, SHA-256
  `b1ca67723600757d022d5f7fc192ccc88bf95af33516b64810ed6da5cb0e8a8b`,
  passed ZIP CRC and package-identity checks. The executable is arm64, embeds
  source `cdcaf3d` and workflow `41263773139`, retains the universal recorder,
  and is unsigned under `com.bookieart1.idared32bit.dev` / `iDared Dev`.
- No compatibility behavior, save data, app-specific branch, ownership rule or
  strict failure check changed. **Physical-device validation is pending.** Test
  only existing-save Hook World followed by Mage Gauntlet, then preserve the
  complete log. Do not delete either save.

Verification: `outputs/iDared-Dev-1.75.verification.json`.


## Dev 1.75 device result — direct restored-preferences origin

The focused log embeds source `cdcaf3d` and workflow `41263773139`; its
SHA-256 is
`19ead50def28badef62777dc67253a929022f0205762215069ef59da9a591a4c`.
The unmodified 116,002-byte file is preserved as
`outputs/device-logs/iDared-Dev-1.75-device-results.txt`. Both requested
launches have the universal recorder enabled, a complete terminal dump and a
retained provenance event.

| App | Dev 1.75 evidence | Revised interpretation |
|---|---|---|
| Hook World 1.10 | Saved version 1.10 loads. Immutable dictionary `0x3647e150` receives `setObject:forKey:` at caller `0x74460`. The retained producer is `NSUserDefaults objectForKey:` sequence 65248, returning that exact pointer. | **Direct application-preferences value; blocked during restored-state startup.** No intervening mutable copy, dictionary factory or decoder produced the receiver. The earlier Dev 1.72 playable path was a first launch with version `(null)` and mutable in-memory settings. |
| Mage Gauntlet 1.30 | Saved version/player/firstPlay load. Immutable dictionary `0x3646d700` receives `setObject:forKey:` at caller `0x1504d`. The retained producer is `NSUserDefaults objectForKey:` sequence 3778, returning that exact pointer. | **The same direct application-preferences pattern; blocked during restored-state startup.** The shared Rocketcat Settings code is the likely common consumer, but the log does not identify the preference key or establish why the shipped games avoided this path on historical iPhone OS. |

This validates the provenance mechanism and falsifies the remaining broad
producer hypotheses for these two concrete receivers. It does **not** justify
making every `NSDictionary` or every value returned by `NSUserDefaults`
mutable. Apple's documented preferences contract says returned property-list
values are immutable. The contemporary open-source CF-550 implementation also
loads the plist with `kCFPropertyListImmutable` and deep-copies values into the
preferences cache with that option specifically to exclude mutable objects.

There is a separate correctness gap in the emulator: its current
`NSUserDefaults setObject:forKey:` retains the supplied object instead of
making the historical immutable property-list snapshot. Correcting that gap
would prevent later mutations of the caller's original object from silently
changing the cached preference. It cannot by itself repair these already
persisted saves or make the games' direct mutation valid, so it must not be
presented as the current crash fix.

The next discriminating evidence, before any accommodation, is bounded decoded
context for the direct defaults lookup: preference key, selected domain and the
failing dictionary key/value classes. That can identify whether both games are
mutating the same Rocketcat settings structure and allow an explicit choice
between a general persistence correction and a narrowly labeled legacy
compatibility mode. Do not delete the preserved saves, globally weaken
collection immutability, or treat the two first-run successes as proof that
the restored-state behavior is correct.


## Dev 1.76 engineering checkpoint — restored-preferences context

Dev 1.76 is diagnostic-only. The universal recorder retains at most eight decoded
details for `NSUserDefaults objectForKey:`/`dictionaryForKey:` and immutable
dictionary `setObject:forKey:` boundaries. A bounded, allocation-free NSString
reader records the defaults key, selected domain, mutation key and value class
without guest calls, ownership changes or return-value changes. The producer's
key/domain also travels with the existing bounded dictionary provenance entry,
so ordinary event-ring wrap does not erase it.

The immediate decoded-detail ring and copied producer key are deliberately
compact so the recorder remains inside its pre-existing 128 KiB ceiling.

This deliberately does **not** make defaults values mutable, alter preference
serialization, tolerate an invalid mutation, migrate/delete saves or branch on
either app. Unit coverage exercises UTF-8/UTF-16/UTF-32 bounds, real host
objects, provenance association and the recorder-size ceiling. Local whitespace
checking passes; the local environment has no Rust toolchain, so hosted library
tests and iOS compilation remain pending and separate.

Prediction: existing-save Hook World and Mage Gauntlet should retain their same
strict failures, but their terminal dumps should now expose the exact restored
preference structure and mutation shape. Test Hook World first, then Mage
Gauntlet, without deleting either save. No gameplay or broad regression replay
is required for a non-mutating diagnostic change. Engineering detail:
`dev-docs/compatibility-1.76-restored-preferences-context.md`.

### Dev 1.76 verified delivery

- Final source and implementation commit:
  `c5005dd4bb636f211c2c49d17b564a9f728dfd10`. The two earlier hosted attempts
  were superseded after revealing fixture-type errors and then an over-budget
  recorder; neither was delivered.
- [Tests 41267274637](https://github.com/bookieart1/iDared32bit/actions/runs/41267274637):
  **122/122 library tests passed on macOS and 122/122 on Windows**. The native
  Foundation reference, decoded-context test, recorder-size ceiling and
  diagnostic reproducers passed. Guest integration remains configured off.
- Android stopped at the allowed lint failure on the same two pre-existing
  long comments; clippy, rustdoc and Android compilation were not reached.
- [IPA build 41267274701](https://github.com/bookieart1/iDared32bit/actions/runs/41267274701):
  arm64 packaging succeeded. Artifact `13068265913`; outer artifact ZIP
  SHA-256
  `ecaaff71d204daeac6af6d7c6f4197cd5215917e3f11a7742afd46dea9e9cc01`
  matches GitHub.
- Local `outputs/iDared-Dev-1.76.ipa`, **19,892,219 bytes**, SHA-256
  `3ce39870c3e3ca4f23da449fe6cd1e54f23adf4cf1b5766066ff9cff822774dd`,
  passed archive/CRC and package-identity checks. The arm64 executable embeds
  source `c5005dd4`, workflow `41267274701`, the decoded-context marker and the
  universal-flight default. It is unsigned under the separate development
  identity and contains no provisioning profile or `_CodeSignature` directory.
- No compatibility behavior or save data changed. Physical-device testing is
  recorded below. Verification:
  `outputs/iDared-Dev-1.76.verification.json`.

### Dev 1.76 physical-device result

The focused November 1 test ran the delivered `c5005dd4` build and launched
existing-save Hook World followed by Mage Gauntlet. The exact 117,729-byte log
is preserved as `outputs/device-logs/iDared-Dev-1.76-device-results.txt`,
SHA-256
`38d5a42c3093e9e000f7d7204511b6e9e0527dd69c21e78e94a012553a95e9b4`.

- Hook World: application-domain `NSUserDefaults objectForKey:` returned the
  terminal immutable receiver for `qh/items`; the app then tried to store an
  `NSNumber` under `player/zelle` in that exact dictionary.
- Mage Gauntlet: application-domain `NSUserDefaults objectForKey:` returned the
  terminal immutable receiver for `trpg/stats`; the app then tried to store an
  `NSString` under `firstPlay` in that exact dictionary.

Both remain blocked at their strict immutable-dictionary checks. The decoded
context succeeded and rejects the single-identical-key/shared-corrupt-object
hypothesis. The stronger model is a shared Rocketcat Settings behavior applied
to different saved structures. This is an origin and call-shape result, not a
verified API-contract fix or compatibility improvement.

Do not make preferences dictionaries globally mutable: historical
CFPreferences requires callers to assume returned property-list values are
immutable. Separately, the emulator currently retains values supplied to
`NSUserDefaults setObject:forKey:` by reference, which explains why a mutable
nested dictionary may work during its creation session and fail only after
serialization/relaunch. Fixing that snapshot semantic is general correctness,
but it will not legitimize the Rocketcat direct mutation and may expose it
earlier. Next inspect the shared Rocketcat Settings call sites and historical
wrapper behavior. Any compatibility accommodation must be explicit, narrow and
provisional rather than a global weakening of the preferences contract.

## Dev 1.77: first user-prioritized survey batch (candidate)

Before editing, all eight files in the Dev 1.71 universal-recorder survey were
re-read and verified byte-for-byte against the preserved manifest and extracted
copies: 7,998,477 bytes and 53,967 lines, all embedding source `50ece57` and
workflow `41250258745`. The selected apps have distinct first blockers:

| App | Dev 1.71 first decisive blocker | Candidate contract change | Status before device test |
|---|---|---|---|
| Parachute Ninja | Missing `_sel_getUid` before a run-loop iteration or EAGL return. | Historical runtime alias uses the existing selector-interning path and preserves identity with `sel_registerName`. | Blocker targeted; no launch/menu/gameplay claim. |
| Dark Nebula 2 | Missing `_feraiseexcept(1)` before a run-loop iteration or EAGL return. | Exception raise/clear/test operate on the active guest ARM FPSCR cumulative bits rather than disconnected host state. | Blocker targeted; traps and the older rounding implementation remain outside this boundary. |
| Dirt | Missing `+[NSData dataWithContentsOfURL:options:error:]` after GL/audio setup; option 2 is `NSDataReadingUncached`. | File URL and file path option/error readers use the guest filesystem, honor option semantics as non-observable hints, and return a Cocoa missing-file error. | Blocker targeted; synchronous network URL loading remains unsupported. |
| Hamlet | `NSLog` requests `-description` from a valid UIImage, but NSObject lacks its inherited fallback. | NSObject returns `<Class: 0x...>` and debugDescription dispatches through description; subclass overrides remain authoritative. | Blocker targeted; HookChamp is a corroborating future test. |
| Glowfish | Anonymous mmap asserts because Darwin VM tag `0xf0000000` is carried in `fd`; Glowfish HD repeats it. | Accept only the documented high-byte `VM_MAKE_TAG` field; reject unrelated bits before allocation. Purgable mappings remain explicitly unsupported. | Blocker targeted; no rendering/gameplay claim. |

This is five general contract work items in one requested batch, not a claim of
one common root cause. No game identifier, selector special-case, invalid
object tolerance, memory-fault suppression or fabricated success was added.
Focused unit tests cover each new boundary. The universal bounded recorder
remains enabled. Engineering detail and exact survey ranges:
`dev-docs/compatibility-1.77-priority-batch-contracts.md`.

Predicted device test order is Parachute Ninja, Dark Nebula 2, Dirt, Hamlet and
Glowfish, followed by Spoing and Blot as narrow VM/Foundation regressions.
Glowfish HD can directly corroborate the tagged-mapping path. Preserve existing
saves; a removed first blocker, menu reached, gameplay observed and functional
compatibility must be reported separately.

### Dev 1.77 verified delivery

- Final implementation/source commit:
  `7640156cc0c421c2d44259fcb4ffc2e8787744ac`. Two earlier hosted attempts
  exposed test-source compilation errors and then a missing synthetic guest
  stack in the new five-argument `NSData` fixture; neither superseded package
  was delivered.
- [Tests 41284825026](https://github.com/bookieart1/iDared32bit/actions/runs/41284825026):
  **128/128 library tests passed on macOS and 128/128 on Windows**. Native
  Foundation reference, bounded diagnostic reproducers and final lint jobs
  passed. Guest executable integration is still skipped by configuration.
- [IPA build 41284825031](https://github.com/bookieart1/iDared32bit/actions/runs/41284825031):
  arm64 archive/package and artifact upload passed. Artifact `13073919356`;
  downloaded outer ZIP SHA-256
  `2e239a702ae061fdf75c7bd4f0e474d4128981aa3f77a82dc5baed82980e7e66`
  matches GitHub's digest.
- Local `outputs/iDared-Dev-1.77.ipa`, **19,898,256 bytes**, SHA-256
  `559f5d979985475b92294a563a01aa678b5c76c377fe118696316a386dafbf3b`,
  passed archive/CRC and identity checks. The arm64 executable embeds source
  `7640156`, workflow `41284825031`, all five new contract symbols and the
  permanent universal-flight default. It is unsigned, has no provisioning
  profile or code-signature load command, and uses the separate development
  identity `com.bookieart1.idared32bit.dev` / `iDared Dev`.
- No save migration/deletion, app-specific branch, permissive memory handling,
  diagnostic expansion or unrelated compatibility change is included.
  **Physical-device testing is pending.** A removed first blocker, menu reached,
  gameplay observed and functional compatibility remain separate claims.

Verification: `outputs/iDared-Dev-1.77.verification.json`. Engineering detail:
`dev-docs/compatibility-1.77-priority-batch-contracts.md`.

### Dev 1.77 physical-device result

The supplied 530,642-byte log embeds source `7640156` and workflow
`41284825031`; its SHA-256 is
`b608c4f5d259f50ace6cbb26f1b360a4d5aa4388637a20f69ed6802994fc0dde`.
The unmodified file is preserved as
`outputs/device-logs/iDared-Dev-1.77-device-results.txt`. The universal bounded
recorder is active for every relevant launch.

| App | Dev 1.77 device result | Status and next first blocker |
|---|---|---|
| Dark Nebula 2 | Smooth menu/interface followed by a crash after selecting gameplay. The log records 13.895 s, 777 main loops and 391 EAGL returns before `_object_getInstanceVariable`. | **Verified major stage advance; partial compatibility.** `_feraiseexcept` is no longer the blocker. Gameplay remains unobserved. |
| Dirt | The NSData URL/options reader succeeds and returns 1,474 bytes. XML initialization maps NSString encoding 4 to CF UTF-8, then calls `_CFStringConvertEncodingToIANACharSetName`. | **Blocker removed; still startup-blocked.** The new URL reader is validated on this file path, not synchronous network URLs. |
| Glowfish / Glowfish HD | The VM-tagged mapping succeeds; both enter bundled runtime/thread initialization and reach `_pthread_setcancelstate`. | **Blocker removed; corroborated shared startup blocker.** No visible compatibility claim. |
| Hamlet | UIImage `description` and NSLog complete; the next call sends missing `drawAsPatternInRect:` to the valid UIImage. | **Blocker removed; still startup-blocked.** No frame or main-loop iteration. |
| Parachute Ninja | `_sel_getUid`, two method lookups and both IMP reads complete; UIApplication swizzling then calls `_method_setImplementation`. | **Blocker removed; still startup-blocked.** Saved-old-IMP behavior must avoid redispatch recursion. |
| Blot | Smooth through the requested death path and exits through emulator Home after 40.083 s, 1,271 loops and 1,259 EAGL returns. | **Regression pass in stated scope.** |
| Spoing | Smooth and exits through emulator Home after 41.215 s, 1,075 loops and 504 EAGL returns. | **Regression pass in stated scope.** |

## Dev 1.78 verified delivery — second-stage shared contracts

Dev 1.78 implements shared Objective-C runtime, Core Foundation, pthread and
UIKit/Core Graphics contracts for the newly exposed blockers. It does not add
game identifiers, permissive memory behavior, swallowed failures or expanded
diagnostics. Cancellation delivery remains outside the partial pthread contract.

- Source commit: `1af64fa2aeb4b6f5aacfb0ab591a56329c13b8a8`.
- [Tests 41287654522](https://github.com/bookieart1/iDared32bit/actions/runs/41287654522):
  **129/129 library tests passed on macOS and 129/129 on Windows**. Native
  Foundation reference and bounded diagnostic reproducers passed. Android lint
  retains its configured allowed failure; guest executable integration remains
  skipped.
- [IPA build 41287654525](https://github.com/bookieart1/iDared32bit/actions/runs/41287654525):
  iOS compilation, separate identity verification, unsigned packaging and
  artifact upload passed. Artifact `13075932358`, 19,855,683 bytes, SHA-256
  `1007502c2df69aa946c1f12f130d084da534ec489eb6455eef4af42b9c930dda`.
- Local `outputs/iDared-Dev-1.78.ipa`, 19,901,982 bytes, SHA-256
  `8169715424956bf86895877f949898236637b2ea704757407b3119249496108f`,
  passed archive/CRC, arm64, identity and unsigned-package checks. The executable
  embeds source `1af64fa`, workflow `41287654525`, the universal recorder default
  and all new contract symbols.

Verification: `outputs/iDared-Dev-1.78.verification.json`. Engineering detail:
`dev-docs/compatibility-1.78-second-stage-contracts.md`. Physical-device results
are recorded below; removed blocker, visible stage advance and functional
compatibility remain separate claims.

### Dev 1.78 physical-device result

The supplied 544,116-byte log embeds source `1af64fa` and workflow
`41287654525`; its SHA-256 is
`c40ca031bcdad068b7c5c1db56662d5fdf158b929f6fa8de715f2c682e916ac3`.
The unmodified file is preserved as
`outputs/device-logs/iDared-Dev-1.78-device-results.txt`. The universal bounded
recorder is active for every relevant launch.

| App | Dev 1.78 device result | Status and next first blocker |
|---|---|---|
| Blot | Smooth through the requested play/death path and Home exit after 35.533 s, 1,275 loops and 1,233 EAGL returns. | **Regression pass in stated scope.** |
| Dark Nebula 2 | Menu remains smooth. Starting gameplay reaches a real ivar handle from `object_getInstanceVariable`, then missing `_ivar_getTypeEncoding` after 6.816 s, 353 loops and 179 EAGL returns. | **Partial; same visible stage, newly precise runtime blocker.** |
| Dirt | Launches directly into smooth gameplay. Movement, mechanics and pause/resume work. The user-observed pickaxe/map transition reaches missing `NSKeyedArchiver encodeBool:forKey:` on thread 6 after 27.984 s, 1,652 loops and 1,651 EAGL returns. | **Verified major stage advance; partial compatibility.** The XML/CF blocker is removed; boolean save/archive is next. |
| Glowfish / Glowfish HD | Both pass VM-tag and pthread-cancellation-state startup. Each reaches `_fchmod` after `flock` in about half a second, before a loop or frame. | **Previous blocker removed; corroborated file-descriptor blocker.** No visible launch claim. |
| Hamlet | The image pattern method now completes. A valid `UINavigationController` receives missing `setNavigationBarHidden:YES animated:NO` after 113 ms, before a loop/frame. | **Previous blocker removed; still startup-blocked.** |
| Parachute Ninja | UIApplication swizzling progresses through mutable copying, but `_UIApplicationLaunchOptionsURLKey` is unresolved; launch code dereferences the null global and faults after 98 ms. | **Previous blocker removed; still startup-blocked.** The null-page origin is now specific and is not an invalid-object tolerance issue. |
| Spoing | Smooth and returns through emulator Home after 41.418 s, 1,132 loops and 522 EAGL returns. | **Regression pass in stated scope.** |

## Dev 1.79: third-stage shared contracts (candidate)

Dev 1.79 implements the five contracts exposed above without app identifiers,
permissive memory handling, fabricated service responses or diagnostic growth:

- Objective-C `ivar_getTypeEncoding` reads the stable type string from real
  ivar metadata and handles a null handle.
- `NSKeyedArchiver encodeBool:forKey:` stores a keyed property-list boolean;
  focused round-trip coverage includes false and true.
- `fchmod` validates descriptors and applies Unix mode bits to host-backed
  writable files, with errno for read-only/socket/I/O failures. Directory mode
  persistence and non-Unix host bits remain an explicitly partial contract.
- `UINavigationController` retains observable navigation-bar-hidden state; the
  animated setter applies the final state synchronously. Bar rendering and
  animation are still unsupported.
- UIKit exports `UIApplicationLaunchOptionsURLKey` with its historical string
  value.

Predicted focused retest: Blot, Dark Nebula 2, Dirt, Glowfish, Glowfish HD,
Hamlet, Parachute Ninja and Spoing. No save reset is requested. Engineering
detail: `dev-docs/compatibility-1.79-third-stage-contracts.md`. Hosted tests,
iOS compilation and package verification are recorded below; physical-device
behavior remains pending.

### Dev 1.79 verified delivery

- Implementation/source commit:
  `4de150e74abe3163ae05e2e9f422df413e2e5621`.
- [Tests 41289274669](https://github.com/bookieart1/iDared32bit/actions/runs/41289274669):
  **129/129 library tests passed on macOS and 129/129 on Windows**. The native
  Foundation reference, bounded diagnostic reproducers and Android
  lint/cross-compilation passed. Guest executable integration and the native
  macOS guest-test sanity step remain skipped by the development workflow.
- [IPA build 41289274667](https://github.com/bookieart1/iDared32bit/actions/runs/41289274667):
  arm64 iOS compilation, separate development identity verification, unsigned
  packaging and artifact upload passed. Artifact `13076855985`; downloaded ZIP
  SHA-256
  `616600dc598851b987172204efde47907b51380653c3107c3b091823ee086d79`
  matches GitHub.
- Local `outputs/iDared-Dev-1.79.ipa`, 19,904,165 bytes, SHA-256
  `f81dd5d701f5d7a6d6c12c67ba24d607d43fbd2293255a195d7a4e4892fe695e`,
  passed archive/CRC, arm64, identity and unsigned-package checks. It embeds
  source `4de150e`, workflow `41289274667`, the permanent universal-recorder
  default and all five new contract markers.
- No application-specific branch, diagnostic expansion, save mutation/deletion
  or weakened crash check is included. `fchmod` directory persistence and
  navigation-bar rendering/animation remain explicitly partial contracts.

Verification: `outputs/iDared-Dev-1.79.verification.json`. **Physical-device
testing is pending; compilation does not establish compatibility.**

## Dev 1.79 device result and Dev 1.80 fourth-stage contracts

The complete Dev 1.79 device log is preserved as
`outputs/device-logs/iDared-Dev-1.79-device-results.txt`: 781,817 bytes,
SHA-256
`9c8c2c78d2cef030b7a14b63ec082ad4a1246499f4ea671f1198064cdfa1c5cc`.
It embeds implementation source `4de150e7` and workflow `41289274667`; the
universal bounded recorder is active for every relevant launch.

| App | Dev 1.79 result | Current classification / Dev 1.80 contract |
|---|---|---|
| Blot | Smooth play through death and clean Home return after 60.495 s, 3,521 loops and 2,649 EAGL returns. | **Regression pass in stated scope.** No change. |
| Dark Nebula 2 | Smooth menu; level loading reaches missing `_ivar_getOffset` after 6.842 s, 350 loops and 178 EAGL returns. | **Partial.** Implement the standard runtime offset lookup over stable guest ivar metadata, including null-handle behavior. |
| Dirt | Smooth traversal through several areas, pause/resume, Home exit, relaunch and saved-position recovery. Clean launches lasted 113.893 s / 6,781 loops / 6,779 EAGL returns and 5.297 s / 293 loops / 292 returns. | **Working checkpoint in the tested scope.** Dev 1.79's archive contract is device-validated; broader levels/mechanics remain untested. No change. |
| Glowfish | Valid `CMMotionManager` reaches missing `startAccelerometerUpdates` after 1.042 s, before a loop/frame. | **Startup-blocked.** Implement bounded polling accelerometer state using the existing real/simulated sensor source. |
| Glowfish HD | Same valid-manager blocker after 1.467 s, before a loop/frame. | **Startup-blocked; corroborates Glowfish.** Same shared Core Motion contract. |
| Hamlet | Valid `UINavigationController` reaches missing `navigationItem` after 113 ms, before a loop/frame. | **Startup-blocked.** Implement lazy stable `UINavigationItem` ownership and core historical state; bar rendering/animation remain out of scope. |
| Parachute Ninja | Reaches a genuine offline error, but unresolved `_NSErrorFailingURLStringKey` and `_NSURLErrorDomain` make guest error handling dereference a null global after 104 ms. | **Startup-blocked.** Export historical URL-error constants and attach failed-URL metadata without fabricating network success. |
| Spoing | Smooth and clean Home return after 46.129 s, 1,392 loops and 642 EAGL returns. | **Regression pass in stated scope.** No change. |

Dev 1.80 implements these four shared contracts without app identifiers,
permissive pointer handling, swallowed failures, diagnostic changes or save
mutation. Focused guest source coverage was added for runtime offsets, polling
Core Motion and struct ABI, navigation identity/ownership/title propagation,
and URL-error constants/user-info. The development workflow still skips the
guest executable; hosted library tests, iOS compilation, package verification
and physical-device results must therefore remain separate.

Predicted focused retest: Blot, Dark Nebula 2, Dirt, Glowfish, Glowfish HD,
Hamlet, Parachute Ninja and Spoing, alphabetically and without deleting saves.
Engineering detail:
`dev-docs/compatibility-1.80-fourth-stage-contracts.md`.

### Dev 1.80 verified delivery

- Final implementation/source commit:
  `764d60e65428e466c2480d95aecf3d54006ee65b` (`00fb3dda` contains the
  functional implementation; the later commits only wrap a comment and remove
  an unused import).
- [Tests 41292778139](https://github.com/bookieart1/iDared32bit/actions/runs/41292778139):
  **129/129 library tests passed on macOS x86_64 and 129/129 on Windows**.
  Each of the two focused diagnostic reproducers also passed its single test.
  The workflow's lint step remains allowed to fail and reported only two
  pre-existing overlong comments (`ns_dictionary.rs:531` and `dlfcn.rs:61`).
  The development workflow still skips the guest executable integration and
  native macOS guest-test sanity steps, so the added guest source coverage was
  compiled into the iOS build but not executed by this workflow. The Android
  job is green at the job level, but lint failed before a reliable Android
  cross-compilation claim could be made.
- [IPA build 41292778118](https://github.com/bookieart1/iDared32bit/actions/runs/41292778118):
  arm64 iOS compilation, separate development identity verification, unsigned
  packaging and artifact upload passed. Artifact `13078650513`, named
  `iDared32bit-dev-unsigned-764d60e65428e466c2480d95aecf3d54006ee65b`;
  downloaded ZIP SHA-256
  `0d87819aeb53a2e145f347ffa9a365ab8d8d1665ea94b1496afab296fb43a953`
  matches GitHub's published digest.
- Local `outputs/iDared-Dev-1.80.ipa`, 19,916,373 bytes, SHA-256
  `991cbc5458e414fc7d4393b93c97b1d53492a6c1ba08f5d7b4174787d40aa32e`,
  passed archive extraction, arm64 Mach-O, development identity and unsigned
  package checks. It embeds source `764d60e`, workflow `41292778118`, the
  universal recorder marker, and all four new contract markers.
- No application-specific branch, diagnostic redesign, save mutation/deletion,
  fabricated network success or weakened memory check is included. Core Motion
  handler delivery and navigation-bar rendering/animation remain explicitly
  out of scope.

Verification: `outputs/iDared-Dev-1.80.verification.json`. **Physical-device
testing is pending; package and hosted tests do not establish functional game
compatibility.**

## Dev 1.80 device result and Dev 1.81 fifth-stage contracts

The three complete Dev 1.80 logs and hashes are recorded in
`dev-docs/compatibility-1.81-fifth-stage-contracts-and-picker.md`. All embed
source `764d60e`, workflow `41292778118`, and the universal bounded recorder.

| App | Dev 1.80 result | Current classification / Dev 1.81 action |
|---|---|---|
| Blot | Normal smooth play and clean Home return. | **Regression pass in stated scope.** |
| Dark Nebula 2 | Menu remains functional; level loading reaches missing `_object_setInstanceVariable` after 6.319 s, 251 loops and 127 EAGL returns. | **Partial.** Implement the standard raw ivar assignment contract; ownership remains deliberately untouched. |
| Dirt | Traversal, pause/resume and save recovery continue to work. A later focused observation established that rotating the phone does not change gravity, and a held directional touch can reverse after entering the black bar outside the old guest viewport. Its Dev 1.80 launch never activated UIAccelerometer or Core Motion; the shared UIDevice implementation reports only the fixed interface orientation. | **Partial, prior “working” scope narrowed.** Dev 1.81 derives requested physical-orientation updates from the existing real/simulated gravity source with bounded sampling/debounce, separately from interface rotation. It also clamps physical input to the guest viewport before transformation. Both are general contracts; device validation is pending. |
| Glowfish | Core Motion now passes; startup reaches missing `_dispatch_once` after 1.465 s. | **Startup-blocked.** Implement zero-initialized once semantics with synchronous block completion and cooperative contention waiting. |
| Glowfish HD | Reaches menu and name entry. After accepting a name it presents 892 frames over 19.966 s, then calls missing `+[NSDictionary dictionaryWithObjects:forKeys:count:]`. | **Major stage advance; partial.** Implement the standard pointer-array constructor. The portrait keyboard in landscape is a separate unresolved orientation defect. |
| Hamlet | Runs and responds for 33.823 s / 4,072 loops / 1,943 EAGL returns, then exits cleanly through Home. A transient initial orientation/layout error self-corrects. | **Working checkpoint in observed scope, with an orientation defect.** No speculative rotation change in this batch. |
| Parachute Ninja | Real offline failures return, then a main-thread selector callback supplies `UIApplication` where guest code sends `domain` as though it were an error. Strict panic occurs after 140 ms. | **Startup-blocked, cause narrowed but unresolved.** Do not add `domain` to UIApplication; investigate callback argument origin later. |
| Spoing | Continues to work and returns cleanly through Home. | **Regression pass in stated scope.** |

Dev 1.81 also changes the native picker from end-of-gesture page cuts to
bounded interactive adjacent-page movement and persists the last page across
normal returns, emulator panics and process restarts. Rust's in-session reload
page remains authoritative; no guest save/container behavior changes.

The same candidate corrects two general input contracts newly exposed by Dirt.
`UIDevice` now samples the existing real/simulated gravity source only while
physical-orientation generation is requested, distinguishes the six historical
orientations, rejects weak/invalid samples, debounces changes, honors nested
begin/end calls and posts the existing notification without rotating the
interface. Physical touches are clamped to the displayed guest viewport and
original touchscreen bounds; controller-synthesized points are unchanged.
These predict working gravity rotation and no direction reversal when a held
finger enters a black bar, but both remain unverified on the iPhone.

### Dev 1.81 verified delivery

- Final implementation commit:
  `178cc5197ebc80415538d12aaa6458d3a3e47abc`; parent `bb95d1e0` contains the
  fifth-stage runtime and picker work, and the final commit contains the two
  general Dirt input corrections.
- [Hosted tests 41294836579](https://github.com/bookieart1/iDared32bit/actions/runs/41294836579):
  **132/132 library tests passed on macOS x86_64 and 132/132 on Windows**.
  Both selected diagnostic reproducers passed. The job-level green result is
  qualified: lint remains allowed to fail and reports the same two overlong
  comments at `src/libc/dlfcn.rs:61` and
  `src/frameworks/foundation/ns_dictionary.rs:563`; guest executable and
  native macOS guest sanity steps are skipped in development; Android
  cross-compilation is not claimed after the allowed lint failure.
- [IPA build 41294836557](https://github.com/bookieart1/iDared32bit/actions/runs/41294836557):
  arm64 compilation, separate development identity, unsigned packaging and
  upload passed. Artifact `13078298211`; downloaded ZIP SHA-256
  `910b341dbc0dcb4616939aa82936474c869b3e443aa67057e32343f3cf348634`
  matches GitHub's published digest.
- Local `outputs/iDared-Dev-1.81.ipa`, 19,924,279 bytes, SHA-256
  `30880c66ca5c286f8e872efcf8aa969f6e6bbc4c26805dc85e5cbc8d4e0a82f8`,
  passed CRC, arm64 Mach-O, separate identity and unsigned-package checks. It
  embeds source `178cc51`, workflow `41294836557`, the universal recorder and
  the new contract/input markers.

Verification: `outputs/iDared-Dev-1.81.verification.json`. Implementation and
package status are verified within those boundaries; **physical-device results
are pending**. The focused retest is Blot, Dark Nebula 2, Dirt, Glowfish,
Glowfish HD, Hamlet, Parachute Ninja and Spoing, followed by one slow picker
swipe and page-return/process-restart continuity check. No save reset is
requested.

## Dev 1.81 device result and Dev 1.82 sixth-stage contracts

The complete Dev 1.81 log is preserved as
`outputs/device-logs/iDared-Dev-1.81-device-results.txt`: 1,318,957 bytes,
SHA-256
`8a4407fd9473b898d0d14e72a0cf030446ffbc0747f0eb1cdeb841eb2773a48b`.
It embeds source `178cc5197ebc80415538d12aaa6458d3a3e47abc`, workflow
`41294836557`, and the universal bounded recorder. Three supplied screenshots
preserve Glowfish HD's portrait keyboard and Hamlet before/after its host scene
corrects to landscape; their hashes and exact classifications are recorded in
`dev-docs/compatibility-1.82-sixth-stage-contracts-and-host-lifecycle.md`.

| App / surface | Dev 1.81 device result | Current classification / Dev 1.82 action |
|---|---|---|
| Native picker | Interactive page movement and same-page return worked across normal exits and crashes. | **Verified in requested scope.** Retain unchanged. |
| Blot | Smooth play and clean Home return after 53.941 s / 2,132 loops / 2,101 EAGL returns. | **Regression pass in stated scope.** |
| Dark Nebula 2 | Level loading progresses to missing `+[NSMutableIndexSet indexSet]` after 18.949 s / 1,041 loops / 525 EAGL returns. | **Partial; later stage.** Implement general range-backed `NSIndexSet` / `NSMutableIndexSet`, including construction, queries, copying and mutation. |
| Dirt | Device rotation changes gravity. A drag started inside the game keeps its intended edge direction when crossing a black bar. | **Working checkpoint in expanded scope.** Refine the broad input rule: only touches beginning inside the guest are captured; already-captured drags remain clamped at the guest edge. |
| Glowfish | Missing `+[UIPasteboard pasteboardWithName:create:]` after 1.259 s; recorded `create:NO`. | **Startup-blocked at a shared UIKit contract.** Add a bounded named-board registry with genuine nil-on-absent semantics. |
| Glowfish HD | Intro, level selection and controls work; level start reports missing bundled `GameData/Levels/Level_01.gfl`, then deliberately aborts after 46.976 s / 3,117 loops / 2,268 EAGL returns. | **Partial; major visible advance.** Preserve strict abort and do not invent the resource. Host scene orientation is addressed separately for the native keyboard. |
| Hamlet | Runs after a first presentation that is visibly rotated/cropped; later menu is correctly landscape. | **Working checkpoint with verified orientation defect.** Synchronize the active host scene to the guest's existing orientation request. |
| Parachute Ninja | Unchanged: guest sends `domain` to a supplied `UIApplication`. | **Blocked at invalid callback-argument origin.** No invalid-object tolerance. |
| Spoing | One early null-page crash at PC `0x000c2c70`, followed by two working launches and clean Home returns. | **Working regression watch with intermittent failure.** Reproduce before patching. |

The user also established that accepting the system screenshot review/save flow
closes the whole iDared process while a guest is running. The event is absent
from this log, but source inspection strongly identifies the current
background handler's explicit `std::process::exit(0)`. Dev 1.82 keeps the
existing bounded guest save/termination notification but returns to the picker
instead of terminating the host. This is not full suspend/resume support.

Dev 1.82 is therefore a coherent shared-contract and host-boundary candidate:

- range-backed Foundation index sets;
- named UIKit pasteboards with stable identity and correct creation semantics;
- modern host `UIWindowScene` orientation synchronized to the guest request;
- direct-touch capture only when a touch begins inside the displayed guest,
  while preserving edge capture for an active drag; and
- background/screenshot transitions that end only the guest and preserve the
  host picker.

The universal recorder remains unchanged. No game identifier, save reset,
fabricated resource, swallowed abort, invalid-object accommodation or unrelated
compatibility patch is included.

### Dev 1.82 verified delivery

- Final implementation/source commit:
  `99580ce5a9230eb3fb7c51588412e6cc93851d51` (`e6adddee` contains the main
  sixth-stage implementation, `f2d8f1a9` makes pasteboard registry keys owned,
  and the final commit resolves the Swift host-orientation bridge at runtime).
- [Hosted tests 41298463237](https://github.com/bookieart1/iDared32bit/actions/runs/41298463237):
  **135/135 library tests passed on macOS x86_64 and 135/135 on Windows**.
  The native Foundation copy/defaults reference and both selected diagnostic
  reproducers also passed. The development-branch lint step is allowed to fail
  and reported the two existing overlong comments; Android likewise reached an
  allowed lint failure, so Android cross-compilation is not claimed. Formatting,
  full guest executable integration and the native macOS guest-test sanity path
  are branch-gated and skipped.
- [IPA build 41298463255](https://github.com/bookieart1/iDared32bit/actions/runs/41298463255):
  arm64 iOS compilation, separate development identity verification, unsigned
  packaging and artifact upload passed. Artifact
  `iDared32bit-dev-unsigned-99580ce5a9230eb3fb7c51588412e6cc93851d51`;
  downloaded ZIP SHA-256
  `e66c636636a0a5e1d293f08e0cc45217cbe275a7b4cc4e61923858f689af2d8d`
  matches GitHub's published digest.
- Local `outputs/iDared-Dev-1.82.ipa`, 19,947,034 bytes, SHA-256
  `6c1856fbfb84a8e293b746e15a83151c4adfb9e00fc1aabfef7a65ab41ad696e`,
  passed archive extraction, arm64 Mach-O, separate
  `com.bookieart1.idared32bit.dev` identity and unsigned-package checks. It
  embeds source `99580ce`, workflow `41298463255`, the permanent universal
  recorder and the host-orientation bridge marker.

Verification: `outputs/iDared-Dev-1.82.verification.json`. The focused device
procedure is recorded in the Dev 1.82 engineering note; no save deletion is
requested. **Physical-device compatibility remains pending.**

## Dev 1.82 device result and Dev 1.83 seventh-stage contracts

The complete Dev 1.82 log is preserved as
`outputs/device-logs/iDared-Dev-1.82-device-results.txt`: 1,093,375 bytes,
SHA-256
`5cdfb404ed870e53430d0594d523c11a81bb898e5ed6eeea733a0a428cc4777c`.
It embeds source `99580ce5`, workflow `41298463255`, and the unchanged
universal bounded recorder.

| App / surface | Dev 1.82 result | Current classification / Dev 1.83 action |
|---|---|---|
| Blot | Smooth play and clean Home return after 52.418 s / 1,839 loops / 1,792 EAGL returns. | **Regression pass in stated scope.** |
| Dark Nebula 2 | Level loading reaches missing `removeObjectsAtIndexes:` after 18.271 s / 1,000 loops / 505 returns. | **Partial; later shared blocker.** Implement validated descending indexed removal on mutable arrays. |
| Dirt | Gravity/traversal/input remain functional; clean Home return after 63.999 s / 3,984 loops / 3,801 returns. | **Working checkpoint in expanded tested scope.** No new compatibility change. |
| Glowfish | Startup reaches missing `keysSortedByValueUsingSelector:` after 1.288 s. | **Startup-blocked.** Implement dictionary key sorting through the supplied value comparator. |
| Glowfish HD | Name/menu/intro/level selection remain reachable; opening missing bundled `Level_01.gfl` fails and the guest deliberately aborts after 27.970 s / 1,315 loops / 1,183 returns. Keyboard remains portrait. | **Partial.** Preserve the real resource failure. Correct the separate host-orientation connection. |
| Hamlet | Runs and exits cleanly, but the initial presentation remains rotated/cropped. | **Working checkpoint with orientation defect.** Same host correction as Glowfish HD. |
| Parachute Ninja | Guest again sends `domain` to a supplied `UIApplication`. | **Blocked at unchanged invalid callback-argument origin.** No invalid-object accommodation. |
| Spoing | Two normal launches work and exit cleanly. Screenshot review on a third launch ends only the guest and returns to the picker without an emulator panic. | **Regression pass twice; lifecycle fallback only partly successful.** Replace deliberate guest termination with bounded suspend/resume. |

The log records `Host orientation bridge is unavailable` for every launch.
Dev 1.82 therefore never executed its intended Swift scene request. Dev 1.83
registers that function directly from the final executable instead of relying
on a cross-image lookup. This is a corrected integration path, not a new
orientation heuristic.

Dev 1.83 also adds general `NSMutableArray removeObjectsAtIndexes:` and
`NSDictionary keysSortedByValueUsingSelector:` contracts with focused
ownership/order coverage. Transient host inactivity now sends the historical
resign-active lifecycle, pauses every guest thread, waits in SDL, and sends
did-become-active before resuming the same guest. It does not claim background
task execution. Real termination and guest lifecycle exceptions remain
strict.

Final implementation commit: `7d6a55ebba55488762bc890452d0768b0740ad0b`.
Engineering detail and the focused no-save-reset procedure are in
`dev-docs/compatibility-1.83-seventh-stage-contracts-and-suspend-recovery.md`.

### Dev 1.83 verified delivery

- [Hosted tests 41300999515](https://github.com/bookieart1/iDared32bit/actions/runs/41300999515):
  **137/137 development-library tests passed on macOS and 137/137 on
  Windows**. The native Foundation reference, two bounded diagnostic
  reproducers and Android compile/lint job passed. Branch-gated full
  guest-executable steps were skipped.
- [IPA build 41300999509](https://github.com/bookieart1/iDared32bit/actions/runs/41300999509):
  arm64 iOS compilation, separate development identity verification, unsigned
  packaging and artifact upload passed. Artifact ID `13081776006`, ZIP
  SHA-256 `ea4aaa25c738cec94481f2b06c7668ac3dc0998097dd0011d5d496abbcb6f933`,
  matching GitHub's published digest.
- Local `outputs/iDared-Dev-1.83.ipa`, 19,959,152 bytes, SHA-256
  `38f0f199075e5b1157e6047551d1e700f49297c7271c657d20988b677fff0c75`,
  passed embedded commit/workflow, arm64, separate bundle identity, universal
  recorder and unsigned/no-provisioning checks.

Verification: `outputs/iDared-Dev-1.83.verification.json`. Device results
remain pending and separate from these build and host-test checkpoints.

## Dev 1.83 device result and Dev 1.84 eighth-stage contracts

The complete Dev 1.83 log is preserved as
`outputs/device-logs/iDared-Dev-1.83-device-results.txt`: 874,547 bytes,
SHA-256
`4f0c7442591e142bde9777e92f5ee01fab552b231ea30e720e0c4cfff6a9c147`.
It embeds source `7d6a55ebba55488762bc890452d0768b0740ad0b`, workflow
`41300999509`, and the unchanged universal bounded recorder.

| App / surface | Dev 1.83 result | Current classification / Dev 1.84 action |
|---|---|---|
| Blot | Smooth play and clean Home return after 70.900 s / 3,814 loops / 3,204 EAGL returns. | **Regression pass in stated scope.** |
| Dark Nebula 2 | Completed a level and accepted name entry. After **No** at automatic score submission, local leaderboard bookkeeping sent missing `dateFromString:` to a valid `NSDateFormatter` after 196.587 s / 11,660 loops / 5,717 returns. | **Major advance; partial post-level blocker.** Add bounded, validating Unicode-pattern parsing with formatter time-zone semantics. This is not evidence of a network crash. |
| Dirt | Continues to work; clean Home return after 58.001 s / 3,945 loops / 3,441 returns. | **Regression pass in expanded scope.** |
| Glowfish | Valid `NSProcessInfo` receives missing `globallyUniqueString` after 1.308 s, before a main loop. | **Startup-blocked.** Return a fresh autoreleased RFC 4122 v4 string per call, without stable host identity. |
| Glowfish HD | Name/menu/intro/selection remain reachable. Missing bundled `Level_01.gfl` causes the guest's deliberate `abort` after 29.425 s / 1,352 loops / 1,261 returns. Keyboard remains portrait. | **Partial with genuine package/resource failure.** Preserve strict abort; correct separate host-orientation request ordering. |
| Hamlet | Runs and exits cleanly, but its initial presentation remains rotated/cropped. | **Working checkpoint with orientation defect.** Defer scene geometry update until SDL's game root controller exists. |
| Parachute Ninja | Guest again sends `domain` to a supplied `UIApplication`. | **Blocked at unchanged invalid callback origin.** No invalid-object accommodation. |
| Spoing / screenshot | After screenshot interruption, the log records resign-active, guest pause, did-become-active, resume and later presented frames; it ends mid-session without panic. | **Immediate suspend/resume pass.** Preserve current code; later host termination is not established by this log. |

Dev 1.84 is limited to `NSDateFormatter dateFromString:`,
`NSProcessInfo globallyUniqueString`, and host scene-request ordering with
bounded orientation evidence. The universal recorder remains unchanged. No
game identifier, resource fabrication, network-success fabrication, abort
suppression, save mutation or permissive invalid-object behavior is included.

Engineering detail and focused no-save-reset procedure:
`dev-docs/compatibility-1.84-eighth-stage-contracts-and-orientation-ordering.md`.

## Additional Dev 1.82 and Dev 1.83 new-game samples

Two November 1 samples are preserved without altering the Dev 1.71 historical
survey. The Dev 1.82 sample embeds `99580ce5` / workflow `41298463255`; the Dev
1.83 sample embeds `7d6a55e` / workflow `41300999509`. The universal bounded
recorder operated for every decrypted title that entered CPU emulation.

| Result family | Apps and evidence |
|---|---|
| Working checkpoints in observed scope | Bumpy Road (clean Home; no logged random crash), Oscura (two clean launches), RastaMonkey, Snail Mail and Sway (two clean launches), supported by the user's gameplay/menu observations. |
| Exact shared missing contracts | Fragger / Cube: `UIFont.fontName`; SpaceMnkey / Zombie Attack! 2nd Wave: `_mbsrtowcs`; SUPERFALL / Silverfish: `NSNumberFormatter`; Rolando / junny: run-loop cancellation; Toy Bot 1 and 2 / Crazy Golf: `UISlider.setContinuous:`. |
| Exact shared subsystem boundaries | 1000:Find'Em / Glyder / Glyder 2 / JumpO'Clock: detached-view/window conversion; decrypted 8bitninja / DoDoFly: Core Foundation dictionary capacity/callback semantics; JellyCar 3 / Decoloro: `UINavigationBar`. |
| New explicit general blockers | Max & the Magic Marker: `_nl_langinfo`; Peggle: localized `NSBundle` resource lookup; Ragdoll 3: `hw.activecpu`. |
| Origin still unresolved | Slingshot: null dereference after an `objectAtIndex:` path; Topple: nil `If-None-Match` value after an unavailable service request; Topple 2: untracked receiver during Objective-C class initialization. These are not permission to weaken checks. |
| Not a compatibility result | 8bitninja 1.4 is encrypted and never entered CPU emulation. |

The complete analysis, implementation boundaries and exact per-app metrics are
in `dev-docs/compatibility-new-game-samples-2026-11-01.md`. Archived logs:
`outputs/device-logs/iDared-Dev-1.82-new-game-sample-2026-11-01.txt` and
`outputs/device-logs/iDared-Dev-1.83-new-game-sample-2026-11-01.txt`.
Machine-readable inventory:
`outputs/iDared-new-game-samples-2026-11-01.csv`.

## Dev 2.01 candidate: five shared contracts and two bounded causal histories

The Dev 2.00 device run verified that all five preceding contracts passed and
that Solomon's Boneyard reached working gameplay in the tested scope. Dev 2.01
implements the newly exposed general blockers: KVC `setValue:forKeyPath:`,
Audio Unit render-notification registration and delivery, dyld executable-path
size negotiation, per-thread `sigaltstack`, and retained segmented-control
tint state. These target LegendaryWar, Peggle, Tiki Totems 2, TrenchesVS and
ZA! 2ndWave respectively.

8bitninja and DoDoFly remain maintained return targets. This candidate changes
neither title's behavior; it adds fixed-size failure-only bundle candidate
history and fixed-size dictionary ownership/callback lifecycle history so the
next strict failure can distinguish origin hypotheses. The universal recorder
remains enabled. Full evidence, boundaries and device predictions are in
`dev-docs/compatibility-2.01-kvc-audio-signal-ui-and-causal-evidence.md`.

## Dev 1.84 device result and Dev 1.85 ninth-stage contracts

The three complete Dev 1.84 logs are preserved under `outputs/device-logs/`.
They embed source `e5045fb063ce2d0c0b8f870ac29feae79c43510f`, workflow
`41303668226`, and the unchanged universal bounded recorder.

| App / family | Dev 1.84 evidence | Dev 1.85 action / boundary |
|---|---|---|
| Blot | Played normally; clean Home return. | Regression pass in stated scope. |
| Dark Nebula 2 | Completed a level; whole iDared process ended when name entry was requested. Log ends after text-field configuration without a guest panic. | Revert the failed deferred scene-ordering change to the Dev 1.83 keyboard-safe path. No post-name score-flow claim yet. |
| Dirt | Not retested in these split logs; preceding expanded checkpoint remains working. | No change; focused regression. |
| Glowfish | Valid `NSDate` receives missing `encodeWithCoder:` during keyed archival after 1.321 s, before a main loop. | Implement standard `NS.time` double encoding and the matching archiver primitive. |
| Glowfish HD | Menu reached; requesting name entry reproducibly ended the whole host without a guest panic. | Same host-ordering rollback. Genuine absent level resource remains separate and will not be fabricated. |
| Hamlet | Runs after unchanged initial rotated/cropped presentation. | Orientation defect remains open; do not retain the regressive ordering attempt. |
| Parachute Ninja | Offline paths culminate in `domain` sent to a real `UIApplication`. | Origin unresolved; strict behavior retained. No invalid-object accommodation. |
| Spoing | Normal play and clean Home return. | Regression pass in stated scope. |
| Toy Bot / Toy Bot 2 / Crazy Golf | Survey logs share missing `UISlider.setContinuous:`; Crazy Golf already has a working gameplay checkpoint. | Store the historical continuous flag; rendering remains incomplete. Crazy Golf is a regression/extended-path test, not a nonworking title. |
| Fragger / Cube | Exact shared missing `UIFont.fontName`; Fragger had already reached a functional menu. | Preserve stable public font identity across resizing. |
| Rolando / junny | Exact shared missing `NSRunLoop.cancelPerformSelectorsWithTarget:`. | Cancel matching queued targets with correct retain and waiter cleanup. |

Implementation rationale, limitations, log hashes and predictions are in
`dev-docs/compatibility-1.85-ninth-stage-shared-contracts.md`. This candidate
contains no game-identifier branch, network-success fabrication, missing-file
fabrication, relaxed memory check, save reset, or diagnostic redesign.

## Dev 1.95 candidate: `NSBundle bundleForClass:nil` fallback

The Dev 1.93 bounded history for DizzyWizards shows its own caller loading a
literal zero as the argument to `+[NSBundle bundleForClass:]`. The prior
missing-class-lookup theory is therefore rejected. iDared returned `nil`, and
the guest later faulted while copying the nil-derived executable path.

Dev 1.95 makes nil and application-class arguments return the existing
main-bundle singleton in iDared's current one-bundle model. The implementation
is general and keeps strict object and null-memory checks. Guest ABI and native
Foundation tests cover identity; physical-device validation remains pending
for DizzyWizards and Marblenauts, with Spoing as regression. Full rationale:
`dev-docs/compatibility-1.95-nsbundle-nil-class-contract.md`.

## Dev 1.95 device result and Dev 1.96 POSIX `sync()` contract

The two complete device logs embed source `389b9d4`, workflow
`41401657404`, and the permanent bounded recorder. They are preserved as
`outputs/device-logs/iDared-Dev-1.95-dizzywizards-marblenauts-2026-11-02.txt`
and `outputs/device-logs/iDared-Dev-1.95-spoing-regression-2026-11-02.txt`.

| App | Dev 1.95 evidence | Current classification / Dev 1.96 action |
|---|---|---|
| DizzyWizards | `bundleForClass:nil` now returns the main bundle; `executablePath` and `UTF8String` also return valid values. The next panic is missing `_sync` at 168 ms. | **Former blocker removed; exact general POSIX blocker exposed.** Implement the void `sync()` contract by flushing open writable guest files. |
| Marblenauts | Runs for at least 55 seconds and advances from 298 to 3,305 main-loop iterations, but records zero EAGL presentation returns. One bundled-resource open fails. The session ends on host resign-active without a recorder dump. | **Active no-present black screen.** The doubled slash is already normalized and is not itself a proved bug. Make no speculative resource change; exit the next run with emulator Home to dump bounded history. |
| Spoing | Clean emulator-Home return after 8.6 seconds, 244 main loops and 116 EAGL presentation returns. | **Regression pass in stated scope.** |

Dev 1.96 adds only the general POSIX `sync()` export plus focused guest ABI
coverage. It does not change Marblenauts rendering/resources, fabricate files,
relax memory checks or change the universal recorder. Engineering detail:
`dev-docs/compatibility-1.96-posix-sync-contract.md`.

## Dev 1.97 candidate: shared runtime/text contracts and graphics evidence

Dev 1.97 combines five independently testable general compatibility contracts
with the bounded evidence needed for the two distinct Dev 1.96 black-screen
families. It accepts positive `CFDictionaryCreateMutable` capacity hints;
implements `class_getClassMethod`, `class_getName`, `CFGetTypeID`, and
restartable UTF-8 `mbsrtowcs`; records no more than three GL state snapshots per
EAGL context; and moves the iOS Home control into a native host overlay so a
zero-present guest cannot remove the user's escape path. It does not alter
guest rendering state, suppress failures, fabricate resources, or branch on an
app identity.

Physical-device status is **pending**. The target batch is 8bitninja 1.2,
DizzyWizards, DoDoFly, LegendaryWar, Marblenauts, Peggle, SpaceMnkey,
TrenchesVS and ZA! 2ndWave. Blot, Cube and Spoing are focused regression tests.
Removing an initial missing-symbol/assertion blocker is not a claim of gameplay
compatibility. Engineering record:
`dev-docs/compatibility-1.97-shared-runtime-text-and-graphics-evidence.md`.

## Dev 1.97 result and Dev 1.98 candidate

The complete Dev 1.97 device run is preserved as
`outputs/device-logs/iDared-Dev-1.97-shared-contract-batch-2026-11-02.txt`
(726,513 bytes; SHA-256
`257e39fa791b5459919de98db4aa00060fd3bda3fbaf1bb03e17a3c5bd456e3b`).
SpaceMnkey reached working gameplay/menu re-entry in the tested scope; Blot,
Cube and Spoing passed; and Marblenauts' zero-present state can now be exited
through native Home. DoDoFly, LegendaryWar, Peggle, TrenchesVS and ZA!
2ndWave advanced respectively to exact `UIFont.familyNames`,
`method_exchangeImplementations`, `CFStringGetTypeID`,
`NSNumber.numberWithUnsignedLong:` and `strcasestr` boundaries.

Dev 1.98 implements those five general contract families with focused tests.
8bitninja's unsupported socket length remains strict but now reports its actual
length, family and raw bytes. The tiled `UNOFFICIAL` picker decoration is
removed without changing fork identification in version/log/About surfaces.
DizzyWizards and Marblenauts receive no speculative rendering change.

Physical-device status is **pending**. Focused targets are 8bitninja 1.2,
DoDoFly, LegendaryWar, Peggle, Solomon's Boneyard, TrenchesVS and ZA!
2ndWave; Blot, SpaceMnkey and Spoing are regressions. Engineering record:
`dev-docs/compatibility-1.98-shared-font-runtime-cf-and-libc-contracts.md`.

## Dev 2.01 device result and Dev 2.02 candidate

The complete Dev 2.01 run verifies that all five Dev 2.01 contracts passed
their former blockers. Blot, Solomon's Boneyard, Space Monkey and Spoing are
clean focused regressions. The new decisive boundaries are: an invalid Audio
Unit handle after Peggle requests unavailable 3D Mixer; nil dictionary copy
initialization in Tiki Totems 2's offline path; `conformsToProtocol:` in
TrenchesVS; and `_wcsrtombs` in ZA! 2ndWave.

Dev 2.02 implements those four general contracts without claiming 3D Mixer
support. 8bitninja, DoDoFly and LegendaryWar remain strict unresolved roots;
their next logs gain bounded path-tail, watched-value lifetime, and
property-list dictionary mutability/origin evidence respectively. No invalid
receiver, null memory, missing resource, immutable collection, network result,
or CF ownership rule is relaxed. Full record and predictions:
`dev-docs/compatibility-2.02-protocol-widechar-null-init-and-causal-origins.md`.
# Dev 2.17 final expanded IPA-evidence candidate — 2026-11-03

- The reviewed Dev 2.17 implementation at `0a9144d8` remains the base. The
  final candidate adds numeric-IPv4 `gethostbyname` semantics for ADVENA and
  ZENONIA 2, bounded data-source-driven UITableView cells/selection for Bungee
  Ball, richer post-input GLES array-source evidence for Flip's Escape, and
  per-callback array mutation provenance for Trenches VS.
- Decoloro, Parachute Ninja, Peggle, Splode and Toy Bot 2 are diagnostic-only
  tests using bounded or permanent existing evidence streams; no result or
  invalid object is fabricated. B'Blast, SUPERFALL and Tiki Totems 2 retain the
  bounded diagnostics from the earlier Dev 2.17 work.
- EPOCH, JellyCar 3, Junny, Ragdoll 3, Rolando, Topple 2, Unmechanical and
  Wind-Up Knight retain their documented general/partial contracts. Spoing is
  the only regression control.
- Hook World, Legendary War, Slingshot Racing and Mage Gauntlet are explicitly
  omitted because this candidate contains no safe attributable change for
  them. See `compatibility-2.17-expanded-ipa-evidence-build.md`.
- The app-picker header now shows `Dev 2.17`, and this meaningful test batch is
  placed before the alphabetized remainder. Physical-device status is pending
  a verified hosted artifact.

## Dev 2.17 physical-device result — 2026-11-03

Device logs verify implementation `eb27d7f` and workflow `41694954194`.
All 22 requested titles were exercised. No title became newly playable and the
user observed every failure at effectively the same visible point. Spoing was a
clean regression. That zero-visible-gain result must remain distinct from the
following internal boundary advances:

- ADVENA / ZENONIA 2: numeric host parsing passed; next blocker `_bcopy`.
- EPOCH / Unmechanical: AVURLAsset creation passed; next blocker
  `+[AVPlayerItem playerItemWithAsset:]`.
- Bungee Ball: data-source cell construction passed; next blocker one-argument
  UITableView reuse lookup.
- Junny: edge-insets parsing and ABI passed; next blocker `drawTextInRect:`.
- Ragdoll 3: `hw.activecpu` passed; next blocker `hw.cachelinesize`.
- Rolando: current-point lookup passed; next blocker `_CGPathAddCurveToPoint`.
- Topple 2: nil-header semantics passed; next blocker the historical UIAlertView
  initializer.
- Wind-up Knight: `CFMakeCollectable` passed; next boundary is
  `setShouldProcessNamespaces:YES`, not external-entity resolution.

The new causal evidence is stronger than the visible result. SUPERFALL's
resource-name NSString is proven created/autoreleased, released to zero,
deallocated, then consumed. Tiki Totems 2's invalid dictionary value is proven
to be a fresh 12-byte raw allocation with nil isa, not a stale Objective-C
object. Peggle recorded `GL_INVALID_FRAMEBUFFER_OPERATION` before Core Animation
composition even though the later status query reports a complete framebuffer.
Toy Bot 2 repeatedly rejects taps as additional touches on a non-multitouch
ComicView, indicating a likely stuck touch lifecycle. Decoloro's live landscape
controls sit outside a portrait-width container/hit region. B'Blast's final
`strstr` miss links character XML selection to preceding synthesized missing
environment XML paths.

The complete review and evidence hashes are in
`../../outputs/iDared-Dev-2.17-device-review-2026-11-03.md`. No new compatibility
build or push was performed for this review.

## Dev 2.18 later-contract implementation — 2026-11-03

- Dev 2.17 (`eb27d7f5`, workflow `41694954194`) produced no visible success
  across the 22-title batch, while exposing eight exact later boundaries.
- Dev 2.18 implements general `bcopy`, AVPlayerItem asset construction,
  UITableView reuse dequeue, `hw.cachelinesize`, cubic CGPath flattening,
  UILabel `drawTextInRect:`, the historical six-argument UIAlertView
  initializer, and partial NSXMLParser namespace-option behavior.
- Primary tests are ADVENA, Bungee Ball, EPOCH, Junny, Ragdoll 3, Rolando,
  Topple 2, Unmechanical, Wind-up Knight, and ZENONIA 2. Spoing is the sole
  regression control. Each requested title has changed code; unchanged
  standstill titles are omitted rather than replayed without a new experiment.
- Contract status: `bcopy` and the cache-line sysctl are verified narrow
  contracts; AVPlayerItem, table reuse, CGPath curve rendering, historical
  UIAlertView, and namespace processing are partial contracts with the limits
  recorded in `compatibility-2.18-later-contracts.md`.
- Physical-device status remains pending. Passing one boundary is not yet a
  menu, gameplay, save/relaunch, or functional-compatibility result.

## Dev 2.18 device result and Dev 2.19 candidate — 2026-11-03

The supplied Dev 2.18 log verifies implementation `9566691` and workflow
`41703580910`. ADVENA reached gameplay but later requested XML property-list
serialization during exit; Bungee Ball displayed level content over the wrong
yellow presentation and failed after selecting a level; EPOCH and Unmechanical
advanced to `AVPlayerItem.status`; Ragdoll 3 advanced to
`hw.l1dcachesize`; Topple 2 exposed a string in the historical alert
initializer's final argument. Junny, Rolando, Wind-up Knight and ZENONIA 2
remained strict later-stage failures. Spoing failed once and then ran normally,
so first-launch state remains an intermittent comparison rather than a passed
single-run regression.

Dev 2.19 implements the four exact general corrections above and adds bounded
causal provenance for the eleven Dev 2.18 omissions: complete `strstr` result
sequence (B'Blast), touch lifecycle (Decoloro/Toy Bot 2), delayed-selector
schedule/delivery (Parachute Ninja), raw dictionary payload words (Tiki Totems
2), autorelease-pool identity (SUPERFALL), array/object classes (Trenches VS),
and symbolizable fault-instruction bytes (JellyCar 3/Splode and strict terminal
failures). Flip's Escape and Peggle receive correlation evidence, not a
speculative graphics correction. No invalid memory, invalid object, missing
resource, media readiness, networking success, or collection index is
fabricated.

The picker now exposes a persistent user-controlled classification: long-press
an icon to mark it Playable or Not Playable, or restore the built-in status.
This changes only the picker filters and remains deliberately separate from
the evidence ledger. Full boundaries and predictions are in
`compatibility-2.19-contracts-provenance-and-picker-status.md`. Physical-device
status is pending.

## Dev 2.25 exact missing-system-contract batch — November 4, 2026

Dev 2.24 phone evidence is now reconciled in the dedicated cocos2d records.
Dev 2.25 batches the full exact-contract queue without app-specific branches:
concrete `NSSet member:` identity (Mini Militia), legacy UIView current-state
animation (Cannon Cadets), programmatic UIAlertView dismissal (Clowning
Around), offline read-stream scheduling validation (Cordy/Current), Darwin
network error text (Guerrilla Bob/Muffin Knight), GLES texture-parameter query
(Kick the Buddy), UITableViewCell editing-indent state (Labyrinth 2), `EBADF`
for a vacant nonnegative descriptor (Sonic Dash), version-0 CFRunLoop source
lifecycle (Trainyard), and copyable 32-bit incremental CommonCrypto SHA-1 state
(ZombieSmash). Spoing is the sole general control.

Final implementation `18719b57629070ee53c8775402f1851eeb346d6c` is packaged
by successful unsigned IPA workflow `41808379942`. Windows and macOS each pass
301/302 library tests; all new contract tests pass and the one failure is the
unchanged XML property-list fixture. The verified arm64 development IPA is
unsigned, embeds the final commit/run, retains the universal recorder, and has
SHA-256 `e07f8f4631b217533f3d4b27abea3e4e7ae8f8555bacc92584c337111e40bad8`.

None of the twelve affected titles is promoted to working; each prediction is
only that its recorded first boundary will be crossed. Detailed scope and
limitations are in
`cocos2d-phase1/dev-2.25-exact-system-contracts.md`. The universal recorder is
unchanged, and unrelated invalid-object, graphics, input and package clusters
remain outside this build.

## Dev 2.24 broader-library retrospective — November 4, 2026

Three later-supplied logs all embed implementation `fb810996` and workflow
`41796486938`, proving that this was a **Dev 2.24** sweep even though the picker
still displayed Dev 2.23. This survey was performed after Dev 2.25 had already
been delivered, so it is historical device evidence rather than a new build
request.

### Verified or strongly supported advances

| App | Current observed reach | Causal assessment |
|---|---|---|
| A Monster Ate My Homework | User reports working gameplay; clean Home return after 28.5 s, 1,360 loops and 548 presentations. | **Genuine broad-fix gain.** The old Dev 1.90 terminal path stopped immediately after `UILabel` shadow setters. Dev 2.10 later implemented retained shadow color/offset and drawing. Crossing that exact boundary and sustaining gameplay strongly implicates the general UILabel correction, although no one-change bisection proves sole causality. |
| FallDown! | Gameplay runs smoothly until death/name entry; the log then ends without an emulator panic. | **Genuine Dev 2.24 advance, still partial.** The former startup blocker was unimplemented `ADBannerView`; Dev 2.24's bounded offline iAd lifecycle now reports server failure and lets the game run. The score-name path appears to terminate the native host and needs a matching iOS `.ips` report; it is not yet a normal emulator panic or a full pass. |
| Rope'n'Fly 2 | User reports working gameplay; clean Home return after 30.0 s, 1,783 loops and 1,782 presentations. | **High-confidence Dev 2.24 fix.** The old terminal error was conversion of negative Apple-epoch seconds into an unsigned Duration. The signed historical date correction directly removes that blocker. |
| Mini Militia | Reaches visible horizontal content, but in the wrong orientation, then stops at missing `NSMutableSet member:` after 288 loops / 144 presentations. | **Real later-stage progress.** Dev 2.24's offline iAd path removed the earlier banner-allocation boundary. The exact set membership contract is implemented in Dev 2.25; orientation remains independent. |
| Super Stick Golf | Menus work; selecting a level reaches a later nil mutable-string consumer after 658 loops / 333 presentations. | **Verified Dev 2.24 boundary removal.** Primitive Boolean keyed-unarchive support explains the menu advance. Gameplay remains blocked by a separate string/lifetime origin. |
| Stick Stunt Biker | Reaches an intro/game frame and 93 presentations, then stops at the ASCII-only NSString guard. | **Verified Dev 2.24 boundary removal.** Signed Apple-epoch handling removes the earlier negative-duration failure and exposes a distinct non-ASCII string limitation. |
| Cordy | Runs 12.2 s / 762 loops / 254 presentations, then sends `statusCode` to a generic `NSURLResponse`. | **Genuine stream-path advance.** Dev 2.24 offline read-stream scheduling crosses the previous stream boundary. Correct HTTP-response typing or failure delivery is next; a fake status is not justified. |
| Trism | Runs 6.4 s / 226 loops, but presents no EAGL frame in this launch; stops at missing `+[NSObject cancelPreviousPerformRequestsWithTarget:]`. | **Cumulative earlier progress, not a Dev 2.24 win.** Prior UIButton-shadow and formatter corrections already advanced this title. The delayed-selector cancellation contract is new exact evidence. |
| Perfect Cell | Runs 11.4 s / 544 loops / 319 presentations, then stops at missing `NSScanner scanLongLong:`. | **Genuine late-stage reach, attribution not isolated to Dev 2.24.** Older ad/framework corrections likely enabled the path. The scanner method is an exact small general-contract candidate. |

### Inconsistent or not established as progress

| App | Current evidence | Interpretation |
|---|---|---|
| QuickHook | Fails at 160 ms because immutable `_touchHLE_NSDictionary` receives `setObject:forKey:`; user previously observed one working run. | Intermittent/restored-state-sensitive, not a stable pass. Preserve strict mutability; compare fresh and restored state before another semantic change. |
| Arcade Claw | Runs 11.9 s / 761 loops / 542 presentations, then calls guest `___assert_rtn`. | Substantial runtime is verified, but no like-for-like recent baseline proves Dev 2.24 caused it. The assertion endpoint is downstream; capture assertion text/caller before patching. |
| Gravity Guy | 47 loops / 26 presentations, wrong orientation, then nil `NSData`. | No verified later stage. Loading imagery alone is insufficient; orientation and the nil producer are separate. |
| JellyCar 3 | 46 loops / 85 presentations, wrong orientation, then the known null-page read at `0x54`. | Existing unresolved failure, not a verified gain. |
| Monsters Ate My Condo | Null-page read at `0x0` after 108 ms, no loops or presentations. | **Confirmed unchanged** from Dev 2.22/2.23; the purple flash is not compatibility progress. |
| Modern Combat 2 / Sandstorm2 | Stops at unimplemented `GKSession alloc` after 141 ms, two loops, no presentations. | Logo flash only; exact GameKit session blocker, no meaningful gameplay progress. |
| Moonsters | Receiver `isa` is nil at `componentsSeparatedByString:` after 1.7 s, 15 loops and one presentation. | No reliable gain; invalid-object origin remains required. |
| Paper Pilot | Guest `___assert_rtn` after 2.7 s, with no loops or presentations. | Menu flash is not proof of progression; caller/assertion provenance is needed. |
| Pig Rush | Manual Home return after 2.3 s, one loop and no presentations. | No crash in this short run, but no presented gameplay and no basis to reverse the historical guest-`exit(1)` result. |
| Pudding Panic | Reproduces the same null-page read at `0x18` twice (112/120 ms), no loops or presentations. | **Confirmed unchanged and deterministic.** |
| Ramps | 276 loops / 212 presentations, then unimplemented `GKLeaderboardViewController alloc`. | Exact GameKit UI blocker; the loading screen alone is not new progress. |
| Red Ball of Goo | Stops at missing `_SecCertificateCreateWithData` after 153 ms, four loops and no presentations. | Logo flash only; exact certificate/service blocker, no meaningful progress. |
| Scarecrow | `UIApplication` incorrectly receives `domain` after 124 ms and no presentation. | Likely wrong receiver/dispatch origin; the flashed frame is not proof of improvement. |
| SPYmouse | Reaches 368 loops / 87 presentations, then the known nil mutable-string resource/path consumer. | Same later blocker as prior focused runs; no new producer identified. |
| Tentacle | Stops at the existing NSString internal assertion after 285 ms, no loops or presentations. | Logo flash only; no verified progression. |

The broad pattern is therefore not that the recorder itself made games work.
General framework corrections accumulated across builds: UILabel rendering
state most likely explains A Monster Ate My Homework; Dev 2.24 iAd, signed-date,
keyed-archive and offline-stream contracts directly explain FallDown,
Rope'n'Fly 2, Mini Militia, Super Stick Golf, Stick Stunt Biker and Cordy.
Other splash-frame differences are either older cumulative progress, uncertain,
or demonstrably unchanged. No new build or compatibility patch is authorized by
this retrospective alone.

## Dev 2.25 physical-device result — November 4, 2026

Both returned logs identify implementation `18719b57629070ee53c8775402f1851eeb346d6c`
and unsigned IPA workflow `41808379942`; this is the verified Dev 2.25 artifact,
not an archive-label inference. All ten changed contracts crossed their former
first blocker. Spoing remains working in the tested scope. No other title is
promoted to fully compatible from this pass.

| App | Furthest verified result | New first boundary / interpretation |
|---|---|---|
| Cannon Cadets | Startup advances beyond the animation-state export. | Missing `_CGGradientCreateWithColorComponents`; exact later Core Graphics contract. |
| Clowning Around | Seven main-loop/frame presentations occur beyond alert dismissal. | `UIProgressView` class is absent; exact later UIKit contract. |
| Cordy | 12.9 s, 732 loops, 283 presentations. | Local-file transport constructs generic `NSURLResponse`, then guest asks for `statusCode`; known later response-typing contract. |
| Current | Clean manual return after 3,634 loops and 416 presentations. | No emulator failure in the log, but no visual/gameplay observation was supplied; status remains uncertain rather than working. |
| Guerrilla Bob | 31.9 s and 1,507 presentations; menu is visible beneath a dark layer but cannot be controlled. | A full-screen, visible, interactive `UIView` at alpha 0.4 receives every recorded touch. Need bounded overlay/controller creation and dismissal provenance; global hit-test changes are unsafe. |
| Kick the Buddy | Advances beyond texture-parameter querying. | `UIImage` rejects unknown bytes after missing `Dust_03b.png` and `particleTexture.png`; capture exact failing resource path/header before adding format support. |
| Labyrinth 2 | 145 loops and 74 presentations. | Missing `UITableView cellForRowAtIndexPath:`; exact later UIKit contract. |
| Mini Militia | 62 loops and 31 presentations. | Property-list serialization rejects `NSMutableData`; exact later Foundation class-cluster/output contract. |
| Muffin Knight | Black screen; the emulator Home control cannot return to the picker. | Four frames and 82 main loops occur, then the main thread remains in guest PCs `0x15b3b50..0x15b3bdc` after offline connect failure while Home events queue unhandled. Symbolize the loop and record bounded socket retry results; do not fabricate network success. |
| Sonic Dash | Advances beyond vacant-descriptor `F_GETFD`. | A later vacant-descriptor `fcntl(F_GETFL)` unwrap panics; all descriptor-dependent commands need historical `EBADF` behavior. |
| Trainyard | Advances through CFRunLoop source scheduling and begins rendering. | `NSMutableData` receives `initWithBytes:length:` through an incompatible host method signature; correct the mutable-data initializer/class-cluster contract, not strict dispatch. |
| ZombieSmash | SHA-1 startup completes. | Missing `NSXMLParser abortParsing`; exact parser cancellation contract. |
| Spoing | User reports normal play; clean return after 1,176 loops and 480 presentations. | Relevant regression control passes. |

Dev 2.25 therefore removed its intended first blockers but mostly revealed the
next layer. The strongest next direct-contract candidates are gradient creation,
`UIProgressView`, table-cell lookup, mutable-data/property-list behavior,
descriptor-wide `fcntl` error handling and XML-parser cancellation. Guerrilla
Bob and Muffin Knight now have bounded causal investigations rather than generic
"overlay" or "black screen" labels. No new IPA is authorized by this log review.

## Dev 2.26 authorized later-contract batch — November 4, 2026

The user authorized one build containing every title from the Dev 2.25 pass.
The batch implements general later contracts for Cannon Cadets, Clowning
Around, Kick the Buddy, Labyrinth 2, Mini Militia, Sonic Dash, Trainyard and
ZombieSmash. It adds bounded causal evidence for Cordy's non-HTTP file response,
Guerrilla Bob's blocking overlay lifecycle and Muffin Knight's offline connect
caller/retry pattern. Current is retained only as the relevant URL-loading
control and Spoing as the general healthy control. No title is promoted before
physical testing, and none of the diagnostics changes guest behavior.

The exact implementation boundary, prediction and falsifier for each title are
in `cocos2d-phase1/dev-2.26-later-contracts-and-causal-probes.md`.

## Dev 2.26 physical-device result — November 4, 2026

All three returned logs identify implementation
`eeb7775480cb8bd7b76089fdcdf64d9f16db1b3f` and workflow `41816961881`.
Every eligible launch retained the universal recorder. The complete hashes,
per-title evidence and next actions are in
`cocos2d-phase1/dev-2.26-device-result.md`.

| App | Verified Dev 2.26 result | Current decision |
|---|---|---|
| Cannon Cadets | Crosses gradient creation, then lacks `_CGContextDrawLinearGradient`. | Safe general drawing contract is next. |
| Clowning Around | Crosses `UIProgressView`; 112 frames precede an absent host-object lookup during stage-select animation. | Record the exact failed receiver/generation; do not tolerate unregistered objects. |
| Cordy | Repeats `statusCode` sent to a local-file `NSURLResponse` at caller `0x008f6320`. | Symbolize guest caller; no fake HTTP response. |
| Current | User sees an unresponsive menu, but 162 touches reach the full-screen EAGL view and 1,965 frames present. | Trace guest callback arguments/coordinates; basic hit-testing theory is rejected. |
| Guerrilla Bob | User reports working; 2,621 presentations and clean Home return. | Working in tested scope; no immediate retest. |
| Kick the Buddy | Crosses image decode and reaches missing `+[NSMutableSet setWithSet:]`. | Safe general Foundation factory/copy contract. |
| Labyrinth 2 | Crosses visible-cell lookup and reaches missing invocation-backed scheduled timer. | Safe general timer contract with run-loop/ownership tests. |
| Mini Militia | Still wrong orientation/offline; teardown reaches missing `GKMatchmaker`. | Dropped from routine work at user's request; preserve for explicit GameKit work only. |
| Muffin Knight | Two honest offline connects fail, then main thread loops at `0x015b3b50..0x015b3be0`; Home cannot be handled. | Offline symbolization first; no fake network or watchdog. |
| Sonic Dash | Runs but is severely corrupted after ES2 creation returns nil and an ES1 fallback renders. | Separate high-cost ES2/fallback investigation; no small safe patch established. |
| Trainyard | Mutable-data initializer now hits strict typed-dispatch mismatch; bundle DB also receives denied writable open. | Correct method ABI/registration first, then reassess DB path. |
| ZombieSmash | XML cancellation crosses; later bitmap creation passes caller storage with zero row bytes. | Research historical contract and preserve buffer safety before patching. |
| Spoing | Working control passes. | Retain only when relevant to changed subsystems. |

This result is evidence and planning only. It does not authorize or claim a
subsequent compatibility implementation, commit, push or IPA.

## New-game encryption inventory — November 4, 2026

Seven supplied evidence files from Dev 2.26 implementation
`eeb7775480cb8bd7b76089fdcdf64d9f16db1b3f` were indexed without performing a
runtime compatibility analysis. Across 70 unique titles, ten packages are
explicitly encrypted, one (Burrito Bison) is explicitly reported as badly
decrypted, and 59 show no encryption blocker. Repeated launches and the later
Legends of Ooo confirmation were de-duplicated by bundle identifier.

The exact title/package mapping, evidence hashes and the user's deferred visual
observations are preserved in
`new-game-encryption-survey-2026-11-04.md`. The preceding Dev 2.26 compatibility
result remains the current analyzed compatibility checkpoint. No emulator code,
commit, push, or IPA build is authorized or produced by this inventory.

## Replacement-package runtime triage — November 4, 2026

The combined Dev 2.26 log at implementation `eeb7775480cb8bd7b76089fdcdf64d9f16db1b3f`
materially revises three package-only conclusions. A decrypted JellyCar 1.5.4
reaches an autorelease-pool use-after-deallocation at a reused `NSNumber`
address; a decrypted Sky Burger 1.2.2 reaches missing `UIActionSheet`; and a
decrypted full-version TowerBloxx 2.0.26 reaches missing
`UIPickerView.dataSource` during nib outlet connection. Nyan Cat ran in the
observed scope and is excluded from further repair work at the user's request.

The survey also establishes a prioritized cross-library queue. Exact
Foundation/POSIX contracts are first, legacy UIKit controls second, bounded
autorelease/lifetime provenance third, followed by audio conversion,
graphics-state capture, null-producer tracing, and CPU/host/service evidence.
The complete title mapping, repeated-versus-new boundaries, predictions, and
external/package exclusions are in `new-game-runtime-triage-2026-11-04.md`.
This was analysis and documentation only; no source implementation or IPA was
created.

## Dev 2.27 authorized exact-Foundation batch — November 4, 2026

The user authorized the complete fourteen-title batch rather than a smaller
single-contract build. Every requested title maps to changed compatibility
code; BikeRacePro and Legends of Ooo share file-manager URL enumeration, while
Carnival and Wire Way share correct short-read behavior. The remaining ten
titles each exercise a distinct general Foundation/Core Foundation/UIKit/libc
contract. No bundle-identifier branch, fake service success, relaxed memory
safety or crash suppression is included.

The exact contract, implementation boundary, prediction and falsifier for each
title are preserved in `dev-2.27-exact-foundation-contracts.md`. Source and host
tests are implementation evidence only. The ledger must not promote any title
until the verified artifact is exercised on the physical device.

## Dev 2.27 physical-device result — November 4, 2026

Three returned logs verify implementation `4b4b3bd8abd91aa1bd65153624ed8071bd40be11`
and workflow `41837856742`; the universal recorder was active. Every Dev 2.27
contract crossed its former first blocker. Wire Way is the meaningful functional
advance: gameplay and input were smooth for about 124 seconds, with 10,070 main
loops and 3,679 presentations, but its pause and post-level states are black
despite continued rendering and delivered touches. It is therefore partial,
not fully compatible.

BikeRacePro and Legends of Ooo now share missing
`NSURL URLByAppendingPathComponent:`; MX Mayhem reaches missing recursive
directory enumeration; ParaPanic HD reaches real SHA-224; Dismount reaches
dynamic `printf` width/precision; The Sims 3 reaches `CGContextBeginPath`;
100 Chambers reaches block-backed operation queues; and Carnival reaches the
Darwin notification center. Those are later general contracts, not repetitions
of the old blockers.

Fieldrunners, NinJump, Scribblenauts and Super Hexagon now end in strict null or
invalid-value consumption whose origin is not yet proven. Fieldrunners 2 enters
a guest initialization wait: six threads continue running but the application
main loop never starts, no frame presents, and queued Home events cannot be
handled. These require bounded provenance/thread diagnostics rather than relaxed
memory behavior. Super Hexagon's valid IPA also has an empty display name, so
the title-based front pin missed it; bundle-identifier/filename fallback is the
correct picker fix.

Complete evidence hashes, per-title boundaries and the recommended next batch
are preserved in `dev-2.27-device-result.md`. This review updates evidence only;
it does not authorize a new implementation, push or IPA.

## Dev 2.28 locked 27-title implementation — November 4, 2026

The user hard-locked the complete batch. Fourteen later general contracts are
implemented for 100 Chambers, Bike Race Pro, Cannon Cadets, Carnival, Dismount,
Kick the Buddy, Labyrinth 2, Legends of Ooo, MX Mayhem, ParaPanic HD, Perfect
Cell, The Sims 3, Trainyard and Trism. Twelve unresolved titles receive bounded
nil-producer, autorelease-enqueue, input, presentation or invocation evidence:
Clowning Around, Current, Fieldrunners, Fieldrunners 2, JellyCar 1.5.4, NinJump,
Peggle, Scribblenauts, SUPERFALL, Super Hexagon, Tiki Totems 2 and Wire Way.
Spoing is the only unchanged regression control.

Exact implementation boundaries, per-title predictions and safety limits are
in `dev-2.28-locked-27-contracts-and-provenance.md`. Source/tests/build evidence
must remain separate from the forthcoming physical-device result.

## Dev 2.28 physical-device result — November 4, 2026

Four supplied logs verify implementation
`5f421ee519024ee137ace6e70e54f4df03d79400` and workflow `41862962461`; the
universal recorder and bounded Dev 2.28 provenance were active. All fourteen
direct compatibility contracts crossed their former boundary, but most expose
a later independent missing contract or ownership failure rather than becoming
fully compatible.

Carnival runs but its presentation rate declines under load and many AIFC
music/effect files fail real decoding. Labyrinth 2 runs but has collision/motion
and menu corruption. Current receives touches at the EAGL view and keeps
presenting near 59 Hz, so its inert menu is downstream of basic host input
delivery. Fieldrunners 2 and Trism stop servicing the application event loop,
which explains why even Home is unresponsive. Wire Way's post-level black state
is actively rendered by the guest with a complete framebuffer and a full-screen
black draw; it is not a frozen compositor.

The exact evidence hashes, every title's later boundary, compatibility status
and next discriminating action are preserved in
`dev-2.28-device-result.md`. This result updates evidence only and does not
authorize a source change, commit, push or IPA build.

## Dev 2.29 physical-device result — November 4, 2026

Two supplied logs verify implementation
`94b8d061cc8563a8fbe7e07803cb97fb5fa6f924` and workflow
`41899732869`; the permanent recorder and focused Dev 2.29 diagnostics were
active. Perfect Cell and Trainyard appear working in the tested scope. Dismount
now reaches gameplay but averages only about 15.4 presentations per second, so
it remains partial rather than fully compatible.

The pass materially clarifies several unresolved titles. Current delivers all
recorded taps to its interactive EAGL view and presents near 58 Hz, placing the
failure in later guest touch/menu dispatch. Kick the Buddy presents near 54 Hz
and audibly reacts to blind taps beneath an all-black result, making this a
rendering/content-state defect rather than input loss or a freeze. Fieldrunners
2 has six blocked/sleeping guest threads, zero main-loop iterations and zero
presentations; queued Home requests cannot run because initialization is in a
guest-wide wait. Wire Way continues to present and deliberately draws opaque
black after level completion; the enlarged rolling selector history was
overwritten by later presentation/layout traffic and did not retain the
decisive fade/audio-stop callback.

Carnival's missing music and sparse effects are now attributed to 119 honest
decode failures for IMA4-compressed AIFC, which Dev 2.29's uncompressed AIFC
support does not cover. Its separate slowdown remains unexplained and is not
proved audio-related. Labyrinth 2's wall-crossing defect was reproduced only
once; one launch also reveals missing `UIActivityIndicatorView.hidesWhenStopped`,
while a restored launch runs without a panic. Trism reaches gameplay and then
stops at missing `_CGContextClosePath`. Bike Race Pro reaches mutable-dictionary
URL writing, ParaPanic HD reaches `NSProcessInfo.processIdentifier`, The Sims 3
reaches Clear blend mode, and Legends of Ooo reaches absent
`NSManagedObjectModel`. Cannon Cadets, MX Mayhem and 100 Chambers remain strict
graphics, lifetime and KVC-origin investigations respectively.

Complete hashes, per-title evidence, compatibility classification and preserved
next actions are in `dev-2.29-device-result.md`. This checkpoint updates
documentation only. It does not authorize or contain another implementation,
commit, push, workflow or IPA.

## Dev 2.31 physical-device result — November 5, 2026

Three supplied logs verify implementation
`62aefdd42441ead701cf79e72f8589fc965ac8b3` and workflow
`41968860889`. Bike Race Pro, Labyrinth 2, Legends of Ooo, ParaPanic HD, The
Sims 3 and Trism crossed their intended Dev 2.31 contract, but each exposed a
later independent exception, null producer, model-loading requirement,
graphics error or guest stall rather than becoming fully compatible.

Carnival's implemented IMA4 path did not decode the real package because its
AIFC `COMM` records use a zero sample-size field while the new code accepts
only 16. This falsifies the fixture assumption, but historical/reference
behavior must be checked before accepting zero. The user still hears almost no
audio and observes sharp intermittent slowdown. Dismount remains interactive
but averages about 15.6 presentations per second.

Current still presents near 58.5 Hz and receives touches without guest-visible
response. Kick the Buddy presents near 54.6 Hz and occasionally reacts audibly
beneath black output. Fieldrunners 2 remains a silent pre-presentation guest
wait, but the recurring blocked PCs now include instruction bytes suitable for
offline symbolization. Wire Way remains active around 29 Hz after its pause or
level-complete transition turns permanently black and silent.

The full evidence hashes, per-title results, compatibility classifications and
bounded next actions are preserved in `dev-2.31-device-result.md`. This is a
documentation checkpoint only; no new source change, commit, push, workflow or
IPA is authorized.

## Dev 2.32 physical-device result — November 5, 2026

Two supplied logs verify implementation
`9d568fc7526acc08f4ce6ee9185ca8fbff5e2cea` and workflow
`41990584956`. Highway Rider now reaches responsive gameplay, but its road and
substantial scene content are missing, leaving gray geometry and invisible
collision hazards. iStunt Reloaded Pro crosses its former plain-`UIView`
producer: a real `UITableView` is present and level packs open, but level
buttons are absent and repeated interaction later stops reaching the guest in
a useful way. These are genuine advances, not functional compatibility.

Sky Burger reaches its name prompt and then closes the entire iDared host when
the prompt is tapped. Its log ends without an emulator panic or normal return,
after repeated rejected frees of the same unknown address; a matching native
iOS `.ips` report is required before making an emulator change. TowerBloxx
reaches actual gameplay, but native Home produces a guest deadlock followed by
autorelease/coroutine teardown panics. Tiki Totems Premium remains fully working
in the user's tested scope and returns cleanly through Home.

The remaining logs expose later exact boundaries: 1112 ep.01 reaches UIKit
line-break mode `2`; Junk Jack Retro reaches missing
`UITapGestureRecognizer`; both Real Racing packages reach
`MPMediaPickerController initWithMediaTypes:`; Stick Stunt Biker still reaches
an ASCII-only string assertion through a path not covered by the new legacy
encoding contract; and Doodle Grub decodes a string where the game expects a
color, requiring archive/save-origin analysis rather than relaxed dispatch.

Complete evidence hashes, per-title classifications and preserved next actions
are in `dev-2.32-device-result.md`. This checkpoint changes documentation only;
it does not authorize or contain another implementation, push, workflow or IPA.

## Dev 2.33 five-contract and bounded-diagnostic batch — November 5, 2026

The user authorized the maximum coherent cross-ledger scope: five direct
correction titles, all ten evidence-backed diagnostic experiments, and two
relevant working controls. The five correction titles map to three general
contracts: `UILineBreakModeClip` for 1112 ep.01, tap/pan gesture recognizers
for Junk Jack Retro and BouncyMouse, and media-picker construction/property
state for R.Racing GTI and Real Racing. The media-picker work is explicitly a
partial service contract: it does not fabricate host music access or a selected
item.

The diagnostic titles receive bounded archive-key/result evidence (Doodle
Grub), table reload/row/selection evidence (Bungee Ball and iStunt), expanded
texture upload/copy history (Highway Rider, Kick the Buddy and Flip's Escape),
and suspicious duplicate-autorelease provenance (JellyCar 1.5.4,
MX Mayhem, SUPERFALL and TowerBloxx). CrazyGolf and Tiki Totems Premium are
unchanged controls, not disguised compatibility attempts.

Exact predictions, falsifiers, implementation limits and the full seventeen
title test scope are preserved in `dev-2.33-contracts-and-diagnostics.md`.
Local compile/targeted tests and hosted build evidence remain distinct from
physical-device compatibility; no title is promoted by source work alone.
