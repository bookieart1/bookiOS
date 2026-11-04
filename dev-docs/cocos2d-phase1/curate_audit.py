"""Deterministically join immutable inspection evidence to reviewed conclusions.
Run after inspect_corpus.py; generates Markdown/JSON research only, never builds.
"""
import collections, gzip, hashlib, json, re
from pathlib import Path

ROOT=Path(__file__).resolve().parent
IPAS=json.loads(gzip.decompress((ROOT/'ipa-index.json.gz').read_bytes()))
LOGS=json.loads((ROOT/'log-index.json').read_text(encoding='utf8'))

# IDs describe contracts, not generic terminal symptoms. Distinct IDs in one
# workstream do not establish a shared root cause.
ROWS='''AbstractWar 2.0|com.manucorporat.abstractwar2.0|ALPHA-01|Alpha-only bitmap|panic|confirmed|primary
Addicus|com.getsetgames.addicus|STREAM-01|Offline CFStream lifecycle|panic after presentation|confirmed boundary; probable upstream cause|primary
Blockoban Premium|org.bonuslevel.blockobanpr|PLUS-01|Invalid error receiver|panic after presentation|unknown origin|primary investigation
Cannon Cadets|com.xmgstudio.cannoncadets|ANIM-01|UIKit transaction API|panic|confirmed|secondary independent contract
Clowning Around|com.aceviral.clowningAroundPaid|ALERT-01|UIKit alert lifecycle|panic after one presentation|confirmed|secondary independent contract
Continuity 2|com.continuitygame.continuity2|KVC-01|Foundation value contracts|panic after presentation|confirmed|primary
Contre Jour HD|com.chillingo.contrejourhd|DLOPEN-01|Optional dynamic-library lookup|panic|confirmed boundary; unknown requested library|primary investigation
Dirt|com.BrokenKings.CaveCarver|CONTROL-01|Working controls|working in reported scope|confirmed user observation|control
Doodle Grub|com.pixowl.doodlegrub|COLOR-CODING-01|Foundation value contracts|panic|confirmed missing selector; archive layout unverified|secondary independent contract
FastBall 2|klikgames.fastball.v2|ORIENT-01|Host orientation transition|panic|confirmed boundary; probable transition mismatch|primary
Flow Free|com.bigduckgames.flow|IAD-01|Unavailable advertising lifecycle|panic|confirmed|primary
Hook Champ|com.rocketcat.HookChamp|INPUT-01|Name-entry interaction|partial; blocked name entry|confirmed symptom; unknown cause|primary investigation
Infinity Field|com.chillingo.infinityfieldiphone|NUMBER-01|Foundation value contracts|panic|confirmed|primary
iStunt Reloaded Pro|com.profusionstudios.istuntpro|TABLE-01|Table-controller root view|partial then panic|probable producer; confirmed wrong class|primary
Junk Jack Retro|com.jack.xsx.junkjack|BLOCK-01|Foundation value contracts|panic|confirmed|secondary independent contract
Kiko: The Last Totem|com.fabulapps.kiko|STREAM-01|Offline CFStream lifecycle|panic after presentation|confirmed boundary; probable upstream cause|second generation representative
League of Evil|com.ravenousgames.leagueofevil|PACKAGE-01|Encrypted executable|encrypted before CPU|confirmed|external package blocker
Legendary Wars|com.liv-games.legendarywars|MUTABILITY-01|Immutable plist / KVC|panic after one presentation|confirmed boundary; historical behavior unresolved|primary investigation
Love Catcher|com.hyperdevbox.LoveCatch|VISUAL-01|White presentation / guest state|partial; white screen with audio|confirmed symptom; unknown cause|primary investigation
Lumi|com.foundthefuture.lumi|SIGNATURE-01|Foundation value contracts|panic after presentation|confirmed|secondary ABI representative
Mega Jump|com.getsetgames.megajump|NULL-01|Unresolved pointer producer|guest memory fault|unknown origin|primary investigation
Mini Militia|com.chadtowns.da2|IAD-01|Unavailable advertising lifecycle|panic|confirmed|second generation representative
Moonlights|org.bonuslevel.MyGame|CONTROL-01|Working controls|working in reported scope|confirmed user observation|older-generation control
Pro Zombie Soccer|com.chilingo.PZS|VISUAL-02|Orientation / coordinate conversion|partial; clipped and unresponsive|confirmed symptom; possible coordinate cause|primary investigation
Robokill: Rescue Titan Prime|com.wandake.robokilliphone|PROFILE-01|Profile-array producer|panic|unknown empty-array origin|primary investigation
Silent Swords ZERO|com.oniricgames.silentswordszero|LIFETIME-01|Invalid object origin|panic after sustained presentation|unknown origin|primary investigation
Stick Stunt Biker|com.rsz.StickStuntBiker|DATE-01|Foundation value contracts|panic|confirmed causal chain|primary
StickWars|com.johnehartzog.stickwars-siege|PACKAGE-01|Encrypted executable|encrypted before CPU|confirmed|external package blocker
Super Stick Golf|com.noodlecake.superstickgolf|ARCHIVE-01|Foundation value contracts|panic after one presentation|confirmed|primary
Super Turbo Action Pig|com.joystickgenerals.STActionPig|FBO-01|GL error provenance candidate|panic after presentation|confirmed GL error; possible shared cause|older-generation representative
Tap the Frog|com.mentals.tapthefrog|VISUAL-03|Orientation / coordinate conversion|partial; zoomed and unresponsive; teardown panic|confirmed symptom; possible coordinate cause|second generation representative
Thief Lupin|com.abluewind.lupin|LIFETIME-02|Invalid string origin|panic after presentation|unknown origin|primary investigation
Tiki Totems Premium|com.spokko.TikiPremium|CONTROL-01|Working controls|working in reported scope|confirmed user observation|control
Trainyard|ca.struct.trainyard|RUNLOOP-01|Run-loop source lifecycle|panic|confirmed missing export; client purpose probable|primary
Trenches / TrenchesVS|com.ThunderGameWorks.Trenches|ARRAY-01|Mutation during selector iteration|panic after presentation|confirmed chain; native mutation contract unresolved|primary investigation
Wispin|9X8UEF6RBG.com.GrumpyFace.Wispin|FBO-01|GL error provenance candidate|panic after two presentations|confirmed GL error; possible shared cause|primary
ZombieSmash (originally listed as Zombie Shock)|com.gamedrs.ZS|SHA1-01|CommonCrypto incremental digest|panic|confirmed|secondary independent contract'''

