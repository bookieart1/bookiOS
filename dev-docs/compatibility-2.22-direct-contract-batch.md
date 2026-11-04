# Dev 2.22 direct-contract batch

Dev 2.22 follows the November 4, 2026 non-playable-library survey. It combines
eighteen independent historical contracts whose first decisive blockers were
visible in that survey. None of the implementations dispatch on a bundle ID or
game name, and the universal flight recorder remains unchanged.

## Direct contracts

| Title | First blocker | Dev 2.22 contract | Boundary |
|---|---|---|---|
| Battle Bears | `UIPasteboard valueForPasteboardType:` | Return the existing typed pasteboard value non-owningly | No new host clipboard formats |
| Dungeon Raid | search-path directory 14 | User Application Support path | User domain only |
| KAMI RETRO | `_CFBundleCreate` | Create an owned NSBundle-backed CFBundle from a file URL | Default allocator; valid local bundles only |
| Kick the Buddy | `setInitialText:` | Historical 140-code-unit validation | Twitter service remains unavailable |
| Labyrinth 2 | `___snprintf_chk` | Checked formatting bounded by the compiler-known destination | Existing formatter limitations remain |
| M.A.M.C. | `_CFURLCopyPath` | Create-rule copy of the URL path | Existing NSURL path semantics |
| Pig Shot | `_srandomdev` | Seed the BSD `random()` state from system entropy/time | Same non-cryptographic PRNG model |
| Robot Unicorn | `encodeCGPoint:forKey:` | Encode through the historical NSString geometry representation | Existing keyed-archive graph model |
| Sailboat PRO | `_nl_langinfo` | Darwin C/POSIX locale values | Non-C locale selection remains unmodelled |
| Sonic Dash | `_nl_langinfo` | Same shared locale contract | Same limitation |
| Trench Run | mutable `initWithObjects:count:` | Retaining pointer/count initializer | Normal strict object validity remains |
| Zaxxon | device-motion interval setter | Stable bounded interval state | Device motion availability remains false |

## Follow-up semantic contracts

| Title | First blocker | Dev 2.22 contract | Boundary |
|---|---|---|---|
| Cordy | `_CFWriteStreamSetClient` | Truthfully reject callback registration on unsupported write streams | No fabricated network stream or callbacks |
| Current | KVC type parser panic | Decode complete Objective-C method encodings including offsets and compounds | Existing supported NSValue conversions only |
| Freeballin' | invalid startup-orientation name | Ignore unrecognized metadata and retain the portrait default | No guess at a nonexistent orientation |
| Guerrilla Bob | `listen()` assertion on datagram socket | Return `-1`/`EOPNOTSUPP` as Darwin does | Existing TCP-listen model unchanged |
| Muffin Knight | same datagram `listen()` assertion | Same shared socket correction | Same limitation |
| NyxQuest | `setAttributes:ofItemAtPath:error:` | Validate known historical fixed metadata on existing virtual nodes | VFS still does not persist Unix modes or timestamps |

## Predictions and falsifiers

Each title should cross its listed missing symbol, selector, assertion or parser
panic. Crossing the boundary is not a promise of gameplay: a later distinct
failure falsifies only the theory that this was the title's sole blocker. If a
title fails at the exact same boundary, the ABI/export/selector implementation
is wrong and must be revisited rather than patched around.

Spoing is the only requested regression control. The picker places these
nineteen tests first and leaves the rest alphabetized.

## Cross-library leverage

The KVC parser, Foundation/Core Foundation path and bundle contracts, checked
formatting, BSD random/locale functions, array initializer and socket error
semantics can affect unrelated titles that use the same APIs. Core Motion can
help other motion-aware games that configure an interval before checking
availability. Pasteboard and legacy Twitter support are comparatively narrow.
Ignoring an invalid orientation string helps only malformed metadata and is not
a general rotation fix. No other title is promoted or predicted to work without
device evidence.
