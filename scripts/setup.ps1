#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Set up the Windows build environment for Fractadyne.

.DESCRIPTION
    Idempotent bootstrap for someone who just downloaded the source. It checks for (and can
    install) the two things a Windows build needs:
      1. The Rust toolchain (rustup + the stable `x86_64-pc-windows-msvc` toolchain).
      2. The MSVC C++ build tools + Windows SDK that the Rust `msvc` toolchain links against.
    Then it runs a verification build so you know the environment actually works.

    Safe to re-run: anything already present is left alone.

.PARAMETER Yes
    Don't prompt before installing anything (for unattended / scripted setup).

.PARAMETER SkipBuild
    Skip the final verification build (just set up the toolchain).

.EXAMPLE
    ./scripts/setup.ps1
.EXAMPLE
    ./scripts/setup.ps1 -Yes
#>
[CmdletBinding()]
param(
    [switch]$Yes,
    [switch]$SkipBuild
)

$ErrorActionPreference = 'Stop'

function Write-Step($msg) { Write-Host "`n==> $msg" -ForegroundColor Cyan }
function Write-Ok($msg)   { Write-Host "  [ok] $msg" -ForegroundColor Green }
function Write-Warn2($msg) { Write-Host "  [!]  $msg" -ForegroundColor Yellow }

function Confirm-Action($msg) {
    if ($Yes) { return $true }
    $ans = Read-Host "$msg [y/N]"
    return $ans -match '^(y|yes)$'
}

# rustup puts binaries here; make them usable within this session even right after install.
$cargoBin = Join-Path $env:USERPROFILE '.cargo\bin'
if ((Test-Path $cargoBin) -and ($env:Path -notlike "*$cargoBin*")) {
    $env:Path = "$cargoBin;$env:Path"
}

function Have($name) { return [bool](Get-Command $name -ErrorAction SilentlyContinue) }

Write-Host "Fractadyne - Windows build environment setup" -ForegroundColor White
Write-Host "(this only installs a Rust toolchain and, if needed, the MSVC C++ build tools)"