# blocker, causal interpretation, next discriminating action, source paths
DETAILS={
'ALPHA-01': ('CGBitmapContextCreate borrows a null CGColorSpace',
 'Recorder call #2697: data=0x3000a000, 128x128, 8 bits/component, rowBytes=128, colorSpace=NULL, bitmapInfo=7 (alpha-only). Guest LR 0x677cf lies in the supplied symbol interval starting at 0x676e4, -[CCTexture2D(Image) initPremultipliedATextureWithImage:pixelsWide:pixelsHigh:]. The source unconditionally dereferences colorSpace. This is a legal alpha-only bitmap, not an arbitrary bad pointer.',
 'Implement a genuine one-channel alpha context only after tests cover image drawing, row stride, alpha extraction, context queries, ownership and text/mask consumers. Prediction: texture initialization crosses this call; later gameplay remains unproven. A non-alpha format at the same failure would refute this specific diagnosis.',
 'src/frameworks/core_graphics/cg_bitmap_context.rs; src/frameworks/core_graphics/cg_context.rs; src/frameworks/core_graphics/cg_image.rs'),
'STREAM-01': ('Missing CFReadStreamScheduleWithRunLoop after NULL socket-stream pair',
 'Both runs enter GANTCPSocket / Google Analytics and request www.google-analytics.com:80. CFStreamCreatePairWithSocketToCFHost returns NULL streams before the missing scheduling export. Cocos initialization and EAGL returns already occur. The shared producer is a third-party analytics transport, not cocos rendering. A nil scheduling no-op alone would merely expose the next call.',
 'Trace the finite create/client/schedule/open/error/unschedule/close chain already in the recorder; implement truthful failed network streams with correct callback-context ownership and run-loop delivery, or document the historical creation-failure route first. Never invent a connection or successful analytics response. Addicus + Kiko must both leave initialization without unbounded retry; Cordy/Current are secondary outside-corpus candidates.',
 'src/frameworks/core_foundation/cf_stream.rs; src/frameworks/cf_network.rs; src/frameworks/foundation/ns_stream.rs; src/frameworks/core_foundation/cf_run_loop.rs'),
'PLUS-01': ('NGDataModel class receives NSError-like domain selector',
 'Synchronous Plus+ requests fail offline, then domain is sent to class 0xf8094, not an NSError. Framebuffer is complete and dozens of presentations precede it. Could be an out-error slot, branch/ABI error, or guest misuse; absence of service alone does not explain a class receiver.',
 'Symbolize caller LR 0x755a8 and trace the stack error-out slots 0xfffff4ac / 0xfffff4e0 across sendSynchronousRequest. Compare actual NSError written, retained, consumed. Do not add domain to NGDataModel or pretend the request succeeded.',
 'src/frameworks/foundation/ns_url_connection.rs; src/frameworks/foundation/ns_error.rs; src/objc/messages.rs'),
'ANIM-01': ('+[UIView setAnimationBeginsFromCurrentState:] missing',
 'A UIKit transaction flag is requested before any EAGL return; source has no matching selector. This says nothing yet about the main cocos scene.',
 'Implement transaction-local flag storage/reset and historical interruption semantics within supported animations; guest test nested transactions. Prediction: gets beyond this selector. Tiki Premium is a UI-transition control; do not advertise full animation fidelity from a no-op.',
 'src/frameworks/uikit/ui_view.rs; src/frameworks/core_animation'),
'ALERT-01': ('-[UIAlertView dismissWithClickedButtonIndex:animated:] missing',
 'Fresh supplementary log uses the same 51a6334 baseline and fails at line125 on a real UIAlertView. This confirms the old Dev2.12 boundary is still present. StoreKit observer/setup occurs beforehand; alert dismissal must preserve callback ordering rather than suppressing an alert.',
 'Implement programmatic dismissal through existing alert lifetime/state and native host bridge; test not-shown/already-dismissed state, delegate reentrancy, callback count and ownership. Prediction: this selector no longer blocks; service/gameplay success is unproven.',
 'src/frameworks/uikit/ui_view/ui_alert_view.rs'),
'KVC-01': ('setValue:forKey: unwraps NSValue even for object parameter @',
 'RTGStaticBoxDef receives an object-valued setter. NSObject implementation chooses unboxing from the incoming NSValue class and lacks the TYPE_ID case, despite reading the setter type. This is a direct general KVC type-dispatch error; it is independent of Legendary Wars immutable dictionaries.',
 'Preserve object identity for @ setters; only unbox when target requires scalar/struct. Guest ABI tests must preserve existing scalar behavior and nil-object handling. Prediction: Continuity crosses level-object construction; a scalar target would falsify this case.',
 'src/frameworks/foundation/ns_object.rs; src/objc.rs'),
'DLOPEN-01': ('dlopen asserts on a library outside the known-library list',
 'Crystal initialization precedes the assertion; the exact requested path is not decoded by the panic. The executable includes Crystal and cocos classes. An optional framework probe is plausible, but not verified. Not evidence that ES2 is required.',
 'Recover R0 C string from retained context or bounded dlopen(path,flags,caller) event, then compare import table and expected dlerror/NULL failure. A legal absent optional library should not assert; real guest-dylib loading is a separate task. No dummy handles for nonexistent libraries.',
 'src/libc/dlfcn.rs; src/dyld.rs'),
'CONTROL-01': ('No blocking failure in the user-reported tested scope',
 'User reports gameplay working; current log has continuing presentation and a manual Home exit. This establishes a scoped healthy control, not all levels, every save path, or every future version. The recorder alone does not verify visual correctness.',
 'Choose only by changed subsystem. Dirt: movement, rotation and pause/resume. Moonlights: older cocos scene/audio/input. Tiki Premium: newer scene/physics and transitions. Preserve saves; no demand to replay all three for a narrow unrelated patch.',
 'tests/TestApp_source; src/frameworks/opengles; src/frameworks/uikit'),
'COLOR-CODING-01': ('UIColor encodeWithCoder: missing',
 'The receiver is a valid UIColor; serialization fails before a frame is presented. This is a coding-contract gap, not proof of missing textures or nil CGColor.',
 'Determine historical keyed archive keys/class clusters and round-trip RGBA/gray colors including alpha. Keep unsupported pattern colors explicit. Do not serialize an arbitrary modern format. Prediction: archive completes and launch proceeds.',
 'src/frameworks/uikit/ui_color.rs; src/frameworks/foundation/ns_keyed_archiver.rs; src/frameworks/foundation/ns_keyed_unarchiver.rs'),
'ORIENT-01': ('SDL rejects screen orientation/display-mode size mismatch',
 'window.rs unwrap panics on a host orientation transition. Binary paths identify cocos2d-iphone-0.99.5. This is separate from the non-crashing clipped games until transform/order evidence agrees.',
 'Use existing requested/current orientation and native-window sizes at first resize. Test portrait-to-landscape while changing drawable dimensions; fix ordering/aspect contract rather than forcing a game orientation. Dirt is the relevant rotation control.',
 'src/window.rs; iphone; src/frameworks/uikit/ui_application.rs'),
'IAD-01': ('ADBannerView alloc is unimplemented',
 'Flow Free and Mini Militia both instantiate the Apple advertising view before presentation; the class and iAd dependencies are present. This is a genuine shared missing UIKit-style class, not an engine rendering problem.',
 'Implement an honest unavailable-ad lifecycle: view sizing, delegate ownership, unloaded state and supported failure callbacks. Inspect the union of selectors both apps use before coding. No ad content, success callback, or bannerLoaded=true fabrication. Predict launch passes allocation in both; later blockers remain possible.',
 'src/frameworks.rs; src/dyld.rs; src/frameworks/uikit/ui_view.rs'),
'INPUT-01': ('Name-entry page has no responsive field/back controls',
 'User observation plus ~1894 EAGL returns rules out a stopped renderer. No decisive panic. IPA has OpenFeint/Texture2D but lacks the characteristic CCNode/CCDirector class family; cocos engine attribution is unconfirmed.',
 'One field tap and one Back tap with hit-tested view, first-responder transition and native text-input request/result. Compare before versus after field tap. If delivery reaches guest and it rejects, inspect guest screen state instead of UIKit coordinates. Do not share a generic fix with other black screens.',
 'src/frameworks/uikit/ui_view.rs; src/frameworks/uikit/ui_responder.rs; src/frameworks/uikit/ui_view/ui_control/ui_text_field.rs; src/window.rs'),
'NUMBER-01': ('NSNumber numberWithUnsignedChar: missing',
 'Valid NSNumber class receives the historical unsigned-byte constructor. No EAGL return. Clear small generic boxed-value surface, independent of KVC implementation.',
 'Add correct unsigned 0..255 boxing and matching numeric conversions through existing NSNumber machinery. Test 0,127,128,255 on ARM guest ABI. Prediction: construction proceeds without sign extension.',
 'src/frameworks/foundation/ns_value.rs; src/frameworks/core_foundation/cf_number.rs'),
'TABLE-01': ('LevelTable.tableView returns plain UIView; setRowHeight: fails',
 'Log explicitly says nib load failed and fell back to plain UIView. Return provenance identifies tableView as producer. UITableViewController currently creates a table in initWithStyle only; tableView just calls view. Address reuse is recorded but the latest allocation is a new UIView, not by itself proof of use-after-free.',
 'Inspect LevelTable superclass, init route and its nib before fixing lazy loadView/root creation. Default UITableViewController must create a UITableView with delegate/dataSource when appropriate, preserving actual nib overrides. Test init, initWithStyle, initWithNibName and subclass loadView. Never add rowHeight to UIView.',
 'src/frameworks/uikit/ui_view_controller.rs; src/frameworks/uikit/ui_view_controller/ui_table_view_controller.rs; src/frameworks/uikit/ui_nib.rs'),
'BLOCK-01': ('NSDictionary enumerateKeysAndObjectsUsingBlock: missing',
 'Valid immutable dictionary requests supported enumeration. No invalid pointer evidence at this boundary. Block invocation ABI and stop flag are the risk, not dictionary mutability.',
 'General block enumeration using existing guest block invocation; test keys/value identity, BOOL* stop, empty dictionary and teardown. Do not mutate the immutable receiver or assume order.',
 'src/frameworks/foundation/ns_dictionary.rs; src/libc/dispatch.rs'),
'PACKAGE-01': ('Mach-O encryption cryptid=1 in supplied ARM slices',
 'ZIP CRC succeeds, but loader explicitly rejects encrypted executable before CPU start. Encrypted sections make negative engine-marker conclusions unreliable. Missing flight recorder is expected here, not a diagnostic failure.',
 'Needs a legitimately obtained runnable unencrypted copy, with bundle/version/hash recorded. No emulator correction or decryption attempt is included.',
 'src/mach_o.rs'),
'MUTABILITY-01': ('Immutable property-list dictionary receives KVC assignment height',
 'Same strict mutability boundary documented in older ledger. KVC object unboxing correction for Continuity does not make immutable dictionaries mutable. Producer mutability option and guest expectation must be reconciled.',
 'Compare original plist flags/options and copy/mutableCopy path against native historical behavior. Test a minimal immutable-dictionary KVC operation separately. Preserve exception semantics and bounds; do not invent setters.',
 'src/frameworks/foundation/ns_property_list_serialization.rs; src/frameworks/foundation/ns_dictionary.rs; src/frameworks/foundation/ns_object.rs'),
'VISUAL-01': ('White screen with sound; no usable input',
 'Log records ~1005 presentations, then a repeated NSThread sleepForTimeInterval:0.5 path; audio is user-observed. Projection/geometry and guest wait-state both remain candidates. Only a generic director string appears, insufficient to confirm cocos membership.',
 'Correlate a screenshot with existing last draw/viewport/matrices and guest caller near LR0x3424. Determine whether white geometry is intentionally redrawn or renderer stops after a state flag. One bounded transition capture if existing data lacks the state; do not turn on full instruction tracing.',
 'src/frameworks/opengles/gles_guest.rs; src/frameworks/opengles/eagl.rs; src/window.rs'),
'SIGNATURE-01': ('NSMethodSignature parser rejects structure encoding {',
 'Real method-signature construction reaches a parser that handles only a small primitive set. Exact aggregate shape and subsequent NSInvocation use must be decoded before ABI support is claimed.',
 'Parse balanced nested encodings and ARMv6/v7 layout/alignment, then cover register/stack and structure returns in guest tests. Prediction: signature can be constructed; a successful parse alone does not verify invocation.',
 'src/frameworks/foundation/ns_method_signature.rs; src/frameworks/foundation/ns_invocation.rs; src/abi.rs'),
'NULL-01': ('Null-page read at address0, followed by CPU MemoryError',
 'A resource/initialization sequence (including fps_images~iphone.png) precedes the fault, but the recorder does not establish a failed producer. The CPU error is downstream of the first memory fault. Addicus sharing developer/SDKs does not prove the same failure.',
 'Symbolize fault PC/LR against this exact executable; follow the source register backward through one producer/call return using existing captured bytes. If needed, watch that producer result only. Validate resource presence and conditional control flow before hypothesizing allocation/decoder semantics.',
 'src/cpu.rs; src/mem.rs; src/diagnostics.rs; src/frameworks/foundation/ns_bundle.rs'),
'VISUAL-02': ('Rotated/clipped menu and input does not respond',
 'User reports the orientation fault; ~1600 returns demonstrate ongoing presentation. Crystal/cocos code exists, but an overlay and mismatched touch/draw coordinates remain competing explanations.',
 'Use one orientation-stable launch and one tap on known menu position; compare drawable dimensions, UIWindow bounds/transform and guest locationInView coordinates. Dirt is rotation control; Tap the Frog is a comparison, not yet a proven same-cause case.',
 'src/window.rs; src/frameworks/uikit/ui_view.rs; src/frameworks/uikit/ui_touch.rs; src/frameworks/opengles/eagl.rs'),
'PROFILE-01': ('Index0 requested from empty profile array',
 'RoboKillProfile initProfiles/getProfiles and defaults keys precede objectAtIndex at LR0x193381. This is a producer-empty collection, unlike Trenches removing entries during dispatch. Filename says iPad but actual bundle is com.wandake.robokilliphone.',
 'Symbolize the profile-init branch; record defaults value class/array count and expected initial-profile creation. Compare preserved save state with a separate isolated fresh fixture only with authorization, never delete current saves. Do not return a fabricated profile.',
 'src/frameworks/foundation/ns_user_defaults.rs; src/frameworks/foundation/ns_array.rs; src/diagnostics.rs'),
'LIFETIME-01': ('hash sent to object with nil isa',
 'After >1300 returns, 0x3651a3e0 is invalid at dictionary/hash consumption. Deallocation versus corruption versus raw pointer confusion is not settled. Not evidence of a generic null-page bug.',
 'Recover retained address generations/events and symbolized insertion/caller. If origin was evicted, watch one producer object through collection retain/release and pool drain with a fixed cap. Preserve invalid-object checks.',
 'src/objc/objects.rs; src/frameworks/foundation/ns_dictionary.rs; src/diagnostics.rs'),
'DATE-01': ('Negative Duration during NSDate description',
 'Last events create dateWithTimeIntervalSince1970:0, assign GameEngineNewsFeed.dateLastDownload, then NSLog invokes NSDate.description. description calls CFAbsoluteTimeGetGregorianDate; current code feeds a negative Apple-epoch interval to unsigned Duration. This is a date-format conversion bug, not proof of bad timer/sleep scheduling.',
 'Signed epoch conversion with finite/range checks and correct negative/fraction handling; test 1970, 2000,2001 and dates before Unix epoch where supported. Prediction: news-feed logging no longer panics. Do not clamp old dates to now.',
 'src/frameworks/foundation/ns_date.rs; src/frameworks/core_foundation/time.rs'),
'ARCHIVE-01': ('NSKeyedUnarchiver rejects primitive Boolean',
 'The object table contains a plist Boolean; the source handles strings/integers/reals but omits Boolean. This is an archive representation gap, not corrupt ZIP evidence.',
 'Decode Boolean to existing NSNumber/CFBoolean semantics with archive object-table identity/ownership. Test true/false, repeated UID and nested container roundtrip. Prediction: archived state loads; gameplay not guaranteed.',
 'src/frameworks/foundation/ns_keyed_unarchiver.rs; src/frameworks/core_foundation/cf_number.rs'),
'FBO-01': ('Prior GL_INVALID_FRAMEBUFFER_OPERATION (0x506) observed at compositor cleanup',
 'Both games retain clear/draw candidate calls before the error while the queried current framebuffer is complete (0x8cd5). Error may belong to an earlier binding/context or guest draw; cleanup is a detector, not proven producer. Older CocosNode-based Action Pig and later CCNode Wispin give a useful cross-generation comparison.',
 'First use existing GL history to reconstruct context/FBO/RBO storage/attachments per call. If ambiguous, enable a capped first-error experiment around clear/draw/present, preserving guest glGetError semantics. Compare before/after compositor state restoration; no error swallowing.',
 'src/frameworks/opengles.rs; src/frameworks/opengles/gles_guest.rs; src/frameworks/opengles/eagl.rs; src/frameworks/core_animation/composition.rs'),
'VISUAL-03': ('Zoomed corner, unresponsive input; invalid controller during Home teardown',
 'Frames keep presenting (~1230); visible/input failure predates manual Home. modalViewController on invalid0xbdd10 occurs during recovery and is a second defect. Binary source paths identify cocos2d-iphone-1.0.0-rc. Treating the teardown panic as the cause of zoom would misdirect the work.',
 'Compare content scale, drawable size, root view frame and touch mapping with PZS and Dirt. Separately trace root/modal controller lifetime only on Home; do not widen invalid memory access.',
 'src/frameworks/uikit/ui_view.rs; src/frameworks/uikit/ui_view_controller.rs; src/window.rs; src/frameworks/opengles/eagl.rs'),
'LIFETIME-02': ('Missing StringHostObject at mutable string consumption',
 '0x36477b80 has no matching host string during mutation after ~270 returns. Need its actual creation and mutation receiver class; cannot assume identical cause to Silent Swords hash receiver.',
 'Link string producer, mutable-copy/initializer return and pool drain using bounded address generations. Compare decoded class at allocation versus final consumption; never treat an invalid string as empty.',
 'src/frameworks/foundation/ns_string.rs; src/objc/objects.rs; src/diagnostics.rs'),
'RUNLOOP-01': ('CFRunLoopSourceCreate missing',
 'A worker NSThread starts and logs connection initialization before the missing source creation call. A run-loop source is not equivalent to a timer; callbacks and contexts are essential.',
 'Decode source context/version and schedule/signal/perform/invalidate chain. Implement source0 lifecycle only if that is the observed request, including context retain/release and owning-thread delivery; do not fabricate a connection. Secondary validation with stream lifecycle is relevant, not a claim of the same callsite.',
 'src/frameworks/core_foundation/cf_run_loop.rs; src/frameworks/foundation/ns_run_loop.rs; src/environment.rs'),
'ARRAY-01': ('Array shrinks during makeObjectsPerformSelector; stale index3 sees count2',
 'Current events416..424 show THImageView removal at indices0,1,2 and perform-return count5->4->3->2, then invalid index3 at LR0xb278. Existing loop snapshots initial count but indexes live array. This explains the immediate chain; whether native API guarantees snapshot traversal is still a historical-contract question.',
 'Minimal native/reference mutation-during-selector iteration test plus exact guest callsite symbolization. Compare retained snapshot, live traversal and documented mutation prohibition before changing semantics. Robokill empty-profile case is not validation for this correction.',
 'src/frameworks/foundation/ns_array.rs'),
'SHA1-01': ('CC_SHA1_Init missing',
 'User corrected Zombie Shock to ZombieSmash; added IPA matches com.gamedrs.ZS v2.0 and imports the digest API. It is a CommonCrypto surface gap before first presentation, not encrypted or confirmed networking failure.',
 'Implement real incremental SHA1 init/update/final using existing cryptographic implementation after checking guest context layout/ABI. Test standard vectors, chunking, empty input and separate contexts; no fabricated digest. Preserve as independent small contract.',
 'src/libc/crypto.rs; src/dyld.rs'),
}

