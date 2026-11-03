# Dev 2.17: five-game causal investigation — 2026-11-03

## Evidence and scope

Baseline source is `c84fdd9758fa0102c2e4d6903847317baadb938f` on
`ci/unsigned-ipa-baseline`. Latest device evidence is **Dev 2.15**, embedded
`9fd3fb59`, IPA workflow `41642274758`. Dev 2.16 was built, not yet device-tested.
Its label does not supersede the 2.15 observations. The two archived 2.15 logs,
earlier loose device logs, 1.71 survey, longitudinal audits, source history, and
current ledger were cross-checked. `outputs/five-217-launch-evidence.json`
indexes matching launches, embedded identities, final panics, and segment hashes.
The expanded index includes original attachments and all three survey ZIPs:
134 launch entries across 72 sources, 66 distinct normalized segments. Archive
copies are not additional independent tests. Earlier session-wide virtual-memory
reservation failures are separate from the later runtime failures below.
Earlier surveys and conclusions remain intact; corrections below supersede
specific interpretations, not the evidence itself.

Upstream HEAD was checked remotely: `ca1de89c578e79746f33c1332e28efd106c334ba`,
matching the inspected local `upstream/trunk`. It lacks the CF epoch exports and
has the same incomplete request-header addition as this fork. No matching game
IPA was found in the searched local game/download locations. A complete package
resource audit is therefore unavailable; missing files are not established.

There is no demonstrated single cause shared by these five games. A null-page
fault, a stale string candidate, and an invalid dictionary value are different
boundaries. None of these latest five traces by itself requires a native `.ips`
report: the emulator captured their failures. A future whole-host termination
would require the matching `.ips`, build UUID, and timestamp.

## Per-game reconstruction and decision

### B'Blast — bounded causal diagnostic

History: earliest audit failures involved `predicateWithFormat:` and process
queries. Later versions successively reached architecture queries, timezone,
number conversion, thread dispatch, file attributes, socket errors, and layer
scale. The 1.71 survey (`50ece57`) still stopped on CFNumber type 11. These were
real intermediate contracts, not proof of a usable game. The user's last visual
result remains black screen/brief audio near launch. In 2.15 the recorder shows
48 presentations in approximately 3.5 seconds: “never presents a frame” is false,
though presenting does not prove that useful pixels were drawn.

The decisive chain is `_strstr` at LR `0x2f4a7` returning zero for haystack
`0x36553420` and needle `0x854e7`; guest arithmetic then produces `0xc9aacbe0`,
passed to `_strncpy` at `0x2f4b7`, before `_strlen(NULL)` at `0x2f4cb` faults.
Failed bundle requests precede it but do not prove which file caused the parser
failure. Audio initialization is not the first demonstrated bad boundary.

**Action:** retain the newest 16 failed `strstr` searches, with caller/thread,
addresses, and at most 80 bytes each of input and pattern. Ordinary unsuccessful
searches remain legal. This changes no return value or memory validation.

Prediction: the same crash, now with the parser's missing delimiter/pattern and
input prefix. A malformed path supports resource/path parsing; unrelated content
refutes that specific explanation. A truncated prefix may still be insufficient.
Then inspect the exact resource in the user's legitimate IPA, including case,
bundle layout, and version, before proposing filesystem or parser semantics.
No missing file, response, or sound is invented.

### JellyCar 3 — general compatibility correction

History: 1.83 failed on missing UINavigationBar. Later work provided navigation
state and bundle enumeration. Dev 2.15 fixed unknown dynamic class lookup so the
optional `iSimulate` lookup can return nil; this exposed a later CPU fault.

The new failure follows `CFAbsoluteTimeGetCurrent`: PC `0x1cc9e6`, LR
`0x1cc9d9`, R3=0. Nearby instructions load a pointer then read a double through
it. Dyld explicitly reports unresolved `_kCFAbsoluteTimeIntervalSince1970` at
`0x1fc714`. Missing epoch data is a strong inference, not a fully reconstructed
literal-pool dependency: the pool is beyond the captured instruction window.
There is no evidence here of premature Objective-C deallocation.

**Action:** export the historical double constants for the 1970 and 1904 epochs:
978307200 and 3061152000 seconds. Export guest data, not NSNumber or an extra
pointer indirection. A focused test reads both words using executed ARM guest
instructions and verifies the resulting double and calendar offsets.

