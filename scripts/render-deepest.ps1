#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Reproduce the deepest zoom documented in this project, as a single still — a real stress test.

.DESCRIPTION
    The deepest documented, reproducible location is the endpoint of the "deep spiral dive" tour:
    a Misiurewicz spiral in Elephant Valley at ~10^420x, with a center specified to ~464 digits.
    (That precision is the point: rendering a specific feature at magnification M needs ~log10(M)
    significant digits in the coordinate. 10^420x needs ~420; deeper "record" zooms would need
    thousands to millions of digits, which is why this is the practical rendered ceiling here.)

    This script reads that coordinate and depth straight from the tour file (so it stays in sync
    with the docs), then renders one image with an iteration budget matched to the depth. It's a
    genuine, long-running stress test of the full deep-zoom pipeline (arbitrary-precision reference
    orbit + GPU perturbation + floatexp + series approximation + BLA).

.PARAMETER Out
    Output image path (PNG or EXR by extension). Parent folder is created if needed.
.PARAMETER Size
    Image size as WIDTHxHEIGHT. Prompted (with a preset menu) if omitted.
.PARAMETER Ss
    Supersampling 1-8 (NxN samples/pixel). The main quality/cost knob; 3 is a solid showcase.
.PARAMETER Iter
    Max iterations per pixel. 0 (default) = auto: ~220 per octave (capped at 500,000), which is what
    the app's auto-scaling uses at this depth. Too low and the whole view reads as interior (blank).
.PARAMETER Light
    Add 3D relief lighting (tracks the escape-derivative; prettier, a little more work).
.PARAMETER Tour
    Tour file to read the deepest coordinate/depth from (default tours/deep-spiral-dive.toml).
.PARAMETER Exe
    Path to fractadyne.exe. Defaults to target\release\fractadyne.exe (built if missing).

.EXAMPLE
    ./scripts/render-deepest.ps1
.EXAMPLE
    ./scripts/render-deepest.ps1 -Size 3840x2160 -Ss 4 -Light      # heavy 4K showcase
#>
[CmdletBinding()]
param(
    [string]$Out = "renders/deepest-spiral-1e420.png",
    [string]$Size,
    [ValidateRange(1, 8)][int]$Ss = 3,
    [int]$Iter = 0,
    [switch]$Light,
    [string]$Tour = "tours/deep-spiral-dive.toml",
    [string]$Exe
)

$ErrorActionPreference = "Stop"
# Run from the repo root so relative paths (tours\..., target\..., renders\...) resolve.
Set-Location (Join-Path $PSScriptRoot "..")
if (-not (Test-Path $Tour)) { throw "Tour file not found: $Tour" }

# --- Read the deepest documented location from the tour ----------------------
# The deep target's center is the coordinate carrying the most digits; the depth is the tour's
# largest declared magnitude (mag_log10 = decimal exponent of the magnification, i.e. 10^N x).
$tourText = Get-Content $Tour -Raw
$cx = ([regex]::Matches($tourText, 'center_x = "([0-9.eE+\-]+)"') | ForEach-Object { $_.Groups[1].Value } | Sort-Object Length -Descending)[0]
$cy = ([regex]::Matches($tourText, 'center_y = "([0-9.eE+\-]+)"') | ForEach-Object { $_.Groups[1].Value } | Sort-Object Length -Descending)[0]
$maxLog10 = ([regex]::Matches($tourText, 'mag_log10 = ([0-9.]+)') | ForEach-Object { [double]$_.Groups[1].Value } | Measure-Object -Maximum).Maximum
if (-not $cx -or -not $cy -or -not $maxLog10) { throw "Could not extract a deep coordinate/depth from $Tour." }
# Magnification = 2^L; the app's --zoom-log2 takes L. L = log10(mag) * log2(10).
$zoomLog2 = [Math]::Round($maxLog10 * [Math]::Log(10, 2), 2)

# --- Iteration budget (matched to depth) -------------------------------------
# Auto = ~220 iterations per octave, capped at 500,000 — the app's own appetite at this depth.
# (At 10^420x that's ~307,000; far below and the whole frame reads as interior => a blank image.)
if ($Iter -le 0) { $Iter = [Math]::Min([int]($zoomLog2 * 220) + 1000, 500000) }