def esc(v): return str(v).replace('|','\\|').replace('\n',' ')
def shortpath(p): return p.rsplit('/',1)[-1]

def generation(m,bid):
    if any(e['cryptid'] for e in m['encryption']): return 'unknown: encrypted code/metadata; residual markers not conclusive'
    vs=sorted(set(v for s in m['cocos_version_strings'] for v in re.findall(r'cocos2d-iphone-([0-9][^/ ]*)',s)))
    if vs: return 'confirmed cocos classes; embedded source-path version '+', '.join(vs)+' (fork/local edits possible)'
    names=set(m['classes']+m['class_name_strings'])
    if 'CCDirector' in names and 'CCNode' in names:
        return 'confirmed cocos class family; '+('CCSpriteBatchNode / CCDirectorIOS generation' if 'CCSpriteBatchNode' in names or 'CCDirectorIOS' in names else 'CCSpriteSheet / pre-CCDirectorIOS generation')+'; exact version unknown'
    if {'Director','CocosNode'}.issubset(names): return 'confirmed older unprefixed cocos class family; exact version unknown'
    if bid=='com.ThunderGameWorks.Trenches': return 'confirmed CocosDenshion/CocosThunder audio integration; full cocos scene engine unconfirmed'
    return 'unconfirmed cocos engine: generic director/Texture2D names are insufficient'

