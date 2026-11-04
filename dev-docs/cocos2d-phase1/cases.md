# Per-game evidence and discriminating next steps

Baseline: Dev 2.23, embedded `51a6334`, workflow `41778866142`. Absolute original log lines are cited; archived originals retain their line numbering. Present-return counts show execution, not proven correct pixels or playable gameplay. The source snapshot is reviewed but none of the proposed changes below is implemented in this phase.

## AbstractWar 2.0

- Identity: `com.manucorporat.abstractwar2.0`; version `2.5` / bundle build `2.5`; minimum OS `3.0`; armv6, armv7.
- Local IPA: `AbstractWar - Below IOS 7.ipa`. SHA-256 `7db175afb68339bfeba58f7972b99eacaf72f9eda9e972f83018a42ab990978d`. ZIP CRC valid; encrypted=False.
- Device source: [1-Log.txt](logs/1-Log.txt), lines 44–518; first panic line 234. Log name `AW 2.0`; launched archive `AbstractWar - Below IOS 7.ipa`.
- Engine: confirmed cocos class family; CCSpriteSheet / pre-CCDirectorIOS generation; exact version unknown. Dependencies: CocosDenshion, OpenFeint, Plus+, OpenAL.
- Outcome: **panic**; confidence: confirmed. Role: primary. Runtime ES1.1=True; observed EAGL-return count=0; Home requested=False.
- Signature: **C2D-ALPHA-01** — CGBitmapContextCreate borrows a null CGColorSpace.

Recorder call #2697: data=0x3000a000, 128x128, 8 bits/component, rowBytes=128, colorSpace=NULL, bitmapInfo=7 (alpha-only). Guest LR 0x677cf lies in the supplied symbol interval starting at 0x676e4, -[CCTexture2D(Image) initPremultipliedATextureWithImage:pixelsWide:pixelsHigh:]. The source unconditionally dereferences colorSpace. This is a legal alpha-only bitmap, not an arbitrary bad pointer.

Next action / prediction: Implement a genuine one-channel alpha context only after tests cover image drawing, row stride, alpha extraction, context queries, ownership and text/mask consumers. Prediction: texture initialization crosses this call; later gameplay remains unproven. A non-alpha format at the same failure would refute this specific diagnosis.

Source investigation: src/frameworks/core_graphics/cg_bitmap_context.rs; src/frameworks/core_graphics/cg_context.rs; src/frameworks/core_graphics/cg_image.rs.

```text
Panic at src/objc/objects.rs:298:13: No host object registered for (null) while borrowing "touchHLE::frameworks::core_graphics::cg_color_space::CGColorSpaceHostObject"
```

Resources (counts by filename extension; not a decoder-validity claim): .fnt=7, .m4a=1, .nib=130, .plist=7, .png=273, .pvr=3, .ttf=1, .wav=25.

## Addicus

- Identity: `com.getsetgames.addicus`; version `1.1.3` / bundle build `1.1.3`; minimum OS `3.0`; armv6.
- Local IPA: `Addicus.ipa`. SHA-256 `077119ba1351f785fc1c43b2d77406d7020f1e250a17da260a937272a47ef99b`. ZIP CRC valid; encrypted=False.
- Device source: [1-Log.txt](logs/1-Log.txt), lines 519–1460; first panic line 720. Log name `Addicus`; launched archive `Addicus.ipa`.
- Engine: confirmed cocos class family; CCSpriteSheet / pre-CCDirectorIOS generation; exact version unknown. Dependencies: CocosDenshion, OpenFeint, Plus+, GoogleAnalytics, OpenAL.
- Outcome: **panic after presentation**; confidence: confirmed boundary; probable upstream cause. Role: primary. Runtime ES1.1=True; observed EAGL-return count=53; Home requested=False.
- Signature: **C2D-STREAM-01** — Missing CFReadStreamScheduleWithRunLoop after NULL socket-stream pair.

Both runs enter GANTCPSocket / Google Analytics and request www.google-analytics.com:80. CFStreamCreatePairWithSocketToCFHost returns NULL streams before the missing scheduling export. Cocos initialization and EAGL returns already occur. The shared producer is a third-party analytics transport, not cocos rendering. A nil scheduling no-op alone would merely expose the next call.

Next action / prediction: Trace the finite create/client/schedule/open/error/unschedule/close chain already in the recorder; implement truthful failed network streams with correct callback-context ownership and run-loop delivery, or document the historical creation-failure route first. Never invent a connection or successful analytics response. Addicus + Kiko must both leave initialization without unbounded retry; Cordy/Current are secondary outside-corpus candidates.

Source investigation: src/frameworks/core_foundation/cf_stream.rs; src/frameworks/cf_network.rs; src/frameworks/foundation/ns_stream.rs; src/frameworks/core_foundation/cf_run_loop.rs.

```text
Panic at src/dyld.rs:846:9: Call to unimplemented function _CFReadStreamScheduleWithRunLoop
```

Resources (counts by filename extension; not a decoder-validity claim): .fnt=7, .nib=127, .plist=1, .png=387.

## Blockoban Premium

- Identity: `org.bonuslevel.blockobanpr`; version `1.0` / bundle build `1.0`; minimum OS `2.2`; armv6.
- Local IPA: `Blockoban Premium 1.0.ipa`. SHA-256 `469a724851e72a5e520241e19011d3cc690737acf56dd1ef308ca85e9c2e14ca`. ZIP CRC valid; encrypted=False.
- Device source: [1-Log.txt](logs/1-Log.txt), lines 1461–2285; first panic line 1537. Log name `Blockoban`; launched archive `Blockoban Premium 1.0.ipa`.
- Engine: confirmed older unprefixed cocos class family; exact version unknown. Dependencies: CocosDenshion, Plus+, OpenAL.
- Outcome: **panic after presentation**; confidence: unknown origin. Role: primary investigation. Runtime ES1.1=True; observed EAGL-return count=58; Home requested=False.
- Signature: **C2D-PLUS-01** — NGDataModel class receives NSError-like domain selector.

Synchronous Plus+ requests fail offline, then domain is sent to class 0xf8094, not an NSError. Framebuffer is complete and dozens of presentations precede it. Could be an out-error slot, branch/ABI error, or guest misuse; absence of service alone does not explain a class receiver.

Next action / prediction: Symbolize caller LR 0x755a8 and trace the stack error-out slots 0xfffff4ac / 0xfffff4e0 across sendSynchronousRequest. Compare actual NSError written, retained, consumed. Do not add domain to NGDataModel or pretend the request succeeded.

Source investigation: src/frameworks/foundation/ns_url_connection.rs; src/frameworks/foundation/ns_error.rs; src/objc/messages.rs.

```text
Panic at src/objc/messages.rs:189:17: Class 0xf8094 (metaclass "NGDataModel", 0xf8080) does not respond to selector "domain"!
```

Resources (counts by filename extension; not a decoder-validity claim): .caf=1, .m4a=1, .mp3=1, .nib=17, .plist=4, .png=197, .wav=7.

## Cannon Cadets

- Identity: `com.xmgstudio.cannoncadets`; version `2.3.6` / bundle build `2.3.6`; minimum OS `3.1`; armv6, armv7.
- Local IPA: `Cannon Cadets - Below IOS 7.ipa`. SHA-256 `9db47c2e838137e7e2ca13dea07a7de1dd5733d312ea04c623a24f463468121e`. ZIP CRC valid; encrypted=False.
- Device source: [1-Log.txt](logs/1-Log.txt), lines 2286–2789; first panic line 2420. Log name `CannonCadet`; launched archive `Cannon Cadets - Below IOS 7.ipa`.
- Engine: confirmed cocos class family; CCSpriteSheet / pre-CCDirectorIOS generation; exact version unknown. Dependencies: CocosDenshion, Box2D, AdMob, Flurry, Tapjoy, OpenAL.
- Outcome: **panic**; confidence: confirmed. Role: secondary independent contract. Runtime ES1.1=True; observed EAGL-return count=0; Home requested=False.
- Signature: **C2D-ANIM-01** — +[UIView setAnimationBeginsFromCurrentState:] missing.

A UIKit transaction flag is requested before any EAGL return; source has no matching selector. This says nothing yet about the main cocos scene.

Next action / prediction: Implement transaction-local flag storage/reset and historical interruption semantics within supported animations; guest test nested transactions. Prediction: gets beyond this selector. Tiki Premium is a UI-transition control; do not advertise full animation fidelity from a no-op.

Source investigation: src/frameworks/uikit/ui_view.rs; src/frameworks/core_animation.

```text
Panic at src/objc/messages.rs:189:17: Class 0x364677b0 (metaclass "UIView", 0x364677a0) does not respond to selector "setAnimationBeginsFromCurrentState:"!
```

Resources (counts by filename extension; not a decoder-validity claim): .caf=74, .mp3=6, .nib=64, .plist=382, .png=996.

## Clowning Around

