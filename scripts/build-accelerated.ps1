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

.EXAMPLE
  .\scripts\build-accelerated.ps1
#>
[CmdletBinding()]
param(
    [string]$Tag,
    [string]$OutDir = "dist",
    [switch]$Deps,
    [switch]$SkipVerify
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

# ---------------------------------------------------------------- build
# `use-system-libs` is the LGPL section 4(d)(1) shape. See the header.
Step "Building (GNU toolchain, MPFR backend, dynamically linked)"
$bash = Join-Path $MSYS "usr\bin\bash.exe"
$cargoBinU = ((Join-Path $env:USERPROFILE ".cargo\bin") -replace '\\', '/') -replace '^([A-Za-z]):', '/$1'
$rootU = ($root -replace '\\', '/') -replace '^([A-Za-z]):', '/$1'
$cmd = "export PATH=`"`$PATH:$cargoBinU`"; cd '$rootU' && cargo +stable-$TRIPLE build --release " +
       "--target $TRIPLE --bin fractadyne --features fractadyne-core/rug --features gmp-mpfr-sys/use-system-libs"
$env:MSYSTEM = "MINGW64"
& $bash -lc $cmd
if ($LASTEXITCODE -ne 0) { Fail "cargo build failed ($LASTEXITCODE)" }

$exe = Join-Path $root "target\$TRIPLE\release\fractadyne.exe"
if (-not (Test-Path $exe)) { Fail "build reported success but $exe is missing" }

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
Extract this folder anywhere and run fractadyne.exe from it. Keep the .dll files next to
the executable; the program will not start without them.

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
    }
    finally {
        $env:PATH = $savedPath
        $env:FRACTADYNE_CONFIG_DIR = $null
        Remove-Item -Recurse -Force $cfg -ErrorAction SilentlyContinue
    }
}

# ---------------------------------------------------------------- zip
$zip = Join-Path $OutDir "$name.zip"
if (Test-Path $zip) { Remove-Item -Force $zip }
Compress-Archive -Path "$dir\*" -DestinationPath $zip
$mb = [math]::Round((Get-Item $zip).Length / 1MB, 1)

Write-Host ""
Write-Host "Packaged: $zip ($mb MB)" -ForegroundColor Green
Get-ChildItem $dir | ForEach-Object { Write-Host ("  " + $_.Name) }
