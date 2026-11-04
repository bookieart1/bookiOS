# Cocos2d candidate corpus — 2026-11-04

**Dev 2.24 update (2026-11-04):** implementation now underway for the user-expanded batch: ranks 1–4 plus the two framebuffer diagnostic canaries. See [implementation and predictions](dev-2.24-implementation.md). Baseline results below are preserved; no new device pass is claimed.

37 requested games, now 37 IPAs and 37 indexed launches in three original logs. Zombie Shock was corrected by the user to ZombieSmash, whose IPA was supplied during the audit. Two encrypted executables are package blockers. All 35 eligible launches enabled the universal recorder. See [cases](cases.md) for exact source lines, histories, competing explanations and next experiments; [manifest.json](manifest.json) includes all metadata, dependencies, hashes and resource counts.

| Game | Version / min OS | Architectures | Observed result | Signature | Confidence |
|---|---|---|---|---|---|
| AbstractWar 2.0 | 2.5 / 3.0 | armv6, armv7 | panic | C2D-ALPHA-01 | confirmed |
| Addicus | 1.1.3 / 3.0 | armv6 | panic after presentation | C2D-STREAM-01 | confirmed boundary; probable upstream cause |
| Blockoban Premium | 1.0 / 2.2 | armv6 | panic after presentation | C2D-PLUS-01 | unknown origin |
| Cannon Cadets | 2.3.6 / 3.1 | armv6, armv7 | panic | C2D-ANIM-01 | confirmed |
| Clowning Around | 1.3.2 / 3.0 | armv6, armv7 | panic after one presentation | C2D-ALERT-01 | confirmed |
| Continuity 2 | 1.2 / 4.0 | armv6, armv7 | panic after presentation | C2D-KVC-01 | confirmed |
| Contre Jour HD | 1.25 / 3.0 | armv7, armv6 | panic | C2D-DLOPEN-01 | confirmed boundary; unknown requested library |
| Dirt | 2.5 / 3.1.3 | armv6, armv7 | working in reported scope | C2D-CONTROL-01 | confirmed user observation |
| Doodle Grub | 2.52 / 3.1 | armv7, armv6 | panic | C2D-COLOR-CODING-01 | confirmed missing selector; archive layout unverified |
| FastBall 2 | 1.5.3 / 3.0 | armv7, armv6 | panic | C2D-ORIENT-01 | confirmed boundary; probable transition mismatch |
| Flow Free | 1.1 / 3.0 | armv6, armv7 | panic | C2D-IAD-01 | confirmed |
| Hook Champ | 1.51 / 3.0 | armv6 | partial; blocked name entry | C2D-INPUT-01 | confirmed symptom; unknown cause |
| Infinity Field | 1.1 / 3.1.3 | armv6, armv7 | panic | C2D-NUMBER-01 | confirmed |
| iStunt Reloaded Pro | 1.8.4 / 3.0 | armv6, armv7 | partial then panic | C2D-TABLE-01 | probable producer; confirmed wrong class |
| Junk Jack Retro | 1.2.1 / 4.2 | armv7, armv6 | panic | C2D-BLOCK-01 | confirmed |
| Kiko: The Last Totem | 1.2.927 / 3.1 | armv7, armv6 | panic after presentation | C2D-STREAM-01 | confirmed boundary; probable upstream cause |
| League of Evil | 1.52 / 3.0 | armv6, armv7 | encrypted before CPU | C2D-PACKAGE-01 | confirmed |
| Legendary Wars | 1.33 / 3.1.3 | armv6, armv7 | panic after one presentation | C2D-MUTABILITY-01 | confirmed boundary; historical behavior unresolved |
| Love Catcher | 1.0 / 2.1 | armv6 | partial; white screen with audio | C2D-VISUAL-01 | confirmed symptom; unknown cause |
| Lumi | 1.31 / 4.0 | armv6, armv7 | panic after presentation | C2D-SIGNATURE-01 | confirmed |
| Mega Jump | 16.0.3 / 3.0 | armv6 | guest memory fault | C2D-NULL-01 | unknown origin |
| Mini Militia | 1.6.0 / 3.0 | armv6 | panic | C2D-IAD-01 | confirmed |
| Moonlights | DWANEO v1.0 / 2.2 | armv6 | working in reported scope | C2D-CONTROL-01 | confirmed user observation |
| Pro Zombie Soccer | 1.2 / 3.0 | armv6 | partial; clipped and unresponsive | C2D-VISUAL-02 | confirmed symptom; possible coordinate cause |
| Robokill: Rescue Titan Prime | 1.4 / 4.0 | armv7, armv7 | panic | C2D-PROFILE-01 | unknown empty-array origin |
| Silent Swords ZERO | 1.1 / 3.0 | armv6 | panic after sustained presentation | C2D-LIFETIME-01 | unknown origin |
| Stick Stunt Biker | 5.9 / 3.0 | armv6, armv7 | panic | C2D-DATE-01 | confirmed causal chain |
| StickWars | 1.8.6 / 3.0 | armv6, armv7 | encrypted before CPU | C2D-PACKAGE-01 | confirmed |
| Super Stick Golf | 1.9 / 3.1 | armv6, armv7 | panic after one presentation | C2D-ARCHIVE-01 | confirmed |
| Super Turbo Action Pig | 1.1.1 / 2.2.1 | armv6 | panic after presentation | C2D-FBO-01 | confirmed GL error; possible shared cause |
| Tap the Frog | 1.0 / 3.0 | armv6, armv7 | partial; zoomed and unresponsive; teardown panic | C2D-VISUAL-03 | confirmed symptom; possible coordinate cause |
| Thief Lupin | 1.1.7 / 3.0 | armv6, armv7 | panic after presentation | C2D-LIFETIME-02 | unknown origin |
| Tiki Totems Premium | 1.5.3 / 3.0 | armv6, armv7 | working in reported scope | C2D-CONTROL-01 | confirmed user observation |
| Trainyard | 1.66 / 3.0 | armv6, armv7 | panic | C2D-RUNLOOP-01 | confirmed missing export; client purpose probable |
| Trenches / TrenchesVS | 1.6 / 3.0 | armv6 | panic after presentation | C2D-ARRAY-01 | confirmed chain; native mutation contract unresolved |
| Wispin | 1.1.2 / 4.0 | armv6 | panic after two presentations | C2D-FBO-01 | confirmed GL error; possible shared cause |
| ZombieSmash (originally listed as Zombie Shock) | 2.0 / 3.1.3 | armv6, armv7 | panic | C2D-SHA1-01 | confirmed |

