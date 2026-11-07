# iDared combined Development and Level 3 diagnostic pass

Status: Dev 2.47 implementation candidate. Hosted and package verification are recorded separately after they complete.

## Source and evidence baseline

- Active checkout: `worktrees/idared-phase2-cold-start`
- Branch: `phase2/persistent-cold-start-index`
- Implementation baseline: `530178895ceaa546768e6ef0deb604a0369cf578` (Dev 2.46)
- Evidence source: Level 3 campaign through Batch 012, imported under the separate evidence checkout at `repo/dev-docs/ipa-evidence/level3-campaign/`
- Archive verification: the Batch 012 transfer manifest covered 91 payload files and 2,816,151 bytes; every declared SHA-256 and JSON parse check passed. Existing `README.md` and `campaign-state.json` differed, so the prior copies were preserved under `import-backups/pre-batch-012-2026-11-07/` before the updated records were imported.

The diagnostic banner records the campaign mode, compiled `crate::VERSION`, bundle identifier, and bundle version. Every bounded campaign event records elapsed milliseconds, emulated thread, guest PC/LR where available, and the current presentation count. The common ring contains the newest 192 campaign events; object, API, callback, caller, presentation, and watched-write histories retain their existing independent bounds.

## Complete combined scope

The 15-title table below is the Batch 012 Level 3 expansion, not the complete physical campaign. The complete picker-front cohort contains **34 exact applications**:

- **23 diagnostic targets:** the 15 Level 3 expansion titles plus eight non-overlapping Development follow-ups from the Dev 2.46 assessment: 100 Floors, A Monster Ate My Homework, AC Rearmed, Air Penguin, Dynamite Jack, Implode!, iShoot 2, and Pang. Forlorn, S.T. ActionPig and Wispin were already present in both lists and are counted once.
- **11 inexpensive retest-only applications:** Current, Fieldrunners 2, Flip's Escape, Highway Rider, iStunt Reloaded Pro, JellyCar 1.5.4, Real Racing, Real Racing GTI, SUPERFALL, TowerBloxx and Wire Way. These retain the normal bounded recorder; their presence does not imply new app-specific instrumentation. Real Racing GTI is kept with Real Racing because the historical media-picker evidence and test history treat the exact packages as a pair.

All 34 are promoted to the picker front in the alphabetical order shown in the run sheet. This separation matters: a diagnostic target is expected to produce new provenance, whereas a retest-only title answers whether current shared runtime work has changed an older outcome.

## Reconciliation

