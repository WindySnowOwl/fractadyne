#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Render the "deep spiral dive" guided tour (tours/deep-spiral-dive.toml) to a movie.

.DESCRIPTION
    Prompts for a target folder and a resolution, then renders the tour to a numbered PNG
    frame sequence (and, with -Mp4, assembles them into an H.264 .mp4 via ffmpeg). Every
    command-line option passed to the renderer is documented inline below.

    RESTARTABLE: if the target folder already has frames from an interrupted render, the script
    checks the last frame + a saved manifest against the current settings, warns about any
    mismatch, and offers to Resume (render only the missing frames) or start Over.

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
    Burn a zoom-level + center-coordinate HUD into each frame. On by default; disable with -Hud:$false.
.PARAMETER Resume
    If frames already exist, resume without prompting (render only the missing ones).
.PARAMETER Restart
    If frames already exist, re-render them all from scratch without prompting.
.PARAMETER Exe
    Path to fractadyne.exe. Defaults to target\release\fractadyne.exe (built if missing).

.EXAMPLE
    ./scripts/render-spiral-dive.ps1
.EXAMPLE
    ./scripts/render-spiral-dive.ps1 -Out renders/spiral -Size 3840x2160 -Ss 3 -Mp4
.EXAMPLE
    ./scripts/render-spiral-dive.ps1 -Out renders/spiral -Resume    # continue an interrupted render
#>
[CmdletBinding()]
param(
    [string]$Out,
    [string]$Size,
    [int]$Fps = 30,
    [ValidateRange(1, 8)][int]$Ss = 2,
    [switch]$Mp4,
    [bool]$Hud = $true,
    [switch]$Resume,
    [switch]$Restart,
    [string]$Exe
)

$ErrorActionPreference = "Stop"
# Run from the repo root so the relative tour path (tours/...) resolves regardless of CWD.
Set-Location (Join-Path $PSScriptRoot "..")

$tour = "tours/deep-spiral-dive.toml"
if (-not (Test-Path $tour)) { throw "Tour file not found: $tour (run this from a Fractadyne checkout)." }

# Read a PNG's pixel dimensions ("WxH") straight from its IHDR header — no image library needed.
# (PNG layout: 8-byte signature, then IHDR whose width/height are big-endian u32 at byte 16/20.)
function Get-PngSize([string]$path) {
    try {
        $fs = [System.IO.File]::OpenRead($path)
        try {
            $b = New-Object byte[] 24
            if ($fs.Read($b, 0, 24) -lt 24) { return $null }
            if ($b[1] -ne 0x50 -or $b[2] -ne 0x4E -or $b[3] -ne 0x47) { return $null } # "PNG"
            $w = ([int]$b[16] -shl 24) -bor ([int]$b[17] -shl 16) -bor ([int]$b[18] -shl 8) -bor [int]$b[19]
            $h = ([int]$b[20] -shl 24) -bor ([int]$b[21] -shl 16) -bor ([int]$b[22] -shl 8) -bor [int]$b[23]
            return "${w}x${h}"
        } finally { $fs.Dispose() }
    } catch { return $null }
}

# Read a key=value manifest into a hashtable (empty if the file is absent).
function Read-Manifest([string]$path) {
    $h = @{}
    if (Test-Path $path) {
        foreach ($line in Get-Content $path) {
            if ($line -match '^\s*([^=#]+?)\s*=\s*(.*)$') { $h[$matches[1]] = $matches[2].Trim() }
        }
    }
    return $h
}

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

# --- Prompt: assemble an MP4? (requires ffmpeg) ------------------------------
# Only ask when -Mp4 wasn't given explicitly on the command line.
if (-not $PSBoundParameters.ContainsKey('Mp4')) {
    $ans = Read-Host "Assemble the frames into an .mp4 when done? Requires ffmpeg on PATH. [y/N]"
    $Mp4 = $ans -match '^(y|yes)$'
}
if ($Mp4 -and -not (Get-Command ffmpeg -ErrorAction SilentlyContinue)) {
    Write-Host "  [!] ffmpeg was not found on PATH. The frames will still render, but the .mp4" -ForegroundColor Yellow
    Write-Host "      won't be assembled (install ffmpeg and re-run with -Mp4, or stitch the frames yourself)." -ForegroundColor Yellow
}

# --- Restart / resume an interrupted render ----------------------------------
# The renderer names frames "<prefix>_00000.png" where <prefix> is the tour file's stem.
$prefix       = [System.IO.Path]::GetFileNameWithoutExtension($tour)   # "deep-spiral-dive"
$manifestPath = Join-Path $Out "render.manifest.txt"
# Settings that must match for a resume to produce one coherent movie (fps/size determine which
# point on the timeline each frame index maps to; ss/hud change how frames look).
$curSettings  = [ordered]@{ tour = $tour; size = $Size; fps = "$Fps"; ss = "$Ss"; hud = "$Hud" }
$mode = "fresh"   # fresh | resume | restart

