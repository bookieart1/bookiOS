# iDared Dev 2.23: broad contracts, bounded diagnostics, and new imports

Dev 2.23 follows the verified Dev 2.22 device run. It combines independent
historical API corrections that have precise later boundaries with four
bounded diagnostic experiments. It also adds picker-only tracking for games
imported after the user's current library baseline. There are no bundle-ID or
app-name compatibility branches.

## Compatibility contracts

| Primary titles | Change | Boundary and limitation | Prediction / falsifier |
|---|---|---|---|
| Battle Bears | `UIApplication` now stores the mask passed to `registerForRemoteNotificationTypes:`, reports it through `enabledRemoteNotificationTypes`, and clears it on unregister. | This is registration-request state; it does not claim that APNS registration or delivery succeeded. | The existing getter panic disappears. A different failure before or at the getter falsifies the identified boundary. |
| Cordy, Current | Export the write-stream schedule/unschedule functions and preserve bounded scheduling state on `NSStream`. | No network stream or callback success is fabricated. Existing offline/unsupported creation and callback limitations remain visible. | Both cross the missing symbol. Failure before scheduling falsifies the shared boundary. |
| Dungeon Raid | `NSInvocation` accepts and invokes pointer-sized Objective-C selector (`:`) arguments. | Only the already-supported invocation transport is extended; dispatch remains strict. | `operationWithTarget:andSelector:` crosses its selector argument. A failure parsing the same `:` encoding falsifies it. |
| Guerrilla Bob, Muffin Knight | Datagram `connect()` now uses real connected UDP semantics, and connected UDP `send()` works; an unconnected datagram `send()` returns `ENOTCONN`. | Offline destinations still return the existing truthful network-unreachable result. The change does not fabricate server responses. | The `SOCK_DGRAM` assertion disappears in both titles. A repeat at the same assertion falsifies it. |
| Labyrinth 2 | `UITableViewCell` stores editing state, reports `isEditing`, implements `setEditing:animated:`, and clears editing on reuse. | No editing-control rendering or animation is claimed. | The live `LevelTableCell` crosses its missing selector. |
| M.A.M.C. | Export `_CFStringGetSystemEncoding` as historical embedded Core Foundation's MacRoman encoding. | This is not a locale or preferred-language answer. | The loader crosses the missing symbol and receives the correct 32-bit value. |
| Robot Unicorn | `NSKeyedArchiver` encodes `CGRect` through the same historical string representation that the existing unarchiver decodes. | This does not expand unrelated archive classes. | The missing selector disappears and the rect round-trip succeeds. |
| Sonic Dash | Negative descriptors passed to `fcntl`, including `F_GETFL`, return `-1`/`EBADF` instead of indexing descriptor state and panicking. | Valid descriptor behavior is unchanged. | The invalid-fd panic disappears. |
| Kick the Buddy | Accept the Twitter composer completion setter while Twitter remains truthfully unavailable. | This is a partial contract. Escaping blocks cannot safely be retained until block-copy support exists, so the handler is not stored or invoked and `canSendTweet` remains false. | The setter panic disappears; a dependency on a fabricated successful tweet would not be satisfied. |
| Zaxxon | Add the device-motion start/stop/active/data polling surface. | This emulated device reports device motion unavailable, so start remains inactive and data is nil. Accelerometer values are not mislabeled as gyro/attitude data. | The unrecognized selector disappears. If the game requires real device-motion samples, it will expose that later limitation. |

During review, the proposed non-stream `connect()` error was rejected: UDP
`connect()` is valid POSIX/Darwin behavior. The implementation therefore uses a
real connected host UDP socket and supports subsequent `send()`, rather than
returning a convenient but incorrect error.

## Diagnostic experiments

All diagnostics remain bounded and produce no guest-visible value changes.

| Title | Added evidence | Question answered by the next run |
|---|---|---|
| Freeballin' | Nil `NSString compare:options:` now records receiver, options, guest PC and LR immediately before preserving the strict failure. | Which guest call site consumed nil? |
| KAMI RETRO | Failed bundle candidates now retain guest PC/LR and thread with the bounded path tail. | Which exact loop call site keeps requesting the absent virtual framework bundle? |
| Pig Shot | The newest 32 defaults lookups retain key description, domain, returned address/class, thread and guest PC/LR. | Which defaults lookup produced the later stale string, without changing its ownership? |
| Trench Run | The terminal GL visual ring grows to 64 entries and includes blend, enable/disable, color-mask, current-color and clear state. | Which state transition most closely precedes the visible full-screen corruption? |

Spoing is the single general graphics/input regression control. Sailboat PRO
left the active queue after its verified Dev 2.22 working run. NyxQuest is not
included because its established blocker is the absence of a real OpenGL ES 2
implementation, not another small API contract.

## Picker behavior

The native picker now keeps two persisted identifier sets: the known library
and the still-unseen newly imported games. On the first Dev 2.23 run, the
existing library becomes the baseline so hundreds of old games are not
incorrectly labelled new. Later imports receive a compact `NEW` badge and are
shown in a dedicated **New** filter. Launching the game or choosing **Mark as
Seen** from its long-press menu clears that status. Newness is independent of
the user's Playable/Needs Work classification.

## Validation boundary

Focused tests cover selector-valued invocation arguments, keyed `CGRect`
round trips, stream scheduling state, embedded string encoding, table-cell
editing state, remote-notification mask state, truthful unavailable device
motion, invalid `fcntl` descriptors, and connected UDP transmission. The
permanent universal recorder and strict invalid-memory/object behavior remain
enabled. Physical-device observations are still required before any title is
promoted.
