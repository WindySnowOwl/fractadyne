#!/usr/bin/env pwsh
<#
  coverage.ps1 - measure test coverage locally and write a report.

  CI is manual-only in this repo, so a local report is the honest version: today's testing gaps
  are inferred from structure rather than measured. This script measures them.

  Usage:
    .\coverage.ps1                 # cargo test + --selftest, HTML + lcov report
    .\coverage.ps1 -TestsOnly      # cargo test only (fast; see the WARNING below)
    .\coverage.ps1 -Livetest       # also fold in the grand-tour livetest (adds ~6 min)
    .\coverage.ps1 -Open           # open the HTML report when it finishes
    .\coverage.ps1 -Install        # install the prerequisites, then exit

  WARNING - why -TestsOnly lies about this project:
    `cargo llvm-cov` measures what `cargo test` executes. In this workspace that is the pure
    logic only: the controllers, the scripting/order/segment functions, parsers, and the schema.
    Everything that touches a GPU lives in the app's OWN harnesses (--selftest, 121 checks and
    17 goldens; --livetest, a 24-checkpoint grand tour), which run as the BUILT BINARY and are
    invisible to a plain `cargo llvm-cov test`. A tests-only number therefore understates real
    coverage badly and would argue for writing unit tests that duplicate what the harnesses
    already prove. So the default run instruments the binary too and merges everything into one
    report: `--no-report` for each phase, then a single `report` at the end.

  Prerequisites (NOT installed silently - this script probes and tells you):
    cargo-llvm-cov          cargo install cargo-llvm-cov
    llvm-tools-preview      rustup component add llvm-tools-preview
  Both are ordinary dev tooling, but `rustup` changes the active toolchain's components, and a
  `rustup toolchain install` in this repo has already moved us a compiler version and introduced
  31 new warnings unasked. So installing is an explicit choice here: pass -Install.

  Output lands in target\llvm-cov\ (target\ is gitignored):
    target\llvm-cov\html\index.html    browsable, per-file, line-by-line
    target\llvm-cov\lcov.info          for editor gutters / external tools
#>
[CmdletBinding()]
param(
  [switch]$TestsOnly,
  [switch]$Livetest,
  [switch]$Open,
  [switch]$Install
)

$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot
Set-Location $repo

function Have-Cmd($name) { return [bool](Get-Command $name -ErrorAction SilentlyContinue) }

function Test-Prereqs {
  $missing = @()
  if (-not (Have-Cmd 'cargo-llvm-cov')) {
    # `cargo llvm-cov` also resolves as a cargo subcommand shim; check both spellings.
    $sub = (& cargo --list 2>$null | Select-String -Pattern '^\s+llvm-cov' -Quiet)
    if (-not $sub) { $missing += 'cargo-llvm-cov' }
  }
  $comp = (& rustup component list --installed 2>$null) -join "`n"
  if ($comp -notmatch 'llvm-tools') { $missing += 'llvm-tools-preview' }
  return $missing
}

if ($Install) {
  Write-Host '== installing coverage prerequisites ==' -ForegroundColor Cyan
  Write-Host 'rustup component add llvm-tools-preview'
  & rustup component add llvm-tools-preview
  if ($LASTEXITCODE -ne 0) { throw "rustup component add failed ($LASTEXITCODE)" }
  Write-Host 'cargo install cargo-llvm-cov'
  & cargo install cargo-llvm-cov
  if ($LASTEXITCODE -ne 0) { throw "cargo install cargo-llvm-cov failed ($LASTEXITCODE)" }
  Write-Host 'Prerequisites installed. Re-run without -Install to measure.' -ForegroundColor Green
  exit 0
}

