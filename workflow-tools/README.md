# Lightweight local validation

`Invoke-IDaredLightLocalValidation.ps1` is an early-warning check for the
active checkout. It does not replace the hosted macOS Fast Preflight and it
does not build an IPA.

The script deliberately avoids Git worktrees and submodule commands. It uses
the already-populated dependencies in the active checkout, pinned Rust 1.99.1,
portable CMake 3.31.6, offline Cargo resolution, a reusable target directory,
and at most two Cargo jobs. Missing dependencies cause it to stop without
trying to download or initialize them.

The repository's existing active-checkout `target/` is reused. The retired
detached-worktree target is never reused because Cargo build-script artifacts
record absolute source paths. Targeted and broad host tests link against the
already-built local SDL static libraries in
`../.idared-toolchain/sdl2-msvc-x64/lib`; they stop with an explanation if
those libraries are absent rather than trying to build SDL or initialize a
submodule.

```powershell
# Every source candidate: compile the library and its unit-test code without
# linking or building bundled SDL.
.\workflow-tools\Invoke-IDaredLightLocalValidation.ps1 -Mode Compile

# A changed subsystem: run one or more directly relevant deterministic tests.
.\workflow-tools\Invoke-IDaredLightLocalValidation.ps1 -Mode Targeted `
    -TestFilter historical_initializer_accepts_one_other_button_title

```

Local success means only that inexpensive Windows-host checks passed. The
hosted macOS Fast Preflight, its warm Cargo cache, one viable IPA archive and
the existing IPA verification remain required.

Use `Compile` for every candidate. Use `Targeted` only for deterministic tests
directly related to the subsystem changed. All modes are capped at two Cargo
jobs, resolve
dependencies offline with `--locked`, disable incremental object caches to
limit disk growth, share one consistent static-SDL feature configuration, and
warn if the reusable target exceeds 1.5 GiB.

The complete Windows host library suite is intentionally not a local gate.
In the Codex Windows environment, its socket and temporary-directory tests can
fail because of host restrictions rather than source regressions. The hosted
macOS Fast Preflight remains the authoritative broad suite.