- Identity: `com.aceviral.clowningAroundPaid`; version `1.3.2` / bundle build `1.3.2`; minimum OS `3.0`; armv6, armv7.
- Local IPA: `Clowning Around - Below IOS 7.ipa`. SHA-256 `f026f62b0820bfdcbc5cdd75b41c347ce2f04819d0a0ebced677408f583b8492`. ZIP CRC valid; encrypted=False.
- Device source: [3-Clowning-Around-Log.txt](logs/3-Clowning-Around-Log.txt), lines 44–641; first panic line 125. Log name `ClownAround`; launched archive `Clowning Around - Below IOS 7.ipa`.
- Engine: confirmed cocos class family; CCSpriteBatchNode / CCDirectorIOS generation; exact version unknown. Dependencies: CocosDenshion, GameKit, iAd, OpenAL.
- Outcome: **panic after one presentation**; confidence: confirmed. Role: secondary independent contract. Runtime ES1.1=True; observed EAGL-return count=1; Home requested=False.
- Signature: **C2D-ALERT-01** — -[UIAlertView dismissWithClickedButtonIndex:animated:] missing.

Fresh supplementary log uses the same 51a6334 baseline and fails at line125 on a real UIAlertView. This confirms the old Dev2.12 boundary is still present. StoreKit observer/setup occurs beforehand; alert dismissal must preserve callback ordering rather than suppressing an alert.

Next action / prediction: Implement programmatic dismissal through existing alert lifetime/state and native host bridge; test not-shown/already-dismissed state, delegate reentrancy, callback count and ownership. Prediction: this selector no longer blocks; service/gameplay success is unproven.

Source investigation: src/frameworks/uikit/ui_view/ui_alert_view.rs.

```text
Panic at src/objc/messages.rs:189:17: Object 0x302102a0 (class "UIAlertView", 0x3020ac10) does not respond to selector "dismissWithClickedButtonIndex:animated:"!
```

Resources (counts by filename extension; not a decoder-validity claim): .caf=1, .m4a=1, .mp3=10, .nib=11, .plist=14, .png=163, .ttf=1, .wav=2.

## Continuity 2

- Identity: `com.continuitygame.continuity2`; version `1.2` / bundle build `1.2`; minimum OS `4.0`; armv6, armv7.
- Local IPA: `Continuity 2 The Continuation - Below IOS 8.ipa`. SHA-256 `36730ebe36297fe9a922759c2ee80ff321fe3ec3f54c1a64fd50a57cd920c52f`. ZIP CRC valid; encrypted=False.
- Device source: [1-Log.txt](logs/1-Log.txt), lines 2790–3651; first panic line 2910. Log name `Continuity 2`; launched archive `Continuity 2 The Continuation - Below IOS 8.ipa`.
- Engine: confirmed cocos class family; CCSpriteSheet / pre-CCDirectorIOS generation; exact version unknown. Dependencies: CocosDenshion, GoogleAnalytics, OpenAL.
- Outcome: **panic after presentation**; confidence: confirmed. Role: primary. Runtime ES1.1=True; observed EAGL-return count=103; Home requested=False.
- Signature: **C2D-KVC-01** — setValue:forKey: unwraps NSValue even for object parameter @.

RTGStaticBoxDef receives an object-valued setter. NSObject implementation chooses unboxing from the incoming NSValue class and lacks the TYPE_ID case, despite reading the setter type. This is a direct general KVC type-dispatch error; it is independent of Legendary Wars immutable dictionaries.

Next action / prediction: Preserve object identity for @ setters; only unbox when target requires scalar/struct. Guest ABI tests must preserve existing scalar behavior and nil-object handling. Prediction: Continuity crosses level-object construction; a scalar target would falsify this case.

Source investigation: src/frameworks/foundation/ns_object.rs; src/objc.rs.

```text
Panic at src/frameworks/foundation/ns_object.rs:666:34: not implemented: TODO: [(NSObject*)0x3647cd10 setValue:0x3647d370 forKey:0x3647d360]: Received an NSValue which must be unwrapped to type @
```

Resources (counts by filename extension; not a decoder-validity claim): .caf=31, .fnt=2, .plist=60, .png=249, .pvr=2.

## Contre Jour HD

- Identity: `com.chillingo.contrejourhd`; version `1.25` / bundle build `1.25`; minimum OS `3.0`; armv7, armv6.
- Local IPA: `Contre Jour v1.25 (iOS 3.0) (com.chillingo.contrejourhd).ipa`. SHA-256 `99c1ba5ee4eba5dd444a322fa46f420aeb23b96924dd6a0d4e1cc44834cd07c1`. ZIP CRC valid; encrypted=False.
- Device source: [1-Log.txt](logs/1-Log.txt), lines 3652–4233; first panic line 3910. Log name `Contre Jour`; launched archive `Contre Jour v1.25 (iOS 3.0) (com.chillingo.contrejourhd).ipa`.
- Engine: confirmed cocos class family; CCSpriteBatchNode / CCDirectorIOS generation; exact version unknown. Dependencies: CocosDenshion, Crystal, GameKit, GoogleAnalytics, OpenAL.
- Outcome: **panic**; confidence: confirmed boundary; unknown requested library. Role: primary investigation. Runtime ES1.1=True; observed EAGL-return count=0; Home requested=False.
- Signature: **C2D-DLOPEN-01** — dlopen asserts on a library outside the known-library list.

Crystal initialization precedes the assertion; the exact requested path is not decoded by the panic. The executable includes Crystal and cocos classes. An optional framework probe is plausible, but not verified. Not evidence that ES2 is required.

Next action / prediction: Recover R0 C string from retained context or bounded dlopen(path,flags,caller) event, then compare import table and expected dlerror/NULL failure. A legal absent optional library should not assert; real guest-dylib loading is a separate task. No dummy handles for nonexistent libraries.

Source investigation: src/libc/dlfcn.rs; src/dyld.rs.

```text
Panic at src/libc/dlfcn.rs:44:5: assertion failed: is_known_library(env.mem.cstr_at_utf8(path).unwrap())
```

Resources (counts by filename extension; not a decoder-validity claim): .caf=48, .mp3=7, .otf=2, .plist=2, .png=1077, .pvr=1.

## Dirt

- Identity: `com.BrokenKings.CaveCarver`; version `2.5` / bundle build `2.5`; minimum OS `3.1.3`; armv6, armv7.
- Local IPA: `Dirt 1.5.ipa`. SHA-256 `c717e787324f6ba09940e33d483530e2627afb4c7fd8d31b0308c4ca6bae7a99`. ZIP CRC valid; encrypted=False.
- Device source: [1-Log.txt](logs/1-Log.txt), lines 4234–5162; first panic line none. Log name `Dirt`; launched archive `Dirt 1.5.ipa`.
- Engine: confirmed cocos class family; CCSpriteBatchNode / CCDirectorIOS generation; exact version unknown. Dependencies: CocosDenshion, Plus+, Flurry, OpenAL.
- Outcome: **working in reported scope**; confidence: confirmed user observation. Role: control. Runtime ES1.1=True; observed EAGL-return count=1219; Home requested=True.
- Signature: **C2D-CONTROL-01** — No blocking failure in the user-reported tested scope.

User reports gameplay working; current log has continuing presentation and a manual Home exit. This establishes a scoped healthy control, not all levels, every save path, or every future version. The recorder alone does not verify visual correctness.

Next action / prediction: Choose only by changed subsystem. Dirt: movement, rotation and pause/resume. Moonlights: older cocos scene/audio/input. Tiki Premium: newer scene/physics and transitions. Preserve saves; no demand to replay all three for a narrow unrelated patch.

Source investigation: tests/TestApp_source; src/frameworks/opengles; src/frameworks/uikit.

Identity/history qualifications: IPA filename says 1.5; actual bundle/log version 2.5.

Resources (counts by filename extension; not a decoder-validity claim): .mp3=21, .nib=1, .plist=2, .png=28, .wav=23.

## Doodle Grub

- Identity: `com.pixowl.doodlegrub`; version `2.52` / bundle build `2.52`; minimum OS `3.1`; armv7, armv6.
- Local IPA: `Doodle_Grub_v2.52_os31.ipa`. SHA-256 `c8e20cc4716e3d870cf841ca630b465a7d4527d088e49bc56e6aadab47856865`. ZIP CRC valid; encrypted=False.
- Device source: [1-Log.txt](logs/1-Log.txt), lines 5163–5793; first panic line 5445. Log name `Doodle Grub`; launched archive `Doodle_Grub_v2.52_os31.ipa`.
- Engine: confirmed cocos class family; CCSpriteSheet / pre-CCDirectorIOS generation; exact version unknown. Dependencies: CocosDenshion, OpenFeint, GameKit, GoogleAnalytics, iAd, AdMob, Flurry, OpenAL.
- Outcome: **panic**; confidence: confirmed missing selector; archive layout unverified. Role: secondary independent contract. Runtime ES1.1=True; observed EAGL-return count=0; Home requested=False.
- Signature: **C2D-COLOR-CODING-01** — UIColor encodeWithCoder: missing.

The receiver is a valid UIColor; serialization fails before a frame is presented. This is a coding-contract gap, not proof of missing textures or nil CGColor.

Next action / prediction: Determine historical keyed archive keys/class clusters and round-trip RGBA/gray colors including alpha. Keep unsupported pattern colors explicit. Do not serialize an arbitrary modern format. Prediction: archive completes and launch proceeds.

Source investigation: src/frameworks/uikit/ui_color.rs; src/frameworks/foundation/ns_keyed_archiver.rs; src/frameworks/foundation/ns_keyed_unarchiver.rs.

