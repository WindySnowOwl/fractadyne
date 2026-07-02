<#
.SYNOPSIS
  Measure the BLA (bilinear approximation) cost/benefit: profile with BLA off vs on, then
  analyze the tradeoff per region.

.DESCRIPTION
  Builds the optimized binary, runs the profiling harness twice — once normally and once with
  `--bla` — and reports, for each BLA-eligible region (floatexp mode-2 Mandelbrot, non-aux):

    * gpu_render  — the per-render GPU cost BLA speeds up.
    * bla_build   — the CPU cost to build the acceleration tree (currently PER FRAME, since the
                    tree isn't cached yet).
    * export net  — build once + render once (the export / settled-still model): bla_build + render.
    * live net    — build EVERY frame + render (today's live-view model, uncached):
                    bla_build + render vs. render.
    * cached net  — render only (build amortized): the ceiling a per-reference BLA cache reaches.

  This turns "is BLA worth enabling by default?" into numbers instead of a guess.

.EXAMPLE
  ./scripts/profile-bla.ps1 -Reps 10
#>
param(
    [int]$Reps = 10
)
$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..")

Stop-Process -Name fractadyne -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 300
cargo build --release -p fractadyne-app -j 1
$exe = "target\release\fractadyne.exe"

New-Item -ItemType Directory -Force logs | Out-Null
Write-Host "`n=== profiling BLA OFF ===" -ForegroundColor Cyan
& $exe --profile --reps $Reps --out logs/bla_off.json | Out-Host
Write-Host "`n=== profiling BLA ON  ===" -ForegroundColor Cyan
& $exe --profile --bla --reps $Reps --out logs/bla_on.json | Out-Host

$off = Get-Content logs/bla_off.json -Raw | ConvertFrom-Json
$on = Get-Content logs/bla_on.json -Raw | ConvertFrom-Json
$onMap = @{}
foreach ($r in $on.regions) { $onMap[$r.name] = $r }

function V($x) { if ($null -eq $x) { 0.0 } else { [double]$x } }
function Pct($base, $new) { if ($base -gt 1e-6) { ($new - $base) / $base * 100 } else { 0 } }
function SpeedLine($label, $base, $new) {
    $p = Pct $base $new
    $x = if ($new -gt 1e-6) { $base / $new } else { 0 }
    $tag = if ($p -lt -3) { "{0:N2}x faster" -f $x } elseif ($p -gt 3) { "SLOWER" } else { "~same" }
    $color = if ($p -lt -3) { "Green" } elseif ($p -gt 3) { "Red" } else { "Gray" }
    Write-Host ("    {0,-12} off {1,8:N2}  on {2,8:N2} ms   {3}" -f $label, $base, $new, $tag) -ForegroundColor $color
}

Write-Host "`n===== BLA cost/benefit =====" -ForegroundColor White
Write-Host "$($off.gpu) / $($off.cpu)`n"
$engaged = 0
foreach ($ro in $off.regions) {
    $rn = $onMap[$ro.name]
    if (-not $rn) { continue }
    $renderOff = V $ro.timings_ms.gpu_render.median
    $renderOn = V $rn.timings_ms.gpu_render.median
    $blaOn = V $rn.timings_ms.bla_build
    Write-Host ("{0}  (mode {1})" -f $ro.name, $rn.mode)
    if ($blaOn -lt 1e-6) {
        Write-Host "    BLA not eligible here (only floatexp mode-2 Mandelbrot, non-aux) — unchanged." -ForegroundColor DarkGray
        continue
    }
    $engaged++
    SpeedLine "gpu_render" $renderOff $renderOn
    Write-Host ("    bla_build    {0,8:N2} ms  (CPU tree build — per frame until cached)" -f $blaOn) -ForegroundColor Yellow
    SpeedLine "export net" $renderOff ($renderOn + $blaOn)   # build once + render
    SpeedLine "live net" $renderOff ($renderOn + $blaOn)     # build every frame (uncached) + render
    SpeedLine "cached net" $renderOff $renderOn              # render only (BLA cache ceiling)
}
if ($engaged -eq 0) {
    Write-Host "No BLA-eligible region in this run — add a deep (>=1e28x) mode-2 Mandelbrot region." -ForegroundColor Yellow
}
Write-Host "`nlogs → logs/bla_off.json, logs/bla_on.json"
