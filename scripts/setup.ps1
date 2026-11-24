#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Set up the Windows build environment for Fractadyne.

.DESCRIPTION
    Idempotent bootstrap for someone who just downloaded the source. It checks for (and can
    install) the three things a Windows build needs:
      1. The Rust toolchain (rustup + the stable `x86_64-pc-windows-msvc` toolchain).
      2. The MSVC C++ build tools + Windows SDK that the Rust `msvc` toolchain links against.
      3. NASM, which assembles rav1e's fast paths (the AVIF export's encoder).
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
Write-Host "(this only installs a Rust toolchain and, if needed, the MSVC C++ build tools and NASM)"

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
    # !!NOT VIA WINGET. It stalled at "0%" here (2026-08-15, elevated), as it did for Git and for
    # ffmpeg in windows-build.ps1 - and while stalled it also scrambled this script's output, because
    # its progress display repositions the cursor and every later line then starts at an arbitrary
    # column. Microsoft's own bootstrapper installs the same workload unattended, with no package
    # manager in between.
    $elevated = (New-Object Security.Principal.WindowsPrincipal(
        [Security.Principal.WindowsIdentity]::GetCurrent())).IsInRole(
        [Security.Principal.WindowsBuiltInRole]::Administrator)
    if (-not $elevated) {
        Write-Warn2 "This window is NOT elevated, and these install machine-wide."
        Write-Warn2 "Re-run from an elevated PowerShell, or install the workload by hand:"
        Write-Warn2 "  https://visualstudio.microsoft.com/downloads/ -> Build Tools -> Desktop development with C++"
    }
    elseif (Confirm-Action "Install Visual Studio 2022 Build Tools (C++ workload) now? (multi-GB download)") {
        Write-Host "  This is the long one: several GB, typically 10-30 minutes, and it prints"
        Write-Host "  NOTHING while it works. That is the installer being silent, not a hang."
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        $boot = Join-Path $env:TEMP 'vs_BuildTools.exe'
        $prev = $ProgressPreference
        $ProgressPreference = 'SilentlyContinue'   # the 5.1 progress bar dominates the transfer
        try {
            Invoke-WebRequest -Uri 'https://aka.ms/vs/17/release/vs_BuildTools.exe' -OutFile $boot -TimeoutSec 900
        }
        finally { $ProgressPreference = $prev }
        # --wait makes the bootstrapper block until the real installer finishes, so the exit code
        # below describes the INSTALL rather than the download. 3010 = success, reboot required.
        $p = Start-Process -FilePath $boot -PassThru -ArgumentList `
            '--quiet', '--wait', '--norestart', '--nocache', `
            '--add', 'Microsoft.VisualStudio.Workload.VCTools', '--includeRecommended'
        if (-not $p.WaitForExit(90 * 60 * 1000)) {
            try { $p.Kill() } catch { }
            Write-Warn2 "The build tools installer did not finish in 90 minutes; install it by hand."
        }
        elseif ($p.ExitCode -eq 0 -or $p.ExitCode -eq 3010) {
            Write-Ok "Build tools installed$(if ($p.ExitCode -eq 3010) { ' (a reboot is pending)' })."
            Write-Warn2 "Open a NEW terminal so the tools are on PATH, then re-run this script."
        }
        else {
            Write-Warn2 "The build tools installer exited $($p.ExitCode). See %TEMP%\dd_setup_*.log."
        }
        Remove-Item $boot -ErrorAction SilentlyContinue
    }
    else {
        Write-Warn2 "Skipping. The verification build will likely fail to link until these are installed."
    }
}

# ---------------------------------------------------------------------------
# 3. NASM (rav1e, the AVIF export's encoder, assembles its fast paths with it)
# ---------------------------------------------------------------------------
Write-Step "Checking for NASM (the assembler the AVIF encoder builds with)"
# The nasm.us installer (asInvoker: no UAC prompt) goes to %LOCALAPPDATA%\bin\NASM unelevated or
# Program Files elevated, and adds NEITHER to PATH, so look in both and put the one found on this
# session's PATH. The version and hash are pinned; nasm.us keeps every release build.
$NasmVersion = '3.02'
$NasmSha256 = '0ddb40310861eb29f4d649feb9466779982a2d251c0db2b9cf0d21cf591171f3'
function Add-NasmPath {
    foreach ($d in @((Join-Path $env:ProgramFiles 'NASM'), (Join-Path $env:LOCALAPPDATA 'bin\NASM'))) {
        if (Test-Path (Join-Path $d 'nasm.exe')) {
            if ($env:Path -notlike "*$d*") { $env:Path = "$d;$env:Path" }
            return $d
        }
    }
    return $null
}
$nasmDir = $null
if (-not (Have 'nasm')) { $nasmDir = Add-NasmPath }
if (Have 'nasm') {
    Write-Ok "nasm found: $((nasm -v) | Select-Object -First 1)"
    # Found only in its folder: on the saved PATH (a terminal opened before it was added), or not?
    $saved = @([Environment]::GetEnvironmentVariable('Path', 'Machine'),
               [Environment]::GetEnvironmentVariable('Path', 'User')) -join ';'
    if ($nasmDir -and (($saved -split ';') -notcontains $nasmDir)) {
        Write-Warn2 "It is in $nasmDir, which is not on your PATH: a cargo build outside these"
        Write-Warn2 "scripts needs it there (or the NASM variable set to $nasmDir\nasm.exe)."
    }
}
elseif (Confirm-Action "Install NASM $NasmVersion now? (the nasm.us installer, about 1 MB)") {
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    $inst = Join-Path $env:TEMP "nasm-$NasmVersion-installer-x64.exe"
    $prev = $ProgressPreference
    $ProgressPreference = 'SilentlyContinue'   # the 5.1 progress bar dominates the transfer
    try {
        Invoke-WebRequest -Uri "https://www.nasm.us/pub/nasm/releasebuilds/$NasmVersion/win64/nasm-$NasmVersion-installer-x64.exe" `
                          -OutFile $inst -TimeoutSec 600
    }
    finally { $ProgressPreference = $prev }
    if ((Get-FileHash -Algorithm SHA256 -LiteralPath $inst).Hash -ne $NasmSha256) {
        Remove-Item $inst -ErrorAction SilentlyContinue
        Write-Warn2 "The NASM installer is not the expected file (SHA-256); not running it."
        Write-Warn2 "Install NASM by hand from https://www.nasm.us and re-run."
    }
    else {
        $p = Start-Process -FilePath $inst -ArgumentList '/S' -PassThru
        if (-not $p.WaitForExit(10 * 60 * 1000)) { try { $p.Kill() } catch { } }
        Remove-Item $inst -ErrorAction SilentlyContinue
        $nasmDir = Add-NasmPath
        if ($nasmDir -and (Have 'nasm')) {
            Write-Ok "NASM installed in $nasmDir."
            # On your PATH too, so a build in a new terminal finds it.
            $user = [Environment]::GetEnvironmentVariable('Path', 'User')
            if (($user -split ';') -notcontains $nasmDir) {
                [Environment]::SetEnvironmentVariable('Path', ((@($user, $nasmDir) | Where-Object { $_ }) -join ';'), 'User')
                Write-Ok "Added $nasmDir to your PATH (new terminals see it)."
            }
        }
        else {
            Write-Warn2 "The NASM installer ran, but nasm.exe was not found; install it by hand: https://www.nasm.us"
        }
    }
}
else {
    Write-Warn2 "Skipping. The build stops at rav1e (the AVIF encoder) until NASM is installed: https://www.nasm.us"
}

# ---------------------------------------------------------------------------
# 4. Verification build
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