```text
Panic at src/objc/messages.rs:189:17: Object 0x3646c340 (class "UIColor", 0x364666c0) does not respond to selector "encodeWithCoder:"!
```

Resources (counts by filename extension; not a decoder-validity claim): .fnt=3, .mp3=19, .nib=226, .plist=7, .png=691, .ttf=2, .wav=21.

## FastBall 2

- Identity: `klikgames.fastball.v2`; version `1.5.3` / bundle build `1.5.3`; minimum OS `3.0`; armv7, armv6.
- Local IPA: `Fast Ball - Below IOS 7.ipa`. SHA-256 `e06bc31a043705d60278bd5f6b0720e0f102aa84e89206641ab9dc4acc65cf9f`. ZIP CRC valid; encrypted=False.
- Device source: [1-Log.txt](logs/1-Log.txt), lines 5794–6190; first panic line 5915. Log name `FastBall 2`; launched archive `Fast Ball - Below IOS 7.ipa`.
- Engine: confirmed cocos classes; embedded source-path version 0.99.5 (fork/local edits possible). Dependencies: CocosDenshion, OpenFeint, GameKit, OpenAL.
- Outcome: **panic**; confidence: confirmed boundary; probable transition mismatch. Role: primary. Runtime ES1.1=True; observed EAGL-return count=0; Home requested=False.
- Signature: **C2D-ORIENT-01** — SDL rejects screen orientation/display-mode size mismatch.

window.rs unwrap panics on a host orientation transition. Binary paths identify cocos2d-iphone-0.99.5. This is separate from the non-crashing clipped games until transform/order evidence agrees.

Next action / prediction: Use existing requested/current orientation and native-window sizes at first resize. Test portrait-to-landscape while changing drawable dimensions; fix ordering/aspect contract rather than forcing a game orientation. Dirt is the relevant rotation control.

Source investigation: src/window.rs; iphone; src/frameworks/uikit/ui_application.rs.

```text
Panic at src/window.rs:2090:18: called `Result::unwrap()` on an `Err` value: "Screen orientation does not match display mode size"
```

Resources (counts by filename extension; not a decoder-validity claim): .fnt=8, .mp3=3, .nib=142, .plist=7, .png=597, .tmx=520, .wav=8.

## Flow Free

- Identity: `com.bigduckgames.flow`; version `1.1` / bundle build `1.1`; minimum OS `3.0`; armv6, armv7.
- Local IPA: `com.bigduckgames.flow_1.1_ios-3.0_flow_eb55323c.ipa`. SHA-256 `a8f06da0556947719c1b8a5a8c5d633fea8a36e8c7cc352f3e9c7755304a4bff`. ZIP CRC valid; encrypted=False.
- Device source: [1-Log.txt](logs/1-Log.txt), lines 6191–6901; first panic line 6442. Log name `Flow`; launched archive `com.bigduckgames.flow_1.1_ios-3.0_flow_eb55323c.ipa`.
- Engine: confirmed cocos class family; CCSpriteBatchNode / CCDirectorIOS generation; exact version unknown. Dependencies: CocosDenshion, GameKit, iAd, AdMob, Flurry, OpenAL.
- Outcome: **panic**; confidence: confirmed. Role: primary. Runtime ES1.1=True; observed EAGL-return count=0; Home requested=False.
- Signature: **C2D-IAD-01** — ADBannerView alloc is unimplemented.

Flow Free and Mini Militia both instantiate the Apple advertising view before presentation; the class and iAd dependencies are present. This is a genuine shared missing UIKit-style class, not an engine rendering problem.

Next action / prediction: Implement an honest unavailable-ad lifecycle: view sizing, delegate ownership, unloaded state and supported failure callbacks. Inspect the union of selectors both apps use before coding. No ad content, success callback, or bannerLoaded=true fabrication. Predict launch passes allocation in both; later blockers remain possible.

Source investigation: src/frameworks.rs; src/dyld.rs; src/frameworks/uikit/ui_view.rs.

```text
Panic at src/objc/messages.rs:238:13: Class "ADBannerView" (0x191340) is unimplemented. Lookup to class method "alloc".
```

Resources (counts by filename extension; not a decoder-validity claim): .caf=4, .plist=4, .png=23.

## Hook Champ

- Identity: `com.rocketcat.HookChamp`; version `1.51` / bundle build `1.51`; minimum OS `3.0`; armv6.
- Local IPA: `HookChamp 1.51.ipa`. SHA-256 `2ebb1547b68a765d876bf4c6d3b63355bf0a41516bf156b752d3eafab4421889`. ZIP CRC valid; encrypted=False.
- Device source: [1-Log.txt](logs/1-Log.txt), lines 6902–7799; first panic line none. Log name `HookChamp`; launched archive `HookChamp 1.51.ipa`.
- Engine: unconfirmed cocos engine: generic director/Texture2D names are insufficient. Dependencies: OpenFeint, OpenAL.
- Outcome: **partial; blocked name entry**; confidence: confirmed symptom; unknown cause. Role: primary investigation. Runtime ES1.1=True; observed EAGL-return count=1894; Home requested=True.
- Signature: **C2D-INPUT-01** — Name-entry page has no responsive field/back controls.

User observation plus ~1894 EAGL returns rules out a stopped renderer. No decisive panic. IPA has OpenFeint/Texture2D but lacks the characteristic CCNode/CCDirector class family; cocos engine attribution is unconfirmed.

Next action / prediction: One field tap and one Back tap with hit-tested view, first-responder transition and native text-input request/result. Compare before versus after field tap. If delivery reaches guest and it rejects, inspect guest screen state instead of UIKit coordinates. Do not share a generic fix with other black screens.

Source investigation: src/frameworks/uikit/ui_view.rs; src/frameworks/uikit/ui_responder.rs; src/frameworks/uikit/ui_view/ui_control/ui_text_field.rs; src/window.rs.

Resources (counts by filename extension; not a decoder-validity claim): .caf=32, .mp3=9, .nib=127, .plist=2, .png=229.

## Infinity Field

- Identity: `com.chillingo.infinityfieldiphone`; version `1.1` / bundle build `1.1`; minimum OS `3.1.3`; armv6, armv7.
- Local IPA: `Infinity Field - Below IOS 7.ipa`. SHA-256 `6ca00de26347f80fe35f694d76ec2d6c19fef0a4d76bde6c68e027b98bb7a9cf`. ZIP CRC valid; encrypted=False.
- Device source: [1-Log.txt](logs/1-Log.txt), lines 7800–8315; first panic line 7926. Log name `Infinity Field`; launched archive `Infinity Field - Below IOS 7.ipa`.
- Engine: confirmed cocos class family; CCSpriteBatchNode / CCDirectorIOS generation; exact version unknown. Dependencies: CocosDenshion, Crystal, GameKit, OpenAL.
- Outcome: **panic**; confidence: confirmed. Role: primary. Runtime ES1.1=True; observed EAGL-return count=0; Home requested=False.
- Signature: **C2D-NUMBER-01** — NSNumber numberWithUnsignedChar: missing.

Valid NSNumber class receives the historical unsigned-byte constructor. No EAGL return. Clear small generic boxed-value surface, independent of KVC implementation.

Next action / prediction: Add correct unsigned 0..255 boxing and matching numeric conversions through existing NSNumber machinery. Test 0,127,128,255 on ARM guest ABI. Prediction: construction proceeds without sign extension.

Source investigation: src/frameworks/foundation/ns_value.rs; src/frameworks/core_foundation/cf_number.rs.

```text
Panic at src/objc/messages.rs:189:17: Class 0x176280 (metaclass "NSNumber", 0x176270) does not respond to selector "numberWithUnsignedChar:"!
```

Resources (counts by filename extension; not a decoder-validity claim): .caf=53, .ccz=22, .fnt=4, .m4a=2, .plist=26, .png=64, .pvr=21, .ttf=2, .wav=4.

## iStunt Reloaded Pro

- Identity: `com.profusionstudios.istuntpro`; version `1.8.4` / bundle build `1.8.4`; minimum OS `3.0`; armv6, armv7.
- Local IPA: `iStunt Reloaded Pro (v1.8.4) [Decrypted].ipa`. SHA-256 `b57eae92fd4e769402e83cdb0b7be5fb24dc4027a8ed1c7cf9cad61045b912ab`. ZIP CRC valid; encrypted=False.
- Device source: [1-Log.txt](logs/1-Log.txt), lines 8316–9268; first panic line 8566. Log name `iStunt Pro`; launched archive `iStunt Reloaded Pro (v1.8.4) [Decrypted].ipa`.
- Engine: confirmed cocos class family; CCSpriteSheet / pre-CCDirectorIOS generation; exact version unknown. Dependencies: CocosDenshion, Box2D, OpenFeint, OpenAL.
- Outcome: **partial then panic**; confidence: probable producer; confirmed wrong class. Role: primary. Runtime ES1.1=True; observed EAGL-return count=249; Home requested=False.
- Signature: **C2D-TABLE-01** — LevelTable.tableView returns plain UIView; setRowHeight: fails.

Log explicitly says nib load failed and fell back to plain UIView. Return provenance identifies tableView as producer. UITableViewController currently creates a table in initWithStyle only; tableView just calls view. Address reuse is recorded but the latest allocation is a new UIView, not by itself proof of use-after-free.

