# Dev 2.04: default audio, cooperative yield, and causal diagnostics

Dev 2.04 follows the physical-device Dev 2.03 run preserved at
`outputs/device-logs/iDared-Dev-2.03-number-audiounit-matrix-palette-batch-2026-11-02.txt`
(SHA-256 `bc6f4bc499ed6fcf2b6fff746d96458645ece9dbd2079a84e8f0a1717e834722`).
It combines two implementation-ready compatibility contracts, one independent
collection correction, and bounded evidence capture for four unresolved
titles. The universal recorder remains enabled.

## Implemented contracts

- **RemoteIO default stream format (Peggle).** A newly created Audio Unit has a
  valid global/default stream format. Rendering now uses that format's sample
  rate when the guest has installed neither an input nor output bus override.
  The existing output-override hardware-rate behavior is unchanged. This
  removes the incorrect assertion that an output override must always exist;
  it does not fabricate an unsupported component or successful audio result.
- **`pthread_yield_np` (ZA! 2ndWave).** Darwin's historical cooperative-yield
  entry point now returns success after yielding the current runnable guest
  thread through the emulator's existing scheduler. It does not busy-wait or
  yield an unrelated host thread.
- **`NSMutableArray removeObject:` duplicate removal.** Matching indexes are
  removed from highest to lowest. The prior ascending removal shifted later
  indexes and could remove the wrong element or fail when duplicates existed.
  Bounds checks remain strict.

## Bounded causal evidence

- Failed bundle resource candidates now include a fixed-size sample of actual
  entry names from their parent directories. This distinguishes a real missing
  resource from common case/suffix lookup mistakes without accepting a nil URL.
- Nested dictionaries obtained from a tracked property-list dictionary inherit
  its immutable/mutable origin record. This should expose the actual source of
  Legendary War's nested immutable `height` dictionary without making it
  writable.
- `NSInvocation` records a fixed-size lifecycle of creation, target/selector,
  argument-slot population, retention, and invoke snapshots. Argument values
  are not copied and unset slots are not fabricated for Tiki Totems 2.
- Mutable-array operations and invalid accesses retain a fixed-size identity,
  index, object and before/after-count history for Trenches. Invalid indexes
  still fail normally.

The recorder-size test ceiling is raised from the stale 128 KiB threshold to
192 KiB. All histories remain fixed-size, emit only with an ordinary recorder
dump, and do not retain guest objects or alter guest state.

## Validation boundary

The local native preflight remains disabled after the November 2 Windows
resource-saturation incident. Local validation is limited to formatting and
diff-integrity review; compilation, library tests, iOS packaging and artifact
checks run on GitHub-hosted machines. A green workflow is not treated as proof
of skipped or allowed-to-fail steps.

The device prediction is deliberately limited:

- Peggle should pass its default-format assertion and expose its next real
  boundary.
- ZA! 2ndWave should pass `_pthread_yield_np` and expose its next real boundary.
- The array correction may affect Trenches only if duplicate-removal index
  drift caused its count mismatch; otherwise the new history should identify
  the earlier mutation.
- 8bit Ninja, Legendary War, and Tiki Totems 2 are diagnostic investigations,
  not claimed compatibility fixes.
- Blot and Spoing are the focused regressions for this batch.

No save reset, network implementation, 3D Mixer fabrication, mutable-dictionary
accommodation, nil-argument substitution, out-of-range array tolerance, or
unrelated compatibility change is included.
