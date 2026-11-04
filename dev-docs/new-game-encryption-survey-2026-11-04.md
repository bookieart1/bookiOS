# New-game encryption survey — November 4, 2026

## Scope and provenance

This is an encryption/package-status inventory only. Runtime compatibility has
not been analyzed. The user's visual observations are preserved below for a
later pass, but they are not interpreted here.

All supplied logs identify iDared implementation
`eeb7775480cb8bd7b76089fdcdf64d9f16db1b3f`, branch
`ci/unsigned-ipa-baseline`, and unsigned IPA workflow `41816961881`.

| Evidence | SHA-256 |
|---|---|
| `2C9C151A-831B-4D7C-876D-0BA6E8DFDE34/1-Log.txt` | `468651C903E48DA4DDB22E4D8C650B9ABAB40B44F4E8C6A7CC8ACCCE9B41A41D` |
| `2C9C151A-831B-4D7C-876D-0BA6E8DFDE34/2-Log.txt` | `FAD9DF4D697BDF270DC67A93819449D6B287C9821665CDC5040FEB4011D73E96` |
| `2C9C151A-831B-4D7C-876D-0BA6E8DFDE34/3-Log.txt` | `38A038811D4BAF428928AE0B4717D92505837CA87B2C0B4931988616E2995392` |
| `2C9C151A-831B-4D7C-876D-0BA6E8DFDE34/4-Log.txt` | `29562E5832856C6ECB90FE830E20CBC7BA0C5C088682D0498592376E2FC5588C` |
| `2C9C151A-831B-4D7C-876D-0BA6E8DFDE34/5-Log.txt` | `B311BD2D1971FE5DB908805ABE349FA12475A569C7AFF0430CED5398AADA794A` |
| `2C9C151A-831B-4D7C-876D-0BA6E8DFDE34/6-Log.txt` | `9782B73509BE936A724147B24B503257461E07925924871599381B61436BFA45` |
| `92424255-4FA2-4D79-AFCD-1E56AA6FF669/1-Log.txt` | `36BA7555E4686C0FF5017800D4C14C50EFF844415CB09E1B1C8C40F6B69F7318` |

The last file repeats the four launches in the sixth survey log and adds
Legends of Ooo. Legends of Ooo was already present in the fourth survey log;
the later file is retained as duplicate standalone confirmation rather than a
new title.

## Explicitly encrypted packages (10)

These ten launches end at the emulator's explicit package boundary: `The game
is encrypted. iDared 32bit can't run encrypted games.` They are package inputs,
not emulator compatibility failures.

| Display name | Version | Bundle identifier | IPA filename |
|---|---:|---|---|
| Doodle Army | 3.0.0 | `com.chadtowns.stickarmy` | `Doodle Army 3.0.0.ipa` |
| Drift Mania | 1.31 | `com.ratrodstudio.DriftmaniaLite` | `Drift Mania 1.31.ipa` |
| FlightControl | 1.9.3 | `com.firemint.flightcontrol` | `FlightControl 1.9.3.ipa` |
| Glow Hockey | 2.2.7 | `com.natenai.glowhockeylite` | `Glow Hockey 2.2.7.ipa` |
| Impossible | 1.5.9 | `com.FlukeDude.TheImpossibleGame` | `Impossible_1.5.9_AppDoc.ipa` |
| JellyCar | 1.5.4 | `com.walaber.jellycar` | `JellyCar-1.5.4.ipa` |
| Nyan Cat! | 1.3.5 | `com.ante.nyancat` | `Nyan Cat! 1.3.5.ipa` |
| Sky Burger | 2.0 | `com.eeenmachine.SkyBurger` | `SkyBurger 2.0.ipa` |
| Tiny Wings | 2.0.1 | `com.andreasilliger.tinywings` | `Tiny Wings 2.0.1.ipa` |
| TowerBloxx3D | 2.0.30 | `com.digitalchocolate.iHouseLite` | `TowerBloxx3D 2.0.30 (CaferiusARCH-LOCKED).ipa` |

## Separately unusable package (1)

| Display name | Version | Bundle identifier | Package result |
|---|---:|---|---|
| Burrito Bison | 1.2 | `com.ravenousgames.burritobison` | The binary reaches metadata parsing, but a selector name is not text; iDared explicitly reports that the package was decrypted badly. This is not classified as still encrypted, and it is not a normal runnable package. |

## All other surveyed games (59)

No explicit encryption blocker appears for these packages. This statement does
not mean that a title works, that its package is otherwise complete, or that a
runtime failure belongs to the emulator; those questions are intentionally
deferred.

100 Chambers; 100 Floors; 1112 ep.01; AC Rearmed; Back to the Future Ep 1;
BikeRacePro; BlockBlast; Boss; Carnival; CataGugl; Crash!; Cube Runner; Dark
Meadow; Dead Trigger; Dismount; Dragon's Lair; Earn to Die; FaceFighter;
Fieldrunners; Fieldrunners2; Forlorn; Granny Smith; Gun Bros.; Highway Rider;
iShoot 2; Lara Croft and the Guardian of Light; Legends of Ooo; M.HUNTER;
Mega Jump; MX Mayhem; NinJump; Paper Pilot; Paper Toss; ParaPanicHD; PES2012;
Pocket Tanks; PREDATORS; R.Racing GTI; Real Racing; RedBallPro; Samurai2;
Sandstorm2; Scribblenauts; ShadowGun; SILENT HILL; Ski Safari; SpongeBob Diner
Dash; Super Hexagon; Swordigo; The Sims 3; TheSimpsons; Thief; Trigger Fist;
Vector; Worms; Zombie Crisis 3D; ZombieCrisis; Zombieville 2; Zuma HD.

## User observations preserved for later analysis

- BlockBlast initially appears normal, fades to black, then remains
  unresponsive without an error message.
- Boss and Dead Trigger exit without an error message.
- Dragon's Lair shows a usable menu and Options; Trailer crashes, while Play
  appears to do nothing.
- Granny Smith becomes black and unresponsive until manually exited.
- Trigger Fist starts black, then displays a displaced/cropped view that is not
  meaningfully interactive.
- Worms closes iDared without an error message.
- The user described one of the two Zombie Crisis titles as “Zombie Crisis HD”:
  it reaches a menu but menu input does not respond. The exact mapping between
  that observation and bundle `com.kongzhong.zombiec` versus
  `com.kongzhong.zcp` is deliberately left unresolved until the runtime-analysis
  pass.

No source change, compatibility diagnosis, commit, push, or IPA build was made
for this inventory.

## Runtime follow-up and replacement packages

The later combined log `1A3F8B76-4D87-421B-9391-99165477E7AA/1-Log.txt`
supersedes the package-only status for three requested games:

- **JellyCar 1.5.4** now has a runnable decrypted replacement and reaches a
  deallocated/reused Objective-C address during autorelease-pool drain.
- **Sky Burger 1.2.2** now has a runnable decrypted replacement and first stops
  at missing `UIActionSheet` construction.
- **TowerBloxx 2.0.26** is a runnable full-version sibling of the encrypted
  `TowerBloxx3D` Lite bundle and first stops because `UIPickerView` lacks the
  `dataSource` contract required by its nib.
- **Nyan Cat 1.1** also has a runnable replacement and worked in the observed
  scope, but is intentionally outside the user's repair queue.

The complete runtime evidence, uncertainty boundaries, and cross-library
priority queue are preserved in `new-game-runtime-triage-2026-11-04.md`.
