# Dev 2.02: protocol, wide-character, nil-init and causal-origin contracts

## Dev 2.01 device evidence

The complete 859,050-byte device log embeds source `17755bb`, IPA workflow
`41437480749`, and the universal bounded recorder. Its SHA-256 is
`92749cf48f9962b015857ba9a726f033976506a59794ed1bd49332f9785257d6`.
Every Dev 2.01 target passed its former blocker. Blot, Solomon's Boneyard,
Space Monkey and Spoing returned cleanly through emulator Home.

| App | Decisive Dev 2.01 evidence | Dev 2.02 decision |
|---|---|---|
| 8bitninja 1.2 | 19 loops and 17 presents; a failed bundle-resource URL is passed as null to `CFURLGetFileSystemRepresentation`, whose current implementation then treats null as an NSString. | Preserve strict behavior. Retain the tail of each failed candidate path so the requested filename survives bounded truncation. Do not make null memory valid. |
| DoDoFly | 177 loops and 176 presents; a CF dictionary created with all four callbacks null returns a value that has already been deallocated, then the guest sends `intValue`. | Preserve correct non-retaining callback semantics. Watch only values inserted into such dictionaries and retain a bounded release/autorelease/deallocation history. |
| LegendaryWar | Its KVC setter now reaches an immutable `_touchHLE_NSDictionary` and attempts to set `height`. | Do not add mutation to immutable `NSDictionary`. Retain property-list dictionary origin, requested mutability option and actual mutability for the failing receiver. |
| Peggle | The render-notify contract passes. It requests unsupported Apple 3D Mixer, gets a null component/unit, then `AudioUnitSetProperty(NULL, ...)` hits a host unwrap. | Invalid Audio Unit handles return Darwin `paramErr`. This is a verified error contract, not implementation of the 3D Mixer. |
| Tiki Totems 2 | Its executable-path contract passes and gameplay presents 62 frames. The offline news path obtains nil data/parser/result and calls `initWithDictionary:nil`. | Treat the nil source as an empty dictionary, consistent with Objective-C nil messaging during copy construction. Nil keys and values remain invalid. |
| TrenchesVS | Its signal-stack contract passes; after 819 loops and 413 presents a valid guest controller receives missing `conformsToProtocol:`. | Parse class/category protocol lists and inherited protocol lists; implement class, instance and C-runtime conformance queries. |
| ZA! 2ndWave | Its segmented-control tint contract passes and startup reaches missing `_wcsrtombs`. | Implement restartable UTF-8 wide-to-multibyte conversion, including count-only, bounded partial conversion, source updates and `EILSEQ`. |

## Implementation boundary

Dev 2.02 adds four general compatibility contracts:

- Objective-C 2 class/category protocol metadata and inherited conformance;
- `wcsrtombs` for the emulator's UTF-8 locale;
- nil-source dictionary copy initialization as an empty dictionary;
- `paramErr` for invalid Audio Unit initialize/property/start/stop calls.

It also adds three bounded diagnostics: tail-preserving failed bundle paths,
lifetime events only for values observed in non-retaining CF dictionaries, and
property-list dictionary origin/mutability records. Recording stays universal,
fixed-size and silent during play.

The build does **not** implement Apple's 3D Mixer, mutate immutable
dictionaries, retain values in null-callback CF dictionaries, fabricate a
missing resource, accept an invalid Objective-C object, or weaken null-page
checks. The three unresolved roots remain evidence-gathering cases.

## Validation and device predictions

Focused guest tests cover direct/inherited/category protocol conformance,
`wcsrtombs` restart/count/error behavior, nil dictionary copy initialization,
and invalid Audio Unit handles. Compilation and automated testing run only on
GitHub-hosted runners because local native preflight is disabled after the
November 2 Windows resource-saturation incident.

The focused device order is 8bitninja, Blot, DoDoFly, LegendaryWar, Peggle,
Solomon's Boneyard, Space Monkey, Spoing, Tiki Totems 2, TrenchesVS and ZA!
2ndWave. Stop each failing app after its first reproducible failure and use
emulator Home for non-crashing sessions. No save reset is required.