$existing = @()
if (Test-Path $Out) {
    $existing = @(Get-ChildItem -Path $Out -Filter "${prefix}_*.png" -ErrorAction SilentlyContinue | Sort-Object Name)
}
if ($existing.Count -gt 0) {
    $last = $existing[-1]
    Write-Host "`nFound $($existing.Count) existing frame(s) in $Out (last: $($last.Name))." -ForegroundColor Yellow

    # Check the LAST frame's actual resolution against what we're about to render.
    $lastSize = Get-PngSize $last.FullName
    if ($lastSize -and $lastSize -ne $Size) {
        Write-Host "  [!] Last frame is ${lastSize}, but this run requests ${Size}." -ForegroundColor Yellow
    }
    # Check the saved manifest (fps/ss/hud/tour aren't visible in a frame's pixels).
    $man = Read-Manifest $manifestPath
    $mismatch = @()
    foreach ($k in $curSettings.Keys) {
        if ($man.ContainsKey($k) -and $man[$k] -ne $curSettings[$k]) {
            $mismatch += "  [!] ${k}: existing = '$($man[$k])', requested = '$($curSettings[$k])'"
        }
    }
    if (-not (Test-Path $manifestPath)) {
        Write-Host "  [!] No manifest found — can't confirm fps/ss/hud match the existing frames." -ForegroundColor Yellow
    }
    if ($mismatch.Count -gt 0) {
        Write-Host "  Settings differ from the interrupted render:" -ForegroundColor Yellow
        $mismatch | ForEach-Object { Write-Host $_ -ForegroundColor Yellow }
        Write-Host "  Resuming with different settings will mix mismatched frames — starting over is safer." -ForegroundColor Yellow
    }

    # Decide what to do (params force it; otherwise ask).
    if     ($Restart) { $mode = "restart" }
    elseif ($Resume)  { $mode = "resume" }
    else {
        $choice = Read-Host "[R]esume (render only missing frames) / start [O]ver / [C]ancel"
        switch -Regex ($choice) {
            '^(o|over|overwrite)$' { $mode = "restart" }
            '^(r|resume)$'         { $mode = "resume" }
            default                { Write-Host "Cancelled."; exit 0 }
        }
    }

    if ($mode -eq "resume") {
        # The frame that was mid-write when the render died can be truncated. Its index isn't
        # deterministic (frames encode in parallel), but the highest-numbered one is the most
        # likely victim — drop it so it's re-rendered rather than left half-written in the movie.
        Write-Host "  Resuming; re-rendering the last frame ($($last.Name)) in case it was cut off." -ForegroundColor Cyan
        Remove-Item $last.FullName -Force -ErrorAction SilentlyContinue
    }
}

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
if ($Hud)             { $cli += "--show-location" }  # Burn a zoom-level + center-coordinate HUD into each frame's top-left corner.
if ($mode -eq "resume")  { $cli += "--resume" }      # Restart: keep frames already on disk, render only the missing ones (no prompt).
if ($mode -eq "restart") { $cli += "--overwrite" }   # Re-render every frame from scratch, replacing existing ones without prompting.
if ($Mp4)             { $cli += "--mp4" }             # After rendering, stitch the frames into <Out>/tour.mp4 via ffmpeg (must be on PATH); frames are kept.

# --- Write the manifest (so a later resume can confirm these settings match) --
New-Item -ItemType Directory -Force -Path $Out | Out-Null
$manifestLines = @("# Fractadyne render manifest — used to validate a resumed render. Do not edit.")
foreach ($k in $curSettings.Keys) { $manifestLines += "$k=$($curSettings[$k])" }
Set-Content -Path $manifestPath -Value $manifestLines -Encoding utf8

# --- Render -------------------------------------------------------------------
$modeNote = switch ($mode) { "resume" { " [resuming]" } "restart" { " [from scratch]" } default { "" } }
Write-Host "`nRendering deep-spiral-dive -> $Out  ($Size, ${Fps}fps, ${Ss}x SSAA, HUD $(if($Hud){'on'}else{'off'})$(if($Mp4){', +mp4'}))$modeNote" -ForegroundColor Green
Write-Host "  $Exe $($cli -join ' ')`n" -ForegroundColor DarkGray
& $Exe @cli
if ($LASTEXITCODE -ne 0) { throw "Render failed (exit $LASTEXITCODE). Re-run with -Resume to continue where it stopped." }

Write-Host "`nDone. Frames are in: $Out" -ForegroundColor Green
if (-not $Mp4) {
    Write-Host "Re-run with -Mp4 to also assemble an .mp4 (requires ffmpeg on PATH)."
}
