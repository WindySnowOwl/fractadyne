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
# IT REFUSES A BINARY IT CANNOT ATTRIBUTE. Every package's exe is asked for its --version, whose
# commit field (build.rs FRACT_GIT) must name HEAD and must not be -dirty. A 2026-08-16 validation
# run measured the previous binary because nothing tied the share's contents to a commit; a stale
# target\release or a leftover accelerated zip in dist\ is exactly that mistake, made at publish
# time. -AllowDirty publishes a -dirty build anyway (a work-in-progress test build), and records it
# as such; nothing publishes a binary built from a commit other than HEAD.
#
# BUILD-ID.txt, written last into <Share>\builds\<tag>\, is what the share trusts: the tag, the full
# commit, each package's sha256, and each exe's own --version line. gpu-validate.ps1 step 00 checks
# the binary under test against it. The source tarball carries BUILD-COMMIT.txt so a build made from
# it (no .git) still names its commit, as g<sha>-archive.
#
# ASCII-only (Windows PowerShell 5.1 reads a BOM-less .ps1 as ANSI).

[CmdletBinding()]
param(
    [string]$Share = "",
    [string]$Tag = "",
    [switch]$SkipSource,
    [switch]$SkipWindows,
    [switch]$AllowDirty
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

# ---------------------------------------------------------------- identity, checked BEFORE copying
$headFull = (& git rev-parse HEAD).Trim()
if ($LASTEXITCODE -ne 0 -or $headFull -notmatch '^[0-9a-f]{40}$') { throw "git rev-parse HEAD failed" }
$headShort = (& git rev-parse --short HEAD).Trim()

# Ask a binary for its identity WITHOUT touching the real config dir. --version runs the unclean-exit
# check, and against the real dir it can take a LIVE session for a crash and write a spurious crash
# report into it (it has, 2026-08-27). Returns the "fractadyne ..." line, or $null.
function Get-ExeVersion([string]$exePath) {
    $ErrorActionPreference = "Continue"   # the app's stderr banner must not become a terminating error
    $scratch = Join-Path ([IO.Path]::GetTempPath()) ("fd-version-" + [guid]::NewGuid().ToString("N"))
    New-Item -ItemType Directory -Force -Path $scratch | Out-Null
    $prev = $env:FRACTADYNE_CONFIG_DIR
    try {
        $env:FRACTADYNE_CONFIG_DIR = $scratch
        $out = & $exePath --version 2>$null
        return ($out | Where-Object { $_ -match '^fractadyne ' } | Select-Object -Last 1)
    }
    finally {
        if ($null -eq $prev) { Remove-Item Env:\FRACTADYNE_CONFIG_DIR -ErrorAction SilentlyContinue }
        else { $env:FRACTADYNE_CONFIG_DIR = $prev }
        Remove-Item -Recurse -Force $scratch -ErrorAction SilentlyContinue
    }
}

# Throws unless $line names this tag's version and a clean build of HEAD. Returns the version part
# (everything after "fractadyne "), which is what BUILD-ID.txt records.
function Assert-Identity([string]$what, [string]$line) {
    if (-not $line) { throw "$what did not answer --version - is it a Fractadyne binary?" }
    if ($line -notmatch '^fractadyne (\S+) \(build (\d+), (g([0-9a-f]{7,40})(-dirty|-archive)?|git unknown)\)$') {
        throw "$what reports '$line', which carries no commit field - it predates build identity; rebuild it."
    }
    $ver = $Matches[1]; $sha = $Matches[4]; $suffix = $Matches[5]
    if ("v$ver" -ne $Tag) { throw "$what is version $ver, but this publish is $Tag." }
    if (-not $sha) { throw "$what was built without a commit ('git unknown') - it cannot be attributed." }
    if (-not $headFull.StartsWith($sha)) {
        throw "$what was built from commit $sha, but HEAD is $headShort - it is STALE. Rebuild it."
    }
    if ($suffix -eq "-dirty") {
        if (-not $AllowDirty) {
            throw "$what was built from a DIRTY tree (g$sha-dirty). Commit first, or pass -AllowDirty for a test build."
        }
        Write-Host "  WARNING: $what is a -dirty build; BUILD-ID.txt records it as such." -ForegroundColor Yellow
    }
    return $line.Substring("fractadyne ".Length)
}

$exeIds = @()   # "<package> | <version line>" for BUILD-ID.txt
if (-not $SkipWindows) {
    $exe = Join-Path $root "target\release\fractadyne.exe"
    if (-not (Test-Path $exe)) { throw "target\release\fractadyne.exe is missing - build it first" }
    $v = Assert-Identity "target\release\fractadyne.exe" (Get-ExeVersion $exe)
    $exeIds += "fractadyne-$Tag-windows-x64.zip | $v"
    Write-Host "  windows exe     : $v"
}
$accel = Join-Path $root ("dist\fractadyne-$Tag-windows-x64-accelerated.zip")
if (Test-Path $accel) {
    # A zip left in dist\ by an earlier build of the same tag is the likeliest stale binary of all,
    # so it is unpacked and asked, not trusted by its name. The MPFR exe needs its DLLs beside it.
    $probe = Join-Path ([IO.Path]::GetTempPath()) ("fd-accel-" + [guid]::NewGuid().ToString("N"))
    try {
        Expand-Archive -Path $accel -DestinationPath $probe -Force
        $aexe = Get-ChildItem $probe -Recurse -Filter "fractadyne.exe" | Select-Object -First 1
        if (-not $aexe) { throw "the accelerated zip holds no fractadyne.exe" }
        $v = Assert-Identity "the accelerated zip's exe" (Get-ExeVersion $aexe.FullName)
        $exeIds += "fractadyne-$Tag-windows-x64-accelerated.zip | $v"
        Write-Host "  accelerated exe : $v"
    }
    finally { Remove-Item -Recurse -Force $probe -ErrorAction SilentlyContinue }
}

$dest = Join-Path (Join-Path $Share "builds") $Tag
New-Item -ItemType Directory -Force -Path $dest | Out-Null
Write-Host "Publishing $Tag to $dest" -ForegroundColor Cyan

$published = @()   # "<sha256>  <name>" for each package THIS run copied - not whatever an earlier run left
function Publish([string]$path) {
    $name = Split-Path -Leaf $path
    Copy-Item $path (Join-Path $dest $name) -Force
    $hash = (Get-FileHash (Join-Path $dest $name) -Algorithm SHA256).Hash.ToLower()
    "$hash  $name" | Out-File -FilePath (Join-Path $dest "$name.sha256") -Encoding ascii
    $script:published += "$hash  $name"
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
    # HEAD, not the worktree: the bundle must be exactly what is committed. Plus BUILD-COMMIT.txt,
    # which build.rs reads when there is no .git, so the Linux build still names its commit. It is
    # added with --add-file (never committed; .gitignore) and must be named exactly that.
    $stampDir = Join-Path ([IO.Path]::GetTempPath()) ("fd-stamp-" + [guid]::NewGuid().ToString("N"))
    New-Item -ItemType Directory -Force -Path $stampDir | Out-Null
    $stamp = Join-Path $stampDir "BUILD-COMMIT.txt"
    [IO.File]::WriteAllText($stamp, "$headFull`n$headShort`n", [Text.Encoding]::ASCII)
    try {
        & git archive --format=tar.gz --prefix="fractadyne-$Tag/" "--add-file=$stamp" -o $src HEAD
        if ($LASTEXITCODE -ne 0) { throw "git archive failed" }
    }
    finally { Remove-Item -Recurse -Force $stampDir -ErrorAction SilentlyContinue }
    Publish $src
    $exeIds += "fractadyne-$Tag-src.tar.gz | builds as g$headShort-archive"
}

# ---------------------------------------------------------------- BUILD-ID.txt, written last
# What the share trusts. One "key: value" per line so a script can read it without a parser:
# gpu-validate.ps1 step 00 matches the binary under test against the `exe:` lines.
$idLines = @(
    "# Fractadyne share build identity - written by scripts/publish-share.ps1",
    "tag: $Tag",
    "commit: $headFull",
    "short: $headShort",
    ("published_utc: " + (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ"))
)
foreach ($e in $exeIds) { $idLines += "exe: $e" }
foreach ($p in $published) { $idLines += "sha256: $p" }
[IO.File]::WriteAllText((Join-Path $dest "BUILD-ID.txt"), (($idLines -join "`r`n") + "`r`n"), [Text.Encoding]::ASCII)

Write-Host ""
Write-Host "Published:" -ForegroundColor Green
Get-ChildItem $dest | Sort-Object Name | Format-Table Name, @{n='MB';e={[Math]::Round($_.Length/1MB,2)}}, LastWriteTime -AutoSize