| Game / exact package | Latest physical boundary | Current-source reconciliation | Diagnostic family and enablement | Historical control |
|---|---|---|---|---|
| Cannon Cadets 2.3.6 — IPA-048, `Cannon Cadets - Below IOS 7.ipa`, `9db47c2e838137e7e2ca13dea07a7de1dd5733d312ea04c623a24f463468121e` | Black output followed by strict bitmap-context rejection. | The exact bitmap layout remains unknown; global validation must not be weakened. | `bitmap-context`: passive campaign logs caller PC/LR plus all `CGBitmapContextCreate` arguments before validation. Auto-enabled for `com.xmgstudio.cannoncadets`. | No |
| Forlorn 1.9c — IPA-174, `f953e14876c586824223eb44af5fac71.ipa`, `cdfb3d117d244272288cde06344502ffa119e56075f4c45762bbd564ec3df53d` | Dev 2.46 displays corrupted graphics and crashes; earlier logs expose compositor error `0x506`. | Nearby guest GL calls do not establish the internal error's origin. | `internal-compositor`: first nonzero internal-context error with stage, layer, operation boundary, bindings, framebuffer status, active texture, viewport, and recent bounded history. Auto-enabled. | No |
| Gun Bros 1.2.0 — IPA-206, `Gun Bros 1.2.0 XGA.ipa`, `6e62b6a0fb308ad2436f7ddf1b40c9b5d1f22ba743410f1ffefc436bf506ba9b` | GLU Error 1; guest does not provide a usable return path. | Current source already exposes media-picker classes, so the old missing-class interpretation is not trustworthy without the exact lookup. | `class-lookup`: exact `NSClassFromString` request/result, pre-registration state, caller/thread context, and following guest control-flow history. Auto-enabled. | No |
| Jelly Invaders 1.32 — IPA-223, `Jelly Invaders - Below IOS 7.ipa`, `60dbf235987bcd989ddacb2bc1b29afbbd40f5dad03c9c183ffc7c89691f8663` | Menu works briefly; video and audio disappear, with EAGL presentations ending around 129. | Presentations stop while other work continues; static evidence cannot identify whether rendering or scheduling stops first. | `presentation-cessation`: bounded EAGL presentation, callback, run-loop, audio, application-state, thread, and caller histories identify the last successful presentation and first missing interval. Auto-enabled. | No |
| Legendary Wars 1.33 — IPA-237, `Legendary.Wars v1.33.ipa`, `bb62bb84d474767afc0ccce9583daa1dd4f40f3e6105726ae423879092bf454c` | Reaches one frame, then KVC mutates a nested `Units.plist` dictionary that is immutable in iDared. | Emulator-side provenance can be captured, but historical Apple nested-mutability behavior is not established. No mutability correction is included. | `collection-provenance`: existing plist/KVC/dynamic-class/object histories plus passive campaign identity. Run from a clean container if one is available without deleting user data. | **Yes** — matching historical plist/KVC control required. |
| Monster Hunter 1.03.00 — IPA-253, `Monster Hunter - Below IOS 7.ipa`, `29e6ec59cf845001aff0f432e81538f1a423094a5cd34478a1e596de3f7d8d03` | Startup null dereference in `-[ES1Renderer render]`; global slot `0x00334f8c` is null. | Drawable creation succeeds. The missing initializer event, not a fabricated singleton, must be found. | `engine-init`: watches every overlapping write to `0x00334f8c`; logs raw store-time PC/LR/thread/value, launch callbacks, display-link creation/scheduling/firing, and presentation history. Auto-enabled; direct guest data access is disabled only for this watched-address run. | No |
| QuickHook 1.20 — IPA-340, `Super QuickHook - Below IOS 7.ipa`, `486f4740aae9555119d418e02f1c358e4001cd0d444f6ee09f3ac74a56c735ca` | Startup crashes while mutating nested data loaded from defaults key `qh/items`. | The producer is persisted `NSUserDefaults`, distinct from Legendary Wars' plist producer. Historical nested mutability after a defaults round trip is unresolved. | `defaults-provenance`: existing defaults decode, dynamic-class, dictionary mutation, object, and exception histories. Test first launch and persisted relaunch using the same container. | **Yes** — matching historical defaults round-trip control required. |
| ShadowGun 1.1.3 — IPA-308, `ShadowGun (iOS).ipa`, `3b1fe08416fca355930733b6f8dc8a280c4df62ff12aace88975f5e5473127a9` | Immediate null-page read. | The old log lacks a trustworthy guest region or producer. Current memory-fault snapshots already provide full registers, code bytes, stack, import/global reconstruction and caller boundaries; the campaign adds bounded event provenance. | `guest-null-fault`: passive campaign plus existing crash/control-flow/API/callback/caller histories. Auto-enabled. | No |
| Silent Swords Zero 1.1 — IPA-310, `Silent Swords Zero - Below IOS 7.ipa`, `07354b59e07a00fd61c550d9bd96f872f9e3d83f4728f9e2b93f5d258a1a8987` | Gameplay reaches enemy interaction, then sends `hash` to a stale object. | The exact filename/key generation in `play:loop:` is still missing. | `focused-object-lifetime`: pins the `play:loop:` filename argument so bounded create/autorelease/retain/release/dictionary/dispatch histories survive until failure. Auto-enabled. | No |
| S.T.ActionPig 1.1.1 — IPA-101, `com.joystickgenerals.STActionPig-iOS2.2.1-(Clutch-2.0.4) 2.ipa`, `89e11c7791f6c66c504c361a816edd650ebbeccf70d051a95b12f3a3dff7f37e` | Immediate crash; compositor later observes `0x506`. | Guest GL calls near the panic do not prove a guest-origin error. | `internal-compositor`, shared with Forlorn and Wispin. Auto-enabled. | No |
| Swordigo 1.1 — IPA-345, `Swordigo-v1.1.ipa`, `76d1d01e9422e27ec9026bb556c03850dac4ab88e1a99407f33d6aaffaa8dd0c` | Immediate failure because the guest probes `[UIDocument class]`. | Dev 2.46 has no coherent `UIDocument` implementation. Passive lookup provenance remains useful; an empty class is not a document implementation. | Run 1: passive `class-lookup`. Run 2: explicitly select `Swordigo UIDocument` in Quick Options; this registers only an empty NSObject-derived class and is logged as an intervention. No cloud/ubiquity/document behavior is claimed. | No, but paired baseline/intervention runs are mandatory. |
| Trench Run 2.1 — IPA-137, `com.universomo.TrenchBuild-iOS3.0-(Clutch-2.0.4).ipa`, `775fc62ce9f1a95ab25354183132cb4b0c9e60959a867981b1fd171165a537f4` | Menus work; selected gameplay stays black with a thin border and no input or sound while presentations continue. | A black guest image must be separated from failed host presentation. | `unity-presentation`: first 160 presentations include context/FBO/viewport/rotation and a bounded center 4x4 RGBA hash/nonzero signature, correlated with callbacks, audio and input histories. Auto-enabled. | No |
| Wispin 1.1.2 — IPA-369, `Wispin_v112_os40.ipa`, `be0f4cc430a0340d8afe60f37c52dac4fb81384ef6e2316d97067535fa2cccad` | Immediate crash; compositor later observes `0x506`. | Same localization gap as ActionPig/Forlorn, without proof of one shared guest defect. | `internal-compositor`, shared facility but separately attributed context/history. Auto-enabled. | No |
| Zaxxon 1.0.0 — IPA-128, `com.sega.zaxxonescape_1.0.0_ios-4.0_zaxxon_4985615b.ipa`, `700e06c735ce3f7c3e21a532051ef4655cd5b09045bed25a1e8ef6454915a805` | Menu is reached, then `maybe_initialize_class` encounters no host object. | Nearby handled CoreMotion warnings are not treated as causal. | `class-host-map`: receiver, `isa`, resolved class name, PC/LR/thread and bounded object/class registration/removal history. Auto-enabled. | No |
| Zuma HD 1.1 — IPA-377, `Zuma HD-(com.popcap.ios.ZumaHD)-1.1-(iOS_3.2)-5e12609e18446fe76f75891dfadc09d7.ipa`, `3fc9777ab99905164b6045217b9fef27f15ffdcbb786b24005cc3be3dcd52794` | Immediate null-page read. | As with ShadowGun, current evidence cannot attribute nil to emulator or application. | `guest-null-fault`: passive campaign plus bounded crash/control-flow/API/callback/caller histories. Auto-enabled. | No |

