#!/usr/bin/env pwsh
<#
  radeon-verify.ps1 - verify the 2026-08-15 device-loss fix on the machine that showed the bug.

  WHY THIS EXISTS. An RX 6800 XT lost the GPU two frames after a live mode 0->2 arithmetic
  crossover (reports/fractadyne-report-2026-08-15.txt). The fix makes the frame-budget opening
  guess derive from a measured per-step rate instead of a fixed step count. The dev RTX 3080
  CANNOT reproduce the crash - BLA hides the regime - so the fix is unproven until this runs.

  THE EXPERIMENT. Phase 5 runs the SAME binary twice over the same tour:
      A) --set MODE_RATE_UNKNOWN_MARGIN=1   restores the pre-fix behaviour exactly. With the
         margin at 1 the undivided mode-0 rate (~8.6e7 steps/ms x 40 ms = 3.46e9) clamps straight
         back to TDR_BOOTSTRAP_STEPS = 4e8, which is bit-for-bit the guess that lost the device.
      B) stock                              the fix.
  A device loss in A and survival in B is the proof. Survival in BOTH means the tour did not
  re-enter the regime, NOT that the fix works - say so rather than claiming success.

  USAGE
    .\scripts\radeon-verify.ps1                 # all phases
    .\scripts\radeon-verify.ps1 -SkipBuild      # use the binary already built
    .\scripts\radeon-verify.ps1 -Phase 5        # just the A/B repro
    .\scripts\radeon-verify.ps1 -Out D:\share\Fractadyne\reports\radeon-20260816

  Phase 5 opens a WINDOW and needs a real desktop session. Phases 1-4 are headless.
  Everything lands in -Out, ready to zip and share.
#>
[CmdletBinding()]
param(
    [string]$Out = "",
    [string]$Exe = "",
    [switch]$SkipBuild,
    [int]$Phase = 0
)

$ErrorActionPreference = 'Continue'

function Say([string]$m) { Write-Host $m }
function Head([string]$m) {
    Write-Host ""
    Write-Host ("=" * 78)
    Write-Host "  $m"
    Write-Host ("=" * 78)
}

# ---------------------------------------------------------------- output dir
if (-not $Out) {
    $stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
    $Out = Join-Path $PWD "radeon-verify-$stamp"
}
if (-not (Test-Path $Out)) { New-Item -ItemType Directory -Force -Path $Out | Out-Null }
$Out = (Resolve-Path $Out).Path
Say "output -> $Out"

# ---------------------------------------------------------------- locate exe
if (-not $Exe) {
    foreach ($c in @(
            (Join-Path $PWD 'target\release\fractadyne.exe'),
            (Join-Path $env:USERPROFILE 'fractadyne\target\release\fractadyne.exe'))) {
        if (Test-Path $c) { $Exe = $c; break }
    }
}

# ---------------------------------------------------------------- phase 0: build
if (-not $SkipBuild -and ($Phase -eq 0)) {
    Head "Phase 0: build current main"
    $bs = Join-Path $PSScriptRoot 'windows-build.ps1'
    if (Test-Path $bs) {
        & $bs
        if ($LASTEXITCODE -ne 0) { Say "WARNING: build script returned $LASTEXITCODE; continuing with whatever binary exists" }
        $cand = Join-Path $env:USERPROFILE 'fractadyne\target\release\fractadyne.exe'
        if (Test-Path $cand) { $Exe = $cand }
    } else {
        Say "no windows-build.ps1 next to this script; skipping build"
    }
}

if (-not $Exe -or -not (Test-Path $Exe)) {
    Say "ERROR: no fractadyne.exe found. Build first, or pass -Exe <path>."
    exit 2
}
Say "binary  -> $Exe"
$repo = Split-Path (Split-Path $Exe -Parent) -Parent
$repo = Split-Path $repo -Parent
Say "repo    -> $repo"

# A device loss makes the app relaunch itself. Leftovers from a previous phase would contend for
# the GPU and muddy the next measurement, so clear them between phases.
function Kill-Stragglers {
    Get-Process fractadyne -ErrorAction SilentlyContinue | ForEach-Object {
        try { $_.Kill() } catch { }
    }
    Start-Sleep -Milliseconds 400
}