Prediction: unresolved epoch warning and this time-conversion fault disappear.
The same fault with the export resolved would falsify the immediate diagnosis.
Menu/gameplay compatibility remains unproven. A related fork's JellyCar3 commit
identifies XML DOM layout and SQLite parser differences at later stages; these
are future comparisons, not adopted changes or proof our version reaches them.

### SUPERFALL — bounded causal diagnostic

History: first sample (1.82) stopped at NSNumberFormatter. Later formatting and
Foundation work enabled gameplay. Device observations establish the death
transition, not ordinary movement, as the repeatable failure. Dev 2.11–2.15
record resource-name candidates `0x3646ddf0`, `0x3646de20`, `0x3646de50`.
Latest 2.15 run reaches about 80 seconds/1985 presentations, then consumes
`0x3646de20` through NSBundle resource lookup (caller `0x13f24`). The object has
no live host record. The ordinary history had 62632 evictions; its absence does
not demonstrate that the address was never an object.

**Action:** persistent diagnostic records for up to eight configured addresses,
plus automatically observed allocated resource-name objects, sharing a 64-record
cap. SUPERFALL's options seed the three historical addresses before linking.
The runtime has no app-name branch. Each record preserves allocation/class,
previous generation, removal/free stamps and eight recent ownership events,
even after ordinary table eviction. No guest object is retained or quarantined.

Prediction: behavior is unchanged. A free/removal before the final resource call
supports stale ownership; a different class/allocation generation supports reuse;
raw allocation without ObjC registration supports a wrong pointer producer.
Address drift, capture-cap rejection, or missing prior events remain explicitly
inconclusive. There is no demonstrated shared producer with Blot, Splode, or
Peggle just because their failures also occur at transitions.

### Tiki Totems 2 — bounded causal diagnostic

History: file attributes/executable path, NSInvocation argument handling, and
dictionary removals advanced startup into menus/level selection. Latest failure
is dictionary `setObject:forKey:` (caller `0xb477`) retaining `0x30455c40`, whose
first word is zero. The valid WallObject `0x303db7d0` is a different address.
Preceding caller instructions appear to construct a 12-byte structure with a
capacity of ten and separate 40-byte backing storage. Thus “freed WallObject”
is not established. `gameLayer` also returns nil earlier; this could be a prior
initialization problem and is not silently fixed.

**Action:** retain the newest 256 raw heap allocation/free boundaries, including
allocations that never become Objective-C objects. On failure, match consumed
addresses to allocation extents and show at most eight addresses, eight matching
events each, and 16 current bytes per address. This complements existing object
and dictionary provenance; it does not change collection retention.

Prediction: a fresh 12-byte non-object allocation at the inferred constructor
supports a custom-container/raw-value or missing dispatch/category contract;
a previously registered and freed object supports lifetime error instead.
No allocation match is inconclusive due to the finite ring. If the raw-value
theory wins, inspect the caller's class/category linkage and dictionary setup
against the game binary before changing collection semantics. Ordinary
NSMutableDictionary must still retain real objects and reject invalid ones.

### Topple 2 — native-reference comparison

History: earlier logs were classified too broadly as an offline/dead-service
problem. Latest evidence is more precise: thread 1 reads an absent
`com.ngmoco.gamescatalog.etag`, then calls `addValue:nil` for `If-None-Match`.
The emulator inserts nil into its backing dictionary and faults, caller
`0x6614c`. This is request construction, before the service response.

**Action:** run a native Foundation comparison for nil header addition both with
and without an existing header. Record return state or exception without
asserting the desired outcome. Existing `setValue:nil` removal semantics do not
prove `addValue:nil` semantics. Historical documentation specifies comma
addition and case-insensitive field names but does not settle nil addition.

Until that observation is available, no runtime change is made. If native
behavior is an exception, do not swallow it. If it ignores nil, evaluate a
narrow request contract correction, noting modern macOS is not definitive proof
of iPhone OS behavior. Do not manufacture an ETag, response, cache entry, or
successful connection. Obsolete services may become a later blocker, but are
not established as the only current blocker. An unchanged device retest adds
no value unless a resulting contract correction is included.

## Diagnostic bounds, risk, and validation

