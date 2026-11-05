[CmdletBinding()]
param(
    [ValidateSet('Compile', 'Targeted')]
    [string]$Mode = 'Compile',

    [string]$TestFilter,

    [ValidateRange(1, 2)]
    [int]$Jobs = 2,

    [string]$RepoPath = (Join-Path $PSScriptRoot '..'),

    [string]$TargetPath
)

$ErrorActionPreference = 'Stop'

$repo = (Resolve-Path -LiteralPath $RepoPath).Path
$workspace = (Resolve-Path -LiteralPath (Join-Path $repo '..')).Path
$cargo = Join-Path $env:USERPROFILE '.cargo\bin\cargo.exe'
$rustc = Join-Path $env:USERPROFILE '.cargo\bin\rustc.exe'
$devCmd = 'C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat'
$sdlLibPath = Join-Path $workspace '.idared-toolchain\sdl2-msvc-x64\lib'
if (-not $TargetPath) {
    $TargetPath = Join-Path $repo 'target'
}

foreach ($required in @(
    $cargo,
    $rustc,
    $devCmd,
    (Join-Path $repo 'Cargo.lock'),
    (Join-Path $repo 'vendor\stb\stb_image.h'),
    (Join-Path $repo 'vendor\SDL\CMakeLists.txt'),
    (Join-Path $sdlLibPath 'SDL2-static.lib'),
    (Join-Path $sdlLibPath 'SDL2main.lib')
)) {
    if (-not (Test-Path -LiteralPath $required)) {
        throw "Required local validation input is missing: $required. No dependency or submodule setup was attempted."
    }
}

$rustVersion = (& $rustc +1.99.1 --version)
if ($LASTEXITCODE -ne 0 -or $rustVersion -notmatch '^rustc 1\.99\.1\b') {
    throw "Expected pinned rustc 1.99.1, found: $rustVersion"
}

# Import only the compiler environment. This does not call Git, create a
# worktree, inspect submodules, or mutate the active checkout.
$compilerEnvironment = & cmd.exe /d /s /c "call `"$devCmd`" -no_logo >nul && set"
if ($LASTEXITCODE -ne 0) {
    throw 'Could not initialize the Visual Studio compiler environment.'
}
foreach ($line in $compilerEnvironment) {
    $separator = $line.IndexOf('=')
    if ($separator -gt 0) {
        $name = $line.Substring(0, $separator)
        $value = $line.Substring($separator + 1)
        [Environment]::SetEnvironmentVariable($name, $value, 'Process')
    }
}

$env:CARGO_TARGET_DIR = [IO.Path]::GetFullPath($TargetPath)
$env:CARGO_BUILD_JOBS = [string]$Jobs
$env:CARGO_INCREMENTAL = '0'
$env:RUSTFLAGS = "-Lnative=$sdlLibPath"

$commonCargoArguments = @(
    '+1.99.1',
    '--locked',
    '--offline',
    '--lib',
    '--no-default-features',
    '--features',
    'sdl2/static-link',
    '--jobs',
    [string]$Jobs
)
switch ($Mode) {
    'Compile' {
        # `cargo check --tests` compiles the library's normal and cfg(test)
        # forms without linking the full SDL-backed host executable.
        $cargoArguments = @('+1.99.1', 'check') + $commonCargoArguments[1..($commonCargoArguments.Length - 1)] + '--tests'
    }
    'Targeted' {
        if ([string]::IsNullOrWhiteSpace($TestFilter)) {
            throw 'Targeted mode requires -TestFilter.'
        }
        $cargoArguments = @('+1.99.1', 'test') + $commonCargoArguments[1..($commonCargoArguments.Length - 1)] + @($TestFilter, '--', '--test-threads=2')
    }
}

$timer = [Diagnostics.Stopwatch]::StartNew()
Push-Location -LiteralPath $repo
try {
    Write-Output "iDared local validation: mode=$Mode jobs=$Jobs target=$env:CARGO_TARGET_DIR"
    Write-Output "Command: cargo $($cargoArguments -join ' ')"
    & $cargo @cargoArguments
    $exitCode = $LASTEXITCODE
}
finally {
    Pop-Location
    $timer.Stop()
}

if ($exitCode -ne 0) {
    throw "Local $Mode validation failed with exit code $exitCode after $([math]::Round($timer.Elapsed.TotalSeconds, 1)) seconds."
}

$targetBytes = (Get-ChildItem -LiteralPath $env:CARGO_TARGET_DIR -Recurse -File -ErrorAction SilentlyContinue |
    Measure-Object -Property Length -Sum).Sum
$targetMiB = if ($null -eq $targetBytes) { 0 } else { [math]::Round($targetBytes / 1MB, 1) }
Write-Output "Local $Mode validation passed in $([math]::Round($timer.Elapsed.TotalSeconds, 1)) seconds with $rustVersion."
Write-Output "Reusable local Cargo target: $targetMiB MiB"
if ($targetMiB -gt 1536) {
    Write-Warning 'The reusable local Cargo target exceeds 1.5 GiB. Review it before allowing further local growth.'
}
