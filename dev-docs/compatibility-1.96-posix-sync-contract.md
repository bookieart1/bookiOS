# Dev 1.96: POSIX `sync()` contract and Marblenauts follow-up

## Evidence from Dev 1.95

The focused device log embeds source `389b9d4` and workflow `41401657404`.
It verifies that the Dev 1.95 `+[NSBundle bundleForClass:nil]` correction
removed DizzyWizards' former nil-path chain: the app receives the main bundle,
its executable path and a valid UTF-8 pointer. Its next first decisive blocker
is a call to the standard Darwin/POSIX `_sync` export.

Marblenauts no longer terminates at its earlier path boundary. It remains alive
for at least 55 seconds and advances from 298 to 3,305 main-loop iterations,
but records zero EAGL presentation returns. This is an active no-present black
screen, not evidence of a deadlock. The log contains a failed open for
`Marblenauts.app//res/gfx_normal/magnifier.png`; the filesystem resolver already
normalizes empty slash components, so the doubled slash alone is not a proved
cause. The session ended with an app-resign-active event and did not produce a
flight-recorder dump.

Spoing returned cleanly through emulator Home after 8.6 seconds with 244 main
loops and 116 EAGL presentation returns.

## Implementation boundary

`sync()` now asks every currently open writable guest file to flush its backing
host file. The function preserves Darwin's void ABI. An individual host flush
failure is logged because the API has no error return; it is not converted into
success for another API and no memory or filesystem safety check is weakened.
Read-only bundle files, resources and sockets are not altered.

A guest ABI test opens a writable temporary file, writes a payload, calls
`sync()`, closes and reopens the file, and verifies the payload. This establishes
the export, calling convention and basic guest-visible persistence path. It
does not prove full host power-loss durability.

## Focused device predictions

- DizzyWizards should advance beyond the `_sync` panic. Any later blocker is a
  separate contract.
- Marblenauts may remain black because no speculative resource or rendering
  change is included. After waiting about 20 seconds, exit with the on-screen
  emulator Home button so the bounded recorder history is dumped.
- Spoing should preserve its working launch/gameplay/Home path.

Implementation status: **partial verified POSIX contract pending device test**.
Marblenauts status: **instrumentation-first active no-present investigation**.
