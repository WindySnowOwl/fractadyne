#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Reproduce the project's deep sample location as a single still — a real stress test.

.DESCRIPTION
    Renders the deep-zoom sample location shipped as scripts/deep-sample.fdn: a Mandelbrot view at
    ~10^1108x with a center specified to ~1138 digits. (That precision is the point: rendering a
    specific feature at magnification M needs ~log10(M) significant digits in the coordinate.)

    It reads the center + scale straight from the .fdn (an app "Share location" blob), so it stays
    in sync with the sample, and renders one image. This is a genuine, long-running stress test of
    the full deep-zoom pipeline (arbitrary-precision reference orbit + GPU perturbation + floatexp +
    series approximation + BLA) — the reference orbit alone runs to the location's iteration budget
    at ~3700-bit precision.

.PARAMETER Out
    Output image path (PNG or EXR by extension). Parent folder is created if needed.
.PARAMETER Size
    Image size as WIDTHxHEIGHT. Prompted (with a preset menu) if omitted.
.PARAMETER Ss
    Supersampling 1-8 (NxN samples/pixel). The main quality/cost knob; 3 is a solid showcase.
    (The sample was captured at 8; that's very heavy at this depth.)
.PARAMETER Iter
    Max iterations per pixel. 0 (default) = use the location's own max_iter (or ~220/octave capped
    at 500,000 if absent). Too low and the whole view reads as interior (blank).
.PARAMETER Light
    Add 3D relief lighting (tracks the escape-derivative; prettier, a little more work).
.PARAMETER Location
    The .fdn location to render (default scripts/deep-sample.fdn).
.PARAMETER Exe
    Path to fractadyne.exe. Defaults to target\release\fractadyne.exe (built if missing).

.EXAMPLE
    ./scripts/render-deepest.ps1
.EXAMPLE
    ./scripts/render-deepest.ps1 -Size 3840x2160 -Ss 4      # heavy 4K showcase
#>
[CmdletBinding()]
param(
    [string]$Out = "renders/deep-sample.png",
    [string]$Size,
    [ValidateRange(1, 8)][int]$Ss = 3,
    [int]$Iter = 0,
    [switch]$Light,
    [string]$Location = "scripts/deep-sample.fdn",
    [string]$Exe
)

$ErrorActionPreference = "Stop"
# Run from the repo root so relative paths (scripts\..., target\..., renders\...) resolve.
Set-Location (Join-Path $PSScriptRoot "..")
if (-not (Test-Path $Location)) { throw "Location file not found: $Location" }

# --- Read the sample location (key=value .fdn / export-metadata blob) ---------
$fdn = Get-Content $Location -Raw
function Get-Fdn([string]$key) { if ($fdn -match "(?m)^\s*$key=(.*)$") { $matches[1].Trim() } else { $null } }
$cx      = Get-Fdn 'center_x'
$cy      = Get-Fdn 'center_y'
$uppLog2 = Get-Fdn 'upp_log2'
$fdnIter = Get-Fdn 'max_iter'
$palette = Get-Fdn 'palette'
if (-not $cx -or -not $cy -or -not $uppLog2) { throw "Missing center_x / center_y / upp_log2 in $Location." }
$uppLog2 = [double]$uppLog2

# --- Prompt: resolution (needed before we can convert upp_log2 -> zoom-log2) --
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
$w, $h = ($Size -split 'x') | ForEach-Object { [int]$_ }

# The .fdn stores units-per-pixel (upp_log2 = log2(upp)); the renderer's --zoom-log2 takes the
# magnification exponent L. From the app's own math (units_per_pixel = (3 / height) * 2^-L, with
# REFERENCE_HEIGHT = 3): L = log2(3 / height) - upp_log2. So the render reproduces this exact scale
# at whatever height we render (larger frames simply reveal more of the surrounding area).
$zoomLog2 = [Math]::Round([Math]::Log(3.0 / $h, 2) - $uppLog2, 4)
$decimalDepth = [Math]::Round($zoomLog2 / [Math]::Log(10, 2), 0)

# --- Iteration budget --------------------------------------------------------
# Prefer the location's own max_iter; else ~220/octave capped at 500,000 (the app's appetite).
if ($Iter -le 0) {
    $Iter = if ($fdnIter) { [int]$fdnIter } else { [Math]::Min([int]($zoomLog2 * 220) + 1000, 500000) }
}

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
    "--center", $cx, $cy              # View center as full-precision decimals (two args: X then Y); ~1138 digits deep.
    "--zoom-log2", "$zoomLog2"        # Magnification = 2^L (from the .fdn's upp_log2). --zoom-log2 (not --zoom) since this is far past f64's ~1e308 range.
    "--size", $Size                   # Image size WIDTHxHEIGHT; rendered in tiles, so large sizes don't hit GPU limits.
    "--ss", $Ss                       # Supersampling 1-8 (NxN samples/pixel anti-aliasing).
    "--iter", "$Iter"                 # Max iterations per pixel; must scale with depth or the frame reads as blank interior.
)
if ($palette) { $cli += @("--palette", $palette) }  # Palette index from the location (falls back to the default if absent).
if ($Light)   { $cli += "--light" }                 # 3D relief lighting (shade by the escape-derivative normal).

# --- Render (timed) -----------------------------------------------------------
$samples = [long]$w * [long]$h * $Ss * $Ss
Write-Host "`nReproducing the deep sample location ($Location):" -ForegroundColor Green
Write-Host ("  Mandelbrot @ ~1e{0}x  (zoom-log2 {1}, center ~{2} digits)" -f $decimalDepth, $zoomLog2, ($cx -replace '[^0-9]', '').Length)
Write-Host ("  {0}, {1}x SSAA => {2:N0} samples, iter {3}{4}" -f $Size, $Ss, $samples, $Iter, $(if ($Light) { ', lit' } else { '' }))
Write-Host "  This is a heavy render at extreme precision; expect it to take a while.`n" -ForegroundColor Yellow
Write-Host "  $Exe --render --out $Out --fractal Mandelbrot --center <$($($cx -replace '[^0-9]','').Length)-digit x> <y> --zoom-log2 $zoomLog2 --size $Size --ss $Ss --iter $Iter$(if($palette){" --palette $palette"})$(if($Light){' --light'})`n" -ForegroundColor DarkGray

$sw = [System.Diagnostics.Stopwatch]::StartNew()
& $Exe @cli
$sw.Stop()
if ($LASTEXITCODE -ne 0) { throw "Render failed (exit $LASTEXITCODE)." }

$elapsed = [TimeSpan]::FromMilliseconds($sw.Elapsed.TotalMilliseconds)
Write-Host ("`nDone in {0:hh\:mm\:ss} -> {1}" -f $elapsed, $Out) -ForegroundColor Green
Write-Host ("  {0:N0} samples at {1:N0} samples/sec" -f $samples, ($samples / [Math]::Max($sw.Elapsed.TotalSeconds, 0.001)))