# Each run gets a WIPED config dir. The app writes a session on exit, and --livetest boots into
# whatever view that session holds - on 2026-08-15 a stale deep view kept a tour from ever
# starting. Wiped, not merely created.
function New-Cfg([string]$tag) {
    $d = Join-Path $Out "cfg-$tag"
    if (Test-Path $d) { Remove-Item -Recurse -Force $d -ErrorAction SilentlyContinue }
    New-Item -ItemType Directory -Force -Path $d | Out-Null
    return $d
}

function Run-Phase {
    # -ExpectNoExit: this mode keeps its window open after the work is done (--play does), so
    # reaching the timeout is the NORMAL ending, not a failure. The 2026-08-15 run reported both
    # repro phases as TIMEOUT for exactly this reason and buried the real result.
    param([string]$Tag, [string[]]$FdArgs, [int]$TimeoutSec = 3600, [switch]$ExpectNoExit)

    Kill-Stragglers
    $cfg = New-Cfg $Tag
    $log = Join-Path $Out "$Tag.log"
    Say ("running: fractadyne " + ($FdArgs -join ' '))
    Say "  log -> $log"

    $env:FRACTADYNE_CONFIG_DIR = $cfg
    $sw = [Diagnostics.Stopwatch]::StartNew()
    $p = Start-Process -FilePath $Exe -ArgumentList $FdArgs -NoNewWindow -PassThru `
        -RedirectStandardOutput $log -RedirectStandardError "$log.err"
    $timedOut = $false
    if (-not $p.WaitForExit($TimeoutSec * 1000)) {
        $timedOut = $true
        if ($ExpectNoExit) { Say "  reached ${TimeoutSec}s - closing (expected for this mode)" }
        else { Say "  TIMEOUT after ${TimeoutSec}s - killing" }
        try { $p.Kill() } catch { }
        Start-Sleep -Milliseconds 500
    }
    $sw.Stop()

    # WARNING:Merge stderr BEFORE any early return. The first version returned straight out of the timeout
    # branch, so the device-loss check below read a near-empty .log while all 122 KB of diagnostics
    # sat in the .err file - and both A/B repro phases were scored "no device loss" without ever
    # looking at the evidence. The whole point of the run was that one boolean.
    if (Test-Path "$log.err") { Get-Content "$log.err" | Add-Content $log; Remove-Item "$log.err" -Force }
    $code = if ($timedOut) { $null } else { $p.ExitCode }
    Say ("  exit {0} in {1:N1}s" -f $code, $sw.Elapsed.TotalSeconds)

    # Harvest any crash reports this phase produced.
    $crashDir = Join-Path $cfg 'logs'
    if (Test-Path $crashDir) {
        Get-ChildItem $crashDir -Filter 'crash-*.txt' -ErrorAction SilentlyContinue | ForEach-Object {
            Copy-Item $_.FullName (Join-Path $Out ("$Tag-" + $_.Name)) -Force
            Say ("  CRASH REPORT: " + $_.Name)
        }
        $applog = Join-Path $crashDir 'fractadyne.log'
        if (Test-Path $applog) { Copy-Item $applog (Join-Path $Out "$Tag-app.log") -Force }
    }

    $lost = $false
    if (Test-Path $log) {
        $lost = (Select-String -Path $log -Pattern 'DEVICE LOST' -SimpleMatch -Quiet) -eq $true
    }
    if (-not $lost) {
        $al = Join-Path $Out "$Tag-app.log"
        if (Test-Path $al) { $lost = (Select-String -Path $al -Pattern 'DEVICE LOST' -SimpleMatch -Quiet) -eq $true }
    }
    if ($lost) { Say "  >>> DEVICE LOST detected" }

    $reportTimeout = $timedOut -and (-not $ExpectNoExit)
    return [pscustomobject]@{ Tag = $Tag; Exit = $code; Seconds = $sw.Elapsed.TotalSeconds; TimedOut = $reportTimeout; DeviceLost = $lost; Cfg = $cfg }
}

$results = @()
$want = { param($n) ($Phase -eq 0) -or ($Phase -eq $n) }

# ---------------------------------------------------------------- 1: identity
if (& $want 1) {
    Head "Phase 1: what this machine is"
    $v = & $Exe --version 2>&1 | Select-Object -Last 1
    Say "version: $v"
    $v | Out-File (Join-Path $Out 'version.txt') -Encoding ascii
    $results += Run-Phase -Tag '01-selftest-list' -FdArgs @('--selftest-list') -TimeoutSec 300
}

# ---------------------------------------------------------------- 2: goldens
if (& $want 2) {
    Head "Phase 2: self-test (goldens use the cross-GPU tolerance off the blessed card)"
    $results += Run-Phase -Tag '02-selftest' -FdArgs @('--selftest') -TimeoutSec 2400
}

# ---------------------------------------------------------------- 3: gpu primitives
if (& $want 3) {
    Head "Phase 3: --gputest (df32/floatexp primitives vs the CPU oracle)"
    Say "This card previously PRESERVED the error-free transforms on Vulkan/GL where every NVIDIA"
    Say "backend folded them, and failed two_prod 256/256 on DX12. Re-record for this build."
    $results += Run-Phase -Tag '03-gputest' -FdArgs @('--gputest') -TimeoutSec 1200
}

# ---------------------------------------------------------------- 4: offline depth ladder
if (& $want 4) {
    Head "Phase 4: torture offline depth ladder (headless; includes the e28 crossover)"
    $results += Run-Phase -Tag '04-torture-depth' -FdArgs @('--torture', 'offline/depth', '--torture-out', (Join-Path $Out 'torture')) -TimeoutSec 3600
}

# ---------------------------------------------------------------- 5: THE EXPERIMENT
if (& $want 5) {
    Head "Phase 5: A/B live crossover repro - THE POINT OF THIS SCRIPT"
    Say "Opens a window. Do not use the machine while it runs; a loaded box changes the frame"
    Say "budget and therefore the result, not merely the duration."
    Say ""
    $tour = Join-Path $repo 'tours\repro-e28-crossover.toml'
    if (-not (Test-Path $tour)) { $tour = 'tours\repro-e28-crossover.toml' }

    # The tour is ~72s of content and --play keeps the window open afterwards, so 150s covers it
    # with margin. -ExpectNoExit stops that normal ending being scored as a TIMEOUT.
    Say "A) pre-fix behaviour (MODE_RATE_UNKNOWN_MARGIN=1) - a device loss here is EXPECTED"
    $results += Run-Phase -Tag '05a-repro-OLD' -FdArgs @('--set', 'MODE_RATE_UNKNOWN_MARGIN=1', '--play', $tour) -TimeoutSec 150 -ExpectNoExit

    Kill-Stragglers
    Start-Sleep -Seconds 5

    Say ""
    Say "B) stock (the fix) - survival here is the result we are after"
    $results += Run-Phase -Tag '05b-repro-NEW' -FdArgs @('--play', $tour) -TimeoutSec 150 -ExpectNoExit

    # ------------------------------------------------------------------ 5c/5d
    # The 2026-08-15 attempt crossed cleanly with bla_skip=5,590,609 -- BLA was LIVE, so nominal
    # steps were a small fraction of real cost and the frame was never expensive enough to be
    # dangerous. The field crash had bla_skip=0 at the fatal frames: no valid BLA tree yet, so
    # nominal steps WERE real cost.
    #
    # FRACTADYNE_NO_PREFETCH=1 is the instrument for this. It disables reference prefetching and
    # leaves the reactive path alone -- which is what a person hand-zooming at depth actually gets,
    # and is how the original crash was produced. Without it the tour arrives at the crossover with
    # a prefetched reference and a ready BLA tree, i.e. the safe version of the same journey.
    Kill-Stragglers
    Start-Sleep -Seconds 5
    Say ""
    Say "C) pre-fix + NO PREFETCH - the reactive path a hand-zoom actually takes"
    $env:FRACTADYNE_NO_PREFETCH = '1'
    $results += Run-Phase -Tag '05c-repro-OLD-noprefetch' -FdArgs @('--set', 'MODE_RATE_UNKNOWN_MARGIN=1', '--play', $tour) -TimeoutSec 150 -ExpectNoExit

    Kill-Stragglers
    Start-Sleep -Seconds 5
    Say ""
    Say "D) stock + NO PREFETCH - the pair that matters if C loses the device"
    $results += Run-Phase -Tag '05d-repro-NEW-noprefetch' -FdArgs @('--play', $tour) -TimeoutSec 150 -ExpectNoExit
    Remove-Item Env:\FRACTADYNE_NO_PREFETCH -ErrorAction SilentlyContinue
}

# ---------------------------------------------------------------- verdict
Head "Summary"
$results | ForEach-Object {
    $d = if ($_.DeviceLost) { 'DEVICE-LOST' } elseif ($_.TimedOut) { 'TIMEOUT' } else { 'ok' }
    Say ("  {0,-20} exit={1,-6} {2,8:N1}s  {3}" -f $_.Tag, $_.Exit, $_.Seconds, $d)
}
$results | Format-Table -AutoSize | Out-File (Join-Path $Out 'summary.txt') -Encoding ascii

function Verdict([string]$label, $old, $new) {
    if (-not $old -or -not $new) { return }
    Say ""
    if ($old.DeviceLost -and -not $new.DeviceLost) {
        Say "$label : reproduced pre-fix and SURVIVED on stock. That is the proof."
    } elseif (-not $old.DeviceLost -and -not $new.DeviceLost) {
        Say "$label : INCONCLUSIVE - neither run lost the device, so the tour did not re-enter"
        Say "  the killing regime. This does NOT show the fix works."
    } elseif ($old.DeviceLost -and $new.DeviceLost) {
        Say "$label : BOTH lost the device - the fix is INSUFFICIENT here. Most valuable outcome"
        Say "  to report; send the whole directory."
    } else {
        Say "$label : stock lost it but pre-fix did not. Unexpected; treat as a real finding."
    }
}

Verdict 'RESULT (prefetch on) ' ($results | Where-Object { $_.Tag -eq '05a-repro-OLD' }) ($results | Where-Object { $_.Tag -eq '05b-repro-NEW' })
Verdict 'RESULT (no prefetch) ' ($results | Where-Object { $_.Tag -eq '05c-repro-OLD-noprefetch' }) ($results | Where-Object { $_.Tag -eq '05d-repro-NEW-noprefetch' })

# Whatever the verdict, report the ONE number that says whether the regime was even entered.
Say ""
Say "Did the run reach the killing regime? Check bla_skip just after the crossover:"
foreach ($t in @('05a-repro-OLD', '05b-repro-NEW', '05c-repro-OLD-noprefetch', '05d-repro-NEW-noprefetch')) {
    $lg = Join-Path $Out "$t.log"
    if (-not (Test-Path $lg)) { continue }
    $sw = Select-String -Path $lg -Pattern 'arithmetic mode 0 . 2' | Select-Object -First 1
    if (-not $sw) { Say ("  {0,-26} no 0->2 crossover found - the tour never crossed" -f $t); continue }
    $after = Get-Content $lg | Select-Object -Skip $sw.LineNumber | Where-Object { $_ -match 'bla_skip=(\d+)' } | Select-Object -First 1
    if ($after -match 'bla_skip=(\d+)') {
        $v = [int64]$Matches[1]
        $note = if ($v -eq 0) { 'BLA NOT live - this IS the killing regime' } else { 'BLA live - safe variant, not the crash regime' }
        Say ("  {0,-26} bla_skip={1,-12} {2}" -f $t, $v, $note)
    } else {
        Say ("  {0,-26} crossed, but no frame diagnostics after it" -f $t)
    }
}

Say ""
Say "Zip and share: $Out"
Say "Anything with 'crash-' in the name is a full crash report."
