<#
.SYNOPSIS
  Build and package the OPTIONAL accelerated (MPFR) Fractadyne build for Windows.

.DESCRIPTION
  The standard Fractadyne binary computes its deep-zoom reference orbits with astro-float, a pure
  Rust library. This packages an alternative binary that uses MPFR/GMP instead, which is 2.5x to
  6.4x faster at that one job. The two produce BYTE-IDENTICAL output; the difference is speed only.

  It is a separate download for two reasons that are not going away:

    1. MPFR does not build on the MSVC toolchain the standard Windows binary uses. This build
       needs the Rust GNU toolchain plus MSYS2 (see -Deps below for exactly what).

    2. rug, gmp-mpfr-sys, GMP and MPFR are all LGPL-3.0-or-later, whereas Fractadyne is
       MIT OR Apache-2.0. Those obligations attach to CONVEYING a binary, so they apply to this
       package and not to the standard one.

  LICENSING SHAPE (why this links DYNAMICALLY, which is not an implementation detail):
  GMP and MPFR are linked as SHARED libraries and shipped beside the executable, which is the
  mechanism LGPLv3 section 4(d)(1) provides for. That keeps the obligations to notices: this
  script ships the GPL and LGPL texts, records the exact library versions, and the user can drop
  in their own build of either DLL.
  Linking them STATICALLY instead would fall under section 4(d)(0), which requires shipping the
  application in a relinkable form with EVERY release. That is a permanent tax on a fast release
  cadence, which is why `--features gmp-mpfr-sys/use-system-libs` below is deliberate rather than
  incidental. Do not "simplify" it into a static build.

.PARAMETER Tag
  Version tag for the package name, e.g. v0.2.40-beta.156. Defaults to the workspace version.

.PARAMETER OutDir
  Where to write the package folder and .zip. Defaults to .\dist.

.PARAMETER Deps
  Print the exact prerequisites (and the commands to install them) and exit.

.PARAMETER SkipVerify
  Skip the clean-room verification. NOT recommended - read the comment above the verify step
  before you reach for this.

.PARAMETER Run
  Launch the freshly PACKAGED fractadyne.exe (from the dist folder, with its DLLs beside it -
  the exact bytes a user would run) once the build, packaging and verification have finished.
  The app starts detached in your normal environment (your real config dir and session), so
  this is "build it, then hand it to me", not another test harness.

.PARAMETER RunArgs
  Extra command-line arguments for the -Run launch, as one string (e.g. -RunArgs "--bignum rug"
  or -RunArgs "--selftest"). Ignored without -Run.

.EXAMPLE
  .\scripts\build-accelerated.ps1

.EXAMPLE
  .\scripts\build-accelerated.ps1 -Run

.EXAMPLE
  .\scripts\build-accelerated.ps1 -Run -RunArgs "--bignum rug"
#>
#Requires -Version 7.0
# (PowerShell 7+, not Windows PowerShell 5.1: under 5.1, `2>&1` wraps a native command's
# stderr in ErrorRecords, and this script's ErrorActionPreference = Stop then turns the
# app's ordinary startup banner into a terminating error at the verify step. Measured,
# not theoretical - the failure is confusing enough to deserve this one-line refusal.)
[CmdletBinding()]
param(
    [string]$Tag,
    [string]$OutDir = "dist",
    [switch]$Deps,
    [switch]$SkipVerify,
    [switch]$Run,
    [string]$RunArgs = ""
)

$ErrorActionPreference = "Stop"
$MSYS = "C:\msys64"
$MINGW_BIN = Join-Path $MSYS "mingw64\bin"
$SHARE = Join-Path $MSYS "mingw64\share"
$TRIPLE = "x86_64-pc-windows-gnu"

# The full runtime closure, not just the two libraries we call directly. libmpfr needs libgcc,
# which needs libwinpthread. Shipping only gmp+mpfr produces a package that builds, verifies on
# the BUILD machine, and then fails on every user's machine with 0xC0000135 (DLL not found) -
# which is exactly what happened before the verify step below was rewritten to use a clean PATH.
$RUNTIME_DLLS = @("libgmp-10.dll", "libmpfr-6.dll", "libgcc_s_seh-1.dll", "libwinpthread-1.dll")

function Fail($msg) { Write-Host "ERROR: $msg" -ForegroundColor Red; exit 1 }
function Step($msg) { Write-Host "== $msg" -ForegroundColor Cyan }