## Exact IPA mapping

| Game | Local IPA filename | Bundle identifier |
|---|---|---|
| AbstractWar 2.0 | AbstractWar - Below IOS 7.ipa | com.manucorporat.abstractwar2.0 |
| Addicus | Addicus.ipa | com.getsetgames.addicus |
| Blockoban Premium | Blockoban Premium 1.0.ipa | org.bonuslevel.blockobanpr |
| Cannon Cadets | Cannon Cadets - Below IOS 7.ipa | com.xmgstudio.cannoncadets |
| Clowning Around | Clowning Around - Below IOS 7.ipa | com.aceviral.clowningAroundPaid |
| Continuity 2 | Continuity 2 The Continuation - Below IOS 8.ipa | com.continuitygame.continuity2 |
| Contre Jour HD | Contre Jour v1.25 (iOS 3.0) (com.chillingo.contrejourhd).ipa | com.chillingo.contrejourhd |
| Dirt | Dirt 1.5.ipa | com.BrokenKings.CaveCarver |
| Doodle Grub | Doodle_Grub_v2.52_os31.ipa | com.pixowl.doodlegrub |
| FastBall 2 | Fast Ball - Below IOS 7.ipa | klikgames.fastball.v2 |
| Flow Free | com.bigduckgames.flow_1.1_ios-3.0_flow_eb55323c.ipa | com.bigduckgames.flow |
| Hook Champ | HookChamp 1.51.ipa | com.rocketcat.HookChamp |
| Infinity Field | Infinity Field - Below IOS 7.ipa | com.chillingo.infinityfieldiphone |
| iStunt Reloaded Pro | iStunt Reloaded Pro (v1.8.4) [Decrypted].ipa | com.profusionstudios.istuntpro |
| Junk Jack Retro | Junk Jack-(com.jack.xsx.junkjack)-4010-(iOS_4.2)-880f6b0cecbee5c33fb400c23479f54a.ipa | com.jack.xsx.junkjack |
| Kiko: The Last Totem | Kiko_v1.2.927_os31.ipa | com.fabulapps.kiko |
| League of Evil | League of Evil 1.52.ipa | com.ravenousgames.leagueofevil |
| Legendary Wars | Legendary.Wars v1.33.ipa | com.liv-games.legendarywars |
| Love Catcher | LoveCatch 1.0-Hexhammer.ipa | com.hyperdevbox.LoveCatch |
| Lumi | Lumi V1.31 HS.ipa | com.foundthefuture.lumi |
| Mega Jump | Mega Jump (V16.0.3).ipa | com.getsetgames.megajump |
| Mini Militia | MiniMilitia.ipa | com.chadtowns.da2 |
| Moonlights | Moonlights-(org.bonuslevel.MyGame)-DWANEO v1.0-(iOS_2.2)-cbd006bc5d3be2522517db02c0e79f44.ipa | org.bonuslevel.MyGame |
| Pro Zombie Soccer | com.chilingo.pzs_1.2_ios-3.0_pzs_e52d121d.ipa | com.chilingo.PZS |
| Robokill: Rescue Titan Prime | Robokill_Rescue_Titan_Prime_for_iPad_v1_4.ipa | com.wandake.robokilliphone |
| Silent Swords ZERO | Silent Swords Zero - Below IOS 7.ipa | com.oniricgames.silentswordszero |
| Stick Stunt Biker | Stick Stunt Biker - Below IOS 7.ipa | com.rsz.StickStuntBiker |
| StickWars | StickWars 1.8.6.ipa | com.johnehartzog.stickwars-siege |
| Super Stick Golf | com.noodlecake.superstickgolf-iOS3.1-(Clutch-2.0.4).ipa | com.noodlecake.superstickgolf |
| Super Turbo Action Pig | com.joystickgenerals.STActionPig-iOS2.2.1-(Clutch-2.0.4) 2.ipa | com.joystickgenerals.STActionPig |
| Tap the Frog | com.mentals.tapthefrog_1.0_ios-3.0_tap-the-frog_d1dda3d2.ipa | com.mentals.tapthefrog |
| Thief Lupin | Thief-(com.abluewind.lupin)-1.1.7-(iOS_3.0)-5e96ea0bf3fbf8fb683ac14bbef2d4d7.ipa | com.abluewind.lupin |
| Tiki Totems Premium | Tiki Totems.ipa | com.spokko.TikiPremium |
| Trainyard | Trainyard.ipa | ca.struct.trainyard |
| Trenches / TrenchesVS | com.ThunderGameWorks.Trenches-iOS3.0-(Clutch-2.0.4).ipa | com.ThunderGameWorks.Trenches |
| Wispin | Wispin_v112_os40.ipa | 9X8UEF6RBG.com.GrumpyFace.Wispin |
| ZombieSmash (originally listed as Zombie Shock) | ZombieSmash20.ipa | com.gamedrs.ZS |
