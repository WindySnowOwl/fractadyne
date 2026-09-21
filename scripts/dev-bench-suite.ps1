# dev-bench-suite.ps1 - the performance + self-check battery this machine runs before a build is
# published, producing the REFERENCE numbers a tester on other hardware compares against.
#
#   .\scripts\dev-bench-suite.ps1 -Label rtx3080-beta111
#   .\scripts\dev-bench-suite.ps1 -Label rtx3080-beta111 -Out D:\share\Fractadyne\builds\...\ref
#
# WHY THIS EXISTS, next to gpu-validate.ps1. That script is the CROSS-GPU battery: the six steps
# that must run identically on every card so their outputs can be diffed. This one adds the
# perf-only harnesses that are meaningless to diff across machines but are exactly what a release
# wants recorded on the dev box - the bignum backend sweep, the standardized dive, the on-screen
# zoom latency, and the motion-presentation gate. Run gpu-validate.ps1 FIRST, then this.
#
# ONE AT A TIME, DELIBERATELY. Every step below either measures GPU time or measures frame
# intervals, and a second consumer on the same card makes both meaningless (the same reason the
# bench-matrix A/B rules say to interleave rather than run in parallel). Nothing here is
# parallelised, and nothing else should be running on the machine while it does.
#
# HERMETIC: a private config directory per run, so the developer's own session is never touched
# and never contributes settings to a number that gets published.
#
# ASCII-only, like gpu-validate.ps1: Windows PowerShell 5.1 reads a BOM-less .ps1 as ANSI, so a
# stray em-dash becomes a parse error on someone else's machine.

[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$Label,
    [string]$Out = "",
    [string]$Exe = "",
    # Skip the long ones (livetest is not here, but zoomtest + benchmark-std are a few minutes).
    [switch]$Quick
)

$ErrorActionPreference = "Continue"
$root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)

if (-not $Exe) { $Exe = Join-Path $root "target\release\fractadyne.exe" }
if (-not (Test-Path $Exe)) { throw "binary not found: $Exe" }

if (-not $Out) { $Out = Join-Path $root ("dist\bench-" + $Label) }
$stamp = Get-Date -Format "yyyyMMdd-HHmmss"
$dir = $Out
$cfg = Join-Path $dir "config"
New-Item -ItemType Directory -Force -Path $cfg | Out-Null

$env:FRACTADYNE_CONFIG_DIR = $cfg
$env:FRACTADYNE_NO_SOUND = "1"

$summary = New-Object System.Collections.Generic.List[string]
$summary.Add("Fractadyne dev bench suite - $Label")
$summary.Add("started : $stamp")
$summary.Add("binary  : $Exe")
$summary.Add(("version : " + ((& $Exe --version 2>&1) -join ' ')))
$summary.Add("")
$summary.Add("step                 exit   duration  output")
$summary.Add("-------------------- ----   --------  ------")

function Step([string]$name, [string]$file, [string[]]$stepArgs) {
    Write-Host ""
    Write-Host ("== " + $name) -ForegroundColor Cyan
    Write-Host ("   " + ($stepArgs -join ' '))
    $path = Join-Path $dir $file
    $t0 = Get-Date
    & $Exe @stepArgs *> $path
    $code = $LASTEXITCODE
    $secs = ((Get-Date) - $t0).TotalSeconds
    $summary.Add(("{0,-20} {1,4}   {2,7:N1}s  {3}" -f $name, $code, $secs, $file))
    Write-Host ("   exit {0} in {1:N1}s -> {2}" -f $code, $secs, $file)
    return $code
}

# ---- self checks -------------------------------------------------------------------------
# The motion-PRESENTATION gate: the one harness that asserts a partial refresh never becomes the
# frame on screen. It is the automated half of the beta.110 verified-present work.
Step "motiontest" "10-motiontest.txt" @("--motiontest") | Out-Null

# ---- performance -------------------------------------------------------------------------
# CPU only: the reference-orbit cost of each arbitrary-precision backend, asserting the two
# backends produce byte-identical orbits before it compares their speed.
Step "bench-bignum" "11-bench-bignum.txt" @("--bench-bignum") | Out-Null

if (-not $Quick) {
    # The standardized dive: one number for "how fast does this machine go deep".
    Step "benchmark-std" "12-benchmark-std.txt" @("--benchmark-std") | Out-Null

    # What a viewer FEELS: frame-interval distribution through a real 40-octave glide at the
    # fastest zoom setting, driven through the production pacer and reference lookahead.
    Step "zoomtest-4x" "13-zoomtest-4x.txt" `
        @("--zoomtest", "40", "--zoomtest-rate", "4.0", "--out", (Join-Path $dir "13-zoomtest-4x.json")) | Out-Null

    # ...and at the default rate, where the presenter cadence differs.
    Step "zoomtest-1x" "14-zoomtest-1x.txt" `
        @("--zoomtest", "40", "--zoomtest-rate", "1.0", "--out", (Join-Path $dir "14-zoomtest-1x.json")) | Out-Null
}

# ---- the app's own log across every step --------------------------------------------------
$log = Join-Path $cfg "logs\fractadyne.log"
if (Test-Path $log) { Copy-Item $log (Join-Path $dir "app.log") -Force }
$crashes = Get-ChildItem (Join-Path $cfg "logs") -Filter "crash-*" -ErrorAction SilentlyContinue
$summary.Add("")
$summary.Add(("crash reports produced: " + $(if ($crashes) { $crashes.Count } else { 0 })))
if ($crashes) { foreach ($c in $crashes) { Copy-Item $c.FullName $dir -Force } }

$summary.Add(("finished: " + (Get-Date -Format "yyyyMMdd-HHmmss")))
$summary -join "`r`n" | Out-File -FilePath (Join-Path $dir "summary.txt") -Encoding ascii
Write-Host ""
Write-Host ("Bundle: " + $dir) -ForegroundColor Green
Get-Content (Join-Path $dir "summary.txt") | Select-Object -Last 12
