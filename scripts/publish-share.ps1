# publish-share.ps1 - assemble the downloadable packages for one version and copy them to the
# local share, with checksums, so another machine (or another OS) can install and test them.
#
#   .\scripts\publish-share.ps1                       # version from Cargo.toml, share auto-detected
#   .\scripts\publish-share.ps1 -Share D:\share\Fractadyne
#   .\scripts\publish-share.ps1 -SkipSource           # Windows packages only
#
# WHAT IT MAKES, mirroring what `.github/workflows/release.yml` publishes on a tag, so a package
# built here is laid out exactly like one a user downloads from a release:
#   fractadyne-<tag>-windows-x64.zip              the MSVC build + docs + validation data
#   fractadyne-<tag>-windows-x64-accelerated.zip  the MPFR build (built by build-accelerated.ps1)
#   fractadyne-<tag>-src.tar.gz                   a clean source tree, for building on Linux
# plus a .sha256 beside each, into  <Share>\builds\<tag>\ .
#
# THE SOURCE TARBALL IS NOT A CONVENIENCE. This machine has no Linux toolchain and the tree is
# not pushed, so the only way a Linux box can run this version is to build it from a tree carried
# across. `git archive` is used rather than a directory copy so the bundle carries exactly what
# is committed - no target/, no dist/, no local/ - and cannot quietly ship an uncommitted edit.
#
# ASCII-only (Windows PowerShell 5.1 reads a BOM-less .ps1 as ANSI).

[CmdletBinding()]
param(
    [string]$Share = "",
    [string]$Tag = "",
    [switch]$SkipSource,
    [switch]$SkipWindows
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $root

if (-not $Tag) {
    $line = Select-String -Path (Join-Path $root "Cargo.toml") -Pattern '^version\s*=\s*"([^"]+)"' | Select-Object -First 1
    if (-not $line) { throw "could not read the version from Cargo.toml" }
    $Tag = "v" + $line.Matches[0].Groups[1].Value
}
if (-not $Share) {
    foreach ($c in @("D:\share\Fractadyne", "\\vger\share\Fractadyne")) {
        if (Test-Path $c) { $Share = $c; break }
    }
}
if (-not $Share) { throw "no share found; pass -Share" }

$dest = Join-Path (Join-Path $Share "builds") $Tag
New-Item -ItemType Directory -Force -Path $dest | Out-Null
Write-Host "Publishing $Tag to $dest" -ForegroundColor Cyan

function Publish([string]$path) {
    $name = Split-Path -Leaf $path
    Copy-Item $path (Join-Path $dest $name) -Force
    $hash = (Get-FileHash (Join-Path $dest $name) -Algorithm SHA256).Hash.ToLower()
    "$hash  $name" | Out-File -FilePath (Join-Path $dest "$name.sha256") -Encoding ascii
    $mb = [Math]::Round((Get-Item $path).Length / 1MB, 1)
    Write-Host ("  {0,-56} {1,6} MB" -f $name, $mb)
}

# ---------------------------------------------------------------- windows x64 (MSVC)
if (-not $SkipWindows) {
    $exe = Join-Path $root "target\release\fractadyne.exe"
    if (-not (Test-Path $exe)) { throw "target\release\fractadyne.exe is missing - build it first" }
    $stage = Join-Path $root ("dist\fractadyne-$Tag-windows-x64")
    if (Test-Path $stage) { Remove-Item -Recurse -Force $stage }
    New-Item -ItemType Directory -Force -Path $stage | Out-Null
    Copy-Item $exe -Destination $stage
    Copy-Item (Join-Path $root "README.md"), (Join-Path $root "CHANGELOG.md"), (Join-Path $root "TOURS.md"), `
              (Join-Path $root "DIAGNOSTICS.md"), (Join-Path $root "THIRD-PARTY-NOTICES.md"), `
              (Join-Path $root "LICENSE-APACHE"), (Join-Path $root "LICENSE-MIT") -Destination $stage
    New-Item -ItemType Directory -Force -Path (Join-Path $stage "tours") | Out-Null
    Copy-Item (Join-Path $root "tours\*.toml") -Destination (Join-Path $stage "tours")
    New-Item -ItemType Directory -Force -Path (Join-Path $stage "scripts") | Out-Null
    Copy-Item (Join-Path $root "scripts\*.example.toml") -Destination (Join-Path $stage "scripts")
    Copy-Item (Join-Path $root "scripts\deep-sample.fdn") -Destination (Join-Path $stage "scripts")
    # The validation battery itself, so a tester can run it from the extracted zip.
    Copy-Item (Join-Path $root "scripts\gpu-validate.ps1"), (Join-Path $root "scripts\gpu-validate.sh") `
              -Destination (Join-Path $stage "scripts")
    # Validation data so --selftest / --bench-matrix work from the extracted install.
    New-Item -ItemType Directory -Force -Path (Join-Path $stage "validation\golden") | Out-Null
    Copy-Item (Join-Path $root "validation\golden\*.png") -Destination (Join-Path $stage "validation\golden")
    Copy-Item (Join-Path $root "validation\golden\BLESSED-GPU.txt") -Destination (Join-Path $stage "validation\golden") -ErrorAction SilentlyContinue
    Copy-Item (Join-Path $root "validation\catalog.toml") -Destination (Join-Path $stage "validation")
    New-Item -ItemType Directory -Force -Path (Join-Path $stage "benchmarks") | Out-Null
    Copy-Item (Join-Path $root "benchmarks\bench-matrix-baseline.json") -Destination (Join-Path $stage "benchmarks")
    $zip = "$stage.zip"
    if (Test-Path $zip) { Remove-Item -Force $zip }
    Compress-Archive -Path $stage -DestinationPath $zip
    Publish $zip
}

# ---------------------------------------------------------------- windows x64 accelerated (MPFR)
$accel = Join-Path $root ("dist\fractadyne-$Tag-windows-x64-accelerated.zip")
if (Test-Path $accel) {
    # The cross-GPU battery is not in the accelerated package by default; put it there too so a
    # tester can validate whichever build they installed.
    Publish $accel
} else {
    Write-Host "  (no accelerated package for $Tag - run scripts\build-accelerated.ps1)" -ForegroundColor Yellow
}

# ---------------------------------------------------------------- source, for the Linux box
if (-not $SkipSource) {
    $src = Join-Path $root ("dist\fractadyne-$Tag-src.tar.gz")
    if (Test-Path $src) { Remove-Item -Force $src }
    # HEAD, not the worktree: the bundle must be exactly what is committed.
    & git archive --format=tar.gz --prefix="fractadyne-$Tag/" -o $src HEAD
    if ($LASTEXITCODE -ne 0) { throw "git archive failed" }
    Publish $src
}

Write-Host ""
Write-Host "Published:" -ForegroundColor Green
Get-ChildItem $dest | Sort-Object Name | Format-Table Name, @{n='MB';e={[Math]::Round($_.Length/1MB,2)}}, LastWriteTime -AutoSize
