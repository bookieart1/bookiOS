# iDared Dev 2.32 implementation record

Date: 2026-11-05

This build is the first real warm-cache development trial. It combines ten
independent, evidence-backed compatibility corrections. Tiki Totems Premium is
the single unchanged regression control. No app-identifier branches, crash
suppression, invalid-memory tolerance, or fabricated service success were
added.

| Game | General contract changed | Status before device testing | Prediction |
|---|---|---|---|
| 1112 ep.01 | Store and return `UIView`/`UIWindow.contentMode` using UIKit's historical default and value range. | Partial UIKit contract | Its former missing-selector blocker should disappear. |
| Doodle Grub | Encode `UIColor` through keyed RGBA/grayscale fields compatible with the existing decoder. | General NSCoding correction | Color archiving should pass its former `encodeWithCoder:` blocker. |
| Highway Rider | Implement copied `UILocalNotification` state, including `alertLaunchImage`, and export the default-sound-name constant. | Partial UIKit contract | Launch should pass the missing notification selector/symbol boundary. |
| iStunt Reloaded Pro | Make a default `UITableViewController` lazily install a real `UITableView`. | General UIKit correction | The controller should no longer expose a plain `UIView` where a table is required. |
| Junk Jack Retro | Implement snapshot-based `NSDictionary enumerateKeysAndObjectsUsingBlock:` with key, value, and stop pointer arguments. | General Foundation correction | Enumeration should execute guest blocks and honor early stop. |
| R.Racing GTI | Parse Objective-C property metadata and implement runtime property lookup/copy/accessors. | General Objective-C runtime correction | `class_getProperty` should find real guest-declared properties. |
| Real Racing | Same Objective-C property metadata/runtime correction. | General Objective-C runtime correction | Its shared property-introspection blocker should disappear. |
| Sky Burger | Implement historical `UIActionSheet` button ordering, presentation methods, indices, and delegate callbacks using the existing alert bridge. | Partial UIKit contract | The action sheet should become actionable and report the selected index. |
| Stick Stunt Biker | Implement non-ASCII MacRoman/Latin-1 conversion and failable NSString C-string semantics. | General Foundation correction | Valid legacy text should no longer hit the ASCII-only assertion. |
| TowerBloxx | Implement `UIPickerView` data source/delegate state, component/row queries, reload, and selection state. | Partial UIKit contract; rendering remains limited | The former picker contract blocker should pass and expose any later UI limitation. |
| Tiki Totems Premium | No compatibility change; selected as a brief control for Foundation/UIKit regressions. | Regression control | Launch, menu, gameplay, input, graphics, and audio should remain normal. |

The requested games are placed first in the picker in alphabetical order, and
the visible build label is Dev 2.32. Physical-device observations remain
required before any game is called fixed.