Next action / prediction: Inspect LevelTable superclass, init route and its nib before fixing lazy loadView/root creation. Default UITableViewController must create a UITableView with delegate/dataSource when appropriate, preserving actual nib overrides. Test init, initWithStyle, initWithNibName and subclass loadView. Never add rowHeight to UIView.

Source investigation: src/frameworks/uikit/ui_view_controller.rs; src/frameworks/uikit/ui_view_controller/ui_table_view_controller.rs; src/frameworks/uikit/ui_nib.rs.

Identity/history qualifications: LevelTable/root-view diagnosis is stronger than treating setRowHeight as an absent UIView API.

```text
Panic at src/objc/messages.rs:189:17: Object 0x3646e540 (class "UIView", 0x36467ff0) does not respond to selector "setRowHeight:"!
```

Resources (counts by filename extension; not a decoder-validity claim): .fnt=1, .mp3=5, .nib=184, .plist=4, .png=455, .wav=16.

## Junk Jack Retro

- Identity: `com.jack.xsx.junkjack`; version `1.2.1` / bundle build `4010`; minimum OS `4.2`; armv7, armv6.
- Local IPA: `Junk Jack-(com.jack.xsx.junkjack)-4010-(iOS_4.2)-880f6b0cecbee5c33fb400c23479f54a.ipa`. SHA-256 `7abc30ac4cb56d7069d456c6be84deabfe2a6b8f9969bb35635c95c66b7236f5`. ZIP CRC valid; encrypted=False.
- Device source: [1-Log.txt](logs/1-Log.txt), lines 9269–9987; first panic line 9335. Log name `Junk Jack`; launched archive `Junk Jack-(com.jack.xsx.junkjack)-4010-(iOS_4.2)-880f6b0cecbee5c33fb400c23479f54a.ipa`.
- Engine: confirmed cocos class family; CCSpriteBatchNode / CCDirectorIOS generation; exact version unknown. Dependencies: CocosDenshion, GameKit, OpenAL.
- Outcome: **panic**; confidence: confirmed. Role: secondary independent contract. Runtime ES1.1=True; observed EAGL-return count=0; Home requested=False.
- Signature: **C2D-BLOCK-01** — NSDictionary enumerateKeysAndObjectsUsingBlock: missing.

Valid immutable dictionary requests supported enumeration. No invalid pointer evidence at this boundary. Block invocation ABI and stop flag are the risk, not dictionary mutability.

Next action / prediction: General block enumeration using existing guest block invocation; test keys/value identity, BOOL* stop, empty dictionary and teardown. Do not mutate the immutable receiver or assume order.

Source investigation: src/frameworks/foundation/ns_dictionary.rs; src/libc/dispatch.rs.

```text
Panic at src/objc/messages.rs:189:17: Object 0x302f4d70 (class "_touchHLE_NSDictionary", 0x3020e350) does not respond to selector "enumerateKeysAndObjectsUsingBlock:"!
```

Resources (counts by filename extension; not a decoder-validity claim): .caf=163, .fnt=8, .mp3=13, .plist=4, .png=156.

## Kiko: The Last Totem

- Identity: `com.fabulapps.kiko`; version `1.2.927` / bundle build `1.2.927`; minimum OS `3.1`; armv7, armv6.
- Local IPA: `Kiko_v1.2.927_os31.ipa`. SHA-256 `fb933a031049f46ea5f26c4e08718db08f891e3fbaf4453dc01fade881b3ea88`. ZIP CRC valid; encrypted=False.
- Device source: [1-Log.txt](logs/1-Log.txt), lines 9988–11221; first panic line 10421. Log name `Kiko`; launched archive `Kiko_v1.2.927_os31.ipa`.
- Engine: confirmed cocos classes; embedded source-path version 0.99.5-rc1 (fork/local edits possible). Dependencies: CocosDenshion, GameKit, GoogleAnalytics, Flurry, OpenAL.
- Outcome: **panic after presentation**; confidence: confirmed boundary; probable upstream cause. Role: second generation representative. Runtime ES1.1=True; observed EAGL-return count=581; Home requested=False.
- Signature: **C2D-STREAM-01** — Missing CFReadStreamScheduleWithRunLoop after NULL socket-stream pair.

Both runs enter GANTCPSocket / Google Analytics and request www.google-analytics.com:80. CFStreamCreatePairWithSocketToCFHost returns NULL streams before the missing scheduling export. Cocos initialization and EAGL returns already occur. The shared producer is a third-party analytics transport, not cocos rendering. A nil scheduling no-op alone would merely expose the next call.

Next action / prediction: Trace the finite create/client/schedule/open/error/unschedule/close chain already in the recorder; implement truthful failed network streams with correct callback-context ownership and run-loop delivery, or document the historical creation-failure route first. Never invent a connection or successful analytics response. Addicus + Kiko must both leave initialization without unbounded retry; Cordy/Current are secondary outside-corpus candidates.

Source investigation: src/frameworks/core_foundation/cf_stream.rs; src/frameworks/cf_network.rs; src/frameworks/foundation/ns_stream.rs; src/frameworks/core_foundation/cf_run_loop.rs.

```text
Panic at src/dyld.rs:846:9: Call to unimplemented function _CFReadStreamScheduleWithRunLoop
```

Resources (counts by filename extension; not a decoder-validity claim): .nib=1, .plist=307, .png=1220, .ttf=1.

## League of Evil

- Identity: `com.ravenousgames.leagueofevil`; version `1.52` / bundle build `1.52`; minimum OS `3.0`; armv6, armv7.
- Local IPA: `League of Evil 1.52.ipa`. SHA-256 `f887dd96196d6d72d336a4a608e1ae8abf1c67628bea037db8051827ff2bab04`. ZIP CRC valid; encrypted=True.
- Device source: [1-Log.txt](logs/1-Log.txt), lines 11222–11282; first panic line none. Log name `League of Evil`; launched archive `League of Evil 1.52.ipa`.
- Engine: unknown: encrypted code/metadata; residual markers not conclusive. Dependencies: CocosDenshion, OpenFeint, Plus+, GameKit, Flurry, OpenAL.
- Outcome: **encrypted before CPU**; confidence: confirmed. Role: external package blocker. Runtime ES1.1=True; observed EAGL-return count=0; Home requested=False.
- Signature: **C2D-PACKAGE-01** — Mach-O encryption cryptid=1 in supplied ARM slices.

ZIP CRC succeeds, but loader explicitly rejects encrypted executable before CPU start. Encrypted sections make negative engine-marker conclusions unreliable. Missing flight recorder is expected here, not a diagnostic failure.

Next action / prediction: Needs a legitimately obtained runnable unencrypted copy, with bundle/version/hash recorded. No emulator correction or decryption attempt is included.

Source investigation: src/mach_o.rs.

Resources (counts by filename extension; not a decoder-validity claim): .fnt=12, .mp3=11, .nib=208, .plist=34, .png=830, .tmx=176, .ttf=1, .wav=7.

## Legendary Wars

- Identity: `com.liv-games.legendarywars`; version `1.33` / bundle build `1.33`; minimum OS `3.1.3`; armv6, armv7.
- Local IPA: `Legendary.Wars v1.33.ipa`. SHA-256 `bb62bb84d474767afc0ccce9583daa1dd4f40f3e6105726ae423879092bf454c`. ZIP CRC valid; encrypted=False.
- Device source: [1-Log.txt](logs/1-Log.txt), lines 11283–11774; first panic line 11395. Log name `LegendaryWar`; launched archive `Legendary.Wars v1.33.ipa`.
- Engine: confirmed cocos class family; CCSpriteBatchNode / CCDirectorIOS generation; exact version unknown. Dependencies: CocosDenshion, GameKit, Flurry, Tapjoy, OpenAL.
- Outcome: **panic after one presentation**; confidence: confirmed boundary; historical behavior unresolved. Role: primary investigation. Runtime ES1.1=True; observed EAGL-return count=1; Home requested=False.
- Signature: **C2D-MUTABILITY-01** — Immutable property-list dictionary receives KVC assignment height.

Same strict mutability boundary documented in older ledger. KVC object unboxing correction for Continuity does not make immutable dictionaries mutable. Producer mutability option and guest expectation must be reconciled.

Next action / prediction: Compare original plist flags/options and copy/mutableCopy path against native historical behavior. Test a minimal immutable-dictionary KVC operation separately. Preserve exception semantics and bounds; do not invent setters.

Source investigation: src/frameworks/foundation/ns_property_list_serialization.rs; src/frameworks/foundation/ns_dictionary.rs; src/frameworks/foundation/ns_object.rs.

```text
Panic at src/frameworks/foundation/ns_object.rs:743:5: Object 0x302b24f0 of class "_touchHLE_NSDictionary" (0x3020c100) does not have a setter for height (0x302c9010)
```

Resources (counts by filename extension; not a decoder-validity claim): .caf=1, .fnt=42, .mp3=10, .otf=2, .plist=832, .png=393, .ttf=2, .wav=92.

## Love Catcher