if ($Deps) {
    Write-Host @"
Prerequisites for the accelerated build
---------------------------------------
  1. Rust GNU toolchain:
       rustup toolchain install stable-$TRIPLE

  2. MSYS2 at $MSYS (winget install MSYS2.MSYS2), then inside it:
       pacman -Syuu --noconfirm
       pacman -S --noconfirm --needed diffutils m4 make mingw-w64-x86_64-gcc \
                                      mingw-w64-x86_64-pkgconf mingw-w64-x86_64-gmp \
                                      mingw-w64-x86_64-mpfr

     diffutils/m4/make/gcc build gmp-mpfr-sys; pkgconf lets it FIND the system libraries
     (without it the build fails with 'Unable to execute pkg-config'); gmp/mpfr are the
     libraries themselves, and their DLLs are what this package ships.
"@
    exit 0
}

# ---------------------------------------------------------------- preconditions
Step "Checking prerequisites"
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

if (-not (Test-Path $MINGW_BIN)) { Fail "MSYS2 not found at $MSYS. Run with -Deps for setup steps." }
foreach ($t in @("gcc.exe", "pkg-config.exe")) {
    if (-not (Test-Path (Join-Path $MINGW_BIN $t))) { Fail "$t missing from $MINGW_BIN. Run with -Deps." }
}
foreach ($t in @("m4.exe", "make.exe")) {
    if (-not (Test-Path (Join-Path $MSYS "usr\bin\$t"))) { Fail "$t missing from MSYS2. Run with -Deps." }
}
foreach ($d in $RUNTIME_DLLS) {
    if (-not (Test-Path (Join-Path $MINGW_BIN $d))) { Fail "$d not found in $MINGW_BIN (see -Deps)." }
}

# Licence texts. LGPLv3 section 4(b) REQUIRES shipping the LGPL and the GPL it refers to, so a
# missing text is a hard failure rather than a warning.
$licenses = @{
    "LICENSE-LGPL-3.0.txt"          = Join-Path $SHARE "doc\mpfr\COPYING.LESSER"
    "LICENSE-GPL-3.0.txt"           = Join-Path $SHARE "doc\mpfr\COPYING"
    "LICENSE-libgcc-runtime.txt"    = Join-Path $SHARE "licenses\gcc-libs\COPYING.RUNTIME"
    "LICENSE-libwinpthread.txt"     = Join-Path $SHARE "licenses\libwinpthread\COPYING"
}
# ALL of them, not just the LGPL pair: there is one entry per shipped DLL, and a conditional
# copy silently omitted the libgcc terms from a package once already. A missing licence text
# is a hard failure, never a warning.
foreach ($k in $licenses.Keys) {
    if (-not (Test-Path $licenses[$k])) { Fail "$k source missing ($($licenses[$k])). Its library ships in this package, so its licence must too; refusing to package." }
}
$toolchains = (& rustup toolchain list) -join "`n"
if ($toolchains -notmatch [regex]::Escape($TRIPLE)) { Fail "Rust toolchain stable-$TRIPLE not installed. Run with -Deps." }

if (-not $Tag) {
    $Tag = (Select-String -Path (Join-Path $root "Cargo.toml") -Pattern '^version = "(.+)"' |
            Select-Object -First 1).Matches[0].Groups[1].Value
    $Tag = "v$Tag"
}
Write-Host "  tag: $Tag"

# ---------------------------------------------------------------- delay-load libraries
# ***DELAY-LOAD GMP AND MPFR*** so a user whose DLLs went missing (they split the zip, antivirus
# quarantined one) gets an in-app warning and a fall back to the built-in astro-float arithmetic,
# instead of a bare Windows 0xC0000135 at startup with no message. See
# `fractadyne_core::resolve_startup_backend` for the app side, and `scripts/make-delay-libs.sh` for
# why delay loading needs dlltool stubs on this toolchain (no `ld --delayload`, empty libdelayimp).
# The stubs are SWAPPED IN for the normal import libraries for the duration of the build -- which is
# deterministic (gmp-mpfr-sys's own `-lmpfr` then resolves to the delay stub, with no linker
# search-order gamble; a `-L` ahead of its search path was tried and did not reliably win) -- and
# restored in a `finally` afterwards, so a shared MSYS2 install is never left modified. The
# eager/delay state is asserted after the build and again in the clean-room verify below; a
# delay-load regression must fail the release, not ship.
Step "Building delay-import libraries for GMP/MPFR"
$bash = Join-Path $MSYS "usr\bin\bash.exe"
$env:MSYSTEM = "MINGW64"
$MINGW_LIB = Join-Path $MSYS "mingw64\lib"
$rootU = ($root -replace '\\', '/') -replace '^([A-Za-z]):', '/$1'
$delayDir = [IO.Path]::GetFullPath((Join-Path $OutDir "_delaylibs"))
if (Test-Path $delayDir) { Remove-Item -Recurse -Force $delayDir }
New-Item -ItemType Directory -Force $delayDir | Out-Null
$delayDirU = ($delayDir -replace '\\', '/') -replace '^([A-Za-z]):', '/$1'
& $bash -lc "cd '$rootU' && bash scripts/make-delay-libs.sh '$delayDirU'"
if ($LASTEXITCODE -ne 0) { Fail "generating the delay-import libraries failed ($LASTEXITCODE)." }
foreach ($imp in @("libmpfr.dll.a", "libgmp.dll.a")) {
    if (-not (Test-Path (Join-Path $delayDir $imp))) { Fail "delay lib $imp was not produced." }
}

