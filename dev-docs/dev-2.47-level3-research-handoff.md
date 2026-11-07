# Level 3 handoff — provenance diagnostic pass after Dev 2.46

## Build/source identity

- Development baseline: `phase2/persistent-cold-start-index` at `530178895ceaa546768e6ef0deb604a0369cf578`.
- Local implementation is uncommitted and unbuilt. Replace this statement with the exact candidate commit and Dev label if a diagnostic IPA is later authorized.
- Each diagnostic log begins with `Level 3 diagnostic campaign:` and embeds the compiled version, package identifier/version, campaign mode, and passive/intervention status.

## Added diagnostic families

The physical campaign is broader than the Batch 012 expansion: it contains 23 diagnostic targets and 11 retest-only applications, 34 unique titles total. The 15 package profiles below the imported research handoff are only the new Level 3 queue. The eight additional Development diagnostic targets are 100 Floors, A Monster Ate My Homework, AC Rearmed, Air Penguin, Dynamite Jack, Implode!, iShoot 2 and Pang; the shared Forlorn/ActionPig/Wispin targets are de-duplicated. The exact retest-only cohort and actions are in the run sheet.

- Shared 192-event Level 3 campaign ring with timestamps, emulated thread, guest PC/LR and presentation count.
- Exact watched-write history (128 entries), including raw store-time guest PC/LR, used for Monster Hunter address `0x00334f8c`.
- `CADisplayLink` creation, scheduling, firing and paused-state events plus application-launch callback entry/return.
- Pre-validation `CGBitmapContextCreate` argument capture for Cannon Cadets.
- `NSClassFromString` request/pre-registration/result provenance for Gun Bros and Swordigo.
- Missing host-object receiver/`isa`/class provenance at `maybe_initialize_class` for Zaxxon.
- Focused filename-object lifetime pinning at Silent Swords Zero's `play:loop:` boundary.
- Internal-compositor first-error state for ActionPig, Forlorn and Wispin, separated from guest context activity.
- EAGL presentation state for all campaign titles and a bounded 4x4 center-frame signature for Trench Run's first 160 presentations.
- Existing bounded fault, registers/code/stack, Objective-C dispatch, object lifetime, collection, defaults, callback, run-loop, audio, and caller histories remain active and are dumped with the campaign records.

## Changed or rejected proposals

- Swordigo's top-level bookkeeping mismatch was resolved in favor of a paired experiment. The default run is passive. An empty NSObject-derived `UIDocument` is available only through the explicitly selected `swordigo-uidocument` intervention. It is not enabled globally and supplies no document, file-coordination, cloud, or ubiquity behavior.
- Legendary Wars and QuickHook received provenance only. Their proposed mutability corrections remain rejected until a matching historical iOS control establishes nested collection behavior for plist loading and `NSUserDefaults` round trips respectively.
- Cannon Cadets receives argument capture, not relaxed bitmap validation.
- Monster Hunter receives an exact global-write watch and display-link/launch chronology; no singleton is fabricated and no callback is forced.
- ShadowGun and Zuma HD reuse the stronger current memory-fault snapshot plus campaign history. No unbounded instruction tracer was added. Static Level 3 must first try the captured registers, code bytes, imports/globals, stack, callbacks and caller/API rings.
- Jelly Invaders and Trench Run use bounded shared presentation/callback evidence rather than game-specific behavioral changes.

## Markers for research

- `Level 3 diagnostic campaign:` — source/build/package/configuration banner.
- `level3 campaign #` — normalized campaign event with time/thread/PC/LR/presentation.
- `level3 guest-write #` — exact watched guest store with raw store-time PC/LR.
- `Bounded internal-compositor GL diagnostic:` — first internal compositor GL failure and captured bindings/state.
- Family names: `bitmap-context`, `class-lookup`, `class-host-map`, `display-link`, `application-launch`, `eagl-presentation`, `unity-frame-signature`, `campaign-passive`, `campaign-intervention`.
- Existing recorder dump headings retain the object, Objective-C, API, callback, caller, presentation, run-loop, resource and collection provenance needed to interpret these events.

## Remaining external dependencies

- Legendary Wars: native historical plist loading plus `setValue:forKeyPath:` control.
- QuickHook: native historical `NSUserDefaults` serialize/relaunch/deserialize/mutate control using `qh/items`.
- Swordigo: both passive and intervention physical runs are needed; progress under intervention would establish only that the availability branch changed.

## Required return evidence

Return the complete untrimmed diagnostic log, the precise package run order, the visible observation for each title, container state for Legendary Wars/QuickHook, and both Swordigo runs. The machine-readable configuration and expected evidence fields are in `dev-docs/dev-2.47-level3-diagnostic-map.json`; the complete run sheet and exact hashes are in `dev-docs/dev-2.47-level3-diagnostic-pass.md`.