$missing = Test-Prereqs
if ($missing.Count -gt 0) {
  Write-Host ''
  Write-Host "Missing coverage prerequisites: $($missing -join ', ')" -ForegroundColor Yellow
  Write-Host ''
  Write-Host 'Install them with EITHER:'
  Write-Host '    .\scripts\coverage.ps1 -Install'
  Write-Host '  or, by hand:'
  Write-Host '    rustup component add llvm-tools-preview'
  Write-Host '    cargo install cargo-llvm-cov'
  Write-Host ''
  Write-Host 'Nothing was installed and nothing was measured.' -ForegroundColor Yellow
  exit 2
}

# The exe holds a lock on itself while running, which fails the build rather than the run.
$live = Get-Process fractadyne -ErrorAction SilentlyContinue
if ($live) {
  Write-Host 'fractadyne.exe is running - stopping it so the instrumented build can link.' -ForegroundColor Yellow
  $live | Stop-Process -Force
  Start-Sleep -Milliseconds 500
}

# Harnesses must not read the developer's live session: an unparseable or unexpected session
# silently falls back to defaults and a gate then measures a view it never claimed to.
$cfg = Join-Path ([System.IO.Path]::GetTempPath()) ("fractadyne-cov-" + [guid]::NewGuid().ToString('N').Substring(0,8))
New-Item -ItemType Directory -Force $cfg | Out-Null
$env:FRACTADYNE_CONFIG_DIR = $cfg

$failed = @()
try {
  Write-Host '== clearing previous coverage data ==' -ForegroundColor Cyan
  & cargo llvm-cov clean --workspace
  if ($LASTEXITCODE -ne 0) { throw "cargo llvm-cov clean failed ($LASTEXITCODE)" }

  Write-Host '== phase 1/3: cargo test (pure logic) ==' -ForegroundColor Cyan
  & cargo llvm-cov --no-report --workspace test
  if ($LASTEXITCODE -ne 0) { $failed += 'cargo test' }

  if (-not $TestsOnly) {
    Write-Host '== phase 2/3: --selftest (GPU checks + goldens) ==' -ForegroundColor Cyan
    & cargo llvm-cov --no-report run --release --bin fractadyne -- --selftest
    if ($LASTEXITCODE -ne 0) { $failed += '--selftest' }

    if ($Livetest) {
      Write-Host '== phase 3/3: --livetest grand tour (~6 min) ==' -ForegroundColor Cyan
      & cargo llvm-cov --no-report run --release --bin fractadyne -- `
          --livetest tours/grand-tour.toml --size 480x270
      if ($LASTEXITCODE -ne 0) { $failed += '--livetest' }
    } else {
      Write-Host '== phase 3/3: skipped (pass -Livetest to include the grand tour) ==' -ForegroundColor DarkGray
    }
  } else {
    Write-Host '== phases 2-3 skipped (-TestsOnly): the report below covers PURE LOGIC ONLY ==' -ForegroundColor Yellow
  }

  Write-Host '== merging into one report ==' -ForegroundColor Cyan
  & cargo llvm-cov report --summary-only
  & cargo llvm-cov report --html    | Out-Null
  & cargo llvm-cov report --lcov --output-path target\llvm-cov\lcov.info | Out-Null
}
finally {
  Remove-Item env:FRACTADYNE_CONFIG_DIR -ErrorAction SilentlyContinue
  Remove-Item -Recurse -Force $cfg -ErrorAction SilentlyContinue
}

$html = Join-Path $repo 'target\llvm-cov\html\index.html'
Write-Host ''
Write-Host "HTML report: $html"
Write-Host "lcov:        $(Join-Path $repo 'target\llvm-cov\lcov.info')"

if ($failed.Count -gt 0) {
  # A phase that failed still contributes its coverage, so the report is real but PARTIAL -
  # say so loudly rather than letting a green-looking percentage stand on a harness that died.
  Write-Host ''
  Write-Host "WARNING: these phases did not pass: $($failed -join ', ')" -ForegroundColor Red
  Write-Host 'The report covers only the code they reached before failing.' -ForegroundColor Red
}
if ($Open -and (Test-Path $html)) { Start-Process $html }
if ($failed.Count -gt 0) { exit 1 }
