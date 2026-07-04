#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Render the "deep spiral dive" guided tour (tours/deep-spiral-dive.toml) to a movie.

.DESCRIPTION
    Prompts for a target folder and a resolution, then renders the tour to a numbered PNG
    frame sequence (and, with -Mp4, assembles them into an H.264 .mp4 via ffmpeg). Every
    command-line option passed to the renderer is documented inline below.

    Any parameter you pass on the command line skips its prompt, so the script also works
    unattended, e.g.  ./scripts/render-spiral-dive.ps1 -Out renders/spiral -Size 1920x1080 -Mp4

.PARAMETER Out
    Target folder for the rendered frames (created if needed). Prompted if omitted.
.PARAMETER Size
    Frame size as WIDTHxHEIGHT (e.g. 1920x1080). Prompted if omitted.
.PARAMETER Fps
    Frames per second sampled along the tour timeline (default 30).
.PARAMETER Ss
    Supersampling factor 1-8 (NxN anti-aliasing samples per pixel; default 2).
.PARAMETER Mp4
    After rendering, assemble the frames into an .mp4 via ffmpeg (must be on PATH).
.PARAMETER Hud
    Burn a zoom-level + center-coordinate HUD into each frame.
.PARAMETER Overwrite
    Replace existing frames in the target folder without the interactive prompt.
.PARAMETER Exe
    Path to fractadyne.exe. Defaults to target\release\fractadyne.exe (built if missing).

.EXAMPLE
    ./scripts/render-spiral-dive.ps1
.EXAMPLE
    ./scripts/render-spiral-dive.ps1 -Out renders/spiral -Size 3840x2160 -Ss 3 -Mp4
#>
[CmdletBinding()]
param(
    [string]$Out,
    [string]$Size,
    [int]$Fps = 30,
    [ValidateRange(1, 8)][int]$Ss = 2,
    [switch]$Mp4,
    [switch]$Hud,
    [switch]$Overwrite,
    [string]$Exe
)

$ErrorActionPreference = "Stop"
# Run from the repo root so the relative tour path (tours/...) resolves regardless of CWD.
Set-Location (Join-Path $PSScriptRoot "..")

$tour = "tours/deep-spiral-dive.toml"
if (-not (Test-Path $tour)) { throw "Tour file not found: $tour (run this from a Fractadyne checkout)." }

# --- Prompt: target folder ----------------------------------------------------
if (-not $Out) {
    $default = "renders/deep-spiral-dive"
    $ans = Read-Host "Target folder for the rendered frames [$default]"
    $Out = if ([string]::IsNullOrWhiteSpace($ans)) { $default } else { $ans.Trim() }
}

# --- Prompt: resolution -------------------------------------------------------
if (-not $Size) {
    Write-Host "Resolution:"
    Write-Host "  1) 1280x720   (720p)"
    Write-Host "  2) 1920x1080  (1080p)   [default]"
    Write-Host "  3) 2560x1440  (1440p)"
    Write-Host "  4) 3840x2160  (2160p / 4K)"
    Write-Host "  5) custom (enter WxH)"
    switch (Read-Host "Choice [2]") {
        "1"     { $Size = "1280x720" }
        "3"     { $Size = "2560x1440" }
        "4"     { $Size = "3840x2160" }
        "5"     { $Size = (Read-Host "Enter size as WIDTHxHEIGHT").Trim() }
        default { $Size = "1920x1080" }
    }
}
if ($Size -notmatch '^\d+x\d+$') { throw "Resolution must be WIDTHxHEIGHT (e.g. 1920x1080); got '$Size'." }

# --- Locate (or build) the renderer ------------------------------------------
if (-not $Exe) { $Exe = "target\release\fractadyne.exe" }
if (-not (Test-Path $Exe)) {
    Write-Host "Building the release binary (first build fetches wgpu/egui; a few minutes)..." -ForegroundColor Cyan
    Stop-Process -Name fractadyne -Force -ErrorAction SilentlyContinue  # release the exe lock if the app is open
    # On the author's page-file-constrained machine, add `-j 1` if the build hits OS error 1455.
    cargo build --release -p fractadyne-app
}

# --- Build the renderer command line -----------------------------------------
# Each option is documented so it's clear what the render is doing:
$cli = @(
    "--render-tour", $tour   # MODE: render this keyframe-tour TOML to a PNG frame sequence, then exit.
    "--out", $Out            # Output DIRECTORY for the frames (deep-spiral-dive_00000.png, ...); created if missing.
    "--size", $Size          # Frame size WIDTHxHEIGHT (bare width is also accepted; height then follows the aspect ratio).
    "--fps", $Fps            # Frames per second sampled along the tour timeline; more FPS = smoother motion and more frames.
    "--ss", $Ss              # Supersampling 1-8 (NxN samples/pixel). 2 balances quality/speed; 3-4 reduces shimmer on the deep spiral.
)
if ($Hud)       { $cli += "--show-location" }  # Burn a zoom-level + center-coordinate HUD into each frame's top-left corner.
if ($Overwrite) { $cli += "--overwrite" }      # Overwrite existing frames in the folder without the [y]es/[a]ll/[n]o/[q]uit prompt.
if ($Mp4)       { $cli += "--mp4" }            # After rendering, stitch the frames into <Out>/tour.mp4 via ffmpeg (must be on PATH); frames are kept.

# --- Render -------------------------------------------------------------------
Write-Host "`nRendering deep-spiral-dive -> $Out  ($Size, ${Fps}fps, ${Ss}x SSAA$(if($Mp4){', +mp4'}))" -ForegroundColor Green
Write-Host "  $Exe $($cli -join ' ')`n" -ForegroundColor DarkGray
& $Exe @cli
if ($LASTEXITCODE -ne 0) { throw "Render failed (exit $LASTEXITCODE)." }

Write-Host "`nDone. Frames are in: $Out" -ForegroundColor Green
if (-not $Mp4) {
    Write-Host "Re-run with -Mp4 to also assemble an .mp4 (requires ffmpeg on PATH)."
}