- Identity: `com.hyperdevbox.LoveCatch`; version `1.0` / bundle build `1.0`; minimum OS `2.1`; armv6.
- Local IPA: `LoveCatch 1.0-Hexhammer.ipa`. SHA-256 `c707b87d4ef8934fcc17e8b29cc0a436d6ec54747fed664d110aec6f21f503c8`. ZIP CRC valid; encrypted=False.
- Device source: [1-Log.txt](logs/1-Log.txt), lines 11775–12221; first panic line none. Log name `LoveCatch`; launched archive `LoveCatch 1.0-Hexhammer.ipa`.
- Engine: unconfirmed cocos engine: generic director/Texture2D names are insufficient. Dependencies: OpenAL.
- Outcome: **partial; white screen with audio**; confidence: confirmed symptom; unknown cause. Role: primary investigation. Runtime ES1.1=True; observed EAGL-return count=1005; Home requested=True.
- Signature: **C2D-VISUAL-01** — White screen with sound; no usable input.

Log records ~1005 presentations, then a repeated NSThread sleepForTimeInterval:0.5 path; audio is user-observed. Projection/geometry and guest wait-state both remain candidates. Only a generic director string appears, insufficient to confirm cocos membership.

Next action / prediction: Correlate a screenshot with existing last draw/viewport/matrices and guest caller near LR0x3424. Determine whether white geometry is intentionally redrawn or renderer stops after a state flag. One bounded transition capture if existing data lacks the state; do not turn on full instruction tracing.

Source investigation: src/frameworks/opengles/gles_guest.rs; src/frameworks/opengles/eagl.rs; src/window.rs.

Resources (counts by filename extension; not a decoder-validity claim): .nib=1, .plist=2, .png=2.

## Lumi