# ---------------------------------------------------------------- build (delay stubs swapped in)
# `use-system-libs` is the LGPL section 4(d)(1) shape. See the header.
Step "Building (GNU toolchain, MPFR backend, dynamically linked, MPFR/GMP delay-loaded)"
$cargoBinU = ((Join-Path $env:USERPROFILE ".cargo\bin") -replace '\\', '/') -replace '^([A-Za-z]):', '/$1'
$cmd = "export PATH=`"`$PATH:$cargoBinU`"; cd '$rootU' && cargo +stable-$TRIPLE build --release " +
       "--target $TRIPLE --bin fractadyne --features fractadyne-core/rug --features gmp-mpfr-sys/use-system-libs"
$swapped = @()
$buildExit = 1
try {
    foreach ($imp in @("libmpfr.dll.a", "libgmp.dll.a")) {
        $orig = Join-Path $MINGW_LIB $imp
        $bak = "$orig.fdbak"
        Copy-Item $orig $bak -Force
        Copy-Item (Join-Path $delayDir $imp) $orig -Force
        $swapped += @{ orig = $orig; bak = $bak }
    }
    & $bash -lc $cmd
    $buildExit = $LASTEXITCODE
}
finally {
    foreach ($s in $swapped) {
        Copy-Item $s.bak $s.orig -Force
        Remove-Item $s.bak -Force -ErrorAction SilentlyContinue
    }
}
if ($buildExit -ne 0) { Fail "cargo build failed ($buildExit)" }

$exe = Join-Path $root "target\$TRIPLE\release\fractadyne.exe"
if (-not (Test-Path $exe)) { Fail "build reported success but $exe is missing" }

# Packaging gate: the DLLs must be DELAY imports, not eager ones. An eager import means the delay
# libs did not win the link, and the whole graceful-fallback design is silently gone.
$eager = & (Join-Path $MINGW_BIN "objdump.exe") -p $exe 2>$null | Select-String 'DLL Name:' |
         Where-Object { $_ -match 'libmpfr-6\.dll|libgmp-10\.dll' }
if ($eager) { Fail "MPFR/GMP are EAGER imports - delay-load did not take, so a missing DLL would fail at startup instead of falling back." }
Write-Host "  MPFR/GMP are delay-imported (a missing DLL falls back to astro-float instead of failing to start)"

# ---------------------------------------------------------------- package
Step "Packaging"
$name = "fractadyne-$Tag-windows-x64-accelerated"
$dir = Join-Path $OutDir $name
if (Test-Path $dir) { Remove-Item -Recurse -Force $dir }
New-Item -ItemType Directory -Force $dir | Out-Null

Copy-Item $exe -Destination $dir
foreach ($d in $RUNTIME_DLLS) { Copy-Item (Join-Path $MINGW_BIN $d) -Destination $dir }
foreach ($k in $licenses.Keys) { Copy-Item $licenses[$k] -Destination (Join-Path $dir $k) }
Copy-Item (Join-Path $root "LICENSE-MIT"), (Join-Path $root "LICENSE-APACHE") -Destination $dir

