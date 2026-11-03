# Dev 2.10: resource, UIKit, runtime, and bounded audio contracts

## Device evidence

The Dev 2.09 combined device log embeds source `20d7197` and workflow run
`41578318522`. The universal recorder remained active across the eligible
launches. The exact first boundaries were:

- ADVENA and ZENONIA2: missing `CFURLCreatePropertyFromResource` after both
  titles consumed the earlier file-stream and URL-key work.
- B'Blast: missing `NSRangeFromString`.
- BouncyMouse: `UIPinchGestureRecognizer` lacked `setDelegate:`.
- Bungee Ball: `NSString` lacked `propertyListFromStringsFileFormat` after the
  matching serialization side was reached in the prior build.
- Flip's Escape: `renderbufferStorage:fromDrawable:` asserted on a nil
  drawable. Apple's contract uses nil to detach storage.
- Junny: `UILabel` lacked the `shadowColor` getter.
- The Legend of Crystal Valley: missing `gzdirect` after `gzopen` and the other
  bounded gzip stream operations became available.
- Peggle: missing `AudioConverterNew` after 645 loops and 253 presentations.
- Unmechanical: the recorder tied the null-page read directly to the unresolved
  standard runtime symbol `___stack_chk_guard`.
- DH Bowling ran for about 161 seconds and returned cleanly through emulator
  Home. The user found it playable, with intermittent graphical corruption
  that recovered by itself.
- Roboto ran for about 67 seconds with 1,724 presentations and a clean Home
  return. No sustained slowdown or crash was captured in this run.
- Spoing remained a healthy regression.

## Selected contracts

Dev 2.10 implements the smallest general contract at each established
boundary:

1. `CFURLCreatePropertyFromResource` supports the already-exported historical
   `kCFURLFileLength` key through real guest filesystem metadata. Unsupported
   keys and missing files fail with an error rather than fabricated values.
2. `NSRangeFromString` parses the Foundation range representation and returns
   the historical zero range for malformed input.
3. `UIGestureRecognizer.delegate` is stored with UIKit's non-retaining `assign`
   ownership.
4. `propertyListFromStringsFileFormat` parses bounded strings-file key/value
   data, comments, common escapes, and bare scalar text into an NSDictionary.
   Malformed input returns nil.
5. `EAGLContext renderbufferStorage:fromDrawable:nil` detaches the drawable
   retained for the currently bound renderbuffer without allocating storage.
6. `UILabel` now owns, reports, and renders `shadowColor` and `shadowOffset`.
7. `gzdirect` reports whether an existing bounded `gz*` stream is in
   transparent/uncompressed mode.
8. libSystem exports one stable, nonzero `___stack_chk_guard`. Memory faults and
   actual stack-check failures are not suppressed.
9. Audio Converter Services gets a deliberately partial create/reset/dispose
   lifecycle. Identity formats succeed. Non-identity formats return the real
   `kAudioConverterErr_FormatNotSupported` and log both ASBDs. No codec,
   resampling, or conversion success is fabricated.

DH Bowling and Roboto leave the focused batch: both completed clean runs, and
their remaining intermittent visual/performance reports do not justify an
unrelated speculative semantic change. Their status remains preserved in the
ledger. Spoing stays as the one broad regression control.

## Validation boundary

No local compiler, preflight, or build helper is used, following the user's
explicit workstation-safety instruction. Source formatting is reviewed
statically because no standalone rustfmt is installed. Hosted tests and the
unsigned iOS build must establish compilation and package results; skipped and
allowed-to-fail hosted steps remain separate from actual passes.
