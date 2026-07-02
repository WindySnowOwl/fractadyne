<#
.SYNOPSIS
  Automated interframe-stutter test: play a stepped deep-zoom dive on the live path and report the
  reference-recompute stalls (the CPU hitches that block frames).

.DESCRIPTION
  Builds the optimized binary and runs `--frametest`, which drives the live `build_params` path
  (reference cache + recompute) frame-by-frame down a dive, then reports per-frame `build_ms` (the
  recompute stall) and a stall count (frames with build_ms > 16 ms). Use it to validate an
  optimization such as async/off-thread recompute: run before and after and compare the stall
  count and build p95/max. The GPU render time is reported for context (it doesn't change).

.EXAMPLE
  ./scripts/frametest.ps1                                  # 40 steps x 4 hold, dive to 1e30x
  ./scripts/frametest.ps1 -Steps 60 -Dive 40 -Out logs/ft_after.json
  ./scripts/frametest.ps1 -Compare logs/ft_before.json logs/ft_after.json   # diff two runs
#>
param(
    [int]$Steps = 40,
    [int]$Hold = 4,
    [double]$Dive = 30,
    [string]$Out = "logs/frametest.json",
    [string[]]$Compare
)
$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..")

function Show($path, $label) {
    $d = Get-Content $path -Raw | ConvertFrom-Json
    Write-Host ("{0}: stalls={1}  build median={2:N1} p95={3:N1} max={4:N1} ms  gpu median={5:N1} ms  (BLA={6})" -f `
        $label, $d.recompute_stalls, $d.build_ms.median, $d.build_ms.p95, $d.build_ms.max, $d.gpu_ms.median, $d.use_bla)
}

if ($Compare -and $Compare.Count -eq 2) {
    Show $Compare[0] "before"
    Show $Compare[1] "after "
    $a = Get-Content $Compare[0] -Raw | ConvertFrom-Json
    $b = Get-Content $Compare[1] -Raw | ConvertFrom-Json
    $ds = $a.recompute_stalls - $b.recompute_stalls
    $col = if ($b.recompute_stalls -lt $a.recompute_stalls) { "Green" } elseif ($b.recompute_stalls -gt $a.recompute_stalls) { "Red" } else { "Gray" }
    Write-Host ("stalls: {0} -> {1}  ({2:+#;-#;0})" -f $a.recompute_stalls, $b.recompute_stalls, -$ds) -ForegroundColor $col
    return
}

Stop-Process -Name fractadyne -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 300
cargo build --release -p fractadyne-app -j 1
$exe = "target\release\fractadyne.exe"
New-Item -ItemType Directory -Force logs | Out-Null
& $exe --frametest --steps $Steps --hold $Hold --dive $Dive --out $Out | Out-Host
Write-Host ""
Show $Out "result"
Write-Host "log -> $Out"
