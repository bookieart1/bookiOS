# Dev 2.30 picker performance instrumentation

Status: instrumentation-only physical-device experiment.

Starting implementation commit: 94b8d061cc8563a8fbe7e07803cb97fb5fa6f924.

External baseline from the 2026-11-04 screen recording:

- cold startup pure black: approximately 14.167 seconds;
- return from Axe in Face to the picker pure black: approximately 14.167 seconds;
- each transition is approximately 14.5 seconds including surrounding
  transition/orientation settling.

## Experimental boundary

This build does not retain picker state, add an index, cache metadata or icons,
change package inspection, make filesystem creation lazy, reconcile the library
asynchronously, or add a loading visual. Existing discovery, validation,
sorting, categorization, launch, save, and exit behavior is intentionally
preserved.

The build adds bounded monotonic timing around:

- loose-import and Games-directory discovery;
- ZIP opening, full archive-entry validation, app-bundle discovery, and entry
  count;
- Info.plist reading/parsing and full guest-filesystem construction;
- localization and icon candidate/read/decode/processing work;
- picker sorting and Rust-to-Swift preparation;
- Swift JSON/image/model/window/controller construction, first layout, first
  visible picker frame, and orientation completion;
- selected-game package reopening, runtime filesystem construction, guest
  Environment creation, CPU start, exit request, execution end, and Environment
  teardown.

Each scan reports attempted/loaded/skipped package counts, median and p95 package
inspection time, and the five slowest packages with bounded stage breakdowns.
Scan identifiers join Rust and Swift records. The first-visible-frame record
also includes the preceding Rust scan so it can be compared directly with the
external video interval.

## Interpretation limits

- viewDidAppear is the nearest safe native proxy for a visible picker frame;
  it is not a photodiode measurement of pixels on the display.
- Initial orientation stability is recorded on the next main-loop turn after
  viewDidAppear; explicit rotations also record their transition completion.
- Failed package inspection retains its total time but may not expose every
  internal child stage when the failure returns before the stage timing object.
- The instrumentation adds clock reads, bounded aggregation, and concise log
  output. It deliberately avoids per-frame or unbounded per-entry logging.

## Physical-device procedure

Preserve the same Games and Saves folders. Force-close iDared Dev, begin a
screen recording, launch Dev 2.30, wait for the picker to settle, launch the
same baseline title (Axe in Face), exit it by the same in-emulator Home/close
control, wait for the picker to settle, and stop recording. Export the complete
Log.txt and Log.previous.txt files along with the recording.

No performance conclusion is valid until the external recording and internal
scan/return summaries from that run are reconciled.