The existing universal recorder remains enabled and strict checks remain fatal.
New storage is fixed: 16 search records, 256 heap events, and at most 64 pinned
lifetime records; tests bound their aggregate storage. Search capture reads at
most 160 bytes per failed search; ordinary allocation hooks append one fixed
record; ownership hooks perform bounded table/hash lookups. There is no file
output on every allocation, message, or frame. Host-only bookkeeping adds CPU
work and could affect timing; no claim of zero overhead is made before testing.
All state belongs to one Environment and is discarded at app teardown. Guest
threads share the existing serialized emulation boundary; no additional native
callback or background thread is introduced. Boundary context is the last
observed API, not a reconstructed allocation stack.

After diagnosis, remove SUPERFALL's address configuration when no longer useful.
Keep generic bounded provenance only if it produces evidence; do not enlarge it
automatically after a miss. No binary patch, fake object, nil relaxation,
network success, memory quarantine, or save reset is included.

Focused tests cover CF double data/ARM loads, search results and null-fault
preservation with diagnostics on/off, fixed search capacity, pin eviction/free/
reuse/VM overlap, raw non-object history, and option parsing. A native Foundation
probe addresses Topple separately. All compilation runs on hosted GitHub;
local validation is limited to source review, Python evidence indexing, and
`git diff --check`. Hosted results and artifact identity will be appended after
completion. Green badges alone are not accepted; skipped and allowed-to-fail
steps are checked separately. Unrelated dirty files are excluded from commits.

The first hosted compile found a test-only type-annotation error, corrected in
`d59409ef`. Its Windows suite passes 244 tests and its IPA compiles successfully.
Mac validation was cancelled before runner assignment with an explicit ARM64
capacity annotation. A retry is also queued. An optional `native_reference_only`
workflow-dispatch input now runs the native comparison alone on `macos-15-intel`;
normal push validation is unchanged. This avoids making a small disputed API
contract wait for another full emulator compilation. No local compiler is used.

## Priority and device predictions

1. JellyCar3 epoch contract: strongest sourced correction, narrow risk, reusable
   by every importer of the constants. Test launch and first level; a later
   different error proves only that this blocker was removed.
2. Topple request-header reference: cheap hosted experiment avoids another
   speculative physical test. Other request constructors are secondary users.
3. Tiki raw-value producer: strong local caller evidence; distinguish container
   misuse/dispatch from deallocation before any ownership correction.
4. B'Blast search inputs: distinguishes resource/path hypothesis from an unrelated
   parser; package contents are the next external evidence if indicated.
5. SUPERFALL persistent provenance: addresses a demonstrated coverage loss, but
   historical-address drift and earlier-than-observed lifetime changes remain.

Spoing is the single device control for universal-diagnostic overhead, rendering,
audio, transition, and relaunch. Do not replay the whole working library for
these changes. The five focused titles are kept at the front of the picker with
the control; the rest remains alphabetical. Test without deleting saves, stop
each failing game after the first reproduced failure, and send complete logs.
Device results must separately record compatibility improvement and diagnostic
information gain. No game is promoted to playable on a hosted test alone.

## Primary references

- Apple CFDate.c, historical epoch definitions:
  https://github.com/apple-oss-distributions/CF/blob/main/CFDate.c
- Apple CF time documentation (archived 2007 edition):
  https://leopard-adc.pepas.com/documentation/CoreFoundation/Reference/CFTimeUtils/CFTimeUtils.pdf
- Apple NSMutableURLRequest reference (archived):
  https://leopard-adc.pepas.com/documentation/Cocoa/Reference/Foundation/Classes/NSMutableURLRequest_Class/Reference/Reference.html
- Apple current request-header contract:
  https://developer.apple.com/documentation/foundation/nsmutableurlrequest/addvalue(_:forhttpheaderfield:)
- Swift FoundationNetworking NSURLRequest source: nonoptional Swift input is
  insufficient evidence for Objective-C nil behavior:
  https://github.com/swiftlang/swift-corelibs-foundation/blob/main/Sources/FoundationNetworking/NSURLRequest.swift
- Compared upstream revision:
  https://github.com/touchHLE/touchHLE/tree/ca1de89c578e79746f33c1332e28efd106c334ba
- Related-fork JellyCar3 work, later-stage lead only:
  https://github.com/KlugKlugTG/HyperHLE-Reborn/commit/c89c6b9
