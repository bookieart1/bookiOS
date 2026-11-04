# Dev 2.26 — later contracts and causal probes

## Scope and evidence

This batch follows the exact Dev 2.25 phone result (`18719b57629070ee53c8775402f1851eeb346d6c`, workflow `41808379942`). Every title below has a distinct reason to be tested; none is included merely to see whether it still crashes.

| Title | Classification | Dev 2.26 change | Prediction / boundary |
|---|---|---|---|
| Cannon Cadets | General semantic correction | Implement `CGGradientCreateWithColorComponents` for RGB and monochrome spaces, including alpha, explicit/even locations, immutable stop snapshots, validation and ownership. | Crosses the missing Core Graphics export. Later rendering or gameplay behavior remains unknown. |
| Clowning Around | Partial historical UIKit contract | Add `UIProgressView` identity, style, progress, tint/image ownership and teardown. Rendering/animation remains intentionally limited. | Crosses class allocation and preserves model state without a title stub. |
| Cordy | Targeted diagnostic / historical control | Log the exact non-HTTP local-file response contract and URL at response creation. Do **not** add `statusCode` to `NSURLResponse` or fabricate HTTP success. | Confirms whether the guest still unconditionally treats a standards-correct file response as HTTP and preserves the precise caller evidence. |
| Current | Relevant regression control | No compatibility behavior is changed specifically for Current; it validates the same offline URL/stream path after the new response diagnostic. | Should retain its clean, sustained presentations. A crash in the shared URL path is a regression. |
| Guerrilla Bob | Bounded causal diagnostic | Retain only the newest 32 UIKit add/remove/alpha/hidden transitions, with class, owner, controller and guest PC/LR; dump them when Home is pressed. Hit testing is unchanged. | Identifies the producer and missing dismissal transition of the alpha-0.4 input-blocking overlay. |
| Kick the Buddy | General UIKit error semantics plus diagnostic | Make failable `UIImage` file/data initializers return `nil` for undecodable input and log only byte length plus a 16-byte fingerprint/error. | Distinguishes missing/corrupt data from a valid unsupported image format without crashing inside the decoder. No format is guessed. |
| Labyrinth 2 | General UIKit correction | Implement `UITableView cellForRowAtIndexPath:` for materialized/visible cells, returning `nil` for invalid or nonmaterialized rows. | Crosses the exact selector while preserving native non-visible behavior. |
| Mini Militia | General Foundation correction | Serialize `NSMutableData` as the `NSData` class-cluster leaf it is, retaining strict plist type validation. | Crosses property-list serialization and exposes the next real boundary. |
| Muffin Knight | Bounded causal diagnostic | Keep the newest 16 offline `connect` transitions and log destination/result plus PC/LR at the call. Networking remains genuinely offline. | Correlates the fixed guest loop with the exact failed connect caller/retry pattern without fabricating service success. |
| Sonic Dash | General POSIX correction | Give standard descriptors historical `fcntl` flag behavior; retain `EBADF`/`EINVAL` discipline for invalid descriptors and commands. | Removes the standard-descriptor `F_GETFL` table unwrap. |
| Trainyard | General Foundation correction | Give concrete `NSMutableData` the inherited byte/data factory and initializer ABI while preserving mutable backing ownership. | Removes the typed-dispatch mismatch and permits later mutation. |
| ZombieSmash | Partial historical Foundation contract | Implement `NSXMLParser abortParsing` state, stop subsequent callbacks, suppress end-document delivery and return failure. | Crosses the missing selector; parser-error object/delegate reporting remains a later completeness item. |
| Spoing | General healthy control | No title-specific change. | Must retain normal gameplay/input/presentation behavior. |

## Safety boundaries

- No bundle identifier branches, invalid-memory tolerance, exception swallowing, fake network success, synthetic HTTP status, or global touch bypass.
- The universal recorder remains enabled.
- UI and socket histories are bounded and store no strong guest references.
- `UIProgressView` and `NSXMLParser` are explicitly partial contracts; success at their former boundary does not prove complete app compatibility.

## Validation plan

Focused tests cover gradient components/locations, progress state, table lookup, mutable-data byte initialization, mutable plist leaves, standard-descriptor flags and parser cancellation state. The required local preflight, hosted test workflow, hosted unsigned IPA workflow, package identity verification and physical device observations remain separate evidence categories.