def dependencies(m):
    names='\n'.join(m['classes']+m['class_name_strings']+m['imports']+[x['name'] for x in m['defined_symbols']])
    pats={'CocosDenshion':r'CocosDenshion|SimpleAudioEngine|CDAudioManager|CDBufferLoadRequest', 'Box2D':r'b2World|b2Body', 'Chipmunk':r'cpSpace|cpBody','OpenFeint':r'OpenFeint|OFGameProfile|OFRequest','Plus+':r'NGDataModel|NGSession|PlusPlus','Crystal':r'CCPrivateSession|CrystalPlayer','GameKit':r'GKLocalPlayer|GKScore','GoogleAnalytics':r'GANTCPSocket|GANTracker','iAd':r'ADBannerView','AdMob':r'AdMob|GADBanner','Flurry':r'Flurry','Tapjoy':r'Tapjoy','OpenAL':r'_alGenBuffers|_alcOpenDevice'}
    return [k for k,p in pats.items() if re.search(p,names)]

man=[]; cases=['# Per-game evidence and discriminating next steps','', 'Baseline: Dev 2.23, embedded `51a6334`, workflow `41778866142`. Absolute original log lines are cited; archived originals retain their line numbering. Present-return counts show execution, not proven correct pixels or playable gameplay. The source snapshot is reviewed but none of the proposed changes below is implemented in this phase.','']
for row in ROWS.splitlines():
    game,bid,sig,cluster,outcome,confidence,role=row.split('|')
    x=next(i for i in IPAS if i['info']['CFBundleIdentifier']==bid)
    l=next(i for i in LOGS if i['bundle_id']==bid)
    m=next((s for s in x['macho'] if s['architecture']=='armv7'),x['macho'][0]); info=x['info']
    raw=(ROOT/l['segment']).read_text(encoding='utf8').splitlines()
    session=raw[:next((i for i,v in enumerate(raw) if v.startswith('No app specified,')),len(raw))]
    events=[(l['start_line']+i,v) for i,v in enumerate(session) if re.search(r'flight recorder v1|flight progress ms=|flight summary|HomeButton|Panic|CGBitmapContextCreate enter|Unable to load .*plain UIView|flight array #4(1[6-9]|2[0-4]) ',v)]
    presents=max((int(v) for s in session for v in re.findall(r'EAGL-returns=(\d+)',s)),default=0)
    renderer=[v for v in session if 'Driver info:' in v and 'OpenGL ES' in v]
    encryption=any(e['cryptid'] for s in x['macho'] for e in s['encryption'])
    blocker,why,action,source=DETAILS[sig]
    architectures=', '.join(s['architecture'] for s in x['macho'])
    notes=[]
    if x['filename']!=l['ipa_filename']: notes.append('Local filename differs from launched filename; matched by exact bundle and version, not name.')
    vv=str(info.get('CFBundleShortVersionString',info.get('CFBundleVersion','unknown')))
    if l['version'] not in {vv,str(info.get('CFBundleVersion',''))}: notes.append('VERSION MISMATCH: log='+l['version']+', IPA='+vv)
    if bid=='com.gamedrs.ZS': notes.append('User confirmed ZombieSmash name and supplied missing IPA during audit. No longer log-only.')
    if bid=='com.profusionstudios.istuntpro': notes.append('LevelTable/root-view diagnosis is stronger than treating setRowHeight as an absent UIView API.')
    if bid=='com.BrokenKings.CaveCarver': notes.append('IPA filename says 1.5; actual bundle/log version 2.5.')
    if bid=='com.wandake.robokilliphone': notes.append('Filename says iPad; bundle identifier says robokilliphone. Device family in plist is authoritative.')
    data={'game':game,'ipa_present':True,'log_present':True,'ipa_filename':x['filename'],'launched_filename':l['ipa_filename'],'ipa_sha256':x['sha256'],'bundle_id':bid,'version':vv,'bundle_version':info.get('CFBundleVersion'), 'minimum_os':info.get('MinimumOSVersion','unspecified'),'architecture':architectures,'executable':x['executable'],'encrypted':encryption,'zip_crc_ok':x['zip_crc_failure'] is None,'cocos_evidence':generation(m,bid),'notable_dependencies':dependencies(m),'linked_libraries':m['dylibs'],'gl_indicators':{'fixed_function_imports':[a for a in m['imports'] if a in ['_glMatrixMode','_glVertexPointer','_glOrthof','_glTexCoordPointer']],'shader_imports':[a for a in m['imports'] if a in ['_glCreateShader','_glShaderSource','_glUseProgram']],'runtime_es11':bool(renderer)},'resource_extensions':x['extensions'],'resource_examples':x['resource_examples'],'observed_outcome':outcome,'first_likely_causal_failure':blocker,'signature_id':'C2D-'+sig,'cluster':cluster,'confidence':confidence,'representative_or_control':role,'interpretation':why,'next_action':action,'source_locations':source.split('; '),'notes':notes,'log':l['log'],'source_log':l['source'],'log_start_line':l['start_line'],'log_end_line':l['end_line'],'first_failure_line':l['panics'][0][0] if l['panics'] else None,'first_panic':l['panics'][0][1] if l['panics'] else None,'build_header':l['build_header'],'recorder_enabled':any('flight recorder v1 enabled' in v for v in session),'cpu_started':l['cpu_started'],'observed_EAGL_returns':presents,'home_requested':l['home'],'evidence_excerpt':events[-12:]}
    man.append(data)
    cases += [f'## {game}', '', f'- Identity: `{bid}`; version `{vv}` / bundle build `{info.get("CFBundleVersion")}`; minimum OS `{data["minimum_os"]}`; {architectures}.',f'- Local IPA: `{x["filename"]}`. SHA-256 `{x["sha256"]}`. ZIP CRC valid; encrypted={encryption}.', f'- Device source: [{l["log"]}](logs/{l["log"]}), lines {l["start_line"]}–{l["end_line"]}; first panic line {data["first_failure_line"] or "none"}. Log name `{l["display_name"]}`; launched archive `{l["ipa_filename"]}`.', f'- Engine: {data["cocos_evidence"]}. Dependencies: {", ".join(data["notable_dependencies"]) or "no selected marker confirmed"}.',f'- Outcome: **{outcome}**; confidence: {confidence}. Role: {role}. Runtime ES1.1={bool(renderer)}; observed EAGL-return count={presents}; Home requested={l["home"]}.', f'- Signature: **C2D-{sig}** — {blocker}.', '', why,'',f'Next action / prediction: {action}','',f'Source investigation: {source}.','']
    if notes: cases += ['Identity/history qualifications: '+' '.join(notes),'']
    if data['first_panic']: cases += ['```text',data['first_panic'],'```','']
    cases += ['Resources (counts by filename extension; not a decoder-validity claim): '+', '.join(f'{k or "[none]"}={v}' for k,v in sorted(x['extensions'].items()) if k in ['.png','.pvr','.ccz','.tmx','.fnt','.ttf','.otf','.caf','.wav','.ogg','.mp3','.m4a','.aac','.plist','.nib'])+'.','']

