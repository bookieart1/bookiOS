# Dev 2.27 exact Foundation contract batch

Date: 2026-11-04

Dev 2.27 is one coherent, user-authorized batch of twelve general historical
contracts covering fourteen requested games. No game is included merely to see
whether it still crashes. The app picker pins all fourteen titles first in
alphabetical order; the remainder of the library remains alphabetical.

| Game | General correction | Prediction / falsifier |
|---|---|---|
| 100 Chambers | Add the missing `NSInvocation.selector` accessor. | It crosses the unrecognized-selector boundary; the theory is false if the same selector is still reported missing. |
| BikeRacePro | Implement `NSFileManager URLsForDirectory:inDomains:` using sandbox search paths and file URLs. | It receives a user-domain URL rather than stopping at the missing method. |
| Carnival | Make `NSFileHandle readDataOfLength:` return short/empty data at EOF instead of asserting. | Reads near EOF continue; a repeat of the exact requested-vs-read assertion falsifies the change. |
| Dismount | Schedule ordered run-loop selectors for the next iteration, retaining target and argument until delivery. | The missing run-loop method disappears and lower orders execute first. |
| Fieldrunners | Implement historical `NSScanner scanHexInt:` prefix, overflow, optional-output and scan-location behavior. | Hex configuration input parses and advances rather than hitting the previous stub. |
| Fieldrunners 2 | Accept Foundation search-path domain bitmasks and return only available sandbox domains. | Multi-domain lookup returns the user path instead of asserting. |
| Legends of Ooo | Same general file-manager URL correction as BikeRacePro. | It crosses the same missing selector; a later blocker remains visible. |
| MX Mayhem | Export C `difftime` with double subtraction semantics. | Dynamic linking succeeds and time comparisons receive signed differences. |
| NinJump | Implement `NSInvocationOperation initWithInvocation:`, invocation ownership/access, and execution. | Invocation-backed work executes instead of stopping at construction. |
| ParaPanic HD | Implement inherited `NSData/NSMutableData subdataWithRange:` as a checked copy. | The previous missing selector disappears without exposing source storage. |
| Scribblenauts | Report no daylight saving for the emulator's deliberately fixed GMT zones. | Startup crosses the missing `NSTimeZone` query. |
| Super Hexagon | Implement `UIPasteboard containsPasteboardTypes:` against typed and standard string representations. | The capability query returns a real match result rather than failing dispatch. |
| The Sims 3 | Export range-bounded `CFStringFindWithOptions`. | The exact missing Core Foundation symbol resolves and range/options govern the match. |
| Wire Way | Same short-read correction as Carnival. | EOF produces partial/empty `NSData`, not an emulator assertion. |

Implementation status is **verified contract at the focused host-test level**
until physical testing. Menu reach, gameplay and full compatibility are not
claimed. Run-loop modes remain deliberately collapsed by the existing
scheduler; Dev 2.27 adds documented ordering, next-iteration and ownership
semantics without claiming distinct private-mode queues. Search paths expose
only the app sandbox's user domain. Time zones remain fixed GMT and therefore
correctly report no daylight saving inside that bounded model.

Physical testing should launch the fourteen titles alphabetically, stop at the
first reproducible failure in each, and return one combined log. A later error
means the old blocker was removed, not that the game is yet fully compatible.