- Identity: `com.foundthefuture.lumi`; version `1.31` / bundle build `1.31`; minimum OS `4.0`; armv6, armv7.
- Local IPA: `Lumi V1.31 HS.ipa`. SHA-256 `78251ee707ad2593516ed4080428957fdb3d6958be3a1b85498ab3092c0993d2`. ZIP CRC valid; encrypted=False.
- Device source: [2-Log.txt](logs/2-Log.txt), lines 44–864; first panic line 198. Log name `Lumi`; launched archive `Lumi V1.31 HS.ipa`.
- Engine: confirmed cocos class family; CCSpriteBatchNode / CCDirectorIOS generation; exact version unknown. Dependencies: CocosDenshion, Plus+, GameKit, Flurry, OpenAL.
- Outcome: **panic after presentation**; confidence: confirmed. Role: secondary ABI representative. Runtime ES1.1=True; observed EAGL-return count=193; Home requested=False.
- Signature: **C2D-SIGNATURE-01** — NSMethodSignature parser rejects structure encoding {.

Real method-signature construction reaches a parser that handles only a small primitive set. Exact aggregate shape and subsequent NSInvocation use must be decoded before ABI support is claimed.

Next action / prediction: Parse balanced nested encodings and ARMv6/v7 layout/alignment, then cover register/stack and structure returns in guest tests. Prediction: signature can be constructed; a successful parse alone does not verify invocation.

Source investigation: src/frameworks/foundation/ns_method_signature.rs; src/frameworks/foundation/ns_invocation.rs; src/abi.rs.

```text
Panic at src/frameworks/foundation/ns_method_signature.rs:127:14: not implemented: parse_signature_inner: {
```

Resources (counts by filename extension; not a decoder-validity claim): .fnt=1, .mp3=63, .plist=3, .png=3533.

## Mega Jump

- Identity: `com.getsetgames.megajump`; version `16.0.3` / bundle build `16.0.3`; minimum OS `3.0`; armv6.
- Local IPA: `Mega Jump (V16.0.3).ipa`. SHA-256 `f513d7a755563954ad6baaa2112680a2072c21fa6efdc41d4aa61ecca7197a61`. ZIP CRC valid; encrypted=False.
- Device source: [2-Log.txt](logs/2-Log.txt), lines 865–1432; first panic line 1101. Log name `Mega Jump`; launched archive `Mega Jump (V16.0.3).ipa`.
- Engine: confirmed cocos class family; CCSpriteBatchNode / CCDirectorIOS generation; exact version unknown. Dependencies: CocosDenshion, OpenFeint, GameKit, GoogleAnalytics, Flurry, Tapjoy, OpenAL.
- Outcome: **guest memory fault**; confidence: unknown origin. Role: primary investigation. Runtime ES1.1=True; observed EAGL-return count=0; Home requested=False.
- Signature: **C2D-NULL-01** — Null-page read at address0, followed by CPU MemoryError.

A resource/initialization sequence (including fps_images~iphone.png) precedes the fault, but the recorder does not establish a failed producer. The CPU error is downstream of the first memory fault. Addicus sharing developer/SDKs does not prove the same failure.

Next action / prediction: Symbolize fault PC/LR against this exact executable; follow the source register backward through one producer/call return using existing captured bytes. If needed, watch that producer result only. Validate resource presence and conditional control flow before hypothesizing allocation/decoder semantics.

Source investigation: src/cpu.rs; src/mem.rs; src/diagnostics.rs; src/frameworks/foundation/ns_bundle.rs.

```text
Panic at src/mem.rs:375:9: Attempted null-page access at 0x0 (0x4 bytes)
```

Resources (counts by filename extension; not a decoder-validity claim): .aac=95, .ccz=22, .fnt=5, .nib=169, .plist=56, .png=512, .ttf=2.

## Mini Militia

- Identity: `com.chadtowns.da2`; version `1.6.0` / bundle build `1.6.0`; minimum OS `3.0`; armv6.
- Local IPA: `MiniMilitia.ipa`. SHA-256 `57057884b05867f4690c16027b2003514d89a4fc01015f8419dc51ed58bad53d`. ZIP CRC valid; encrypted=False.
- Device source: [2-Log.txt](logs/2-Log.txt), lines 1433–2111; first panic line 1539. Log name `Mini Militia`; launched archive `MiniMilitia.ipa`.
- Engine: confirmed cocos classes; embedded source-path version 0.99.5-beta3 (fork/local edits possible). Dependencies: CocosDenshion, Chipmunk, GameKit, iAd, OpenAL.
- Outcome: **panic**; confidence: confirmed. Role: second generation representative. Runtime ES1.1=True; observed EAGL-return count=0; Home requested=False.
- Signature: **C2D-IAD-01** — ADBannerView alloc is unimplemented.

Flow Free and Mini Militia both instantiate the Apple advertising view before presentation; the class and iAd dependencies are present. This is a genuine shared missing UIKit-style class, not an engine rendering problem.

Next action / prediction: Implement an honest unavailable-ad lifecycle: view sizing, delegate ownership, unloaded state and supported failure callbacks. Inspect the union of selectors both apps use before coding. No ad content, success callback, or bannerLoaded=true fabrication. Predict launch passes allocation in both; later blockers remain possible.

Source investigation: src/frameworks.rs; src/dyld.rs; src/frameworks/uikit/ui_view.rs.

```text
Panic at src/objc/messages.rs:238:13: Class "ADBannerView" (0x364696b0) is unimplemented. Lookup to class method "alloc".
```

Resources (counts by filename extension; not a decoder-validity claim): .mp3=1, .plist=11, .png=64, .tmx=11, .wav=95.

## Moonlights

- Identity: `org.bonuslevel.MyGame`; version `DWANEO v1.0` / bundle build `DWANEO v1.0`; minimum OS `2.2`; armv6.
- Local IPA: `Moonlights-(org.bonuslevel.MyGame)-DWANEO v1.0-(iOS_2.2)-cbd006bc5d3be2522517db02c0e79f44.ipa`. SHA-256 `2c5a9799a4f6d6d33deaf9a8b15f30053b0168080f452a82efabb6c435d8127b`. ZIP CRC valid; encrypted=False.
- Device source: [2-Log.txt](logs/2-Log.txt), lines 2112–2987; first panic line none. Log name `Moonlights`; launched archive `Moonlights-(org.bonuslevel.MyGame)-DWANEO v1.0-(iOS_2.2)-cbd006bc5d3be2522517db02c0e79f44.ipa`.
- Engine: confirmed older unprefixed cocos class family; exact version unknown. Dependencies: CocosDenshion, Plus+, OpenAL.
- Outcome: **working in reported scope**; confidence: confirmed user observation. Role: older-generation control. Runtime ES1.1=True; observed EAGL-return count=1253; Home requested=True.
- Signature: **C2D-CONTROL-01** — No blocking failure in the user-reported tested scope.

User reports gameplay working; current log has continuing presentation and a manual Home exit. This establishes a scoped healthy control, not all levels, every save path, or every future version. The recorder alone does not verify visual correctness.

Next action / prediction: Choose only by changed subsystem. Dirt: movement, rotation and pause/resume. Moonlights: older cocos scene/audio/input. Tiki Premium: newer scene/physics and transitions. Preserve saves; no demand to replay all three for a narrow unrelated patch.

Source investigation: tests/TestApp_source; src/frameworks/opengles; src/frameworks/uikit.

Resources (counts by filename extension; not a decoder-validity claim): .mp3=1, .plist=3, .png=11, .pvr=5, .wav=14.

## Pro Zombie Soccer

- Identity: `com.chilingo.PZS`; version `1.2` / bundle build `1.2`; minimum OS `3.0`; armv6.
- Local IPA: `com.chilingo.pzs_1.2_ios-3.0_pzs_e52d121d.ipa`. SHA-256 `a2565b49aee95809b1751c64d44e8d370c6685f7aa283f9a66908fbd52d36c4d`. ZIP CRC valid; encrypted=False.
- Device source: [2-Log.txt](logs/2-Log.txt), lines 2988–3691; first panic line none. Log name `PZS`; launched archive `com.chilingo.pzs_1.2_ios-3.0_pzs_e52d121d.ipa`.
- Engine: confirmed cocos class family; CCSpriteBatchNode / CCDirectorIOS generation; exact version unknown. Dependencies: CocosDenshion, Plus+, Crystal, OpenAL.
- Outcome: **partial; clipped and unresponsive**; confidence: confirmed symptom; possible coordinate cause. Role: primary investigation. Runtime ES1.1=True; observed EAGL-return count=1600; Home requested=True.
- Signature: **C2D-VISUAL-02** — Rotated/clipped menu and input does not respond.

User reports the orientation fault; ~1600 returns demonstrate ongoing presentation. Crystal/cocos code exists, but an overlay and mismatched touch/draw coordinates remain competing explanations.

Next action / prediction: Use one orientation-stable launch and one tap on known menu position; compare drawable dimensions, UIWindow bounds/transform and guest locationInView coordinates. Dirt is rotation control; Tap the Frog is a comparison, not yet a proven same-cause case.

Source investigation: src/window.rs; src/frameworks/uikit/ui_view.rs; src/frameworks/uikit/ui_touch.rs; src/frameworks/opengles/eagl.rs.

Resources (counts by filename extension; not a decoder-validity claim): .mp3=21, .plist=42, .png=477, .ttf=1, .wav=59.

## Robokill: Rescue Titan Prime

- Identity: `com.wandake.robokilliphone`; version `1.4` / bundle build `1.4`; minimum OS `4.0`; armv7, armv7.
- Local IPA: `Robokill_Rescue_Titan_Prime_for_iPad_v1_4.ipa`. SHA-256 `298bc155633d48dd2ce1e5b38bf4b12f4a2758d71460b6bc504253a55582036b`. ZIP CRC valid; encrypted=False.
- Device source: [2-Log.txt](logs/2-Log.txt), lines 3692–4271; first panic line 3819. Log name `Robokill`; launched archive `Robokill_Rescue_Titan_Prime_for_iPad_v1_4.ipa`.
- Engine: confirmed cocos class family; CCSpriteSheet / pre-CCDirectorIOS generation; exact version unknown. Dependencies: GameKit, OpenAL.
- Outcome: **panic**; confidence: unknown empty-array origin. Role: primary investigation. Runtime ES1.1=True; observed EAGL-return count=0; Home requested=False.
- Signature: **C2D-PROFILE-01** — Index0 requested from empty profile array.

RoboKillProfile initProfiles/getProfiles and defaults keys precede objectAtIndex at LR0x193381. This is a producer-empty collection, unlike Trenches removing entries during dispatch. Filename says iPad but actual bundle is com.wandake.robokilliphone.

Next action / prediction: Symbolize the profile-init branch; record defaults value class/array count and expected initial-profile creation. Compare preserved save state with a separate isolated fresh fixture only with authorization, never delete current saves. Do not return a fabricated profile.

Source investigation: src/frameworks/foundation/ns_user_defaults.rs; src/frameworks/foundation/ns_array.rs; src/diagnostics.rs.

Identity/history qualifications: Filename says iPad; bundle identifier says robokilliphone. Device family in plist is authoritative.

```text
Panic at src/frameworks/foundation/ns_array.rs:868:51: index out of bounds: the len is 0 but the index is 0
```

Resources (counts by filename extension; not a decoder-validity claim): .mp3=88, .nib=21, .otf=10, .plist=2, .png=2813, .ttf=4, .wav=6.

## Silent Swords ZERO

- Identity: `com.oniricgames.silentswordszero`; version `1.1` / bundle build `1.1`; minimum OS `3.0`; armv6.
- Local IPA: `Silent Swords Zero - Below IOS 7.ipa`. SHA-256 `07354b59e07a00fd61c550d9bd96f872f9e3d83f4728f9e2b93f5d258a1a8987`. ZIP CRC valid; encrypted=False.
- Device source: [2-Log.txt](logs/2-Log.txt), lines 4272–5391; first panic line 4536. Log name `Zero`; launched archive `Silent Swords Zero - Below IOS 7.ipa`.
- Engine: confirmed older unprefixed cocos class family; exact version unknown. Dependencies: CocosDenshion, Chipmunk, OpenFeint, Plus+, OpenAL.
- Outcome: **panic after sustained presentation**; confidence: unknown origin. Role: primary investigation. Runtime ES1.1=True; observed EAGL-return count=1345; Home requested=False.
- Signature: **C2D-LIFETIME-01** — hash sent to object with nil isa.

After >1300 returns, 0x3651a3e0 is invalid at dictionary/hash consumption. Deallocation versus corruption versus raw pointer confusion is not settled. Not evidence of a generic null-page bug.

Next action / prediction: Recover retained address generations/events and symbolized insertion/caller. If origin was evicted, watch one producer object through collection retain/release and pool drain with a fixed cap. Preserve invalid-object checks.

Source investigation: src/objc/objects.rs; src/frameworks/foundation/ns_dictionary.rs; src/diagnostics.rs.

```text
Panic at src/objc/messages.rs:314:5: Cannot send selector "hash" to receiver 0x3651a3e0: its isa is nil (the object may have been deallocated or corrupted)
```

Resources (counts by filename extension; not a decoder-validity claim): .aac=1, .fnt=1, .m4a=7, .nib=122, .plist=3, .png=236, .pvr=59, .wav=21.

## Stick Stunt Biker

- Identity: `com.rsz.StickStuntBiker`; version `5.9` / bundle build `5.9`; minimum OS `3.0`; armv6, armv7.
- Local IPA: `Stick Stunt Biker - Below IOS 7.ipa`. SHA-256 `1ad0688fae69f08bbef0558875bd1b569c084ab1a406fc5f0f4852febda9fd31`. ZIP CRC valid; encrypted=False.
- Device source: [2-Log.txt](logs/2-Log.txt), lines 5392–5843; first panic line 5520. Log name `Stunt Biker`; launched archive `Stick Stunt Biker - Below IOS 7.ipa`.
- Engine: confirmed cocos class family; CCSpriteBatchNode / CCDirectorIOS generation; exact version unknown. Dependencies: CocosDenshion, OpenFeint, GameKit, OpenAL.
- Outcome: **panic**; confidence: confirmed causal chain. Role: primary. Runtime ES1.1=True; observed EAGL-return count=0; Home requested=False.
- Signature: **C2D-DATE-01** — Negative Duration during NSDate description.

Last events create dateWithTimeIntervalSince1970:0, assign GameEngineNewsFeed.dateLastDownload, then NSLog invokes NSDate.description. description calls CFAbsoluteTimeGetGregorianDate; current code feeds a negative Apple-epoch interval to unsigned Duration. This is a date-format conversion bug, not proof of bad timer/sleep scheduling.

Next action / prediction: Signed epoch conversion with finite/range checks and correct negative/fraction handling; test 1970, 2000,2001 and dates before Unix epoch where supported. Prediction: news-feed logging no longer panics. Do not clamp old dates to now.

Source investigation: src/frameworks/foundation/ns_date.rs; src/frameworks/core_foundation/time.rs.

```text
Panic at /rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/core/src/time.rs:962:23: cannot convert float seconds to Duration: value is negative
```

Resources (counts by filename extension; not a decoder-validity claim): .fnt=4, .mp3=15, .nib=154, .plist=16, .png=412, .pvr=3, .wav=9.

## StickWars

- Identity: `com.johnehartzog.stickwars-siege`; version `1.8.6` / bundle build `1.8.6`; minimum OS `3.0`; armv6, armv7.
- Local IPA: `StickWars 1.8.6.ipa`. SHA-256 `56a0ea7acb09417c40a8e291159ea92d90443c7d4cdd1e536f19f00c4def69db`. ZIP CRC valid; encrypted=True.
- Device source: [2-Log.txt](logs/2-Log.txt), lines 5844–5904; first panic line none. Log name `StickWars`; launched archive `StickWars 1.8.6.ipa`.
- Engine: unknown: encrypted code/metadata; residual markers not conclusive. Dependencies: Chipmunk, OpenFeint, Plus+, iAd, AdMob, Flurry, Tapjoy.
- Outcome: **encrypted before CPU**; confidence: confirmed. Role: external package blocker. Runtime ES1.1=True; observed EAGL-return count=0; Home requested=False.
- Signature: **C2D-PACKAGE-01** — Mach-O encryption cryptid=1 in supplied ARM slices.

ZIP CRC succeeds, but loader explicitly rejects encrypted executable before CPU start. Encrypted sections make negative engine-marker conclusions unreliable. Missing flight recorder is expected here, not a diagnostic failure.

Next action / prediction: Needs a legitimately obtained runnable unencrypted copy, with bundle/version/hash recorded. No emulator correction or decryption attempt is included.

Source investigation: src/mach_o.rs.

Resources (counts by filename extension; not a decoder-validity claim): .fnt=3, .mp3=6, .nib=179, .plist=4, .png=433, .pvr=1, .ttf=1, .wav=22.

## Super Stick Golf

- Identity: `com.noodlecake.superstickgolf`; version `1.9` / bundle build `1.9`; minimum OS `3.1`; armv6, armv7.
- Local IPA: `com.noodlecake.superstickgolf-iOS3.1-(Clutch-2.0.4).ipa`. SHA-256 `28469401db1027fbb51615c8a9ede0c209ba5335c29ddbc4c39a538bdd8894ff`. ZIP CRC valid; encrypted=False.
- Device source: [2-Log.txt](logs/2-Log.txt), lines 5905–6401; first panic line 6065. Log name `SSG`; launched archive `com.noodlecake.superstickgolf-iOS3.1-(Clutch-2.0.4).ipa`.
- Engine: confirmed cocos class family; CCSpriteBatchNode / CCDirectorIOS generation; exact version unknown. Dependencies: CocosDenshion, GameKit, Flurry, OpenAL.
- Outcome: **panic after one presentation**; confidence: confirmed. Role: primary. Runtime ES1.1=True; observed EAGL-return count=1; Home requested=False.
- Signature: **C2D-ARCHIVE-01** — NSKeyedUnarchiver rejects primitive Boolean.

The object table contains a plist Boolean; the source handles strings/integers/reals but omits Boolean. This is an archive representation gap, not corrupt ZIP evidence.

Next action / prediction: Decode Boolean to existing NSNumber/CFBoolean semantics with archive object-table identity/ownership. Test true/false, repeated UID and nested container roundtrip. Prediction: archived state loads; gameplay not guaranteed.

Source investigation: src/frameworks/foundation/ns_keyed_unarchiver.rs; src/frameworks/core_foundation/cf_number.rs.

```text
Panic at src/frameworks/foundation/ns_keyed_unarchiver.rs:353:14: not implemented: Unarchive: Boolean(
```

Resources (counts by filename extension; not a decoder-validity claim): .caf=24, .ccz=1039, .fnt=2, .mp3=9, .plist=17, .png=546, .ttf=1.

## Super Turbo Action Pig

- Identity: `com.joystickgenerals.STActionPig`; version `1.1.1` / bundle build `1.1.1`; minimum OS `2.2.1`; armv6.
- Local IPA: `com.joystickgenerals.STActionPig-iOS2.2.1-(Clutch-2.0.4) 2.ipa`. SHA-256 `89e11c7791f6c66c504c361a816edd650ebbeccf70d051a95b12f3a3dff7f37e`. ZIP CRC valid; encrypted=False.
- Device source: [2-Log.txt](logs/2-Log.txt), lines 6402–7151; first panic line 6601. Log name `S.T.ActionPig`; launched archive `com.joystickgenerals.STActionPig-iOS2.2.1-(Clutch-2.0.4) 2.ipa`.
- Engine: confirmed older unprefixed cocos class family; exact version unknown. Dependencies: CocosDenshion, OpenFeint, Plus+, OpenAL.
- Outcome: **panic after presentation**; confidence: confirmed GL error; possible shared cause. Role: older-generation representative. Runtime ES1.1=True; observed EAGL-return count=34; Home requested=False.
- Signature: **C2D-FBO-01** — Prior GL_INVALID_FRAMEBUFFER_OPERATION (0x506) observed at compositor cleanup.

Both games retain clear/draw candidate calls before the error while the queried current framebuffer is complete (0x8cd5). Error may belong to an earlier binding/context or guest draw; cleanup is a detector, not proven producer. Older CocosNode-based Action Pig and later CCNode Wispin give a useful cross-generation comparison.

Next action / prediction: First use existing GL history to reconstruct context/FBO/RBO storage/attachments per call. If ambiguous, enable a capped first-error experiment around clear/draw/present, preserving guest glGetError semantics. Compare before/after compositor state restoration; no error swallowing.

Source investigation: src/frameworks/opengles.rs; src/frameworks/opengles/gles_guest.rs; src/frameworks/opengles/eagl.rs; src/frameworks/core_animation/composition.rs.

```text
Panic at src/frameworks/core_animation/composition.rs:96:5: Core Animation compositor failure at stage "frame-cleanup", layer (null): prior GL error 0x506, framebuffer status 0x8cd5, status-query GL error 0x0
```

Resources (counts by filename extension; not a decoder-validity claim): .fnt=1, .nib=116, .plist=3, .png=195, .pvr=2.

## Tap the Frog

- Identity: `com.mentals.tapthefrog`; version `1.0` / bundle build `1.0`; minimum OS `3.0`; armv6, armv7.
- Local IPA: `com.mentals.tapthefrog_1.0_ios-3.0_tap-the-frog_d1dda3d2.ipa`. SHA-256 `faada76de44830390a9b10604aed2cdc9a97bd06ee6045306daac50cc6a66c41`. ZIP CRC valid; encrypted=False.
- Device source: [2-Log.txt](logs/2-Log.txt), lines 7152–7877; first panic line 7294. Log name `Tap The Frog`; launched archive `com.mentals.tapthefrog_1.0_ios-3.0_tap-the-frog_d1dda3d2.ipa`.
- Engine: confirmed cocos classes; embedded source-path version 1.0.0-rc (fork/local edits possible). Dependencies: CocosDenshion, GameKit, OpenAL.
- Outcome: **partial; zoomed and unresponsive; teardown panic**; confidence: confirmed symptom; possible coordinate cause. Role: second generation representative. Runtime ES1.1=True; observed EAGL-return count=1230; Home requested=True.
- Signature: **C2D-VISUAL-03** — Zoomed corner, unresponsive input; invalid controller during Home teardown.

Frames keep presenting (~1230); visible/input failure predates manual Home. modalViewController on invalid0xbdd10 occurs during recovery and is a second defect. Binary source paths identify cocos2d-iphone-1.0.0-rc. Treating the teardown panic as the cause of zoom would misdirect the work.

Next action / prediction: Compare content scale, drawable size, root view frame and touch mapping with PZS and Dirt. Separately trace root/modal controller lifetime only on Home; do not widen invalid memory access.

Source investigation: src/frameworks/uikit/ui_view.rs; src/frameworks/uikit/ui_view_controller.rs; src/window.rs; src/frameworks/opengles/eagl.rs.

```text
Panic at src/objc/messages.rs:314:5: Cannot send selector "modalViewController" to receiver 0xbdd10: its isa is nil (the object may have been deallocated or corrupted)
```

Resources (counts by filename extension; not a decoder-validity claim): .caf=29, .ccz=48, .fnt=1, .m4a=2, .plist=99, .png=17.

## Thief Lupin

- Identity: `com.abluewind.lupin`; version `1.1.7` / bundle build `1.1.7`; minimum OS `3.0`; armv6, armv7.
- Local IPA: `Thief-(com.abluewind.lupin)-1.1.7-(iOS_3.0)-5e96ea0bf3fbf8fb683ac14bbef2d4d7.ipa`. SHA-256 `37f49909fd3d0097a2e1256c1aec82ed42c9fa96beabb474502923d4a99e95bc`. ZIP CRC valid; encrypted=False.
- Device source: [2-Log.txt](logs/2-Log.txt), lines 7878–9333; first panic line 8495. Log name `Thief`; launched archive `Thief-(com.abluewind.lupin)-1.1.7-(iOS_3.0)-5e96ea0bf3fbf8fb683ac14bbef2d4d7.ipa`.
- Engine: confirmed cocos class family; CCSpriteSheet / pre-CCDirectorIOS generation; exact version unknown. Dependencies: CocosDenshion, GameKit, iAd, AdMob, Flurry, Tapjoy, OpenAL.
- Outcome: **panic after presentation**; confidence: unknown origin. Role: primary investigation. Runtime ES1.1=True; observed EAGL-return count=270; Home requested=False.
- Signature: **C2D-LIFETIME-02** — Missing StringHostObject at mutable string consumption.

0x36477b80 has no matching host string during mutation after ~270 returns. Need its actual creation and mutation receiver class; cannot assume identical cause to Silent Swords hash receiver.

Next action / prediction: Link string producer, mutable-copy/initializer return and pool drain using bounded address generations. Compare decoded class at allocation versus final consumption; never treat an invalid string as empty.

Source investigation: src/frameworks/foundation/ns_string.rs; src/objc/objects.rs; src/diagnostics.rs.

```text
Panic at src/objc/objects.rs:327:13: No host object registered for 0x36477b80 while mutably borrowing "touchHLE::frameworks::foundation::ns_string::StringHostObject"
```

Resources (counts by filename extension; not a decoder-validity claim): .mp3=29, .plist=4, .png=1538.

## Tiki Totems Premium

- Identity: `com.spokko.TikiPremium`; version `1.5.3` / bundle build `1.5.3`; minimum OS `3.0`; armv6, armv7.
- Local IPA: `Tiki Totems.ipa`. SHA-256 `158394e00af3435faf56d7efb04203592bdc2a597e897855e5fc25d5a8816338`. ZIP CRC valid; encrypted=False.
- Device source: [2-Log.txt](logs/2-Log.txt), lines 9334–10434; first panic line none. Log name `Tiki Premium`; launched archive `Tiki Totems.ipa`.
- Engine: confirmed cocos class family; CCSpriteBatchNode / CCDirectorIOS generation; exact version unknown. Dependencies: Box2D, OpenFeint, GameKit.
- Outcome: **working in reported scope**; confidence: confirmed user observation. Role: control. Runtime ES1.1=True; observed EAGL-return count=744; Home requested=True.
- Signature: **C2D-CONTROL-01** — No blocking failure in the user-reported tested scope.

User reports gameplay working; current log has continuing presentation and a manual Home exit. This establishes a scoped healthy control, not all levels, every save path, or every future version. The recorder alone does not verify visual correctness.

Next action / prediction: Choose only by changed subsystem. Dirt: movement, rotation and pause/resume. Moonlights: older cocos scene/audio/input. Tiki Premium: newer scene/physics and transitions. Preserve saves; no demand to replay all three for a narrow unrelated patch.

Source investigation: tests/TestApp_source; src/frameworks/opengles; src/frameworks/uikit.

Resources (counts by filename extension; not a decoder-validity claim): .nib=263, .plist=27, .png=617, .wav=8.

## Trainyard

- Identity: `ca.struct.trainyard`; version `1.66` / bundle build `1.66`; minimum OS `3.0`; armv6, armv7.
- Local IPA: `Trainyard.ipa`. SHA-256 `11f4f96f83cd8f0befaa500937512fdacd67442b3924d7140f6c9b9e4bb640af`. ZIP CRC valid; encrypted=False.
- Device source: [2-Log.txt](logs/2-Log.txt), lines 10435–11590; first panic line 10904. Log name `Trainyard`; launched archive `Trainyard.ipa`.
- Engine: confirmed cocos class family; CCSpriteBatchNode / CCDirectorIOS generation; exact version unknown. Dependencies: CocosDenshion, GameKit, OpenAL.
- Outcome: **panic**; confidence: confirmed missing export; client purpose probable. Role: primary. Runtime ES1.1=True; observed EAGL-return count=0; Home requested=False.
- Signature: **C2D-RUNLOOP-01** — CFRunLoopSourceCreate missing.

A worker NSThread starts and logs connection initialization before the missing source creation call. A run-loop source is not equivalent to a timer; callbacks and contexts are essential.

Next action / prediction: Decode source context/version and schedule/signal/perform/invalidate chain. Implement source0 lifecycle only if that is the observed request, including context retain/release and owning-thread delivery; do not fabricate a connection. Secondary validation with stream lifecycle is relevant, not a claim of the same callsite.

Source investigation: src/frameworks/core_foundation/cf_run_loop.rs; src/frameworks/foundation/ns_run_loop.rs; src/environment.rs.

```text
Panic at src/dyld.rs:846:9: Call to unimplemented function _CFRunLoopSourceCreate
```

Resources (counts by filename extension; not a decoder-validity claim): .caf=29, .fnt=2, .plist=5, .png=58.

## Trenches / TrenchesVS

- Identity: `com.ThunderGameWorks.Trenches`; version `1.6` / bundle build `1.6`; minimum OS `3.0`; armv6.
- Local IPA: `com.ThunderGameWorks.Trenches-iOS3.0-(Clutch-2.0.4).ipa`. SHA-256 `277f668a8d5b3c8f3586a5077a54f722a6fe766f075c3a1b3b7a9a3f60fd0364`. ZIP CRC valid; encrypted=False.
- Device source: [2-Log.txt](logs/2-Log.txt), lines 11591–12683; first panic line 11888. Log name `TrenchesVS`; launched archive `com.ThunderGameWorks.Trenches-iOS3.0-(Clutch-2.0.4).ipa`.
- Engine: confirmed CocosDenshion/CocosThunder audio integration; full cocos scene engine unconfirmed. Dependencies: CocosDenshion, OpenFeint, OpenAL.
- Outcome: **panic after presentation**; confidence: confirmed chain; native mutation contract unresolved. Role: primary investigation. Runtime ES1.1=True; observed EAGL-return count=647; Home requested=False.
- Signature: **C2D-ARRAY-01** — Array shrinks during makeObjectsPerformSelector; stale index3 sees count2.

Current events416..424 show THImageView removal at indices0,1,2 and perform-return count5->4->3->2, then invalid index3 at LR0xb278. Existing loop snapshots initial count but indexes live array. This explains the immediate chain; whether native API guarantees snapshot traversal is still a historical-contract question.

Next action / prediction: Minimal native/reference mutation-during-selector iteration test plus exact guest callsite symbolization. Compare retained snapshot, live traversal and documented mutation prohibition before changing semantics. Robokill empty-profile case is not validation for this correction.

Source investigation: src/frameworks/foundation/ns_array.rs.

```text
Panic at src/frameworks/foundation/ns_array.rs:868:51: index out of bounds: the len is 2 but the index is 3
```

Resources (counts by filename extension; not a decoder-validity claim): .fnt=1, .mp3=7, .nib=135, .plist=4, .png=289, .pvr=1, .wav=63.

## Wispin

- Identity: `9X8UEF6RBG.com.GrumpyFace.Wispin`; version `1.1.2` / bundle build `112`; minimum OS `4.0`; armv6.
- Local IPA: `Wispin_v112_os40.ipa`. SHA-256 `be0f4cc430a0340d8afe60f37c52dac4fb81384ef6e2316d97067535fa2cccad`. ZIP CRC valid; encrypted=False.
- Device source: [2-Log.txt](logs/2-Log.txt), lines 12684–13388; first panic line 12818. Log name `Wispin`; launched archive `Wispin_v112_os40.ipa`.
- Engine: confirmed cocos class family; CCSpriteBatchNode / CCDirectorIOS generation; exact version unknown. Dependencies: CocosDenshion, Box2D, GameKit, OpenAL.
- Outcome: **panic after two presentations**; confidence: confirmed GL error; possible shared cause. Role: primary. Runtime ES1.1=True; observed EAGL-return count=2; Home requested=False.
- Signature: **C2D-FBO-01** — Prior GL_INVALID_FRAMEBUFFER_OPERATION (0x506) observed at compositor cleanup.

Both games retain clear/draw candidate calls before the error while the queried current framebuffer is complete (0x8cd5). Error may belong to an earlier binding/context or guest draw; cleanup is a detector, not proven producer. Older CocosNode-based Action Pig and later CCNode Wispin give a useful cross-generation comparison.

Next action / prediction: First use existing GL history to reconstruct context/FBO/RBO storage/attachments per call. If ambiguous, enable a capped first-error experiment around clear/draw/present, preserving guest glGetError semantics. Compare before/after compositor state restoration; no error swallowing.

Source investigation: src/frameworks/opengles.rs; src/frameworks/opengles/gles_guest.rs; src/frameworks/opengles/eagl.rs; src/frameworks/core_animation/composition.rs.

```text
Panic at src/frameworks/core_animation/composition.rs:96:5: Core Animation compositor failure at stage "frame-cleanup", layer (null): prior GL error 0x506, framebuffer status 0x8cd5, status-query GL error 0x0
```

Resources (counts by filename extension; not a decoder-validity claim): .caf=79, .ccz=30, .fnt=14, .mp3=6, .nib=3, .plist=36, .png=27, .wav=2.

## ZombieSmash (originally listed as Zombie Shock)

- Identity: `com.gamedrs.ZS`; version `2.0` / bundle build `2.0`; minimum OS `3.1.3`; armv6, armv7.
- Local IPA: `ZombieSmash20.ipa`. SHA-256 `3d482e5f838eca065eb43e9ff6054015ffef568ea79874e850f3a551be5a8dd9`. ZIP CRC valid; encrypted=False.
- Device source: [2-Log.txt](logs/2-Log.txt), lines 13389–13934; first panic line 13598. Log name `ZS`; launched archive `ZombieSmash20.ipa`.
- Engine: confirmed cocos class family; CCSpriteBatchNode / CCDirectorIOS generation; exact version unknown. Dependencies: CocosDenshion, Chipmunk, Plus+, GameKit, AdMob, Flurry, OpenAL.
- Outcome: **panic**; confidence: confirmed. Role: secondary independent contract. Runtime ES1.1=True; observed EAGL-return count=0; Home requested=False.
- Signature: **C2D-SHA1-01** — CC_SHA1_Init missing.

User corrected Zombie Shock to ZombieSmash; added IPA matches com.gamedrs.ZS v2.0 and imports the digest API. It is a CommonCrypto surface gap before first presentation, not encrypted or confirmed networking failure.

Next action / prediction: Implement real incremental SHA1 init/update/final using existing cryptographic implementation after checking guest context layout/ABI. Test standard vectors, chunking, empty input and separate contexts; no fabricated digest. Preserve as independent small contract.

Source investigation: src/libc/crypto.rs; src/dyld.rs.

Identity/history qualifications: User confirmed ZombieSmash name and supplied missing IPA during audit. No longer log-only.

```text
Panic at src/dyld.rs:846:9: Call to unimplemented function _CC_SHA1_Init
```

Resources (counts by filename extension; not a decoder-validity claim): .caf=99, .ccz=22, .fnt=26, .mp3=7, .nib=7, .plist=8, .png=509, .pvr=5, .ttf=1.