assert len(man)==37 and len({r['bundle_id'] for r in man})==37
assert len(IPAS)==37 and len(LOGS)==37
assert all('51a6334' in r['build_header'][0] and '41778866142' in r['build_header'][1] for r in man)
assert sum(r['recorder_enabled'] for r in man)==35
assert all((ROOT.parents[1]/p).exists() for r in man for p in r['source_locations'])
assert all(r['zip_crc_ok'] for r in man)
assert not any('VERSION MISMATCH' in n for r in man for n in r['notes'])
(ROOT/'manifest.json').write_text(json.dumps(man,indent=2,ensure_ascii=False)+'\n',encoding='utf8')
(ROOT/'cases.md').write_text('\n'.join(cases),encoding='utf8')
table=['# Cocos2d candidate corpus — 2026-11-04','', '37 requested games, now 37 IPAs and 37 indexed launches in three original logs. Zombie Shock was corrected by the user to ZombieSmash, whose IPA was supplied during the audit. Two encrypted executables are package blockers. All 35 eligible launches enabled the universal recorder. See [cases](cases.md) for exact source lines, histories, competing explanations and next experiments; [manifest.json](manifest.json) includes all metadata, dependencies, hashes and resource counts.','', '| Game | Version / min OS | Architectures | Observed result | Signature | Confidence |','|---|---|---|---|---|---|']
for r in man: table.append('| '+' | '.join(map(esc,[r['game'],r['version']+' / '+r['minimum_os'],r['architecture'],r['observed_outcome'],r['signature_id'],r['confidence']]))+' |')
table += ['','## Exact IPA mapping','','| Game | Local IPA filename | Bundle identifier |','|---|---|---|']
for r in man: table.append('| '+' | '.join(map(esc,[r['game'],r['ipa_filename'],r['bundle_id']]))+' |')
(ROOT/'manifest.md').write_text('\n'.join(table)+'\n',encoding='utf8')
stats={'games':len(man),'ipas':len(IPAS),'launches':len(LOGS),'raw_logs':len({r['log'] for r in man}),'encrypted':sum(r['encrypted'] for r in man),'crc_valid':sum(r['zip_crc_ok'] for r in man),'recorder_enabled':sum(r['recorder_enabled'] for r in man),'working_controls':[r['game'] for r in man if r['signature_id']=='C2D-CONTROL-01'],'cluster_counts':dict(collections.Counter(r['cluster'] for r in man)),'identity_warnings':{r['game']:r['notes'] for r in man if r['notes']},'checks':'Index cardinality, bundle uniqueness, embedded build/workflow and recorder eligibility assertions passed. No emulator tests/builds run.'}
(ROOT/'validation.json').write_text(json.dumps(stats,indent=2)+'\n',encoding='utf8')
print(json.dumps(stats,indent=2))
