# iDared Dev 2.43 physical test plan

Use one continuous screen recording and retain the matching `Log.txt`. The ten
apps below are promoted to the beginning of the picker in this same order. The
installed title for Bring Me Sandwiches is displayed as **Sandwiches!!**.

| Game | Purpose | What to do | What to observe |
|---|---|---|---|
| Blot | Working regression control | Launch, enter gameplay, interact for about 30 seconds, then exit normally to the picker. | Gameplay, input, rendering, audio, and return remain normal; no new crash or delay. |
| Bring Me Sandwiches | Test `NSKeyedUnarchiver finishDecoding` | Launch, continue past the prior startup/archive boundary, enter the first playable sequence if available, then exit normally. | Whether launch advances beyond the former `finishDecoding` failure; any next error, hang, visual fault, or successful gameplay. |
| Continuity 2 | Test `NSArray arrayByAddingObjectsFromArray:` | Launch, choose Play, pass the instructions, and start Level 1-1. Move through the level briefly if it opens. | Whether Level 1-1 advances beyond the former missing-array-selector crash; rendering, movement, and any next exact stopping point. |
| Implode! | Test programmatic `UITableView` selection | Launch past the Loading frame, navigate the first menu/profile or level selection, and attempt to start gameplay. | Whether startup advances beyond the table-selection boundary; menu selection/scrolling, duplicate callbacks, crash, or gameplay. |
| Luxor | Test per-Class `NSObject` version state | Launch and continue through the first menus into gameplay if possible. | Whether all inherited `+version` calls complete and the game advances beyond its immediate launch crash; any next blocker. |
| Pig Rush | Test `NSKeyedUnarchiver finishDecoding` after the Dev 2.42 entropy fix | Launch, start a run, play briefly, and exit normally. | Whether it advances beyond the new unarchiver boundary into usable gameplay; input, rendering, save/archive behavior, or next error. |
| Ski Safari | Test true `NSMutableDictionary` subclass allocation/init | Launch through the intro, enter the menu, start a run, and play briefly. | Whether startup advances beyond `ApItems` allocation/init without losing subclass behavior; gameplay, graphics, or the next blocker. |
| Steam Wars | Test `_sigsetjmp` / `_siglongjmp` compatibility exports | Launch through its startup milestones and attempt to begin gameplay. Exercise any ordinary retry/back path if available. | Whether it advances beyond `_sigsetjmp`; whether a recovery path using `_siglongjmp` works, or the exact next failure. |
| ZA! 2ndWave | Test reentrant programmatic table selection | Launch Survival, choose New Game, and attempt to reach gameplay. | Whether row selection and requested scrolling work without a synthesized tap callback or crash. Record the pre-existing corrupted text separately. |
| Zombieville 2 | Test invalid pthread-key error handling | Launch, start a game, move and interact briefly, then exit normally if possible. | Whether key zero returns an error instead of terminating iDared; gameplay progress and the exact next blocker, if any. |

Do not classify a title as fully compatible from launch alone. For every title,
record the furthest visible state, whether iDared itself closed, whether an error
screen appeared, and whether return to the picker remained fast.