# --- Prompt: resolution -------------------------------------------------------
if (-not $Size) {
    Write-Host "Resolution (bigger = much longer at this depth):"
    Write-Host "  1) 1280x720   (720p)"
    Write-Host "  2) 1920x1080  (1080p)   [default]"
    Write-Host "  3) 3840x2160  (2160p / 4K)   heavy"
    Write-Host "  4) custom (enter WxH)"
    switch (Read-Host "Choice [2]") {
        "1"     { $Size = "1280x720" }
        "3"     { $Size = "3840x2160" }
        "4"     { $Size = (Read-Host "Enter size as WIDTHxHEIGHT").Trim() }
        default { $Size = "1920x1080" }
    }
}
if ($Size -notmatch '^\d+x\d+$') { throw "Resolution must be WIDTHxHEIGHT; got '$Size'." }

# --- Locate (or build) the renderer ------------------------------------------
if (-not $Exe) { $Exe = "target\release\fractadyne.exe" }
if (-not (Test-Path $Exe)) {
    Write-Host "Building the release binary (first build fetches wgpu/egui; a few minutes)..." -ForegroundColor Cyan
    Stop-Process -Name fractadyne -Force -ErrorAction SilentlyContinue  # release the exe lock if the app is open
    # On the author's page-file-constrained machine, add `-j 1` if the build hits OS error 1455.
    cargo build --release -p fractadyne-app
}
$outDir = Split-Path -Parent $Out
if ($outDir -and -not (Test-Path $outDir)) { New-Item -ItemType Directory -Force -Path $outDir | Out-Null }

# --- Build the renderer command line -----------------------------------------
# Every option is documented so it's clear what the render is doing:
$cli = @(
    "--render"                        # MODE: render one still image and exit (tiled export + perturbation pipeline).
    "--out", $Out                     # Output image path; PNG (8-bit sRGB) or EXR (32-bit linear) by extension.
    "--fractal", "Mandelbrot"         # Fractal family.
    "--center", $cx, $cy              # View center as full-precision decimals (two args: X then Y); ~464 digits deep.
    "--zoom-log2", "$zoomLog2"        # Magnification = 2^L. Used (not --zoom) because 10^420x is far past f64's ~1e308 range.
    "--size", $Size                   # Image size WIDTHxHEIGHT; rendered in tiles, so large sizes don't hit GPU limits.
    "--ss", $Ss                       # Supersampling 1-8 (NxN samples/pixel anti-aliasing).
    "--iter", "$Iter"                 # Max iterations per pixel; must scale with depth or the frame reads as blank interior.
)
if ($Light) { $cli += "--light" }     # 3D relief lighting (shade by the escape-derivative normal).

# --- Render (timed) -----------------------------------------------------------
$w, $h = $Size -split 'x'
$samples = [long]$w * [long]$h * $Ss * $Ss
Write-Host "`nReproducing the deepest documented zoom:" -ForegroundColor Green
Write-Host ("  Misiurewicz spiral @ 10^{0}x  (zoom-log2 {1}, center ~{2} digits)" -f $maxLog10, $zoomLog2, ($cx -replace '[^0-9]', '').Length)
Write-Host ("  {0}, {1}x SSAA => {2:N0} samples, iter {3}{4}" -f $Size, $Ss, $samples, $Iter, $(if ($Light) { ', lit' } else { '' }))
Write-Host "  This is a heavy render at extreme precision; expect it to take a while.`n" -ForegroundColor Yellow
Write-Host "  $Exe --render --out $Out --fractal Mandelbrot --center <464-digit x> <y> --zoom-log2 $zoomLog2 --size $Size --ss $Ss --iter $Iter$(if($Light){' --light'})`n" -ForegroundColor DarkGray

$sw = [System.Diagnostics.Stopwatch]::StartNew()
& $Exe @cli
$sw.Stop()
if ($LASTEXITCODE -ne 0) { throw "Render failed (exit $LASTEXITCODE)." }

$elapsed = [TimeSpan]::FromMilliseconds($sw.Elapsed.TotalMilliseconds)
Write-Host ("`nDone in {0:hh\:mm\:ss} -> {1}" -f $elapsed, $Out) -ForegroundColor Green
Write-Host ("  {0:N0} samples at {1:N0} samples/sec" -f $samples, ($samples / [Math]::Max($sw.Elapsed.TotalSeconds, 0.001)))