## What the pass can and cannot prove

The pass is observational except for the explicitly selected Swordigo experiment. It does not implement `UIDocument`, loosen bitmap validation, force nested collections mutable, fabricate Monster Hunter initialization, or attribute internal-compositor errors to guest GL calls. The general memory-fault snapshot decodes the relevant bytes and records complete registers, but it is not an instruction-by-instruction guest branch tracer; a later Level 3 pass may still need to reconstruct a producer from the bounded history and captured code. Likewise, Jelly Invaders' exact guest branch that stops presenting can only be named if it crosses an observed callback/API boundary.

## Physical run sheet

The 23 diagnostic targets inherit `--diagnostic-campaign=level3-provenance` from exact bundle defaults except that Swordigo's intervention must be selected manually. The 11 retest-only titles use the normal bounded recorder. One diagnostic build can test the full list. Do not delete an existing save/container merely to create a clean run.

| Order | Game | Required action | Visible observation to report |
|---:|---|---|---|
| 1 | 100 Floors | Enter a puzzle, exercise the white regions, then return/exit. | Photograph the menu and level if the white rectangles remain; note whether labels/content appear and whether exit fails. |
| 2 | A Monster Ate My Homework | Launch through the loading frame until failure. | Last frame, sound, timing and whether the null-context failure visibly changed. |
| 3 | AC Rearmed | Launch through the loading screen until failure. | Last frame and whether it advances beyond the prior class-initialization boundary. |
| 4 | Air Penguin | Launch through logos/loading until failure. | Last frame, audio and whether the regression stage changes. |
| 5 | Cannon Cadets | Launch and attempt to start gameplay until black output or failure. | Last visible frame and whether any frame is presented before failure. |
| 6 | Current *(retest only)* | Exercise the menu/input path for at least 10 seconds. | Whether touches now produce a guest-visible response while frames continue. |
| 7 | Dynamite Jack | Wait for music, tap the black surface once in the center and once elsewhere, then return from any App Store page. | Whether any game frame appears and which regions trigger the external page. |
| 8 | Fieldrunners 2 *(retest only)* | Launch and wait at least 15 seconds; try Home once. | Whether a frame/main loop appears and whether Home can be serviced. |
| 9 | Flip's Escape *(retest only)* | Launch and attempt normal input/gameplay. | Screenshot the first corrupt scene and note whether input produces any useful response. |
| 10 | Forlorn | Launch through the corrupted graphic and crash. | First corrupt frame, whether it changes, and crash timing. |
| 11 | Gun Bros | Launch until the GLU error or another boundary. | Exact error text, whether the picker exit control responds, and any new screen. |
| 12 | Highway Rider *(retest only)* | Start gameplay and drive long enough to encounter scenery/collisions. | Whether road/environment graphics remain absent and invisible collisions remain. |
| 13 | Implode! | Perform one deliberately slow bomb drag and one fast drag, then place/detonate if possible. | Exact point where a drag locks/snaps and whether later touches inherit the stale touch. |
| 14 | iShoot 2 | Launch once; repeat only if it silently exits without a complete marker. | Last frame/resource and whether guest `_exit(1)` still ends the launch. |
| 15 | iStunt Reloaded Pro *(retest only)* | Open several level packs and tap expected level-button positions, then attempt to return. | Whether level buttons/rows appear and whether touch becomes unresponsive. |
| 16 | Jelly Invaders | Enter the menu/game and wait at least 15 seconds after video/audio disappear. | Whether audio stops with video, whether touch responds, and whether the app exits itself. |
| 17 | JellyCar 1.5.4 *(retest only)* | Launch and attempt to reach gameplay. | Black-screen duration and exact crash/return behavior. |
| 18 | Legendary Wars | Use a clean container only if naturally available; otherwise record existing-container state. Reach the `Units.plist`/KVC failure. | First frame, menu reach, and exact action before failure. A separate historical-iOS control is still required. |
| 19 | Monster Hunter | Launch and wait through startup without forcing input. | Last frame, any animation/audio, and delay to failure. |
| 20 | Pang | Play through at least one level, then use iDared's return control. | Gameplay speed and whether exit reproduces the stale AVAudioPlayer failure. |
| 21 | QuickHook | Run once in the available container, relaunch without clearing it, and repeat the same path. | Whether first and persisted runs differ. Record whether the container was genuinely new. Historical-iOS control remains required. |
| 22 | Real Racing *(retest only)* | Launch through logo/startup and wait for the next state. | Last frame, black-screen duration and crash/return behavior. |
| 23 | Real Racing GTI *(retest only)* | Repeat the same startup path as Real Racing. | Whether its boundary still matches the related package. |
| 24 | ShadowGun | Launch once until the null fault. | Last visible frame/audio and delay to failure. |
| 25 | Silent Swords Zero | Play until the enemy/attack/sound action that previously crashed. | Exact action and sound immediately before failure. |
| 26 | S.T.ActionPig | Launch until the compositor failure. | First visible frame and any corruption before failure. |
| 27 | SUPERFALL *(retest only)* | Play through a death/ending if practical, then return through iDared. | Gameplay stability and whether death or Home still triggers the lifetime/teardown failure. |
| 28 | Swordigo | First launch with App default/passive. Then choose `Swordigo UIDocument` in Quick Options and launch again; change no other option. | Compare the exact last screen/branch in both runs. The second run is an intervention, not a compatibility result. |
| 29 | TowerBloxx *(retest only)* | Reach gameplay, then exit through iDared. | Whether gameplay remains functional and whether Home/teardown still deadlocks or crashes. |
| 30 | Trench Run | Select a level, wait through the first black gameplay presentation, then try input for 10 seconds. | Whether audio/input/game state continues and whether black is full-frame or bordered. |
| 31 | Wire Way *(retest only)* | Play through pause or level completion. | Whether the scene still turns black/silent while presentation continues. |
| 32 | Wispin | Launch until failure. | First visible frame and corruption timing. |
| 33 | Zaxxon | Launch, enter the menu, and select the path that crashes. | Exact selection and last responsive screen. |
| 34 | Zuma HD | Launch once until the null fault. | Last visible frame/audio and delay to failure. |

The matching log must be retained in full. Level 3 should search for `Level 3 diagnostic campaign:`, `level3 campaign #`, `level3 guest-write #`, `Bounded internal-compositor GL diagnostic:`, and the existing diagnostic dump sections.