# **The SAME user payload the standard Windows package ships** (release.yml's `windows` job) and
# the accelerated LINUX script (`build-accelerated.sh`) already stage. This script used to ship only
# the exe + DLLs + licences, so `--selftest`, `--uitest` and `--bench-matrix` all FAILED from the
# accelerated download -- the goldens, tours, scripts and benchmark baseline were simply absent (a
# tester reported `--uitest` failing on missing goldens, 2026-09-12). A green backend build is not a
# green PACKAGE: the presence check below is the packaging gate that would have caught this, and the
# three copy sites (here, the standard windows job, build-accelerated.sh) must stay in step.
Copy-Item (Join-Path $root "README.md"), (Join-Path $root "CHANGELOG.md"), (Join-Path $root "TOURS.md"), `
          (Join-Path $root "DIAGNOSTICS.md"), (Join-Path $root "THIRD-PARTY-NOTICES.md") -Destination $dir
New-Item -ItemType Directory -Force (Join-Path $dir "tours") | Out-Null
Copy-Item (Join-Path $root "tours\*.toml") -Destination (Join-Path $dir "tours")
New-Item -ItemType Directory -Force (Join-Path $dir "scripts") | Out-Null
Copy-Item (Join-Path $root "scripts\*.example.toml") -Destination (Join-Path $dir "scripts")
Copy-Item (Join-Path $root "scripts\deep-sample.fdn") -Destination (Join-Path $dir "scripts")
# Validation data so `--selftest` / `--uitest` / `--bench-matrix` work from the extracted install:
# `anchored()` walks up from the binary and finds this `validation/` tree beside it.
New-Item -ItemType Directory -Force (Join-Path $dir "validation\golden") | Out-Null
Copy-Item (Join-Path $root "validation\golden\*.png") -Destination (Join-Path $dir "validation\golden")
# BLESSED-GPU.txt records which card produced the goldens; without it the self-test uses the strict
# tolerance (the safe direction), so its absence degrades gracefully.
Copy-Item (Join-Path $root "validation\golden\BLESSED-GPU.txt") -Destination (Join-Path $dir "validation\golden") -ErrorAction SilentlyContinue
Copy-Item (Join-Path $root "validation\catalog.toml") -Destination (Join-Path $dir "validation")
New-Item -ItemType Directory -Force (Join-Path $dir "benchmarks") | Out-Null
Copy-Item (Join-Path $root "benchmarks\bench-matrix-baseline.json") -Destination (Join-Path $dir "benchmarks")

# **Packaging gate.** A missing golden is invisible until a tester runs `--uitest` on the
# download -- exactly how this was found. Fail the build if the user-facing payload is incomplete,
# so a green CI run cannot ship a package that cannot self-test. Goldens are a floor, not an exact
# count (adding one must not break the release).
$goldenCount = @(Get-ChildItem (Join-Path $dir "validation\golden") -Filter *.png -ErrorAction SilentlyContinue).Count
if ($goldenCount -lt 15) { Fail "package staged only $goldenCount golden PNGs (< 15) - validation data is incomplete" }
foreach ($req in @("validation\catalog.toml", "benchmarks\bench-matrix-baseline.json", "DIAGNOSTICS.md",
                   "tours", "scripts\deep-sample.fdn")) {
    if (-not (Test-Path (Join-Path $dir $req))) { Fail "package is missing '$req' - the accelerated payload is incomplete" }
}
Write-Host "  staged $goldenCount goldens + validation/tours/scripts/benchmarks (matches the standard package)"

$pacman = Join-Path $MSYS "usr\bin\pacman.exe"
$gmpVer = (& $pacman -Q mingw-w64-x86_64-gmp) -replace '.*\s'
$mpfrVer = (& $pacman -Q mingw-w64-x86_64-mpfr) -replace '.*\s'

$readme = @"
Fractadyne - accelerated build (optional)
=========================================

This is the SAME Fractadyne as the standard download, with one difference: it computes
deep-zoom reference orbits using MPFR/GMP instead of the pure-Rust library the standard
build uses. On the reference-orbit build - the CPU pause before a deep view starts
resolving - it is roughly 2.5x to 6.4x faster, and more so the deeper you go.

The two builds produce BYTE-IDENTICAL images. This is verified, not assumed: the same
reference orbits, every fractal formula, at arithmetic widths from 64 bits to 132,000
bits, plus the full 38-location deep-zoom comparison corpus. If you ever see a difference
in output between the two builds, that is a bug - please report it.

HOW TO USE IT
-------------
Extract this folder anywhere and run fractadyne.exe from it. Keep all four .dll files
(libgmp-10.dll, libmpfr-6.dll, libgcc_s_seh-1.dll, libwinpthread-1.dll) next to the
executable - that is what makes this build the fast one.

IF A .DLL IS MISSING, the program still starts: it shows a notice, falls back to the
built-in pure-Rust arithmetic (astro-float), and runs normally - only the pause while a
deep view's reference orbit is built is slower. Your images are unaffected; the two are
byte-identical. To get the fast path back, either:

  * restore the libraries next to fractadyne.exe - re-extract the whole zip so the DLLs sit
    beside the exe, or obtain compatible builds of libgmp-10.dll and libmpfr-6.dll (plus
    libgcc_s_seh-1.dll and libwinpthread-1.dll) from MSYS2 (https://www.msys2.org/ -
    packages mingw-w64-x86_64-gmp and mingw-w64-x86_64-mpfr), or from https://gmplib.org/
    and https://www.mpfr.org/ , and drop them in this folder; or

  * just use the STANDARD (non-accelerated) download, which needs no DLLs at all:
    https://github.com/WindySnowOwl/fractadyne/releases

Your settings, saved session and locations are SHARED with the standard build - they live
in your user profile, not next to the executable - so you can switch between the two
freely and everything carries over. Nothing needs importing or converting.

To confirm which arithmetic you are running, open Help -> Report an issue: the system
information block names the backend in use. You can also force either one:
    fractadyne.exe --bignum astro
    fractadyne.exe --bignum rug

WHY IT IS A SEPARATE DOWNLOAD
-----------------------------
Two reasons, neither of which is going away:

  * MPFR cannot be built with the Microsoft compiler the standard Windows binary uses.
    This build is produced with a different toolchain.

  * MPFR and GMP are licensed under the GNU LGPL v3, while Fractadyne itself is
    MIT OR Apache-2.0. Keeping them in a separate, clearly-labelled download keeps the
    standard build free of those terms.

LICENSING
---------
Fractadyne's own code: MIT OR Apache-2.0 (LICENSE-MIT, LICENSE-APACHE).

This package also contains these separate, unmodified shared libraries:

  * GNU MP (GMP) $gmpVer         - https://gmplib.org/        - LGPL-3.0-or-later
  * GNU MPFR $mpfrVer            - https://www.mpfr.org/      - LGPL-3.0-or-later
  * libgcc_s_seh-1.dll                                        - GPL-3.0 with the GCC
                                                                Runtime Library Exception
  * libwinpthread-1.dll (mingw-w64)                           - see its licence file

Copies of the GNU Lesser General Public License and of the GNU General Public License it
refers to are included as LICENSE-LGPL-3.0.txt and LICENSE-GPL-3.0.txt; the other two
libraries' terms are in LICENSE-libgcc-runtime.txt and LICENSE-libwinpthread.txt.

GMP and MPFR are linked dynamically and shipped as ordinary DLLs beside the executable
specifically so that you can replace them: build or obtain your own libgmp-10.dll or
libmpfr-6.dll with a compatible interface, drop it in this folder, and this program will
use it instead.

Source for the exact versions above is available from the project sites listed. The builds
used here are the MSYS2 packages mingw-w64-x86_64-gmp $gmpVer and mingw-w64-x86_64-mpfr
$mpfrVer, whose sources are published at https://packages.msys2.org/ .
"@
Set-Content -Path (Join-Path $dir "README-ACCELERATED.txt") -Value $readme -Encoding ASCII

# ---------------------------------------------------------------- verify (CLEAN ROOM)
# This runs the PACKAGED binary with MSYS2 removed from PATH, i.e. under the conditions a user
# actually has - not the build machine's.
#
# WARNING: the first version of this script verified the binary in the build tree with $MINGW_BIN
# PREPENDED to PATH. It passed, and the package it blessed failed on any machine without MSYS2
# because libmpfr's own dependency on libgcc was missing. A check run under conditions the user
# will never have is not a check. Do not "fix" a failure here by widening PATH.
if (-not $SkipVerify) {
    Step "Verifying the PACKAGE on a clean PATH (no MSYS2)"

    $pkgExe = Join-Path $dir "fractadyne.exe"
    $bytes = [IO.File]::ReadAllBytes($pkgExe)
    $ascii = [Text.Encoding]::ASCII.GetString($bytes)
    foreach ($d in @("libgmp-10.dll", "libmpfr-6.dll")) {
        if ($ascii -notmatch [regex]::Escape($d)) {
            Fail "$pkgExe does not import $d - it linked statically, which is the WRONG licensing shape (see header)."
        }
    }
    Write-Host "  imports libgmp-10.dll + libmpfr-6.dll (dynamic: correct)"

    $savedPath = $env:PATH
    $cfg = Join-Path $env:TEMP ("fd-accel-" + [guid]::NewGuid().ToString("N").Substring(0, 8))
    New-Item -ItemType Directory -Force $cfg | Out-Null
    try {
        $env:PATH = (Join-Path $env:SystemRoot "System32") + ";" + $env:SystemRoot
        $env:FRACTADYNE_CONFIG_DIR = $cfg
        $env:FRACTADYNE_NO_SOUND = "1"

        $ver = & $pkgExe --version 2>&1 | Out-String
        if ($LASTEXITCODE -ne 0) {
            Fail ("the packaged binary does not start on a clean PATH (exit $LASTEXITCODE). " +
                  "0xC0000135 means a DLL is missing from the package - add it to `$RUNTIME_DLLS.`n$ver")
        }
        Write-Host "  starts with no MSYS2 on PATH"

        # ...and it must actually iterate in MPFR. --bench-bignum reports the backend that produced
        # its numbers, taken from what ran rather than from a flag, and exits non-zero if the
        # backends ever disagree.
        $out = & $pkgExe --bench-bignum --iters 0.02 2>&1 | Out-String
        if ($LASTEXITCODE -ne 0) { Fail "--bench-bignum failed (exit $LASTEXITCODE):`n$out" }
        if ($out -notmatch "rug") { Fail "the packaged binary did not report the MPFR backend:`n$out" }
        if ($out -match "DIFFERS") { Fail "backends disagreed - refusing to package:`n$out" }
        Write-Host "  runs, reports the MPFR backend, and both backends agree"

        # ***THE DELAY-LOAD GATE.*** Copy JUST the exe (no DLLs) to a clean folder and confirm it
        # STARTS and falls back to astro-float instead of dying with 0xC0000135. This is the whole
        # point of the delay-load work, and the one check that proves a user with missing DLLs gets
        # a warning rather than a broken program. Without it a delay-load regression ships silently.
        $noDll = Join-Path $env:TEMP ("fd-accel-nodll-" + [guid]::NewGuid().ToString("N").Substring(0, 8))
        New-Item -ItemType Directory -Force $noDll | Out-Null
        try {
            Copy-Item $pkgExe (Join-Path $noDll "fractadyne.exe")
            $fb = & (Join-Path $noDll "fractadyne.exe") --version 2>&1 | Out-String
            if ($LASTEXITCODE -ne 0) {
                Fail ("with its DLLs removed the binary did not start (exit $LASTEXITCODE) - delay-load is not " +
                      "working, so a user missing a DLL gets 0xC0000135 instead of a fallback.`n$fb")
            }
            if ($fb -notmatch "fell back to astro-float") {
                Fail "with its DLLs removed the binary started but did not fall back to astro-float:`n$fb"
            }
            Write-Host "  with the DLLs removed it starts and falls back to astro-float (delay-load works)"
        }
        finally { Remove-Item -Recurse -Force $noDll -ErrorAction SilentlyContinue }
    }
    finally {
        $env:PATH = $savedPath
        $env:FRACTADYNE_CONFIG_DIR = $null
        Remove-Item -Recurse -Force $cfg -ErrorAction SilentlyContinue
    }
}

