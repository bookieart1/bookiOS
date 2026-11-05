# Dev 2.34 retained-library return experiment

## Boundary

- Rollback baseline: `b989e54206b1a6209fdf557340c2015e3880231b` (Dev 2.33).
- This experiment changes only the process-lifetime guest-to-picker return path.
- Cold startup still performs the existing complete package scan.
- Guest `Environment` creation, execution, teardown, filesystem, memory, graphics,
  audio, threads, and Objective-C state are not retained.

## Design

After the normal cold scan, Rust retains the already-discovered picker records:
package paths, display metadata, picker classification, and decoded picker icons.
The Swift picker window, controller, model, and `UIImage` objects remain
short-lived and are released when a game is selected.

On guest return, iDared snapshots only the direct `.ipa`/`.app` entries in the
Games directory and loose `.ipa` entries in Documents, using path, kind, size,
and high-resolution modification time. It does not open an IPA, validate ZIP
entries, hash package contents, or construct guest filesystems.

- Unchanged snapshot: rebuild the inexpensive native picker from the retained
  catalog.
- Changed or unavailable snapshot, missing/purged catalog, or picker-requested
  reload: use the pre-existing complete scan.
- iOS memory warning: discard the retained catalog; the next return uses the
  complete scan.

The log identifies normal, crash, failed-launch, guest-`exit()`, external-URL,
and otherwise-unclassified returns, and records whether the retained catalog or
full-rescan path was used.

## Device benchmark

Keep the Dev 2.33 library unchanged (approximately 377 packages / 15.97 GiB),
then capture:

1. cold startup (control),
2. normal Home/exit return,
3. recoverable crash return,
4. one second launch after a completed return to ensure no guest state leaks,
5. optionally add/remove a package while a guest runs to verify full-scan
   fallback.

Return the complete iDared log. The key markers are `iDared Picker Fast
Return`, `Picker Return Mode`, `Fallback reason`, `Library cheap-check`,
`Retained apps`, and the correlated `Picker Performance` first-visible-frame
events.

## Known first-experiment limitation

The cheap identity check intentionally does not recursively walk `.app`
directories. Normal add/delete/rename/replace operations change the direct
entry identity, but an unusual in-place edit of only a nested file that leaves
the `.app` directory's own modification metadata unchanged may require the
picker's existing directory watcher or a later strengthened snapshot. IPA
archives, which dominate the current library, are checked by file path, size,
and modification time.
