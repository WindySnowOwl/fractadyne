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
    param([string]$Tag, [string[]]$FdArgs, [int]$TimeoutSec = 3600)

    Kill-Stragglers
    $cfg = New-Cfg $Tag
    $log = Join-Path $Out "$Tag.log"
    Say ("running: fractadyne " + ($FdArgs -join ' '))
    Say "  log -> $log"

    $env:FRACTADYNE_CONFIG_DIR = $cfg
    $sw = [Diagnostics.Stopwatch]::StartNew()
    $p = Start-Process -FilePath $Exe -ArgumentList $FdArgs -NoNewWindow -PassThru `
        -RedirectStandardOutput $log -RedirectStandardError "$log.err"
    if (-not $p.WaitForExit($TimeoutSec * 1000)) {
        Say "  TIMEOUT after ${TimeoutSec}s - killing"
        try { $p.Kill() } catch { }
        $sw.Stop()
        return [pscustomobject]@{ Tag = $Tag; Exit = $null; Seconds = $sw.Elapsed.TotalSeconds; TimedOut = $true; Cfg = $cfg }
    }
    $sw.Stop()
    if (Test-Path "$log.err") { Get-Content "$log.err" | Add-Content $log; Remove-Item "$log.err" -Force }
    $code = $p.ExitCode
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

    return [pscustomobject]@{ Tag = $Tag; Exit = $code; Seconds = $sw.Elapsed.TotalSeconds; TimedOut = $false; DeviceLost = $lost; Cfg = $cfg }
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

    Say "A) pre-fix behaviour (MODE_RATE_UNKNOWN_MARGIN=1) - a device loss here is EXPECTED"
    $results += Run-Phase -Tag '05a-repro-OLD' -FdArgs @('--set', 'MODE_RATE_UNKNOWN_MARGIN=1', '--play', $tour) -TimeoutSec 600

    Kill-Stragglers
    Start-Sleep -Seconds 5

    Say ""
    Say "B) stock (the fix) - survival here is the result we are after"
    $results += Run-Phase -Tag '05b-repro-NEW' -FdArgs @('--play', $tour) -TimeoutSec 600
}

# ---------------------------------------------------------------- verdict
Head "Summary"
$results | ForEach-Object {
    $d = if ($_.DeviceLost) { 'DEVICE-LOST' } elseif ($_.TimedOut) { 'TIMEOUT' } else { 'ok' }
    Say ("  {0,-20} exit={1,-6} {2,8:N1}s  {3}" -f $_.Tag, $_.Exit, $_.Seconds, $d)
}
$results | Format-Table -AutoSize | Out-File (Join-Path $Out 'summary.txt') -Encoding ascii

$a = $results | Where-Object { $_.Tag -eq '05a-repro-OLD' }
$b = $results | Where-Object { $_.Tag -eq '05b-repro-NEW' }
if ($a -and $b) {
    Say ""
    if ($a.DeviceLost -and -not $b.DeviceLost) {
        Say "RESULT: reproduced on the pre-fix setting and SURVIVED on stock. That is the proof."
    } elseif (-not $a.DeviceLost -and -not $b.DeviceLost) {
        Say "RESULT: INCONCLUSIVE. Neither run lost the device, so the tour did not re-enter the"
        Say "        killing regime. This does NOT show the fix works. Check the app log for"
        Say "        'arithmetic mode 0 -> 2' and whether bla_skip was 0 just after it; if BLA was"
        Say "        live through the crossover, the tour needs a centre with a short escaped"
        Say "        reference. Send the logs rather than calling this a pass."
    } elseif ($a.DeviceLost -and $b.DeviceLost) {
        Say "RESULT: BOTH lost the device. The fix is INSUFFICIENT for this regime. This is the"
        Say "        most valuable outcome to report - send the whole output directory."
    } else {
        Say "RESULT: stock lost the device but the pre-fix setting did not. Unexpected ordering;"
        Say "        treat as a real finding and send the logs."
    }
}

Say ""
Say "Zip and share: $Out"
Say "Anything with 'crash-' in the name is a full crash report."