# The delay-import stubs were a build input, not a package artifact - drop them.
Remove-Item -Recurse -Force $delayDir -ErrorAction SilentlyContinue

# ---------------------------------------------------------------- zip
$zip = Join-Path $OutDir "$name.zip"
if (Test-Path $zip) { Remove-Item -Force $zip }
Compress-Archive -Path "$dir\*" -DestinationPath $zip
$mb = [math]::Round((Get-Item $zip).Length / 1MB, 1)

Write-Host ""
Write-Host "Packaged: $zip ($mb MB)" -ForegroundColor Green
Get-ChildItem $dir | ForEach-Object { Write-Host ("  " + $_.Name) }

# ---------------------------------------------------------------- run (optional)
# Launches the PACKAGED exe from the dist folder - its DLLs sit beside it, so this runs the
# exact bytes a user would, with no PATH additions (the same discipline as the verify step).
# Detached: the script's job is done; the app is yours.
if ($Run) {
    $runExe = Join-Path $dir "fractadyne.exe"
    Step "Launching $runExe"
    if ($RunArgs) {
        Write-Host "  args: $RunArgs"
        Start-Process -FilePath $runExe -WorkingDirectory $dir -ArgumentList $RunArgs
    } else {
        Start-Process -FilePath $runExe -WorkingDirectory $dir
    }
}
