# Dev 1.95: NSBundle nil-class fallback

## Evidence and corrected hypothesis

The Dev 1.93 universal recorder already captured the decisive DizzyWizards
sequence. `MacGetReadOnlyPath` calls `+[NSBundle bundleForClass:]` with a zero
class argument, receives `nil`, sends `executablePath` and `UTF8String` through
that nil chain, and finally faults while copying the resulting C string.

The bounded caller bytes show the game loading a literal zero into the class
argument immediately before the message send. This contradicts the earlier
hypothesis that an unknown class lookup first returned nil. No deeper
instruction recorder is justified for this particular boundary.

Apple documents `bundleForClass:` as returning the associated loadable or
framework bundle, or the main bundle when the class is not dynamically loaded
or defined in a framework. Native Foundation also returns the process main
bundle for a nil class argument. The emulator instead contained a fork-added
special case returning nil.

## Contract change

`+[NSBundle bundleForClass:]` now returns the existing main-bundle singleton
for a nil argument as well as for application classes. This matches the only
bundle model currently represented by iDared: system framework and loadable
framework bundles remain unmodeled. The change does not create a fake path,
make null memory accessible, tolerate an invalid object, or branch on a game
identifier.

## Validation

- The guest Objective-C test checks nil-class and application-class fallback
  identity against `mainBundle`.
- The independent macOS Foundation fixture checks the same identity using the
  host Foundation implementation.
- Hosted macOS and Windows library tests remain the compilation/test gate.
  The macOS native fixture, allowed-to-fail lint, branch-gated integration
  steps and unsigned iOS package must be reported separately according to
  what actually runs.

## Device prediction and focused test

1. **DizzyWizards:** should pass the read-only executable-path construction
   that currently faults and either launch farther or expose the next genuine
   blocker.
2. **Marblenauts:** should pass the matching nil bundle/path chain and either
   launch farther or expose its next blocker.
3. **Spoing:** play briefly and return through emulator Home as a broad
   Foundation/launch regression control.

Do not delete saves. Reaching a later failure is evidence that this contract
was corrected, not proof of full compatibility.