# ---------------------------------------------------------------------------
# 1. Rust toolchain (rustup)
# ---------------------------------------------------------------------------
Write-Step "Checking for the Rust toolchain (rustup)"
if (Have 'rustup') {
    Write-Ok "rustup found: $((rustup --version) -split "`n" | Select-Object -First 1)"
}
else {
    Write-Warn2 "rustup is not installed."
    if (-not (Confirm-Action "Install rustup now?")) {
        Write-Host "Aborting. Install Rust from https://rustup.rs and re-run this script." -ForegroundColor Red
        exit 1
    }
    # rustup-init IS the vendor's unattended installer and needs no administrator rights, so it is
    # preferred over winget rather than used as its fallback. winget would install the same thing
    # while adding two ways to stall at "0%": a UAC prompt that may never surface in this window,
    # and the `msstore` source waiting on an agreement. Neither can happen here.
    Write-Host "  Downloading rustup-init.exe..."
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    $init = Join-Path $env:TEMP 'rustup-init.exe'
    Invoke-WebRequest -Uri 'https://win.rustup.rs/x86_64' -OutFile $init -TimeoutSec 600
    & $init -y --default-toolchain stable --profile default
    Remove-Item $init -ErrorAction SilentlyContinue
    if ((Test-Path $cargoBin) -and ($env:Path -notlike "*$cargoBin*")) {
        $env:Path = "$cargoBin;$env:Path"
    }
    if (-not (Have 'rustup')) {
        Write-Host "rustup still isn't on PATH. Open a NEW terminal and re-run this script." -ForegroundColor Red
        exit 1
    }
    Write-Ok "rustup installed."
}

Write-Step "Ensuring the stable MSVC toolchain is installed and default"
rustup toolchain install stable | Out-Null
rustup default stable | Out-Null
$host_tuple = (rustc -vV | Select-String '^host:' ) -replace 'host:\s*', ''
Write-Ok "Default toolchain host: $host_tuple"
if ($host_tuple -notlike '*windows-msvc*') {
    Write-Warn2 "Host is not the MSVC toolchain ($host_tuple). This project targets x86_64-pc-windows-msvc."
    Write-Warn2 "If you hit issues, run: rustup default stable-x86_64-pc-windows-msvc"
}

# ---------------------------------------------------------------------------
# 2. MSVC C++ build tools (the linker + Windows SDK the `msvc` toolchain needs)
# ---------------------------------------------------------------------------
Write-Step "Checking for the MSVC C++ build tools (linker + Windows SDK)"
$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
$haveMsvc = $false
if (Have 'cl') {
    $haveMsvc = $true  # already inside a Developer prompt
}
elseif (Test-Path $vswhere) {
    $vc = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null
    if ($vc) { $haveMsvc = $true }
}
if ($haveMsvc) {
    Write-Ok "MSVC C++ build tools present."
}
else {
    Write-Warn2 "The MSVC C++ build tools (with the Windows SDK) were not detected."
    Write-Warn2 "The Rust msvc toolchain needs them to link. Install the 'Desktop development with C++' workload."
    if (Have 'winget') {
        # This one genuinely installs machine-wide, so it needs administrator rights. Unelevated,
        # winget waits on a UAC prompt that may never appear in this window and simply sits at
        # "0%" - so say that BEFORE starting a multi-gigabyte download, not after.
        $elevated = (New-Object Security.Principal.WindowsPrincipal(
            [Security.Principal.WindowsIdentity]::GetCurrent())).IsInRole(
            [Security.Principal.WindowsBuiltInRole]::Administrator)
        if (-not $elevated) {
            Write-Warn2 "This window is NOT elevated. Installing the build tools needs administrator"
            Write-Warn2 "rights; without them winget stalls at 0% on a UAC prompt you may never see."
            Write-Warn2 "Re-run this script from an elevated PowerShell, or install the workload by hand:"
            Write-Warn2 "  https://visualstudio.microsoft.com/downloads/ -> Build Tools -> Desktop development with C++"
        }
        elseif (Confirm-Action "Install Visual Studio 2022 Build Tools (C++ workload) via winget now? (large download)") {
            # --source winget skips msstore (another silent stall); --disable-interactivity makes
            # winget FAIL instead of waiting, which is what turns a hang into a message.
            winget install --id Microsoft.VisualStudio.2022.BuildTools -e --source winget `
                --disable-interactivity --accept-source-agreements --accept-package-agreements `
                --override "--quiet --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
            Write-Warn2 "After it finishes, open a NEW terminal so the tools are on PATH, then re-run this script."
        }
        else {
            Write-Warn2 "Skipping. The verification build will likely fail to link until these are installed."
        }
    }
    else {
        Write-Warn2 "Install from https://visualstudio.microsoft.com/downloads/ (Build Tools for Visual Studio -> 'Desktop development with C++')."
    }
}

# ---------------------------------------------------------------------------
# 3. Verification build
# ---------------------------------------------------------------------------
if ($SkipBuild) {
    Write-Step "Skipping the verification build (-SkipBuild)."
}
else {
    Write-Step "Verification build (cargo build --bin fractadyne)"
    Write-Host "  The first build fetches wgpu/egui and may take several minutes..."
    # Deliberately NOT using -j1 / no-debuginfo - those are the author machine's page-file
    # workaround (see Cargo.toml), not something a normal build environment needs.
    Push-Location (Join-Path $PSScriptRoot '..')
    try {
        cargo build --bin fractadyne
        Write-Ok "Build succeeded."
        Write-Host "`nEnvironment is ready. Next:" -ForegroundColor White
        Write-Host "  cargo run --release -p fractadyne-app        # launch the app"
        Write-Host "  cargo test  -p fractadyne-core               # exact-math tests"
        Write-Host "  cargo run -p fractadyne-app -- --selftest    # GPU validation (needs a GPU)"
    }
    catch {
        Write-Host "`nBuild failed. If it's a linker error (link.exe / kernel32.lib not found)," -ForegroundColor Red
        Write-Host "install the MSVC 'Desktop development with C++' workload (see above) and re-run." -ForegroundColor Red
        Pop-Location
        exit 1
    }
    Pop-Location
}

Write-Host "`nDone." -ForegroundColor Green
